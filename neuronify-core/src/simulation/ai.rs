use std::cmp::Ordering;

use glam::Vec3;
use hecs::Entity;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::simulation::game;

pub struct AiState {
    pub tick_counter: u32,
    pub rng: rand::rngs::ThreadRng,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            tick_counter: 0,
            rng: rand::thread_rng(),
        }
    }
}

impl AiState {
    pub fn new() -> Self {
        Self::default()
    }
}

fn dist_2d(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}

fn cmp_f32(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

pub fn ai_tick(world: &mut hecs::World, ai: &mut AiState, dish: &game::PetriDish) {
    ai_tick_for_player(world, ai, dish, PlayerId::Player2);
}

pub fn ai_tick_for_player(
    world: &mut hecs::World,
    ai: &mut AiState,
    dish: &game::PetriDish,
    player: PlayerId,
) {
    ai.tick_counter += 1;
    if !ai.tick_counter.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }

    let opponent = match player {
        PlayerId::Player1 => PlayerId::Player2,
        PlayerId::Player2 => PlayerId::Player1,
    };

    let roll: f64 = ai.rng.gen();
    if roll < AI_EXPAND_CHANCE {
        ai_expand(world, ai, dish, player);
    } else if roll < AI_EXPAND_CHANCE + AI_DEFEND_CHANCE {
        ai_defend(world, ai, player, opponent);
    } else {
        ai_attack(world, ai, player, opponent);
    }
}

/// Find all neurons owned by the given player.
fn ai_neurons(world: &hecs::World, player: PlayerId) -> Vec<(Entity, Vec3, f64)> {
    world
        .query::<(&Position, &Ownership, &MetabolicState)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o, _))| o.player == player)
        .map(|(e, (p, _, m))| (e, p.position, m.energy))
        .collect()
}

/// Find the nearest resource node that doesn't have a neuron of this player near it.
fn find_target_resource(
    world: &hecs::World,
    dish: &game::PetriDish,
    player: PlayerId,
) -> Option<Vec3> {
    let ai_neuron_positions: Vec<Vec3> = ai_neurons(world, player)
        .iter()
        .map(|(_, p, _)| *p)
        .collect();
    let ai_origin = match ai_neuron_positions.first() {
        Some(p) => *p,
        None => return None,
    };

    world
        .query::<(&ResourceNode, &Position)>()
        .iter()
        .filter_map(|(_, (_, pos))| {
            let rpos = pos.position;
            // Skip if we already have a neuron near this resource
            let has_nearby = ai_neuron_positions
                .iter()
                .any(|np| dist_2d(*np, rpos) < RESOURCE_NODE_RADIUS);
            if has_nearby {
                return None;
            }
            // Skip if outside dish
            if dist_2d(rpos, dish.center) > dish.radius {
                return None;
            }
            Some((rpos, dist_2d(rpos, ai_origin)))
        })
        .min_by(|a, b| cmp_f32(a.1, b.1))
        .map(|(p, _)| p)
}

/// Expand: place a neuron toward the nearest unclaimed resource, connected via axon.
fn ai_expand(world: &mut hecs::World, ai: &mut AiState, dish: &game::PetriDish, player: PlayerId) {
    let neurons = ai_neurons(world, player);
    if neurons.is_empty() {
        return;
    }

    let target = match find_target_resource(world, dish, player) {
        Some(t) => t,
        None => return,
    };

    // Find the AI neuron closest to the target
    let (source_entity, source_pos, source_energy) = match neurons
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(a.1, target), dist_2d(b.1, target)))
        .copied()
    {
        Some(n) => n,
        None => return,
    };

    let total_cost = NEURON_SPAWN_COST + COMPARTMENT_SPAWN_COST * 3.0;
    if source_energy < total_cost + 10.0 {
        return;
    }

    let dir = Vec3::new(target.x - source_pos.x, 0.0, target.z - source_pos.z);
    let dist = dir.length();
    if dist < 2.0 {
        return;
    }
    let dir_norm = dir / dist;
    let step = dist.min(BUILD_RANGE * 0.7);
    let new_pos = source_pos + dir_norm * step;

    if dist_2d(new_pos, dish.center) > dish.radius * 0.95 {
        return;
    }

    // Deduct cost
    if let Ok(mut metab) = world.get::<&mut MetabolicState>(source_entity) {
        metab.energy -= total_cost;
    }

    let neuron_type = if ai.rng.gen::<f64>() < 0.8 {
        NeuronType::Excitatory
    } else {
        NeuronType::Inhibitory
    };

    let new_entity = world.spawn((
        Position { position: new_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        neuron_type.clone(),
        MetabolicState::default(),
        Ownership { player },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
        Deletable {},
        DepolarizationBlock {
            time_above_threshold: 0.0,
            blocked: false,
            recovery_timer: 0.0,
        },
    ));

    if matches!(neuron_type, NeuronType::Inhibitory) {
        let _ = world.insert_one(new_entity, Inhibitory);
    }

    spawn_axon_between(world, source_entity, new_entity, &neuron_type, player);
}

/// Spawn compartment chain between two neurons and connect them.
fn spawn_axon_between(
    world: &mut hecs::World,
    from_neuron: Entity,
    to_neuron: Entity,
    neuron_type: &NeuronType,
    player: PlayerId,
) {
    let from_pos = world
        .get::<&Position>(from_neuron)
        .map(|p| p.position)
        .unwrap_or(Vec3::ZERO);
    let to_pos = world
        .get::<&Position>(to_neuron)
        .map(|p| p.position)
        .unwrap_or(Vec3::ZERO);

    let dir = to_pos - from_pos;
    let dist = dir.length();
    if dist < 0.1 {
        return;
    }
    let num_compartments = ((dist / (MIN_CREATION_DISTANCE_AXON * 1.5)) as usize).max(1);

    let mut prev_entity = from_neuron;
    for i in 1..=num_compartments {
        let t = i as f32 / (num_compartments + 1) as f32;
        let pos = from_pos + dir * t;

        let comp_entity = world.spawn((
            Position { position: pos },
            Compartment {
                voltage: -50.0,
                m: 0.0,
                h: 1.0,
                n: 0.0,
                influence: 0.0,
                capacitance: COUPLING_CAPACITANCE,
                injected_current: 0.0,
                fire_impulse: 0.0,
            },
            neuron_type.clone(),
            SpatialDynamics {
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
            },
            Ownership { player },
            Deletable {},
        ));

        world.spawn((
            Connection {
                from: prev_entity,
                to: comp_entity,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        prev_entity = comp_entity;
    }

    world.spawn((
        Connection {
            from: prev_entity,
            to: to_neuron,
            strength: 1.0,
            directional: true,
        },
        CompartmentCurrent {
            capacitance: COUPLING_CAPACITANCE,
        },
    ));
}

/// Defend: place an inhibitory neuron near the border facing the opponent.
fn ai_defend(world: &mut hecs::World, ai: &mut AiState, player: PlayerId, opponent: PlayerId) {
    let neurons = ai_neurons(world, player);
    if neurons.len() < 2 {
        return;
    }

    let p1_positions: Vec<Vec3> = world
        .query::<(&Position, &Ownership)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o))| o.player == opponent)
        .map(|(_, (p, _))| p.position)
        .collect();

    if p1_positions.is_empty() {
        return;
    }

    // Find AI neuron closest to any P1 neuron
    let (source_entity, source_pos, source_energy) = match neurons.iter().min_by(|a, b| {
        let da = p1_positions
            .iter()
            .map(|p| dist_2d(a.1, *p))
            .fold(f32::INFINITY, f32::min);
        let db = p1_positions
            .iter()
            .map(|p| dist_2d(b.1, *p))
            .fold(f32::INFINITY, f32::min);
        cmp_f32(da, db)
    }) {
        Some(n) => (n.0, n.1, n.2),
        None => return,
    };

    let cost = NEURON_SPAWN_COST + COMPARTMENT_SPAWN_COST * 2.0;
    if source_energy < cost + 10.0 {
        return;
    }

    let nearest_enemy = p1_positions
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(source_pos, **a), dist_2d(source_pos, **b)))
        .copied()
        .unwrap_or(Vec3::ZERO);

    let dir = Vec3::new(
        nearest_enemy.x - source_pos.x,
        0.0,
        nearest_enemy.z - source_pos.z,
    );
    let dist = dir.length();
    if dist < 3.0 {
        return;
    }
    let offset_x: f32 = ai.rng.gen::<f32>() * 4.0 - 2.0;
    let offset_z: f32 = ai.rng.gen::<f32>() * 4.0 - 2.0;
    let new_pos = source_pos
        + (dir / dist) * (dist.min(BUILD_RANGE * 0.5))
        + Vec3::new(offset_x, 0.0, offset_z);

    if let Ok(mut metab) = world.get::<&mut MetabolicState>(source_entity) {
        metab.energy -= cost;
    }

    let new_entity = world.spawn((
        Position { position: new_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Inhibitory,
        Inhibitory,
        MetabolicState::default(),
        Ownership { player },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
        Deletable {},
        DepolarizationBlock {
            time_above_threshold: 0.0,
            blocked: false,
            recovery_timer: 0.0,
        },
    ));

    spawn_axon_between(
        world,
        source_entity,
        new_entity,
        &NeuronType::Inhibitory,
        player,
    );
}

/// Attack: project an excitatory axon toward nearby enemy neurons.
fn ai_attack(world: &mut hecs::World, ai: &mut AiState, player: PlayerId, opponent: PlayerId) {
    let neurons = ai_neurons(world, player);
    if neurons.len() < 4 {
        return; // Need critical mass before attacking
    }

    let enemy_neurons: Vec<(Entity, Vec3)> = world
        .query::<(&Position, &Ownership)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o))| o.player == opponent)
        .map(|(e, (p, _))| (e, p.position))
        .collect();

    if enemy_neurons.is_empty() {
        return;
    }

    // Pick a rich neuron
    let rich_neurons: Vec<_> = neurons
        .iter()
        .filter(|(_, _, e)| *e > NEURON_SPAWN_COST * 2.0)
        .collect();
    if rich_neurons.is_empty() {
        return;
    }

    let idx = ai.rng.gen_range(0..rich_neurons.len());
    let &(source_entity, source_pos, _) = rich_neurons[idx];

    // Find nearest enemy neuron
    let (target_entity, _target_pos) = match enemy_neurons
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(source_pos, a.1), dist_2d(source_pos, b.1)))
        .copied()
    {
        Some(n) => n,
        None => return,
    };

    let target_dist = dist_2d(source_pos, _target_pos);
    if target_dist > BUILD_RANGE * 3.0 {
        return;
    }

    let cost = COMPARTMENT_SPAWN_COST * 5.0;
    if !game::try_spend_energy(world, source_pos, player, cost) {
        return;
    }

    spawn_axon_between(
        world,
        source_entity,
        target_entity,
        &NeuronType::Excitatory,
        player,
    );
}
