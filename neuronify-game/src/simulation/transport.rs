use std::collections::{HashMap, VecDeque};

use hecs::Entity;

use neuronify_core::{CompartmentCurrent, Connection, Deletable, LeakyNeuron, Position, COUPLING_CAPACITANCE};

use crate::components::*;
use crate::constants::*;
use crate::map::{HexTerrain, TerrainType};

/// Directly harvest glucose from vessel terrain hexes adjacent to each glial cell.
/// No packet entities are created — resources are credited instantly each tick.
/// Harvest rate scales with the number of adjacent vessel hexes.
pub fn harvest_glucose(
    world: &mut hecs::World,
    terrain: &HashMap<(i32, i32), HexTerrain>,
    dt: f64,
) {
    let glial_entities: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();

    for entity in glial_entities {
        let pos = match world.get::<&Position>(entity) {
            Ok(p) => p.position,
            Err(_) => continue,
        };

        let hex = crate::map::hex::world_to_hex(pos.x, pos.z);
        let neighbors = crate::map::hex::hex_neighbors(hex.col, hex.row);

        let vessel_count = neighbors.iter().filter(|nb| {
            terrain.get(&(nb.col, nb.row))
                .map(|t| t.terrain == TerrainType::Vessel)
                .unwrap_or(false)
        }).count();

        if vessel_count == 0 {
            continue;
        }

        let glucose_gain = VESSEL_GLUCOSE_RATE * vessel_count as f64 * dt;
        let block_gain   = VESSEL_BLOCK_RATE   * vessel_count as f64 * dt;

        if let Ok(mut glial) = world.get::<&mut GlialCell>(entity) {
            glial.glucose_stored = (glial.glucose_stored + glucose_gain).min(glial.max_glucose);
            glial.blocks_stored  = (glial.blocks_stored  + block_gain  ).min(glial.max_blocks);
        }
    }
}

/// Spawn lactate packets from glial cells to neurons connected via glial processes.
/// Paths are found by BFS (with parent tracking) through GlialCell/GlialProcess
/// CompartmentCurrent connections, then reconstructed as ordered entity lists so
/// packets travel hop-by-hop along the process chain.
pub fn spawn_lactate_packets(world: &mut hecs::World, dt: f64) {
    let glial_entities: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();

    // Build adjacency restricted to GlialCell/GlialProcess edges (once, shared across all cells).
    let glial_adjacency: HashMap<Entity, Vec<Entity>> = {
        let mut adj: HashMap<Entity, Vec<Entity>> = HashMap::new();
        for (_, conn) in world.query::<&Connection>().with::<&CompartmentCurrent>().iter() {
            let from_glial = world.get::<&GlialProcess>(conn.from).is_ok()
                || world.get::<&GlialCell>(conn.from).is_ok();
            let to_glial = world.get::<&GlialProcess>(conn.to).is_ok()
                || world.get::<&GlialCell>(conn.to).is_ok();
            if from_glial || to_glial {
                adj.entry(conn.from).or_default().push(conn.to);
                adj.entry(conn.to).or_default().push(conn.from);
            }
        }
        adj
    };

    let mut packets_to_spawn: Vec<(Vec<Entity>, f64)> = Vec::new(); // (path, energy)
    let mut glucose_to_deduct: Vec<(Entity, f64)> = Vec::new();

    for glial_entity in glial_entities {
        let (should_spawn, glucose_stored, glial_player) = {
            let Ok(glial) = world.get::<&GlialCell>(glial_entity) else { continue };
            let player = world.get::<&Ownership>(glial_entity).map(|o| o.player).ok();
            (
                glial.lactate_timer <= 0.0 && glial.glucose_stored > 0.0,
                glial.glucose_stored,
                player,
            )
        };

        if !should_spawn {
            if let Ok(mut glial) = world.get::<&mut GlialCell>(glial_entity) {
                glial.lactate_timer -= dt;
            }
            continue;
        }

        // Reset timer
        if let Ok(mut glial) = world.get::<&mut GlialCell>(glial_entity) {
            glial.lactate_timer = LACTATE_PACKET_INTERVAL;
        }

        // BFS with parent tracking to find connected neurons and reconstruct paths.
        let mut visited: HashMap<Entity, Option<Entity>> = HashMap::new();
        let mut queue = VecDeque::new();
        visited.insert(glial_entity, None);
        queue.push_back(glial_entity);
        let mut neuron_paths: Vec<Vec<Entity>> = Vec::new();

        while let Some(current) = queue.pop_front() {
            if current != glial_entity && world.get::<&LeakyNeuron>(current).is_ok() {
                if world.get::<&Ownership>(current).map(|o| o.player).ok() == glial_player {
                    // Reconstruct path from glial to this neuron
                    let mut path = Vec::new();
                    let mut node = current;
                    path.push(node);
                    while let Some(&Some(parent)) = visited.get(&node) {
                        path.push(parent);
                        node = parent;
                    }
                    path.reverse(); // [glial_entity, ..., neuron]
                    neuron_paths.push(path);
                }
                continue; // don't traverse further through neurons
            }
            if let Some(neighbors) = glial_adjacency.get(&current) {
                for &nb in neighbors {
                    if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(nb) {
                        e.insert(Some(current));
                        queue.push_back(nb);
                    }
                }
            }
        }

        if neuron_paths.is_empty() {
            continue;
        }

        let max_distribution = (RESOURCE_FLOW_RATE * LACTATE_PACKET_INTERVAL).min(glucose_stored);
        let per_neuron = max_distribution / neuron_paths.len() as f64;
        let total = per_neuron * neuron_paths.len() as f64;

        for path in neuron_paths {
            packets_to_spawn.push((path, per_neuron));
        }
        glucose_to_deduct.push((glial_entity, total));
    }

    for (glial_entity, amount) in glucose_to_deduct {
        if let Ok(mut glial) = world.get::<&mut GlialCell>(glial_entity) {
            glial.glucose_stored = (glial.glucose_stored - amount).max(0.0);
        }
    }

    for (path, energy) in packets_to_spawn {
        if path.len() < 2 {
            continue;
        }
        let start_pos = world
            .get::<&Position>(path[0])
            .map(|p| p.position)
            .unwrap_or(glam::Vec3::ZERO);
        world.spawn((
            Position { position: start_pos },
            LactatePacket {
                path,
                path_index: 0,
                progress: 0.0,
                speed: LACTATE_PACKET_SPEED,
                energy_amount: energy,
            },
            Deletable {},
        ));
    }
}

/// Move lactate packets hop-by-hop along their glial process paths.
/// Deposit energy into the target neuron on arrival at the final entity.
pub fn move_lactate_packets(world: &mut hecs::World, dt: f64) {
    let packet_entities: Vec<Entity> = world
        .query::<&LactatePacket>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    let mut arrived: Vec<(Entity, Entity, f64)> = Vec::new(); // (packet, neuron, energy)
    let mut destroyed: Vec<Entity> = Vec::new();

    for packet_entity in packet_entities {
        let update = if let Ok(mut packet) = world.get::<&mut LactatePacket>(packet_entity) {
            let from_entity = packet.path[packet.path_index];
            let to_entity = packet.path[packet.path_index + 1];

            if !world.contains(from_entity) || !world.contains(to_entity) {
                Some(Err(packet_entity))
            } else {
                let from_pos = world
                    .get::<&Position>(from_entity)
                    .map(|p| p.position)
                    .unwrap_or(glam::Vec3::ZERO);
                let to_pos = world
                    .get::<&Position>(to_entity)
                    .map(|p| p.position)
                    .unwrap_or(glam::Vec3::ZERO);
                let edge_length = from_pos.distance(to_pos).max(0.01);
                packet.progress += packet.speed * dt as f32 / edge_length;

                if packet.progress >= 1.0 {
                    if packet.path_index + 2 >= packet.path.len() {
                        // Arrived at destination (neuron soma)
                        let neuron_entity = *packet.path.last().unwrap();
                        Some(Ok((packet_entity, neuron_entity, packet.energy_amount)))
                    } else {
                        packet.path_index += 1;
                        packet.progress = 0.0;
                        None
                    }
                } else {
                    None
                }
            }
        } else {
            None
        };

        match update {
            Some(Ok(arrival)) => arrived.push(arrival),
            Some(Err(dead)) => destroyed.push(dead),
            None => {}
        }

        // Update visual position by interpolating along current edge
        if let Ok(packet) = world.get::<&LactatePacket>(packet_entity) {
            if packet.path_index + 1 < packet.path.len() {
                let from_entity = packet.path[packet.path_index];
                let to_entity = packet.path[packet.path_index + 1];
                let progress = packet.progress;
                let from_pos = world.get::<&Position>(from_entity).map(|p| p.position).ok();
                let to_pos = world.get::<&Position>(to_entity).map(|p| p.position).ok();
                if let (Some(a), Some(b)) = (from_pos, to_pos) {
                    let interpolated = a + (b - a) * progress;
                    if let Ok(mut pos) = world.get::<&mut Position>(packet_entity) {
                        pos.position = interpolated;
                    }
                }
            }
        }
    }

    for (packet_entity, neuron_entity, energy) in arrived {
        if let Ok(mut metab) = world.get::<&mut MetabolicState>(neuron_entity) {
            let room = metab.max_energy - metab.energy;
            metab.energy += energy.min(room);
        }
        let _ = world.despawn(packet_entity);
    }
    for e in destroyed {
        let _ = world.despawn(e);
    }
}

/// Automatically wire each GlialCell to nearby neurons with Connection +
/// CompartmentCurrent edges so that lactate packets can travel to them.
/// Connections are created once and cleaned up by `cleanup_orphans` when
/// either endpoint is despawned.
pub fn tick_glial_connect_neurons(world: &mut hecs::World) {
    let glial_entities: Vec<hecs::Entity> =
        world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();

    let neurons: Vec<(hecs::Entity, glam::Vec3)> = world
        .query::<(&LeakyNeuron, &Position)>()
        .iter()
        .map(|(e, (_, p))| (e, p.position))
        .collect();

    let existing: std::collections::HashSet<(hecs::Entity, hecs::Entity)> = world
        .query::<&Connection>()
        .iter()
        .map(|(_, c)| (c.from, c.to))
        .collect();

    let mut to_spawn: Vec<(hecs::Entity, hecs::Entity)> = Vec::new();

    for glial_entity in glial_entities {
        let (glial_pos, distribute_r) = match world.get::<&GlialCell>(glial_entity).ok() {
            Some(g) => (
                world.get::<&Position>(glial_entity).map(|p| p.position).unwrap_or_default(),
                g.distribute_radius as f32,
            ),
            None => continue,
        };

        for &(neuron_entity, neuron_pos) in &neurons {
            if glial_pos.distance(neuron_pos) <= distribute_r
                && !existing.contains(&(glial_entity, neuron_entity))
            {
                to_spawn.push((glial_entity, neuron_entity));
            }
        }
    }

    for (from, to) in to_spawn {
        world.spawn((
            Connection { from, to, strength: 1.0, directional: true },
            CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
            Deletable {},
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{HexTerrain, TerrainType};
    use crate::map::hex::{world_to_hex, hex_neighbors};
    use neuronify_core::Position;
    use glam::Vec3;

    fn vessel_terrain(col: i32, row: i32) -> HexTerrain {
        HexTerrain {
            col,
            row,
            terrain: TerrainType::Vessel,
            passable: false,
            speed_mult: 0.0,
            resource_glucose: true,
        }
    }

    fn spawn_glial_at(world: &mut hecs::World, pos: Vec3) -> hecs::Entity {
        world.spawn((
            Position { position: pos },
            GlialCell::default(),
        ))
    }

    /// Glial with no adjacent vessel hexes should receive zero glucose.
    #[test]
    fn test_no_harvest_far_from_vessel() {
        let mut world = hecs::World::new();
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let entity = spawn_glial_at(&mut world, pos);

        // Terrain has only open hexes — no vessel
        let terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();

        harvest_glucose(&mut world, &terrain, 1.0);

        let glial = world.get::<&GlialCell>(entity).unwrap();
        assert_eq!(glial.glucose_stored, 0.0, "no glucose when not adjacent to vessel");
        assert_eq!(glial.blocks_stored, 0.0);
    }

    /// Glial with exactly one adjacent vessel hex should harvest at base rate.
    #[test]
    fn test_harvest_one_adjacent_vessel() {
        let mut world = hecs::World::new();
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let entity = spawn_glial_at(&mut world, pos);

        // Put a vessel in the first neighbour of the glial's hex
        let hex = world_to_hex(pos.x, pos.z);
        let nb = hex_neighbors(hex.col, hex.row)[0];
        let mut terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();
        terrain.insert((nb.col, nb.row), vessel_terrain(nb.col, nb.row));

        harvest_glucose(&mut world, &terrain, 1.0);

        let glial = world.get::<&GlialCell>(entity).unwrap();
        assert!(
            (glial.glucose_stored - VESSEL_GLUCOSE_RATE).abs() < 1e-6,
            "expected base rate glucose, got {}", glial.glucose_stored
        );
        assert!(
            (glial.blocks_stored - VESSEL_BLOCK_RATE).abs() < 1e-6,
            "expected base rate blocks, got {}", glial.blocks_stored
        );
    }

    /// Glial at a vessel bend/intersection (2 adjacent vessel hexes) should harvest at 2× rate.
    #[test]
    fn test_harvest_two_adjacent_vessels_doubles_rate() {
        let mut world = hecs::World::new();
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let entity = spawn_glial_at(&mut world, pos);

        let hex = world_to_hex(pos.x, pos.z);
        let nbs = hex_neighbors(hex.col, hex.row);
        let mut terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();
        terrain.insert((nbs[0].col, nbs[0].row), vessel_terrain(nbs[0].col, nbs[0].row));
        terrain.insert((nbs[1].col, nbs[1].row), vessel_terrain(nbs[1].col, nbs[1].row));

        harvest_glucose(&mut world, &terrain, 1.0);

        let glial = world.get::<&GlialCell>(entity).unwrap();
        assert!(
            (glial.glucose_stored - 2.0 * VESSEL_GLUCOSE_RATE).abs() < 1e-6,
            "expected 2× rate glucose, got {}", glial.glucose_stored
        );
    }

    /// Glucose is capped at max_glucose even with many vessel neighbors over many ticks.
    #[test]
    fn test_harvest_caps_at_max_glucose() {
        let mut world = hecs::World::new();
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let entity = spawn_glial_at(&mut world, pos);

        let hex = world_to_hex(pos.x, pos.z);
        let nb = hex_neighbors(hex.col, hex.row)[0];
        let mut terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();
        terrain.insert((nb.col, nb.row), vessel_terrain(nb.col, nb.row));

        // Run for a very long time — should cap at max_glucose
        for _ in 0..10000 {
            harvest_glucose(&mut world, &terrain, 1.0);
        }

        let glial = world.get::<&GlialCell>(entity).unwrap();
        assert!(
            glial.glucose_stored <= glial.max_glucose + 1e-6,
            "glucose exceeded cap: {} > {}", glial.glucose_stored, glial.max_glucose
        );
        assert_eq!(glial.glucose_stored, glial.max_glucose);
    }
}
