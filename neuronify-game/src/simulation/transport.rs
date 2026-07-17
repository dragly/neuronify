use std::collections::{HashMap, VecDeque};

use hecs::Entity;

use neuronify_core::{CompartmentCurrent, Connection, Deletable, LeakyNeuron, Position, COUPLING_CAPACITANCE};

use crate::components::*;
use crate::constants::*;
use crate::map::{HexTerrain, TerrainType};

/// Check if a world position is near vessel terrain, using both the hex grid
/// and direct world-space probing.
fn is_near_vessel(
    pos: glam::Vec3,
    terrain: &HashMap<(i32, i32), HexTerrain>,
    voronoi_model: Option<&neuronify_game_lib::voronoi_map::terrain_model::MapModel>,
) -> bool {
    // Check hex grid.
    let hex = crate::map::hex::world_to_hex(pos.x, pos.z);
    if terrain.get(&(hex.col, hex.row))
        .map(|t| t.terrain == TerrainType::Vessel)
        .unwrap_or(false)
    {
        return true;
    }
    for nb in crate::map::hex::hex_neighbors(hex.col, hex.row) {
        if terrain.get(&(nb.col, nb.row))
            .map(|t| t.terrain == TerrainType::Vessel)
            .unwrap_or(false)
        {
            return true;
        }
    }
    // Check Voronoi model directly.
    if let Some(model) = voronoi_model {
        use neuronify_game_lib::voronoi_map::game_integration;
        use neuronify_game_lib::voronoi_map::terrain_model::EditorTerrain;
        // Probe the position and nearby offsets.
        // Probe radius should be large enough to span the gap between
        // a process tip and the nearest Voronoi cell center (~half cell spacing).
        let probe_dist = 25.0;
        let probes = [
            pos,
            pos + glam::Vec3::new(probe_dist, 0.0, 0.0),
            pos + glam::Vec3::new(-probe_dist, 0.0, 0.0),
            pos + glam::Vec3::new(0.0, 0.0, probe_dist),
            pos + glam::Vec3::new(0.0, 0.0, -probe_dist),
            pos + glam::Vec3::new(probe_dist * 0.7, 0.0, probe_dist * 0.7),
            pos + glam::Vec3::new(-probe_dist * 0.7, 0.0, probe_dist * 0.7),
            pos + glam::Vec3::new(probe_dist * 0.7, 0.0, -probe_dist * 0.7),
            pos + glam::Vec3::new(-probe_dist * 0.7, 0.0, -probe_dist * 0.7),
        ];
        for p in &probes {
            if game_integration::sample_terrain_at(model, p.x, p.z) == EditorTerrain::Vessel {
                return true;
            }
        }
    }
    false
}

/// Directly harvest glucose from vessel terrain adjacent to each glial cell's processes.
/// Harvest rate scales with the number of process tips touching vessel terrain.
pub fn harvest_glucose(
    world: &mut hecs::World,
    terrain: &HashMap<(i32, i32), HexTerrain>,
    dt: f64,
) {
    harvest_glucose_with_model(world, terrain, None, dt);
}

/// Harvest glucose with optional Voronoi model for direct terrain sampling.
pub fn harvest_glucose_with_model(
    world: &mut hecs::World,
    terrain: &HashMap<(i32, i32), HexTerrain>,
    voronoi_model: Option<&neuronify_game_lib::voronoi_map::terrain_model::MapModel>,
    dt: f64,
) {
    let glial_entities: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();

    // Build adjacency to find glial process compartments reachable from each soma.
    let mut glial_process_positions: HashMap<Entity, Vec<glam::Vec3>> = HashMap::new();
    {
        let connections: Vec<(Entity, Entity)> = world
            .query::<&Connection>()
            .with::<&CompartmentCurrent>()
            .iter()
            .map(|(_, c)| (c.from, c.to))
            .collect();
        for &glial_entity in &glial_entities {
            let mut positions = Vec::new();
            // BFS from soma through GlialProcess connections.
            let mut visited = std::collections::HashSet::new();
            let mut frontier = vec![glial_entity];
            visited.insert(glial_entity);
            while let Some(current) = frontier.pop() {
                if let Ok(p) = world.get::<&Position>(current) {
                    positions.push(p.position);
                }
                for &(from, to) in &connections {
                    let neighbor = if from == current { to } else if to == current { from } else { continue };
                    if visited.insert(neighbor) {
                        if world.get::<&GlialProcess>(neighbor).is_ok() {
                            frontier.push(neighbor);
                        }
                    }
                }
            }
            glial_process_positions.insert(glial_entity, positions);
        }
    }

    let mut glucose_packets_to_spawn: Vec<(Entity, glam::Vec3, glam::Vec3)> = Vec::new();

    for entity in glial_entities {
        // Check all process positions for proximity to vessel terrain.
        // Sample the terrain (hex grid OR direct Voronoi) around each process tip.
        let positions = glial_process_positions.get(&entity).cloned().unwrap_or_default();
        let mut vessel_contact_count = 0usize;
        let mut vessel_contact_pos: Option<glam::Vec3> = None;

        // Count vessel contacts: unique vessel hexes adjacent to any process,
        // OR Voronoi vessel cells near any process.
        let mut vessel_hex_set = std::collections::HashSet::new();
        for pos in &positions {
            // Hex grid check.
            let hex = crate::map::hex::world_to_hex(pos.x, pos.z);
            if terrain.get(&(hex.col, hex.row))
                .map(|t| t.terrain == TerrainType::Vessel).unwrap_or(false) {
                vessel_hex_set.insert((hex.col, hex.row));
                if vessel_contact_pos.is_none() { vessel_contact_pos = Some(*pos); }
            }
            for nb in crate::map::hex::hex_neighbors(hex.col, hex.row) {
                if terrain.get(&(nb.col, nb.row))
                    .map(|t| t.terrain == TerrainType::Vessel).unwrap_or(false) {
                    vessel_hex_set.insert((nb.col, nb.row));
                    if vessel_contact_pos.is_none() { vessel_contact_pos = Some(*pos); }
                }
            }
            // Voronoi model check.
            if let Some(model) = voronoi_model {
                use neuronify_game_lib::voronoi_map::game_integration;
                use neuronify_game_lib::voronoi_map::terrain_model::EditorTerrain;
                let probe_dist = 25.0;
                let probes = [
                    *pos,
                    *pos + glam::Vec3::new(probe_dist, 0.0, 0.0),
                    *pos + glam::Vec3::new(-probe_dist, 0.0, 0.0),
                    *pos + glam::Vec3::new(0.0, 0.0, probe_dist),
                    *pos + glam::Vec3::new(0.0, 0.0, -probe_dist),
                    *pos + glam::Vec3::new(probe_dist * 0.7, 0.0, probe_dist * 0.7),
                    *pos + glam::Vec3::new(-probe_dist * 0.7, 0.0, probe_dist * 0.7),
                    *pos + glam::Vec3::new(probe_dist * 0.7, 0.0, -probe_dist * 0.7),
                    *pos + glam::Vec3::new(-probe_dist * 0.7, 0.0, -probe_dist * 0.7),
                ];
                for p in &probes {
                    if game_integration::sample_terrain_at(model, p.x, p.z) == EditorTerrain::Vessel {
                        vessel_contact_count += 1;
                        if vessel_contact_pos.is_none() { vessel_contact_pos = Some(*pos); }
                        break; // one contact per process position
                    }
                }
            }
        }
        vessel_contact_count += vessel_hex_set.len();

        if vessel_contact_count == 0 {
            continue;
        }
        let vessel_count = vessel_contact_count;

        let glucose_gain = VESSEL_GLUCOSE_RATE * vessel_count as f64 * dt;
        let block_gain   = VESSEL_BLOCK_RATE   * vessel_count as f64 * dt;

        if let Ok(mut glial) = world.get::<&mut GlialCell>(entity) {
            glial.glucose_stored = (glial.glucose_stored + glucose_gain).min(glial.max_glucose);
            glial.blocks_stored  = (glial.blocks_stored  + block_gain  ).min(glial.max_blocks);

            // Spawn visual glucose packets toward the glial soma.
            glial.glucose_spawn_timer -= dt;
            if glial.glucose_spawn_timer <= 0.0 {
                glial.glucose_spawn_timer = 0.5;
                if let (Ok(soma_pos), Some(contact_pos)) = (
                    world.get::<&Position>(entity),
                    vessel_contact_pos,
                ) {
                    glucose_packets_to_spawn.push((entity, contact_pos, soma_pos.position));
                }
            }
        }
    }

    // Spawn glucose packets.
    for (target, start, _end) in glucose_packets_to_spawn {
        world.spawn((
            Position { position: start },
            GlucosePacket {
                target,
                start,
                progress: 0.0,
                speed: LACTATE_PACKET_SPEED,
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

/// Move glucose packets from vessel positions toward glial somas. Despawn on arrival.
pub fn move_glucose_packets(world: &mut hecs::World, dt: f64) {
    let packets: Vec<(Entity, Entity, glam::Vec3)> = world
        .query::<&GlucosePacket>()
        .iter()
        .map(|(e, p)| (e, p.target, p.start))
        .collect();

    let mut to_despawn = Vec::new();

    for (packet_entity, target, start) in &packets {
        if !world.contains(*target) {
            to_despawn.push(*packet_entity);
            continue;
        }
        let target_pos = world.get::<&Position>(*target)
            .map(|p| p.position).unwrap_or(*start);
        let total_dist = start.distance(target_pos).max(0.01);

        let arrived = if let Ok(mut packet) = world.get::<&mut GlucosePacket>(*packet_entity) {
            packet.progress += packet.speed * dt as f32 / total_dist;
            packet.progress >= 1.0
        } else { true };

        if arrived {
            to_despawn.push(*packet_entity);
        } else if let Ok(packet) = world.get::<&GlucosePacket>(*packet_entity) {
            let pos = start + (target_pos - *start) * packet.progress;
            drop(packet);
            if let Ok(mut p) = world.get::<&mut Position>(*packet_entity) {
                p.position = pos;
            }
        }
    }

    for e in to_despawn {
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

    /// Build a minimal test scenario: one vessel hex, one glial cell with processes
    /// reaching it, one neuron connected via the glial process chain.
    /// Returns (world, terrain, glial_entity, neuron_entity).
    pub fn build_glial_test_scenario() -> (
        hecs::World,
        HashMap<(i32, i32), HexTerrain>,
        Entity,
        Entity,
    ) {
        use glam::Vec3;
        use crate::map::hex;

        let mut world = hecs::World::new();

        // Pick a hex for the vessel and the glial cell.
        // Place glial at hex (5, 5), vessel at neighbor (6, 5).
        let glial_world_pos = hex::hex_to_world(5, 5);
        let vessel_hex = hex::hex_neighbors(5, 5)[0]; // first neighbor

        // Build terrain: vessel at one neighbor, open elsewhere.
        let mut terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();
        terrain.insert((5, 5), HexTerrain {
            col: 5, row: 5,
            terrain: TerrainType::Open,
            passable: true, speed_mult: 1.0, resource_glucose: false,
        });
        terrain.insert((vessel_hex.col, vessel_hex.row), HexTerrain {
            col: vessel_hex.col, row: vessel_hex.row,
            terrain: TerrainType::Vessel,
            passable: false, speed_mult: 1.0, resource_glucose: true,
        });

        // Spawn glial cell with processes.
        let glial_entity = crate::spawning::spawn_glial(
            &mut world, glial_world_pos, PlayerId::Player1, 3,
        );

        // Spawn a neuron nearby (one hex away in the opposite direction).
        let neuron_hex = hex::hex_neighbors(5, 5)[3]; // opposite side
        let neuron_pos = hex::hex_to_world(neuron_hex.col, neuron_hex.row);
        let neuron_entity = world.spawn((
            Position { position: neuron_pos },
            neuronify_core::LeakyNeuron::default(),
            neuronify_core::LeakyDynamics::default(),
            neuronify_core::LeakCurrent::default(),
            neuronify_core::NeuronType::Excitatory,
            MetabolicState::default(),
            Ownership { player: PlayerId::Player1 },
            Health::new(crate::constants::NEURON_HEALTH),
        ));

        (world, terrain, glial_entity, neuron_entity)
    }

    /// Verify that the Voronoi scenario's player glial cells actually find vessel hexes.
    #[test]
    fn voronoi_scenario_glial_finds_vessels() {
        use crate::simulation::voronoi_scenario;
        use neuronify_game_lib::voronoi_map::game_integration;

        let mut world = hecs::World::new();
        let model = game_integration::load_voronoi_model();
        let mut terrain = HashMap::new();
        voronoi_scenario::setup_voronoi_scenario(&mut world, &model, &mut terrain);

        // Count vessel hexes in terrain.
        let vessel_count = terrain.values().filter(|h| h.terrain == TerrainType::Vessel).count();
        assert!(vessel_count > 0, "Scenario should have vessel terrain hexes, got 0");

        // Find all glial cells and check their process positions against terrain.
        let glial_entities: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();
        assert!(!glial_entities.is_empty(), "Scenario should have glial cells");

        let mut any_harvesting = false;
        for &glial_entity in &glial_entities {
            // Check soma position.
            let soma_pos = world.get::<&Position>(glial_entity).unwrap().position;
            let soma_hex = crate::map::hex::world_to_hex(soma_pos.x, soma_pos.z);

            // Check all process positions.
            let mut process_positions = vec![soma_pos];
            // BFS through glial processes.
            let connections: Vec<(Entity, Entity)> = world
                .query::<&Connection>()
                .with::<&CompartmentCurrent>()
                .iter()
                .map(|(_, c)| (c.from, c.to))
                .collect();
            let mut visited = std::collections::HashSet::new();
            visited.insert(glial_entity);
            let mut frontier = vec![glial_entity];
            while let Some(current) = frontier.pop() {
                for &(from, to) in &connections {
                    let neighbor = if from == current { to } else if to == current { from } else { continue };
                    if visited.insert(neighbor) && world.get::<&GlialProcess>(neighbor).is_ok() {
                        if let Ok(p) = world.get::<&Position>(neighbor) {
                            process_positions.push(p.position);
                        }
                        frontier.push(neighbor);
                    }
                }
            }

            // Check if any process position is adjacent to a vessel hex.
            for pos in &process_positions {
                let hex = crate::map::hex::world_to_hex(pos.x, pos.z);
                for nb in crate::map::hex::hex_neighbors(hex.col, hex.row) {
                    if terrain.get(&(nb.col, nb.row))
                        .map(|t| t.terrain == TerrainType::Vessel)
                        .unwrap_or(false)
                    {
                        any_harvesting = true;
                    }
                }
            }

            // Print diagnostic info.
            let is_player = world.get::<&Ownership>(glial_entity).is_ok();
            eprintln!(
                "Glial entity {:?} (player={}) at hex ({},{}) world ({:.1},{:.1}), {} processes, reaches vessel: {}",
                glial_entity, is_player,
                soma_hex.col, soma_hex.row,
                soma_pos.x, soma_pos.z,
                process_positions.len() - 1,
                process_positions.iter().any(|pos| {
                    let hex = crate::map::hex::world_to_hex(pos.x, pos.z);
                    crate::map::hex::hex_neighbors(hex.col, hex.row).iter().any(|nb| {
                        terrain.get(&(nb.col, nb.row))
                            .map(|t| t.terrain == TerrainType::Vessel)
                            .unwrap_or(false)
                    })
                })
            );
        }

        // Print vessel hex positions for debugging.
        let vessel_hexes: Vec<_> = terrain.iter()
            .filter(|(_, h)| h.terrain == TerrainType::Vessel)
            .map(|(&(c, r), _)| {
                let wp = crate::map::hex::hex_to_world(c, r);
                eprintln!("  Vessel hex ({},{}) at world ({:.1},{:.1})", c, r, wp.x, wp.z);
                (c, r)
            })
            .collect();
        eprintln!("Total vessel hexes: {}", vessel_hexes.len());

        // Run harvest_glucose with the Voronoi model for direct terrain sampling.
        let dt = 1.0;
        super::harvest_glucose_with_model(&mut world, &terrain, Some(&model), dt);

        let mut any_glucose = false;
        for &glial_entity in &glial_entities {
            let glial = world.get::<&GlialCell>(glial_entity).unwrap();
            if glial.glucose_stored > 0.0 {
                any_glucose = true;
                eprintln!("  -> Glial {:?} harvested glucose: {:.2}", glial_entity, glial.glucose_stored);
            }
        }
        // Debug: directly sample terrain at glial positions.
        for &glial_entity in &glial_entities {
            let pos = world.get::<&Position>(glial_entity).unwrap().position;
            let sampled = game_integration::sample_terrain_at(&model, pos.x, pos.z);
            eprintln!("  Direct sample at glial ({:.1},{:.1}): {:?}", pos.x, pos.z, sampled);
            // Also check with offsets.
            for dx in [-25.0f32, 0.0, 25.0] {
                for dz in [-25.0f32, 0.0, 25.0] {
                    let s = game_integration::sample_terrain_at(&model, pos.x + dx, pos.z + dz);
                    if s == neuronify_game_lib::voronoi_map::terrain_model::EditorTerrain::Vessel {
                        eprintln!("    VESSEL at offset ({},{}) -> world ({:.1},{:.1})", dx, dz, pos.x+dx, pos.z+dz);
                    }
                }
            }
        }

        assert!(any_glucose, "At least one glial cell should have harvested glucose from the terrain");
    }

    /// Glial cell with processes reaching a vessel should harvest glucose
    /// and deliver energy to a connected neuron.
    #[test]
    fn glial_harvests_and_feeds_neuron() {
        let (mut world, terrain, glial_entity, neuron_entity) = build_glial_test_scenario();

        // Drain the neuron's initial energy to see if glial restores it.
        if let Ok(mut m) = world.get::<&mut MetabolicState>(neuron_entity) {
            m.energy = 10.0;
        }

        let dt = 0.01; // 10ms steps
        // Run for 10 simulated seconds.
        for _ in 0..1000 {
            super::tick_glial_connect_neurons(&mut world);
            super::harvest_glucose(&mut world, &terrain, dt);
            super::spawn_lactate_packets(&mut world, dt);
            super::move_lactate_packets(&mut world, dt);
        }

        // Check glial harvested glucose.
        let glial = world.get::<&GlialCell>(glial_entity).unwrap();
        assert!(
            glial.glucose_stored > 0.0,
            "Glial should have harvested glucose, got {}", glial.glucose_stored
        );

        // Check neuron received energy.
        let metab = world.get::<&MetabolicState>(neuron_entity).unwrap();
        assert!(
            metab.energy > 10.0,
            "Neuron should have gained energy from glial, got {}", metab.energy
        );
    }
}
