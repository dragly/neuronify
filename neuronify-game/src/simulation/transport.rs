use std::collections::{HashMap, HashSet, VecDeque};

use hecs::Entity;

use neuronify_core::{CompartmentCurrent, Connection, Deletable, LeakyNeuron, Position};

use crate::components::*;
use crate::constants::*;

/// Spawn glucose packets at blood vessels connected to glial cells.
/// Each glial cell has a timer; when it fires, packets are spawned at each
/// connected vessel and travel along the process chain to the glial cell.
pub fn spawn_glucose_packets(world: &mut hecs::World, dt: f64) {
    // Build undirected adjacency from CompartmentCurrent connections
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        adjacency.entry(conn.from).or_default().push(conn.to);
        adjacency.entry(conn.to).or_default().push(conn.from);
    }

    // Collect vessel data
    let vessel_data: HashMap<Entity, (f64, f64)> = world
        .query::<&BloodVessel>()
        .iter()
        .map(|(e, v)| (e, (v.glucose_rate, v.block_rate)))
        .collect();

    // Collect glial cells that need to spawn packets
    let glial_entities: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();

    let mut packets_to_spawn = Vec::new();

    for glial_entity in glial_entities {
        // Decrement timer
        let should_spawn = if let Ok(mut glial) = world.get::<&mut GlialCell>(glial_entity) {
            glial.packet_timer -= dt;
            if glial.packet_timer <= 0.0 {
                glial.packet_timer = GLUCOSE_PACKET_INTERVAL;
                true
            } else {
                false
            }
        } else {
            false
        };

        if !should_spawn {
            continue;
        }

        // BFS from glial cell to find connected vessels and their paths
        let mut visited: HashMap<Entity, Option<Entity>> = HashMap::new();
        let mut queue = VecDeque::new();
        visited.insert(glial_entity, None);
        queue.push_back(glial_entity);

        let mut reached_vessels = Vec::new();

        while let Some(current) = queue.pop_front() {
            if vessel_data.contains_key(&current) && current != glial_entity {
                reached_vessels.push(current);
                continue; // Don't traverse through vessels
            }
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(neighbor) {
                        e.insert(Some(current));
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // For each reached vessel, reconstruct path from vessel to glial
        for vessel_entity in reached_vessels {
            let (glucose_rate, block_rate) = vessel_data[&vessel_entity];

            // Trace path from vessel back to glial cell via parent pointers
            let mut path = Vec::new();
            let mut current = vessel_entity;
            path.push(current);
            while let Some(Some(parent)) = visited.get(&current) {
                path.push(*parent);
                current = *parent;
            }
            // path is now [vessel, ..., glial_cell]

            if path.len() < 2 {
                continue;
            }

            let glucose_amount = glucose_rate * GLUCOSE_PACKET_INTERVAL;
            let block_amount = block_rate * GLUCOSE_PACKET_INTERVAL;

            // Get vessel position for initial spawn
            let vessel_pos = world
                .get::<&Position>(vessel_entity)
                .map(|p| p.position)
                .unwrap_or(glam::Vec3::ZERO);

            packets_to_spawn.push((vessel_pos, path, glucose_amount, block_amount));
        }
    }

    // Spawn packet entities
    for (pos, path, glucose, blocks) in packets_to_spawn {
        world.spawn((
            Position { position: pos },
            GlucosePacket {
                path,
                path_index: 0,
                progress: 0.0,
                speed: GLUCOSE_PACKET_SPEED,
                glucose_amount: glucose,
                block_amount: blocks,
            },
            Deletable {},
        ));
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

/// Move glucose packets along their paths. When a packet reaches its
/// destination (the glial cell), deposit resources. If the path is broken
/// (an entity in the chain was destroyed), destroy the packet.
pub fn move_glucose_packets(world: &mut hecs::World, dt: f64) {
    // Check which connections still exist (for path validation)
    let mut connection_set: HashSet<(Entity, Entity)> = HashSet::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        connection_set.insert((conn.from, conn.to));
        connection_set.insert((conn.to, conn.from));
    }

    // Collect packet updates
    let mut arrived: Vec<(Entity, Entity, f64, f64)> = Vec::new(); // (packet, glial, glucose, blocks)
    let mut destroyed: Vec<Entity> = Vec::new();

    let packet_entities: Vec<Entity> = world
        .query::<&GlucosePacket>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for packet_entity in packet_entities {
        let update = if let Ok(mut packet) = world.get::<&mut GlucosePacket>(packet_entity) {
            let from_entity = packet.path[packet.path_index];
            let to_entity = packet.path[packet.path_index + 1];

            // Check if both entities still exist
            if !world.contains(from_entity) || !world.contains(to_entity) {
                Some(Err(packet_entity))
            } else {
                // Check if connection still exists
                if !connection_set.contains(&(from_entity, to_entity))
                    && from_entity != to_entity
                    // Allow direct vessel→first-hop even without connection lookup
                    // (the vessel itself isn't in the connection graph as a compartment)
                    && world.get::<&BloodVessel>(from_entity).is_err()
                {
                    Some(Err(packet_entity))
                } else {
                    // Compute edge length and advance
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
                        // Reached next node
                        if packet.path_index + 2 >= packet.path.len() {
                            // Arrived at destination (glial cell)
                            let glial_entity = *packet.path.last().unwrap();
                            Some(Ok((
                                packet_entity,
                                glial_entity,
                                packet.glucose_amount,
                                packet.block_amount,
                            )))
                        } else {
                            // Move to next edge
                            packet.path_index += 1;
                            packet.progress = 0.0;
                            // Update position to the new from-node
                            None
                        }
                    } else {
                        None
                    }
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

        // Update position by interpolating along current edge
        if let Ok(packet) = world.get::<&GlucosePacket>(packet_entity) {
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

    // Deposit resources from arrived packets
    for (packet_entity, glial_entity, glucose, blocks) in arrived {
        if let Ok(mut glial) = world.get::<&mut GlialCell>(glial_entity) {
            glial.glucose_stored = (glial.glucose_stored + glucose).min(glial.max_glucose);
            glial.blocks_stored = (glial.blocks_stored + blocks).min(glial.max_blocks);
        }
        let _ = world.despawn(packet_entity);
    }

    // Destroy packets with broken paths
    for packet_entity in destroyed {
        let _ = world.despawn(packet_entity);
    }
}
