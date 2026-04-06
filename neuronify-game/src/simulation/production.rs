//! Production, migration, and maturation systems for the radial-glial cell flow.
//!
//! Call order each frame:
//!   tick_production → move_neuroblasts → tick_maturation → advance_growth_cones

use glam::Vec3;
use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, Deletable, NeuronType, Position, Selectable,
    SpatialDynamics, StaticConnectionSource, COUPLING_CAPACITANCE,
};

use crate::components::{
    DendriteDepth, Faction, GrowthCone, MaturingNeuron, MovePath, Neuroblast, OriginNeuron,
    Ownership, PlayerEconomy, PlayerId, ProducibleCell, ProducibleItem, ProductionQueue, QueuedItem,
};
use crate::simulation::pathfinding::HexGrid;
use crate::spawning;
use crate::constants;
use crate::simulation::economy;

// ── Production ────────────────────────────────────────────────────────────────

/// Advance the front item of each origin's `ProductionQueue`.
/// On completion: spawn the produced unit/neuroblast and pop the queue.
pub fn tick_production(world: &mut hecs::World, dt: f32) {
    // Collect origins and their current front item if complete.
    let mut completions: Vec<(hecs::Entity, Vec3, ProducibleItem)> = Vec::new();

    for (entity, (queue, pos)) in
        world.query::<(&mut ProductionQueue, &Position)>().with::<&OriginNeuron>().iter()
    {
        if let Some(front) = queue.items.front_mut() {
            front.timer += dt;
            if front.timer >= front.duration {
                completions.push((entity, pos.position, front.item));
            }
        }
    }

    for (entity, origin_pos, item) in completions {
        // Pop the completed item.
        if let Ok(mut queue) = world.get::<&mut ProductionQueue>(entity) {
            queue.items.pop_front();
        }
        spawn_produced_item(world, item, origin_pos);
    }
}

/// Dispatch a completed production item to the appropriate spawn function.
fn spawn_produced_item(world: &mut hecs::World, item: ProducibleItem, origin_pos: Vec3) {
    // Count existing mobile units + neuroblasts for a unique exit angle each spawn.
    let unit_count = world.query::<&crate::components::MobileUnit>().iter().count()
        + world.query::<&Neuroblast>().iter().count();
    let offset = spawn_offset(unit_count);

    match item {
        ProducibleItem::ExcitatoryNeuron => {
            let e = spawning::spawn_neuroblast(world, origin_pos, ProducibleCell::ExcitatoryNeuroblast);
            // Give the neuroblast a migration destination so move_neuroblasts will move it.
            let dest = origin_pos + offset.normalize_or_zero() * 10.0;
            let _ = world.insert(e, (MovePath { waypoints: vec![dest], replan_timer: 0.5 },));
        }
        ProducibleItem::InhibitoryNeuron => {
            let e = spawning::spawn_neuroblast(world, origin_pos, ProducibleCell::InhibitoryNeuroblast);
            let dest = origin_pos + offset.normalize_or_zero() * 10.0;
            let _ = world.insert(e, (MovePath { waypoints: vec![dest], replan_timer: 0.5 },));
        }
        ProducibleItem::MicroglialCell => {
            spawning::spawn_microglial_cell(world, origin_pos + offset, Faction::Biological);
        }
        ProducibleItem::Macrophage => {
            spawning::spawn_macrophage(world, origin_pos + offset, Faction::Biological);
        }
        ProducibleItem::TCell => {
            spawning::spawn_t_cell(world, origin_pos + offset, Faction::Biological);
        }
    }
}

/// Compute a spawn exit offset from origin, distributing units around a ring.
/// Using the unit count gives each successive spawn a different angle.
fn spawn_offset(unit_count: usize) -> Vec3 {
    // Golden angle increment (≈137.5°) ensures no two consecutive spawns overlap
    // even when many units have already been produced.
    let angle = unit_count as f32 * 2.399_963; // radians — golden angle
    Vec3::new(angle.cos(), 0.0, angle.sin()) * 8.0
}

// ── Neuroblast migration ──────────────────────────────────────────────────────

/// Sample a "carrot" point that is `lookahead` world units ahead along the path.
/// Steering toward this point rather than the immediate waypoint rounds corners
/// and produces smooth curved motion through the hex grid.
fn path_lookahead(current: Vec3, waypoints: &[Vec3], lookahead: f32) -> Vec3 {
    if waypoints.is_empty() {
        return current;
    }
    let mut remaining = lookahead;
    let mut pos = current;
    for &wp in waypoints {
        let dist = pos.distance(wp);
        if dist < 0.001 {
            pos = wp;
            continue;
        }
        if remaining <= dist {
            return pos + (wp - pos) / dist * remaining;
        }
        remaining -= dist;
        pos = wp;
    }
    *waypoints.last().unwrap()
}

/// Move all neuroblasts with an active `MovePath`.
/// Uses a short lookahead to steer smoothly through Chaikin waypoints,
/// with proximity-based waypoint popping and snap-to-destination to stop cleanly.
pub fn move_neuroblasts(world: &mut hecs::World, grid: &mut HexGrid, dt: f32) {
    const REPLAN_INTERVAL: f32 = 0.5;
    // Lookahead: steer toward a point ~1 hex width ahead on the path.
    // Short enough that the entity actually curves through each waypoint,
    // long enough to round corners smoothly.
    const LOOKAHEAD_DIST: f32 = 5.0;
    // Pop a waypoint once we are this close to it.
    const ARRIVAL_RADIUS: f32 = 1.5;

    let movers: Vec<(hecs::Entity, Vec3, f32)> = world
        .query::<(&Neuroblast, &MovePath, &Position)>()
        .iter()
        .map(|(e, (nb, _, pos))| (e, pos.position, nb.speed))
        .collect();

    let mut arrived: Vec<hecs::Entity> = Vec::new();

    for (entity, current_pos, speed) in movers {
        if let Ok(mut path) = world.get::<&mut MovePath>(entity) {
            path.replan_timer -= dt;
            if path.replan_timer <= 0.0 {
                path.replan_timer = REPLAN_INTERVAL;
                // Replanning delegated to app.rs periodic rebuild — nothing to do here.
            }

            if path.waypoints.is_empty() {
                arrived.push(entity);
                continue;
            }

            // Fast-path: if we are already within ARRIVAL_RADIUS of the goal (last
            // waypoint), snap there and clear the whole path.  This handles the case
            // where Chaikin smoothing creates an overshoot hump at the end — the
            // entity may arrive near the goal while intermediate waypoints still
            // describe a detour ahead.
            let goal = *path.waypoints.last().unwrap();
            if current_pos.distance(goal) <= ARRIVAL_RADIUS {
                path.waypoints.clear();
                arrived.push(entity);
                if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                    pos.position = goal;
                }
                continue;
            }

            let step = speed * dt;

            // Pop intermediate waypoints (all but the last) using three criteria:
            // 1. Proximity: already within ARRIVAL_RADIUS.
            // 2. "Passed": the entity is on the far side of the waypoint's
            //    perpendicular (i.e. it has moved past without touching it).
            // 3. "Closer to next": the entity is already nearer the next waypoint
            //    than the current one (handles overshoot of individual waypoints).
            while path.waypoints.len() >= 2 {
                let wp0 = path.waypoints[0];
                let wp1 = path.waypoints[1];
                let d0 = current_pos.distance(wp0);
                let d1 = current_pos.distance(wp1);
                let close  = d0 <= ARRIVAL_RADIUS;
                let passed = (current_pos - wp0).dot(wp1 - wp0) > 0.0;
                let closer_to_next = d1 < d0;
                if close || passed || closer_to_next {
                    path.waypoints.remove(0);
                } else {
                    break;
                }
            }

            if path.waypoints.is_empty() {
                arrived.push(entity);
                continue;
            }

            // Snap exactly onto the final waypoint when within one step to stop cleanly.
            let new_pos = if path.waypoints.len() == 1
                && current_pos.distance(path.waypoints[0]) <= step
            {
                let dest = path.waypoints[0];
                path.waypoints.clear();
                arrived.push(entity);
                dest
            } else {
                // Steer toward the lookahead carrot for smooth curved motion.
                let carrot = path_lookahead(current_pos, &path.waypoints, LOOKAHEAD_DIST);
                let dir = (carrot - current_pos).normalize_or_zero();
                current_pos + dir * step
            };

            // Update position and hex occupancy.
            let old_hex = grid.world_to_hex(current_pos);
            let new_hex = grid.world_to_hex(new_pos);
            if old_hex != new_hex {
                grid.occupied.remove(&old_hex);
                grid.occupied.insert(new_hex, entity);
            }
            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                pos.position = new_pos;
            }
        }
    }

    for entity in arrived {
        let pos = world.get::<&Position>(entity).ok().map(|p| p.position).unwrap_or(Vec3::ZERO);
        grid.occupied.remove(&grid.world_to_hex(pos));
        let _ = world.remove_one::<MovePath>(entity);
    }
}

/// Chaikin curve subdivision: each interior segment is replaced by two new points
/// at the 25% and 75% positions. The first and last points are preserved so the
/// path still starts and ends at the exact intended positions.
/// `iterations = 2` is enough to turn sharp hex-grid corners into smooth arcs.
fn chaikin_smooth(points: Vec<Vec3>, iterations: usize) -> Vec<Vec3> {
    if points.len() < 3 {
        return points;
    }
    let mut pts = points;
    for _ in 0..iterations {
        let n = pts.len();
        let mut next = Vec::with_capacity(n * 2);
        next.push(pts[0]); // preserve start
        for i in 0..n - 1 {
            next.push(pts[i] * 0.75 + pts[i + 1] * 0.25);
            next.push(pts[i] * 0.25 + pts[i + 1] * 0.75);
        }
        next.push(pts[n - 1]); // preserve end
        pts = next;
    }
    pts
}

/// Issue a move order: run A* from `entity`'s current position to `goal`, and
/// store the resulting `MovePath`. Idempotent — replaces any existing path.
pub fn set_neuroblast_destination(
    world: &mut hecs::World,
    grid: &HexGrid,
    entity: hecs::Entity,
    goal: Vec3,
) {
    let current_pos = match world.get::<&Position>(entity).ok() {
        Some(p) => p.position,
        None => return,
    };

    let start_hex = grid.world_to_hex(current_pos);
    let goal_hex = grid.world_to_hex(goal);

    let hex_path = grid.a_star(entity, start_hex, goal_hex);

    // Convert hex path to world positions; append the exact goal position.
    let mut waypoints: Vec<Vec3> = hex_path
        .iter()
        .map(|&h| {
            let mut wp = grid.hex_to_world(h);
            wp.y = 0.0;
            wp
        })
        .collect();

    // Prepend current position so the spline starts from where the entity actually is.
    waypoints.insert(0, current_pos);

    // Make sure the last waypoint is exactly the goal.
    if waypoints.last().map(|wp| wp.distance(goal) > 0.5).unwrap_or(true) {
        waypoints.push(goal);
    }

    // Apply Chaikin curve subdivision to smooth the hex-grid corners into arcs.
    // 2 iterations give a visually smooth result without excessive waypoint density.
    waypoints = chaikin_smooth(waypoints, 2);

    // Drop the first point (it's the entity's current position — don't re-navigate there).
    if waypoints.len() > 1 {
        waypoints.remove(0);
    }

    let path = MovePath {
        waypoints,
        replan_timer: 0.5,
    };

    if world.get::<&MovePath>(entity).is_ok() {
        if let Ok(mut mp) = world.get::<&mut MovePath>(entity) {
            *mp = path;
        }
    } else {
        let _ = world.insert(entity, (path,));
    }
}

// ── Neuroblast repulsion ──────────────────────────────────────────────────────

/// Push idle neuroblasts apart so they spread into a ring around the origin
/// rather than stacking on top of each other.
/// Also pushes them away from the origin neuron itself.
pub fn tick_neuroblast_repulsion(world: &mut hecs::World, dt: f32) {
    const REPULSION_RADIUS: f32 = 7.0; // minimum centre-to-centre distance
    const REPULSION_STRENGTH: f32 = 18.0; // units/s²
    const ORIGIN_REPULSION_RADIUS: f32 = 9.0;

    // Snapshot positions of all neuroblasts (only idle ones — those without MovePath).
    let neuroblasts: Vec<(hecs::Entity, Vec3)> = world
        .query::<(&Neuroblast, &Position)>()
        .without::<&MovePath>()
        .iter()
        .map(|(e, (_, p))| (e, p.position))
        .collect();

    // Origin neuron position (push away from it).
    let origin_pos: Option<Vec3> = world
        .query::<(&OriginNeuron, &Position)>()
        .iter()
        .next()
        .map(|(_, (_, p))| p.position);

    let mut moves: Vec<(hecs::Entity, Vec3)> = Vec::new();

    for (i, &(ea, pa)) in neuroblasts.iter().enumerate() {
        let mut push = Vec3::ZERO;

        // Repulsion from other idle neuroblasts.
        for (j, &(_, pb)) in neuroblasts.iter().enumerate() {
            if i == j { continue; }
            let delta = pa - pb;
            let dist = delta.length();
            if dist < REPULSION_RADIUS && dist > 0.001 {
                let overlap = REPULSION_RADIUS - dist;
                push += delta.normalize() * overlap * REPULSION_STRENGTH * dt;
            }
        }

        // Repulsion from origin neuron.
        if let Some(op) = origin_pos {
            let delta = pa - op;
            let dist = delta.length();
            if dist < ORIGIN_REPULSION_RADIUS && dist > 0.001 {
                let overlap = ORIGIN_REPULSION_RADIUS - dist;
                push += delta.normalize() * overlap * REPULSION_STRENGTH * dt;
            } else if dist <= 0.001 {
                // Exactly on origin: use entity ID to break symmetry.
                let a = ea.id() as f32 * 2.399_963;
                push += Vec3::new(a.cos(), 0.0, a.sin()) * REPULSION_STRENGTH * dt;
            }
        }

        if push != Vec3::ZERO {
            moves.push((ea, pa + push));
        }
    }

    for (entity, new_pos) in moves {
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.position = new_pos;
        }
    }
}

// ── Maturation ────────────────────────────────────────────────────────────────

/// Advance maturation timers. When a `MaturingNeuron` completes, it is replaced
/// by a full `LeakyNeuron` (via `spawning::spawn_neuron_in_place`) and gets an
/// `IsolationTimer`.
pub fn tick_maturation(world: &mut hecs::World, dt: f32) {
    let mut completed: Vec<(hecs::Entity, Vec3, ProducibleCell)> = Vec::new();

    for (entity, (maturing, pos)) in world.query::<(&mut MaturingNeuron, &Position)>().iter() {
        maturing.timer += dt;
        if maturing.timer >= maturing.duration {
            completed.push((entity, pos.position, maturing.cell_type));
        }
    }

    for (entity, _pos, cell_type) in completed {
        let neuron_type = match cell_type {
            ProducibleCell::ExcitatoryNeuroblast => NeuronType::Excitatory,
            ProducibleCell::InhibitoryNeuroblast => NeuronType::Inhibitory,
        };

        // Remove the maturation marker
        let _ = world.remove::<(MaturingNeuron,)>(entity);

        // Upgrade this entity in-place: add neuron components
        spawning::mature_neuroblast(world, entity, neuron_type);
    }
}


/// Query: does the origin neuron have anything in its production queue?
#[cfg(test)]
pub fn is_producing(world: &hecs::World) -> bool {
    world
        .query::<&ProductionQueue>()
        .with::<&OriginNeuron>()
        .iter()
        .any(|(_, q)| !q.items.is_empty())
}

/// Get a snapshot of the origin neuron's production queue for display.
pub fn queue_snapshot(world: &hecs::World) -> Vec<QueuedItem> {
    world
        .query::<&ProductionQueue>()
        .with::<&OriginNeuron>()
        .iter()
        .next()
        .map(|(_, q)| q.items.iter().cloned().collect())
        .unwrap_or_default()
}

// ── Growth cones ──────────────────────────────────────────────────────────────

/// Advance all in-flight growth cones one timestep.
///
/// Each cone moves toward its target at `GrowthCone.speed`, placing a new axon
/// compartment every `GROWTH_CONE_COMP_SPACING` world units.  When the cone
/// reaches within `GROWTH_CONE_SNAP_RADIUS` of its target it creates the final
/// connection and despawns.
pub fn advance_growth_cones(
    world: &mut hecs::World,
    economy: &mut PlayerEconomy,
    dt: f32,
) {
    #[derive(Clone)]
    struct ConeSnapshot {
        entity: hecs::Entity,
        pos: Vec3,
        cone: GrowthCone,
    }

    let snapshots: Vec<ConeSnapshot> = world
        .query::<(&GrowthCone, &Position)>()
        .iter()
        .map(|(e, (gc, pos))| ConeSnapshot {
            entity: e,
            pos: pos.position,
            cone: gc.clone(),
        })
        .collect();

    let mut to_despawn: Vec<hecs::Entity> = Vec::new();

    for mut snap in snapshots {
        if !snap.cone.waypoints.is_empty() {
            // ── Waypoint mode ─────────────────────────────────────────────────────
            // Snap to each painted waypoint in turn and place a compartment exactly
            // there.  No distance-based interpolation: the built path matches the
            // painted path waypoint for waypoint.
            let wp = *snap.cone.waypoints.front().unwrap();
            let dist_to_wp = snap.pos.distance(wp);
            let wp_snap = constants::GROWTH_CONE_COMP_SPACING * 0.5;

            if dist_to_wp <= wp_snap {
                // Arrived — snap cone to the waypoint position.
                if let Ok(mut pos) = world.get::<&mut Position>(snap.entity) {
                    pos.position = wp;
                }

                if economy::try_spend_blocks(economy, constants::COMPARTMENT_SPAWN_COST) {
                    let neuron_type = snap.cone.neuron_type.clone();
                    let depth = snap.cone.depth;
                    let owner = snap.cone.owner;
                    let compartment = world.spawn((
                        Position { position: wp },
                        neuron_type,
                        Compartment {
                            voltage: -10.0,
                            m: -0.625,
                            h: 0.0,
                            n: 0.0,
                            influence: 0.0,
                            capacitance: 1.0,
                            injected_current: 0.0,
                            fire_impulse: 0.0,
                        },
                        DendriteDepth(depth),
                        Ownership { player: owner },
                        StaticConnectionSource {},
                        Deletable {},
                        Selectable { selected: false },
                        SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
                    ));
                    world.spawn((
                        Connection {
                            from: snap.cone.last_comp,
                            to: compartment,
                            strength: 1.0,
                            directional: true,
                        },
                        Deletable {},
                        CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
                    ));
                    if let Ok(mut gc) = world.get::<&mut GrowthCone>(snap.entity) {
                        gc.last_comp = compartment;
                        gc.last_comp_pos = wp;
                        gc.waypoints.pop_front();
                        gc.depth += 1;
                    }
                    snap.cone.last_comp = compartment;
                    snap.cone.last_comp_pos = wp;
                } else {
                    // Can't afford — pop waypoint anyway to avoid stalling.
                    if let Ok(mut gc) = world.get::<&mut GrowthCone>(snap.entity) {
                        gc.waypoints.pop_front();
                    }
                }
            } else {
                // Move toward waypoint.
                let dir = (wp - snap.pos).normalize_or_zero();
                let step = (snap.cone.speed * dt).min(dist_to_wp);
                if let Ok(mut pos) = world.get::<&mut Position>(snap.entity) {
                    pos.position = snap.pos + dir * step;
                }
            }
            continue;
        }

        // ── Final-target mode (no waypoints remaining) ────────────────────────
        let target_pos = snap
            .cone
            .target_entity
            .and_then(|te| world.get::<&Position>(te).ok().map(|p| p.position))
            .unwrap_or(snap.cone.target);

        let to_target = target_pos - snap.pos;
        let dist_to_target = to_target.length();

        if dist_to_target <= constants::GROWTH_CONE_SNAP_RADIUS {
            if let Some(target_entity) = snap.cone.target_entity {
                let already = world
                    .query::<&Connection>()
                    .iter()
                    .any(|(_, c)| c.from == snap.cone.last_comp && c.to == target_entity);
                if !already && snap.cone.last_comp != target_entity {
                    world.spawn((
                        Connection {
                            from: snap.cone.last_comp,
                            to: target_entity,
                            strength: 1.0,
                            directional: true,
                        },
                        Deletable {},
                        CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
                    ));
                }
            }
            to_despawn.push(snap.entity);
            continue;
        }

        // Move cone forward and lay compartments by distance.
        let dir = to_target / dist_to_target;
        let step = (snap.cone.speed * dt).min(dist_to_target);
        let new_pos = snap.pos + dir * step;

        if let Ok(mut pos) = world.get::<&mut Position>(snap.entity) {
            pos.position = new_pos;
        }
        snap.pos = new_pos;

        let dist_since_last = new_pos.distance(snap.cone.last_comp_pos);
        if dist_since_last >= constants::GROWTH_CONE_COMP_SPACING
            && economy::try_spend_blocks(economy, constants::COMPARTMENT_SPAWN_COST)
        {
            let neuron_type = snap.cone.neuron_type.clone();
            let depth = snap.cone.depth;
            let owner = snap.cone.owner;
            let compartment = world.spawn((
                Position { position: new_pos },
                neuron_type,
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                DendriteDepth(depth),
                Ownership { player: owner },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
            ));
            world.spawn((
                Connection {
                    from: snap.cone.last_comp,
                    to: compartment,
                    strength: 1.0,
                    directional: true,
                },
                Deletable {},
                CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
            ));
            if let Ok(mut gc) = world.get::<&mut GrowthCone>(snap.entity) {
                gc.last_comp = compartment;
                gc.last_comp_pos = new_pos;
                gc.depth += 1;
            }
            snap.cone.last_comp = compartment;
            snap.cone.last_comp_pos = new_pos;
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

/// Find the entity of the player 1 origin neuron.
pub fn origin_entity(world: &hecs::World) -> Option<hecs::Entity> {
    world
        .query::<(&OriginNeuron, &Ownership)>()
        .iter()
        .find(|(_, (_, o))| o.player == PlayerId::Player1)
        .map(|(e, _)| e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_core::{LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronType, Position, VisualRadius, NODE_RADIUS};
    use crate::components::Anchored;
    use crate::components::{
        MetabolicState, OriginNeuron, Ownership, PlayerId,
        ProducibleItem, ProductionQueue, QueuedItem,
    };
    use crate::constants::{NEURON_HEALTH, MAX_NEURON_ENERGY};
    use crate::components::Health;

    /// Spawn a minimal origin neuron with a ProductionQueue.
    fn spawn_origin(world: &mut hecs::World) -> hecs::Entity {
        world.spawn((
            Position { position: glam::Vec3::ZERO },
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
            MetabolicState { energy: MAX_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
            Health::new(NEURON_HEALTH),
            OriginNeuron { player: PlayerId::Player1 },
            ProductionQueue::default(),
            Ownership { player: PlayerId::Player1 },
            Anchored,
            VisualRadius { radius: NODE_RADIUS * 1.5 },
        ))
    }

    #[test]
    fn test_origin_entity_found() {
        let mut world = hecs::World::new();
        let origin = spawn_origin(&mut world);
        let found = origin_entity(&world);
        assert_eq!(found, Some(origin), "origin_entity should find the spawned origin");
    }

    #[test]
    fn test_queue_append_and_tick_spawns_microglia() {
        let mut world = hecs::World::new();
        let origin = spawn_origin(&mut world);

        // Manually push a near-instant T-Cell build.
        {
            let mut queue = world.get::<&mut ProductionQueue>(origin).unwrap();
            queue.items.push_back(QueuedItem {
                item: ProducibleItem::TCell,
                timer: 1.4,   // 0.1 s left until completion (duration = 1.5)
                duration: 1.5,
            });
        }

        assert!(is_producing(&world), "queue should be non-empty before tick");

        // One tick of 0.2 s should complete the item.
        tick_production(&mut world, 0.2);

        // Queue should now be empty.
        let queue_empty = world
            .get::<&ProductionQueue>(origin)
            .map(|q| q.items.is_empty())
            .unwrap_or(false);
        assert!(queue_empty, "queue should be empty after item completes");

        // A TCellUnit should have been spawned.
        let tcell_count = world
            .query::<&crate::components::TCellUnit>()
            .iter()
            .count();
        assert_eq!(tcell_count, 1, "one T-Cell should have been spawned");
    }

    #[test]
    fn test_queue_sequential_processing() {
        let mut world = hecs::World::new();
        let origin = spawn_origin(&mut world);

        // Push two items — both near completion.
        {
            let mut queue = world.get::<&mut ProductionQueue>(origin).unwrap();
            queue.items.push_back(QueuedItem {
                item: ProducibleItem::TCell,
                timer: 1.4,
                duration: 1.5,
            });
            queue.items.push_back(QueuedItem {
                item: ProducibleItem::MicroglialCell,
                timer: 1.9,
                duration: 2.0,
            });
        }

        // First tick completes T-Cell.
        tick_production(&mut world, 0.2);
        let tcell = world.query::<&crate::components::TCellUnit>().iter().count();
        assert_eq!(tcell, 1, "T-Cell spawned after first tick");

        // Queue should still have Microglia.
        let queue_len = world
            .get::<&ProductionQueue>(origin)
            .map(|q| q.items.len())
            .unwrap_or(0);
        assert_eq!(queue_len, 1, "Microglia still queued");

        // Second tick completes Microglia.
        tick_production(&mut world, 0.2);
        let micro = world.query::<&crate::components::MicroglialCell>().iter().count();
        assert_eq!(micro, 1, "Microglia spawned after second tick");
    }

    #[test]
    fn test_neuroblast_spawned_for_excitatory_neuron() {
        let mut world = hecs::World::new();
        let origin = spawn_origin(&mut world);

        {
            let mut queue = world.get::<&mut ProductionQueue>(origin).unwrap();
            queue.items.push_back(QueuedItem {
                item: ProducibleItem::ExcitatoryNeuron,
                timer: 2.9,
                duration: 3.0,
            });
        }

        tick_production(&mut world, 0.2);

        let nb_count = world.query::<&Neuroblast>().iter().count();
        assert_eq!(nb_count, 1, "one Neuroblast should have been spawned");
    }

    // ── Pathfinding + movement tests ─────────────────────────────────────────

    use crate::simulation::pathfinding::HexGrid;
    use crate::constants::{HEX_GRID_CELL_SIZE, NEUROBLAST_SPEED};
    use glam::Vec3;

    fn spawn_neuroblast_at(world: &mut hecs::World, pos: Vec3) -> hecs::Entity {
        world.spawn((
            Position { position: pos },
            Neuroblast { cell_type: ProducibleCell::ExcitatoryNeuroblast, speed: NEUROBLAST_SPEED },
            Health::new(50.0),
            Ownership { player: PlayerId::Player1 },
        ))
    }

    #[test]
    fn test_hex_roundtrip() {
        let grid = HexGrid::new(HEX_GRID_CELL_SIZE);
        let positions = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(20.0, 0.0, 15.0),
        ];
        for pos in positions {
            let hex = grid.world_to_hex(pos);
            let back = grid.hex_to_world(hex);
            assert!(
                pos.distance(back) < HEX_GRID_CELL_SIZE,
                "round-trip should land within one cell: {:?} → {:?} → {:?} (dist {:.2})",
                pos, hex, back, pos.distance(back)
            );
        }
    }

    #[test]
    fn test_astar_finds_path_in_empty_grid() {
        let grid = HexGrid::new(HEX_GRID_CELL_SIZE);
        // Use a dummy entity that the grid will ignore (no occupants registered).
        let world = hecs::World::new();
        // Need a dummy entity for the mover arg.
        let mover = world.reserve_entity();
        let start = grid.world_to_hex(Vec3::new(0.0, 0.0, 0.0));
        let goal  = grid.world_to_hex(Vec3::new(20.0, 0.0, 0.0));
        let path  = grid.a_star(mover, start, goal);
        assert!(!path.is_empty(), "A* must find a path in an empty grid");
        assert_eq!(*path.last().unwrap(), goal, "path must end at goal hex");
    }

    #[test]
    fn test_set_destination_creates_movepath() {
        let mut world = hecs::World::new();
        let grid = HexGrid::new(HEX_GRID_CELL_SIZE);
        let e = spawn_neuroblast_at(&mut world, Vec3::ZERO);
        let goal = Vec3::new(20.0, 0.0, 0.0);
        set_neuroblast_destination(&mut world, &grid, e, goal);

        let has_path = world.get::<&MovePath>(e).is_ok();
        assert!(has_path, "neuroblast should have a MovePath after set_destination");

        let wp_count = world.get::<&MovePath>(e).map(|p| p.waypoints.len()).unwrap_or(0);
        assert!(wp_count > 0, "MovePath must have at least one waypoint, got 0");

        // Last waypoint should be at or near the exact goal.
        let last_wp = world.get::<&MovePath>(e).map(|p| *p.waypoints.last().unwrap()).unwrap();
        assert!(
            last_wp.distance(goal) < 0.1,
            "last waypoint must be near the goal: {:?} vs {:?} (dist {:.3})",
            last_wp, goal, last_wp.distance(goal)
        );
        let _ = grid; // used above
    }

    #[test]
    fn test_neuroblast_moves_toward_destination() {
        let mut world = hecs::World::new();
        let mut grid = HexGrid::new(HEX_GRID_CELL_SIZE);
        let start = Vec3::ZERO;
        let goal  = Vec3::new(20.0, 0.0, 0.0);
        let e = spawn_neuroblast_at(&mut world, start);
        set_neuroblast_destination(&mut world, &grid, e, goal);

        let dt = 0.016_f32;
        move_neuroblasts(&mut world, &mut grid, dt);

        let pos = world.get::<&Position>(e).map(|p| p.position).unwrap();
        let dist_after  = pos.distance(goal);
        let dist_before = start.distance(goal);
        assert!(
            dist_after < dist_before,
            "neuroblast must move closer to goal after one tick: before={:.2} after={:.2}",
            dist_before, dist_after
        );
    }

    #[test]
    fn test_neuroblast_reaches_destination() {
        let mut world = hecs::World::new();
        let mut grid = HexGrid::new(HEX_GRID_CELL_SIZE);
        let start = Vec3::ZERO;
        let goal  = Vec3::new(20.0, 0.0, 0.0);
        let e = spawn_neuroblast_at(&mut world, start);
        set_neuroblast_destination(&mut world, &grid, e, goal);

        // Run up to 5 seconds of simulation (plenty for 20 units at speed 25).
        let dt = 0.016_f32;
        let max_ticks = (5.0 / dt) as usize;
        let mut arrived = false;
        for _tick in 0..max_ticks {
            grid.rebuild(&world);
            move_neuroblasts(&mut world, &mut grid, dt);
            // Arrived = no longer has a MovePath
            if world.get::<&MovePath>(e).is_err() {
                arrived = true;
                break;
            }
        }

        let final_pos = world.get::<&Position>(e).map(|p| p.position).unwrap_or(start);
        assert!(
            arrived,
            "neuroblast should arrive (MovePath removed) within 5 s; final pos {:?}, dist to goal {:.2}",
            final_pos, final_pos.distance(goal)
        );
        assert!(
            final_pos.distance(goal) < 1.0,
            "neuroblast should stop near goal: final {:?}, goal {:?}, dist {:.2}",
            final_pos, goal, final_pos.distance(goal)
        );
    }

    #[test]
    fn test_setup_game_origin_has_production_queue() {
        let mut world = hecs::World::new();
        let dish = crate::simulation::setup::PetriDish {
            center: glam::Vec3::ZERO,
            radius: 120.0,
        };
        crate::simulation::setup::setup_game(&mut world, &dish);

        let origin = origin_entity(&world)
            .expect("origin entity must exist after setup_game");

        let has_queue = world.get::<&ProductionQueue>(origin).is_ok();
        assert!(has_queue, "origin neuron must have a ProductionQueue component after setup_game");
    }
}
