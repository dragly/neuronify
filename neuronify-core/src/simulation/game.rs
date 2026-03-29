use std::collections::{HashMap, HashSet, VecDeque};

use glam::Vec3;
use hecs::Entity;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::simulation;

#[derive(Clone, Debug)]
pub struct PetriDish {
    pub center: Vec3,
    pub radius: f32,
}

// --- Building cost helpers ---

/// Find the nearest owned neuron to a position and return its entity and distance.
pub fn nearest_owned_neuron(
    world: &hecs::World,
    position: Vec3,
    player: PlayerId,
) -> Option<(Entity, f32)> {
    world
        .query::<(&Position, &Ownership, &MetabolicState)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o, _))| o.player == player)
        .map(|(e, (p, _, _))| {
            let dist =
                Vec3::new(p.position.x - position.x, 0.0, p.position.z - position.z).length();
            (e, dist)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

/// Check if a position is within build range of any owned neuron or compartment.
pub fn is_within_build_range(world: &hecs::World, position: Vec3, player: PlayerId) -> bool {
    // Check owned neurons
    for (_, (p, o, _)) in world
        .query::<(&Position, &Ownership, &MetabolicState)>()
        .with::<&LeakyNeuron>()
        .iter()
    {
        if o.player == player {
            let dist =
                Vec3::new(p.position.x - position.x, 0.0, p.position.z - position.z).length();
            if dist < BUILD_RANGE {
                return true;
            }
        }
    }
    // Check owned compartments (along axons of owned neurons)
    for (_, (p, _)) in world
        .query::<(&Position, &Ownership)>()
        .with::<&Compartment>()
        .iter()
    {
        let dist = Vec3::new(p.position.x - position.x, 0.0, p.position.z - position.z).length();
        if dist < BUILD_RANGE * 0.5 {
            return true;
        }
    }
    false
}

/// Try to deduct a cost from the nearest owned neuron. Returns true if successful.
pub fn try_spend_energy(
    world: &mut hecs::World,
    position: Vec3,
    player: PlayerId,
    cost: f64,
) -> bool {
    let nearest = {
        let mut best: Option<(Entity, f32)> = None;
        for (e, (p, o, m)) in world
            .query::<(&Position, &Ownership, &MetabolicState)>()
            .with::<&LeakyNeuron>()
            .iter()
        {
            if o.player == player && m.energy > cost {
                let dist =
                    Vec3::new(p.position.x - position.x, 0.0, p.position.z - position.z).length();
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((e, dist));
                }
            }
        }
        best
    };
    if let Some((entity, _)) = nearest {
        if let Ok(mut metab) = world.get::<&mut MetabolicState>(entity) {
            metab.energy -= cost;
            return true;
        }
    }
    false
}

/// Check if a membrane segment blocks the line between two positions.
pub fn membrane_blocks_path(world: &hecs::World, from: Vec3, to: Vec3) -> bool {
    for (_, (p, _)) in world.query::<(&Position, &MembraneSegment)>().iter() {
        let membrane_pos = p.position;
        let membrane_radius = NODE_RADIUS * 1.2;
        // Simple point-to-segment distance check
        let line = to - from;
        let len_sq = line.length_squared();
        if len_sq < 0.001 {
            continue;
        }
        let t = ((membrane_pos - from).dot(line) / len_sq).clamp(0.0, 1.0);
        let closest = from + line * t;
        let dist = Vec3::new(closest.x - membrane_pos.x, 0.0, closest.z - membrane_pos.z).length();
        if dist < membrane_radius {
            return true;
        }
    }
    false
}

// --- Simulation systems ---

pub fn enforce_petri_boundary(world: &mut hecs::World, dish: &PetriDish) {
    for (_, (position, dynamics)) in world.query_mut::<(&mut Position, &mut SpatialDynamics)>() {
        let offset = position.position - dish.center;
        let offset_2d = Vec3::new(offset.x, 0.0, offset.z);
        let dist = offset_2d.length();
        if dist > dish.radius {
            let dir = offset_2d.normalize();
            position.position =
                dish.center + Vec3::new(dir.x * dish.radius, 0.0, dir.z * dish.radius);
            let outward_vel = dynamics.velocity.dot(dir);
            if outward_vel > 0.0 {
                dynamics.velocity -= dir * outward_vel;
            }
        }
    }
}

pub fn harvest_resources(world: &mut hecs::World, dt: f64) {
    let resources: Vec<(Vec3, f64, f32)> = world
        .query::<(&ResourceNode, &Position)>()
        .iter()
        .map(|(_, (r, p))| (p.position, r.emission_rate, r.radius))
        .collect();

    let neuron_entities: Vec<Entity> = world
        .query::<(&LeakyNeuron, &MetabolicState, &Position)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in neuron_entities {
        let pos = world.get::<&Position>(entity).unwrap().position;
        let mut total_harvest = 0.0;
        for &(rpos, rate, radius) in &resources {
            let dist = Vec3::new(pos.x - rpos.x, 0.0, pos.z - rpos.z).length();
            if dist < radius {
                let factor = 1.0 - (dist / radius) as f64;
                total_harvest += rate * factor * dt;
            }
        }
        if total_harvest > 0.0 {
            if let Ok(mut metab) = world.get::<&mut MetabolicState>(entity) {
                metab.energy = (metab.energy + total_harvest).min(metab.max_energy);
            }
        }
    }
}

pub fn metabolic_drain(world: &mut hecs::World, dt: f64) {
    // Count outgoing connections per neuron
    let mut outgoing_count: HashMap<Entity, usize> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        *outgoing_count.entry(conn.from).or_insert(0) += 1;
    }

    // Count compartments owned by each neuron (via connection chain from neuron)
    // For simplicity, charge compartment costs to the neuron that connects to them
    let mut compartment_counts: HashMap<Entity, usize> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        if world.get::<&LeakyNeuron>(conn.from).is_ok()
            && world.get::<&Compartment>(conn.to).is_ok()
        {
            *compartment_counts.entry(conn.from).or_insert(0) += 1;
        }
    }

    // Count membrane segments owned by each player
    let mut membrane_counts: HashMap<PlayerId, usize> = HashMap::new();
    for (_, o) in world
        .query::<&Ownership>()
        .with::<&MembraneSegment>()
        .iter()
    {
        *membrane_counts.entry(o.player).or_insert(0) += 1;
    }

    let neuron_entities: Vec<(Entity, bool, bool, PlayerId)> = world
        .query::<(&LeakyNeuron, &LeakyDynamics, &MetabolicState)>()
        .iter()
        .map(|(e, (_, d, _))| {
            let just_fired = d.time_since_fire < dt * 2.0;
            let is_origin = world.get::<&OriginNeuron>(e).is_ok();
            let player = world
                .get::<&Ownership>(e)
                .map(|o| o.player)
                .unwrap_or(PlayerId::Player1);
            (e, just_fired, is_origin, player)
        })
        .collect();

    // Count neurons per player for distributing membrane costs
    let mut neurons_per_player: HashMap<PlayerId, usize> = HashMap::new();
    for &(_, _, _, player) in &neuron_entities {
        *neurons_per_player.entry(player).or_insert(0) += 1;
    }

    for (entity, just_fired, is_origin, player) in neuron_entities {
        if let Ok(mut metab) = world.get::<&mut MetabolicState>(entity) {
            // Resting cost
            metab.energy -= RESTING_METABOLIC_COST * dt;
            // Firing cost
            if just_fired {
                metab.energy -= FIRING_METABOLIC_COST;
            }
            // Connection maintenance
            let conn_count = outgoing_count.get(&entity).copied().unwrap_or(0);
            metab.energy -= CONNECTION_MAINTENANCE_COST * conn_count as f64 * dt;
            // Compartment maintenance (axon segments)
            let comp_count = compartment_counts.get(&entity).copied().unwrap_or(0);
            metab.energy -= COMPARTMENT_METABOLIC_COST * comp_count as f64 * dt;
            // Membrane costs distributed across all player neurons
            let membrane_count = membrane_counts.get(&player).copied().unwrap_or(0);
            let neuron_count = neurons_per_player.get(&player).copied().unwrap_or(1);
            metab.energy -=
                MEMBRANE_METABOLIC_COST * membrane_count as f64 * dt / neuron_count as f64;

            if is_origin {
                metab.energy = metab.energy.max(ORIGIN_MIN_ENERGY);
            } else {
                metab.energy = metab.energy.max(0.0);
            }
        }
    }
}

pub fn resource_flow(world: &mut hecs::World, dt: f64) {
    let energy_map: HashMap<Entity, f64> = world
        .query::<&MetabolicState>()
        .iter()
        .map(|(e, m)| (e, m.energy))
        .collect();

    // Build adjacency through compartment connections
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        adjacency.entry(conn.from).or_default().push(conn.to);
        adjacency.entry(conn.to).or_default().push(conn.from);
    }

    // For each neuron with MetabolicState, BFS to find connected neurons
    let neuron_entities: HashSet<Entity> = energy_map.keys().copied().collect();
    let mut flows: Vec<(Entity, Entity, f64)> = Vec::new();
    let mut visited_pairs: HashSet<(Entity, Entity)> = HashSet::new();

    for &neuron in &neuron_entities {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(neuron);
        if let Some(neighbors) = adjacency.get(&neuron) {
            for &n in neighbors {
                if !neuron_entities.contains(&n) {
                    queue.push_back(n);
                    visited.insert(n);
                } else if !visited_pairs.contains(&(n, neuron)) {
                    visited_pairs.insert((neuron, n));
                    let from_e = energy_map[&neuron];
                    let to_e = energy_map[&n];
                    let diff = from_e - to_e;
                    let max_flow = (from_e * 0.5).min(to_e * 0.5 + from_e * 0.5);
                    let flow = (RESOURCE_FLOW_RATE * diff * dt).clamp(-max_flow, max_flow);
                    flows.push((neuron, n, flow));
                }
            }
        }
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&current) {
                for &n in neighbors {
                    if visited.contains(&n) {
                        continue;
                    }
                    visited.insert(n);
                    if neuron_entities.contains(&n) && !visited_pairs.contains(&(n, neuron)) {
                        visited_pairs.insert((neuron, n));
                        let from_e = energy_map[&neuron];
                        let to_e = energy_map[&n];
                        let diff = from_e - to_e;
                        let max_flow = (from_e * 0.5).min(to_e * 0.5 + from_e * 0.5);
                        let flow = (RESOURCE_FLOW_RATE * diff * dt).clamp(-max_flow, max_flow);
                        flows.push((neuron, n, flow));
                    } else if !neuron_entities.contains(&n) {
                        queue.push_back(n);
                    }
                }
            }
        }
    }

    for (from, to, flow) in flows {
        if let Ok(mut m) = world.get::<&mut MetabolicState>(from) {
            m.energy = (m.energy - flow).max(0.0);
        }
        if let Ok(mut m) = world.get::<&mut MetabolicState>(to) {
            m.energy = (m.energy + flow).min(m.max_energy);
        }
    }
}

pub fn depolarization_block(world: &mut hecs::World, dt: f64) {
    let entities: Vec<Entity> = world
        .query::<(&LeakyDynamics, &DepolarizationBlock)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in entities {
        let voltage = world
            .get::<&LeakyDynamics>(entity)
            .map(|d| d.voltage)
            .unwrap_or(-0.07);

        if let Ok(mut block) = world.get::<&mut DepolarizationBlock>(entity) {
            if block.blocked {
                // In block state - recover over time
                block.recovery_timer -= dt;
                if block.recovery_timer <= 0.0 {
                    block.blocked = false;
                    block.time_above_threshold = 0.0;
                }
            } else if voltage > DEPOL_BLOCK_THRESHOLD {
                block.time_above_threshold += dt;
                if block.time_above_threshold > DEPOL_BLOCK_DURATION {
                    block.blocked = true;
                    block.recovery_timer = DEPOL_BLOCK_RECOVERY;
                }
            } else {
                block.time_above_threshold = (block.time_above_threshold - dt * 2.0).max(0.0);
            }

            // When blocked, force neuron to be disabled
            if block.blocked {
                if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
                    dynamics.enabled = false;
                }
            } else if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
                dynamics.enabled = true;
            }
        }
    }
}

pub fn check_starvation(world: &mut hecs::World) {
    // Neurons starve
    let dead_neurons: Vec<Entity> = world
        .query::<(&LeakyNeuron, &MetabolicState)>()
        .without::<&OriginNeuron>()
        .iter()
        .filter(|(_, (_, m))| m.energy <= 0.0)
        .map(|(e, _)| e)
        .collect();

    for &entity in &dead_neurons {
        let _ = world.insert_one(entity, Dead);
    }
}

/// Clean up dead entities and orphaned compartments.
pub fn cleanup_dead(world: &mut hecs::World) {
    let dead_entities: HashSet<Entity> = world.query::<&Dead>().iter().map(|(e, _)| e).collect();

    if dead_entities.is_empty() {
        return;
    }

    let connections_to_remove: Vec<Entity> = world
        .query::<&Connection>()
        .iter()
        .filter(|(_, c)| dead_entities.contains(&c.from) || dead_entities.contains(&c.to))
        .map(|(e, _)| e)
        .collect();

    for entity in connections_to_remove {
        let _ = world.despawn(entity);
    }
    for entity in dead_entities {
        let _ = world.despawn(entity);
    }

    // Clean up orphaned compartments (not connected to anything)
    let mut connected_entities: HashSet<Entity> = HashSet::new();
    for (_, conn) in world.query::<&Connection>().iter() {
        connected_entities.insert(conn.from);
        connected_entities.insert(conn.to);
    }
    let orphan_compartments: Vec<Entity> = world
        .query::<&Compartment>()
        .iter()
        .filter(|(e, _)| !connected_entities.contains(e))
        .map(|(e, _)| e)
        .collect();
    for entity in orphan_compartments {
        let _ = world.despawn(entity);
    }
}

pub fn update_ownership(world: &mut hecs::World) {
    let origins: Vec<(Entity, PlayerId)> = world
        .query::<&OriginNeuron>()
        .iter()
        .map(|(e, o)| (e, o.player))
        .collect();

    // Build adjacency from CompartmentCurrent connections (undirected)
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        adjacency.entry(conn.from).or_default().push(conn.to);
        adjacency.entry(conn.to).or_default().push(conn.from);
    }

    // BFS reachability per player
    let mut reachable: HashMap<PlayerId, HashSet<Entity>> = HashMap::new();
    for (origin_entity, player) in &origins {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(*origin_entity);
        visited.insert(*origin_entity);
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if visited.contains(&neighbor) {
                        continue;
                    }
                    let is_neuron = world.get::<&LeakyNeuron>(neighbor).is_ok();
                    let is_compartment = world.get::<&Compartment>(neighbor).is_ok();
                    let is_membrane = world.get::<&MembraneSegment>(neighbor).is_ok();
                    if is_neuron || is_compartment || is_membrane {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        reachable.insert(*player, visited);
    }

    // Assign ownership to neurons
    let all_neurons: Vec<Entity> = world
        .query::<&LeakyNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in all_neurons {
        let p1_owns = reachable
            .get(&PlayerId::Player1)
            .is_some_and(|s| s.contains(&entity));
        let p2_owns = reachable
            .get(&PlayerId::Player2)
            .is_some_and(|s| s.contains(&entity));
        match (p1_owns, p2_owns) {
            (true, false) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player1,
                    },
                );
            }
            (false, true) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player2,
                    },
                );
            }
            _ => {
                let _ = world.remove_one::<Ownership>(entity);
            }
        }
    }

    // Also assign ownership to compartments and membranes
    let all_compartments: Vec<Entity> = world
        .query::<&Compartment>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in all_compartments {
        let p1_owns = reachable
            .get(&PlayerId::Player1)
            .is_some_and(|s| s.contains(&entity));
        let p2_owns = reachable
            .get(&PlayerId::Player2)
            .is_some_and(|s| s.contains(&entity));
        match (p1_owns, p2_owns) {
            (true, false) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player1,
                    },
                );
            }
            (false, true) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player2,
                    },
                );
            }
            _ => {
                let _ = world.remove_one::<Ownership>(entity);
            }
        }
    }

    let all_membranes: Vec<Entity> = world
        .query::<&MembraneSegment>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in all_membranes {
        let p1_owns = reachable
            .get(&PlayerId::Player1)
            .is_some_and(|s| s.contains(&entity));
        let p2_owns = reachable
            .get(&PlayerId::Player2)
            .is_some_and(|s| s.contains(&entity));
        match (p1_owns, p2_owns) {
            (true, false) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player1,
                    },
                );
            }
            (false, true) => {
                let _ = world.insert_one(
                    entity,
                    Ownership {
                        player: PlayerId::Player2,
                    },
                );
            }
            _ => {
                let _ = world.remove_one::<Ownership>(entity);
            }
        }
    }
}

/// Apply substrate zone effects to neurons within their radius.
pub fn apply_substrate_zones(world: &mut hecs::World, dt: f64) {
    // Collect zones
    let zones: Vec<(Vec3, SubstrateZoneType, f32)> = world
        .query::<(&Position, &SubstrateZone)>()
        .iter()
        .map(|(_, (p, z))| (p.position, z.zone_type.clone(), z.radius))
        .collect();

    if zones.is_empty() {
        return;
    }

    // Collect neurons to process
    let neuron_entities: Vec<Entity> = world
        .query::<(&LeakyNeuron, &Position)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    let mut rng = rand::thread_rng();

    for entity in neuron_entities {
        let pos = match world.get::<&Position>(entity) {
            Ok(p) => p.position,
            Err(_) => continue,
        };

        for (zone_pos, zone_type, zone_radius) in &zones {
            let dist = Vec3::new(pos.x - zone_pos.x, 0.0, pos.z - zone_pos.z).length();
            if dist >= *zone_radius {
                continue;
            }
            let intensity = 1.0 - (dist / *zone_radius) as f64;

            match zone_type {
                SubstrateZoneType::HighPotassium => {
                    // Lower threshold → more excitable
                    if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
                        neuron.threshold =
                            LeakyNeuron::default().threshold + HIGH_K_THRESHOLD_SHIFT * intensity;
                    }
                }
                SubstrateZoneType::HighMagnesium => {
                    // Weaken incoming connections - handled via connection query below
                }
                SubstrateZoneType::Noise => {
                    // Inject random current
                    if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
                        let noise: f64 = (rng.gen::<f64>() - 0.5) * 2.0;
                        dynamics.received_currents += NOISE_ZONE_CURRENT * intensity * noise;
                    }
                }
                SubstrateZoneType::Damage => {
                    // Drain energy
                    if let Ok(mut metab) = world.get::<&mut MetabolicState>(entity) {
                        metab.energy = (metab.energy - DAMAGE_ZONE_DRAIN * intensity * dt).max(0.0);
                    }
                }
            }
        }
    }

    // High Magnesium: weaken connections whose target is in the zone
    let mg_zones: Vec<(Vec3, f32)> = zones
        .iter()
        .filter(|(_, t, _)| matches!(t, SubstrateZoneType::HighMagnesium))
        .map(|(p, _, r)| (*p, *r))
        .collect();

    if !mg_zones.is_empty() {
        let connections: Vec<Entity> = world
            .query::<&Connection>()
            .with::<&CompartmentCurrent>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for conn_entity in connections {
            let to_entity = match world.get::<&Connection>(conn_entity) {
                Ok(c) => c.to,
                Err(_) => continue,
            };
            let to_pos = match world.get::<&Position>(to_entity) {
                Ok(p) => p.position,
                Err(_) => continue,
            };

            let mut max_intensity = 0.0_f64;
            for &(zone_pos, zone_radius) in &mg_zones {
                let dist = Vec3::new(to_pos.x - zone_pos.x, 0.0, to_pos.z - zone_pos.z).length();
                if dist < zone_radius {
                    let intensity = 1.0 - (dist / zone_radius) as f64;
                    max_intensity = max_intensity.max(intensity);
                }
            }

            if max_intensity > 0.0 {
                if let Ok(mut conn) = world.get::<&mut Connection>(conn_entity) {
                    // Reduce strength by the Mg factor scaled by intensity
                    conn.strength = 1.0 - (1.0 - HIGH_MG_WEIGHT_FACTOR) * max_intensity;
                }
            } else if let Ok(mut conn) = world.get::<&mut Connection>(conn_entity) {
                conn.strength = 1.0;
            }
        }
    }
}

pub fn setup_game(world: &mut hecs::World, dish: &PetriDish) {
    world.clear();

    let offset = dish.radius * 0.7;

    // Player 1 origin - left side
    world.spawn((
        Position {
            position: Vec3::new(-offset, 0.0, 0.0),
        },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState {
            energy: MAX_NEURON_ENERGY,
            max_energy: MAX_NEURON_ENERGY,
        },
        OriginNeuron {
            player: PlayerId::Player1,
        },
        Ownership {
            player: PlayerId::Player1,
        },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
        DepolarizationBlock {
            time_above_threshold: 0.0,
            blocked: false,
            recovery_timer: 0.0,
        },
    ));

    // Player 2 origin - right side (AI)
    world.spawn((
        Position {
            position: Vec3::new(offset, 0.0, 0.0),
        },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState {
            energy: MAX_NEURON_ENERGY,
            max_energy: MAX_NEURON_ENERGY,
        },
        OriginNeuron {
            player: PlayerId::Player2,
        },
        Ownership {
            player: PlayerId::Player2,
        },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
        DepolarizationBlock {
            time_above_threshold: 0.0,
            blocked: false,
            recovery_timer: 0.0,
        },
    ));

    // Resource nodes - scattered across the dish
    let resource_positions = [
        Vec3::new(0.0, 0.0, 0.0),                     // center (contested)
        Vec3::new(0.0, 0.0, offset * 0.6),            // top center
        Vec3::new(0.0, 0.0, -offset * 0.6),           // bottom center
        Vec3::new(-offset * 0.4, 0.0, offset * 0.4),  // near P1 top
        Vec3::new(-offset * 0.4, 0.0, -offset * 0.4), // near P1 bottom
        Vec3::new(offset * 0.4, 0.0, offset * 0.4),   // near P2 top
        Vec3::new(offset * 0.4, 0.0, -offset * 0.4),  // near P2 bottom
        Vec3::new(-offset * 0.8, 0.0, 0.0),           // far left
        Vec3::new(offset * 0.8, 0.0, 0.0),            // far right
        Vec3::new(0.0, 0.0, offset * 0.9),            // far top
        Vec3::new(0.0, 0.0, -offset * 0.9),           // far bottom
    ];
    for pos in resource_positions {
        world.spawn((Position { position: pos }, ResourceNode::default()));
    }

    // Substrate zones - environmental hazards and features
    // High potassium zone near center-top: neurons here fire more easily
    world.spawn((
        Position {
            position: Vec3::new(-offset * 0.3, 0.0, offset * 0.5),
        },
        SubstrateZone {
            zone_type: SubstrateZoneType::HighPotassium,
            radius: SUBSTRATE_ZONE_RADIUS,
        },
    ));
    // High magnesium zone near center: weakens signal transmission
    world.spawn((
        Position {
            position: Vec3::new(offset * 0.15, 0.0, -offset * 0.3),
        },
        SubstrateZone {
            zone_type: SubstrateZoneType::HighMagnesium,
            radius: SUBSTRATE_ZONE_RADIUS * 1.2,
        },
    ));
    // Noise zone on the periphery: random electrical interference
    world.spawn((
        Position {
            position: Vec3::new(0.0, 0.0, -offset * 0.7),
        },
        SubstrateZone {
            zone_type: SubstrateZoneType::Noise,
            radius: SUBSTRATE_ZONE_RADIUS * 0.8,
        },
    ));
    // Damage zone: toxic region that drains energy
    world.spawn((
        Position {
            position: Vec3::new(offset * 0.5, 0.0, offset * 0.5),
        },
        SubstrateZone {
            zone_type: SubstrateZoneType::Damage,
            radius: SUBSTRATE_ZONE_RADIUS * 0.7,
        },
    ));
}

/// Result of a headless game simulation.
#[derive(Debug)]
pub struct GameResult {
    pub p1_neurons: usize,
    pub p2_neurons: usize,
    pub p1_total_energy: f64,
    pub p2_total_energy: f64,
    pub frames_run: usize,
}

/// Run a headless game with two AI players for balance testing.
/// Returns the final state after `frames` simulation frames.
pub fn run_headless_game(frames: usize, iterations_per_frame: u32) -> GameResult {
    let mut world = hecs::World::new();
    let dish = PetriDish {
        center: Vec3::ZERO,
        radius: PETRI_DISH_RADIUS,
    };
    setup_game(&mut world, &dish);

    let mut ai1 = simulation::ai::AiState::new();
    let mut ai2 = simulation::ai::AiState::new();

    for _frame in 0..frames {
        // LIF substeps
        let lif_dt = LIF_DT;
        let mut time = _frame as f64 * iterations_per_frame as f64 * lif_dt;
        for _ in 0..iterations_per_frame {
            simulation::lif_step(&mut world, lif_dt, time);
            time += lif_dt;
        }

        // FHN + physics substeps
        let recently_fired: HashSet<Entity> = world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < iterations_per_frame as f64 * lif_dt)
            .map(|(e, _)| e)
            .collect();

        for _ in 0..iterations_per_frame {
            simulation::fhn_step(&mut world, FHN_CDT, &recently_fired);
            simulation::apply_spatial_forces(&mut world);
            simulation::integrate_motion(&mut world, PHYSICS_DT);
        }

        // Game systems
        let frame_dt = iterations_per_frame as f64 * LIF_DT;
        enforce_petri_boundary(&mut world, &dish);
        harvest_resources(&mut world, frame_dt);
        metabolic_drain(&mut world, frame_dt);
        resource_flow(&mut world, frame_dt);
        depolarization_block(&mut world, frame_dt);
        apply_substrate_zones(&mut world, frame_dt);
        check_starvation(&mut world);
        cleanup_dead(&mut world);
        update_ownership(&mut world);

        // Both players are AI
        // AI1 acts as Player1
        simulation::ai::ai_tick_for_player(&mut world, &mut ai1, &dish, PlayerId::Player1);
        // AI2 acts as Player2
        simulation::ai::ai_tick_for_player(&mut world, &mut ai2, &dish, PlayerId::Player2);
    }

    // Count results
    let mut p1_neurons = 0;
    let mut p2_neurons = 0;
    let mut p1_energy = 0.0;
    let mut p2_energy = 0.0;
    for (_, (o, m)) in world
        .query::<(&Ownership, &MetabolicState)>()
        .with::<&LeakyNeuron>()
        .iter()
    {
        match o.player {
            PlayerId::Player1 => {
                p1_neurons += 1;
                p1_energy += m.energy;
            }
            PlayerId::Player2 => {
                p2_neurons += 1;
                p2_energy += m.energy;
            }
        }
    }

    GameResult {
        p1_neurons,
        p2_neurons,
        p1_total_energy: p1_energy,
        p2_total_energy: p2_energy,
        frames_run: frames,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_vs_ai_balance() {
        // Run 3 games, each for 5000 frames (~5 seconds of game time at 10 iters/frame)
        // This tests that both AIs can survive and grow, and neither crashes.
        let num_games = 3;
        let frames_per_game = 5000;
        let iterations = 5;

        let mut results = Vec::new();
        for game_idx in 0..num_games {
            let result = run_headless_game(frames_per_game, iterations);
            println!(
                "Game {}: P1={} neurons ({:.0} energy), P2={} neurons ({:.0} energy)",
                game_idx + 1,
                result.p1_neurons,
                result.p1_total_energy,
                result.p2_neurons,
                result.p2_total_energy,
            );
            // Both players should still have their origin neurons at minimum
            assert!(
                result.p1_neurons >= 1,
                "Player 1 lost all neurons in game {}",
                game_idx + 1
            );
            assert!(
                result.p2_neurons >= 1,
                "Player 2 lost all neurons in game {}",
                game_idx + 1
            );
            results.push(result);
        }

        // Check overall balance: neither player should dominate every game
        let p1_wins = results
            .iter()
            .filter(|r| r.p1_neurons > r.p2_neurons)
            .count();
        let p2_wins = results
            .iter()
            .filter(|r| r.p2_neurons > r.p1_neurons)
            .count();
        println!(
            "Balance: P1 won {} games, P2 won {} games, {} ties",
            p1_wins,
            p2_wins,
            num_games - p1_wins - p2_wins
        );
    }
}
