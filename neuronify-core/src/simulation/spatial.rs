use std::collections::{HashMap, HashSet};

use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;

const REPULSION_RANGE: f32 = 2.0 * NODE_RADIUS;

pub fn apply_spatial_forces(world: &mut hecs::World, held: &HashSet<Entity>) {
    let nothing_can_move = world.query::<&SpatialDynamics>().iter().next().is_none();
    if nothing_can_move {
        return;
    }

    let mut slot_of = HashMap::new();
    let mut positions = Vec::new();
    let mut moving = Vec::new();
    let mut dendrite = Vec::new();
    let mut axon = Vec::new();
    for (entity, (position, dynamics, in_dendrite, compartment)) in world
        .query::<(
            &Position,
            Option<&SpatialDynamics>,
            Option<&Dendrite>,
            Option<&Compartment>,
        )>()
        .iter()
    {
        slot_of.insert(entity, positions.len());
        positions.push(position.position);
        moving.push(dynamics.is_some() && !held.contains(&entity));
        dendrite.push(in_dendrite.is_some());
        axon.push(compartment.is_some() && in_dendrite.is_none());
    }
    let (cables, rest_lengths): (Vec<(usize, usize, bool)>, Vec<f32>) = world
        .query::<(&Connection, Option<&Cable>)>()
        .with::<hecs::Or<&Cable, &ConductanceSynapse>>()
        .iter()
        .filter_map(|(_, (connection, cable))| {
            let from = *slot_of.get(&connection.from)?;
            let to = *slot_of.get(&connection.to)?;
            let rest_length = cable.map_or(SEGMENT_LENGTH, |cable| {
                (cable.length / MICROMETRES_PER_UNIT) as f32
            });
            Some(((from, to, connection.directional), rest_length))
        })
        .unzip();

    let mut forces = vec![Vec3::ZERO; positions.len()];
    add_repulsion(&positions, &moving, &dendrite, &axon, &mut forces);
    add_cable_bending(&positions, &cables, &dendrite, &mut forces);
    add_cable_springs(&positions, &cables, &rest_lengths, &mut forces);

    for (entity, dynamics) in world.query_mut::<&mut SpatialDynamics>() {
        if held.contains(&entity) {
            continue;
        }
        if let Some(&slot) = slot_of.get(&entity) {
            dynamics.acceleration += forces[slot];
        }
    }
}

fn repulsion(from: Vec3, to: Vec3) -> Vec3 {
    let overlap = (from.distance_squared(to) - REPULSION_RANGE.powi(2)).min(0.0);
    REPULSION_STRENGTH * overlap * (to - from).normalize_or_zero()
}

fn add_repulsion(
    positions: &[Vec3],
    moving: &[bool],
    dendrite: &[bool],
    axon: &[bool],
    forces: &mut [Vec3],
) {
    let cell_of = |position: Vec3| {
        let coordinate = |x: f32| (x / REPULSION_RANGE).floor() as i32;
        (coordinate(position.x), coordinate(position.z))
    };
    let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (slot, &position) in positions.iter().enumerate() {
        cells.entry(cell_of(position)).or_default().push(slot);
    }
    for (slot, &position) in positions.iter().enumerate() {
        if !moving[slot] {
            continue;
        }
        let (x, z) = cell_of(position);
        for dx in -1..=1 {
            for dz in -1..=1 {
                let cell = (x.saturating_add(dx), z.saturating_add(dz));
                for &other in cells.get(&cell).into_iter().flatten() {
                    if other != slot && !(dendrite[slot] && axon[other]) {
                        forces[slot] += repulsion(position, positions[other]);
                    }
                }
            }
        }
    }
}

fn turn(towards: Vec3, onwards: Vec3) -> f32 {
    Vec3::Y
        .dot(towards.cross(onwards))
        .atan2(towards.dot(onwards))
}

fn add_cable_bending(
    positions: &[Vec3],
    cables: &[(usize, usize, bool)],
    dendrite: &[bool],
    forces: &mut [Vec3],
) {
    let mut children = vec![Vec::new(); positions.len()];
    for &(parent, child, _) in cables {
        children[parent].push(child);
    }
    let joints: Vec<(usize, usize, usize)> = cables
        .iter()
        .filter(|&&(_, _, directional)| !directional)
        .flat_map(|&(a, b, _)| children[b].iter().map(move |&c| (a, b, c)))
        .collect();
    let (axon_joints, dendrite_joints): (Vec<_>, Vec<_>) =
        joints.into_iter().partition(|&(_, b, _)| !dendrite[b]);
    smooth_axon_bends(positions, &axon_joints, dendrite, forces);
    for (a, b, c) in dendrite_joints {
        let ab = (positions[b] - positions[a]).normalize();
        let bc = (positions[c] - positions[b]).normalize();
        let bend = 1.0 - ab.dot(bc);
        let normal = ab.cross(bc);
        let force_a = ANGLE_ALIGNMENT_STRENGTH * bend * ab.cross(normal).normalize();
        let force_c = ANGLE_ALIGNMENT_STRENGTH * bend * bc.cross(normal).normalize();
        if !(force_a.is_finite() && force_c.is_finite()) {
            continue;
        }
        forces[a] += force_a;
        forces[b] -= force_a + force_c;
        forces[c] += force_c;
    }
}

fn smooth_axon_bends(
    positions: &[Vec3],
    joints: &[(usize, usize, usize)],
    dendrite: &[bool],
    forces: &mut [Vec3],
) {
    let turns: Vec<f32> = joints
        .iter()
        .map(|&(a, b, c)| turn(positions[b] - positions[a], positions[c] - positions[b]))
        .collect();
    let mut into = vec![(0.0, 0.0); positions.len()];
    let mut at = vec![(0.0, 0.0); positions.len()];
    for (&(_, b, c), &turn) in joints.iter().zip(&turns) {
        into[c] = (into[c].0 + turn, into[c].1 + 1.0);
        at[b] = (at[b].0 + turn, at[b].1 + 1.0);
    }
    for (&(a, b, c), &turn) in joints.iter().zip(&turns) {
        let (sum, count) = (into[b].0 + at[c].0, into[b].1 + at[c].1);
        let neighbours = if count > 0.0 { sum / count } else { 0.0 };
        let torque = AXON_SMOOTHING_STRENGTH * (turn - neighbours).sin();
        let towards = positions[b] - positions[a];
        let onwards = positions[c] - positions[b];
        let force_a = -torque * Vec3::Y.cross(towards) / towards.length_squared();
        let force_c = -torque * Vec3::Y.cross(onwards) / onwards.length_squared();
        if !(force_a.is_finite() && force_c.is_finite()) {
            continue;
        }
        forces[a] += force_a;
        forces[b] -= force_a + force_c;
        if !dendrite[c] {
            forces[c] += force_c;
        }
    }
}

fn add_cable_springs(
    positions: &[Vec3],
    cables: &[(usize, usize, bool)],
    rest_lengths: &[f32],
    forces: &mut [Vec3],
) {
    for (&(from, to, directional), rest_length) in cables.iter().zip(rest_lengths) {
        let stretch = positions[from].distance_squared(positions[to]) - rest_length.powi(2);
        let force =
            SPRING_STRENGTH * stretch * (positions[to] - positions[from]).normalize_or_zero();
        forces[from] += force;
        if !directional {
            forces[to] -= force;
        }
    }
}

pub fn integrate_motion(world: &mut hecs::World, dt: f64, held: &HashSet<Entity>) {
    for (entity, (position, dynamics)) in world.query_mut::<(&mut Position, &mut SpatialDynamics)>()
    {
        if held.contains(&entity) {
            continue;
        }
        let gravity = -position.position.y;
        dynamics.acceleration += Vec3::new(0.0, gravity, 0.0);
        dynamics.velocity += dynamics.acceleration * dt as f32;
        position.position += dynamics.velocity * dt as f32;
        if position.position.y < 0.0 {
            position.position.y = 0.0;
            dynamics.velocity.y = dynamics.velocity.y.max(0.0);
        }
        dynamics.acceleration = Vec3::new(0.0, 0.0, 0.0);
        dynamics.velocity -= dynamics.velocity * dt as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_cable, spawn_internode};
    use hecs::{Entity, World};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    fn at(x: f32, z: f32) -> Position {
        Position {
            position: Vec3::new(x, 0.0, z),
        }
    }

    fn compartment(world: &mut World, x: f32, z: f32) -> Entity {
        world.spawn((at(x, z), SpatialDynamics::default()))
    }

    fn neuron(world: &mut World, x: f32, z: f32) -> Entity {
        world.spawn((at(x, z),))
    }

    fn cable(world: &mut World, from: Entity, to: Entity) {
        spawn_cable(world, from, to, false);
    }

    fn axon_ending(world: &mut World, from: Entity, to: Entity) {
        spawn_cable(world, from, to, true);
    }

    #[test]
    fn internodes_rest_at_their_own_length() {
        let mut world = World::new();
        let soma = neuron(&mut world, 0.0, 0.0);
        let node = compartment(&mut world, INTERNODE_LENGTH, 0.0);
        spawn_internode(&mut world, soma, node);
        apply_spatial_forces(&mut world, &HashSet::new());
        assert!(acceleration(&world, node).length() < 1e-3);
    }

    fn acceleration(world: &World, entity: Entity) -> Vec3 {
        world.get::<&SpatialDynamics>(entity).unwrap().acceleration
    }

    fn position(world: &World, entity: Entity) -> Vec3 {
        world.get::<&Position>(entity).unwrap().position
    }

    #[test]
    fn overlapping_nodes_push_compartments_apart() {
        let mut world = World::new();
        let left = compartment(&mut world, 0.0, 0.0);
        let right = compartment(&mut world, 0.0, 1.0);
        let below = neuron(&mut world, -1.0, 1.0);
        let far_away = compartment(&mut world, 50.0, 50.0);

        apply_spatial_forces(&mut world, &HashSet::new());

        assert!(acceleration(&world, left).z < 0.0);
        assert!(acceleration(&world, right).z > 0.0);
        assert!(acceleration(&world, right).x > 0.0);
        assert_eq!(position(&world, below), Vec3::new(-1.0, 0.0, 1.0));
        assert_eq!(acceleration(&world, far_away), Vec3::ZERO);
    }

    #[test]
    fn repulsion_finds_every_node_in_range() {
        let mut rng = StdRng::seed_from_u64(1);
        let positions: Vec<Vec3> = (0..500)
            .map(|_| Vec3::new(rng.gen_range(-15.0..15.0), 0.0, rng.gen_range(-15.0..15.0)))
            .collect();
        let moving: Vec<bool> = (0..positions.len()).map(|slot| slot % 3 != 0).collect();
        let mut forces = vec![Vec3::ZERO; positions.len()];

        let none = vec![false; positions.len()];
        add_repulsion(&positions, &moving, &none, &none, &mut forces);

        for (slot, &force) in forces.iter().enumerate() {
            let from_every_other_node: Vec3 = positions
                .iter()
                .enumerate()
                .filter(|&(other, _)| moving[slot] && other != slot)
                .map(|(_, &other)| repulsion(positions[slot], other))
                .sum();
            assert!((force - from_every_other_node).length() < 1e-3);
        }
        assert!(forces.iter().filter(|force| **force != Vec3::ZERO).count() > 100);
    }

    #[test]
    fn cables_straighten_bent_chains() {
        let mut world = World::new();
        let soma = neuron(&mut world, 0.0, 0.0);
        let first = compartment(&mut world, 2.0, 0.0);
        let second = compartment(&mut world, 2.0, 2.0);
        cable(&mut world, soma, first);
        cable(&mut world, first, second);
        let bend = |world: &World| {
            let towards_first = position(world, first) - position(world, soma);
            let towards_second = position(world, second) - position(world, first);
            towards_first.angle_between(towards_second)
        };
        let initial_bend = bend(&world);

        for _ in 0..2000 {
            apply_spatial_forces(&mut world, &HashSet::new());
            integrate_motion(&mut world, PHYSICS_DT, &HashSet::new());
        }

        assert!(bend(&world) < initial_bend / 2.0, "bend {}", bend(&world));
    }

    #[test]
    fn an_axon_drawn_with_square_corners_settles_into_one_smooth_bend() {
        let mut world = World::new();
        let soma = neuron(&mut world, 0.0, 0.0);
        let target = neuron(&mut world, 20.0, 0.0);
        let corners: [(f32, f32); 4] = [(0.0, 0.0), (0.0, -8.0), (20.0, -8.0), (20.0, 0.0)];
        let mut path = Vec::new();
        for pair in corners.windows(2) {
            let [(x0, z0), (x1, z1)] = [pair[0], pair[1]];
            let steps = (x1 - x0).hypot(z1 - z0) as usize / 2;
            path.extend((1..=steps).map(|step| {
                let along = step as f32 / steps as f32;
                (x0 + (x1 - x0) * along, z0 + (z1 - z0) * along)
            }));
        }
        path.pop();
        let mut chain = vec![soma];
        for (x, z) in path {
            let node = compartment(&mut world, x, z);
            cable(&mut world, *chain.last().unwrap(), node);
            chain.push(node);
        }
        axon_ending(&mut world, *chain.last().unwrap(), target);
        chain.push(target);

        for _ in 0..(30.0 * PHYSICS_STEPS_PER_SECOND) as usize {
            apply_spatial_forces(&mut world, &HashSet::new());
            integrate_motion(&mut world, PHYSICS_DT, &HashSet::new());
        }

        let turns: Vec<f32> = chain
            .windows(3)
            .map(|joint| {
                let [a, b, c] = [joint[0], joint[1], joint[2]].map(|e| position(&world, e));
                turn(b - a, c - b).to_degrees()
            })
            .collect();
        let direction = turns.iter().sum::<f32>().signum();
        assert!(
            turns.iter().all(|&turn| turn * direction > -3.0),
            "{turns:?}"
        );
        assert!(turns.iter().all(|&turn| turn.abs() < 45.0), "{turns:?}");
    }

    #[test]
    fn an_axon_ending_does_not_bend_the_dendrite_it_ends_on() {
        let mut world = World::new();
        let axon = compartment(&mut world, 0.0, 0.0);
        let dendrite = compartment(&mut world, 2.0, 0.0);
        let beyond = compartment(&mut world, 2.0, 2.0);
        axon_ending(&mut world, axon, dendrite);
        cable(&mut world, dendrite, beyond);

        apply_spatial_forces(&mut world, &HashSet::new());

        assert_eq!(acceleration(&world, beyond), Vec3::ZERO);
    }

    #[test]
    fn axons_make_way_for_dendrites_without_pushing_them() {
        let mut world = World::new();
        let axon = compartment(&mut world, 0.0, 0.0);
        let dendrite = compartment(&mut world, 1.0, 0.0);
        world.insert_one(axon, Compartment).unwrap();
        world.insert(dendrite, (Compartment, Dendrite)).unwrap();

        apply_spatial_forces(&mut world, &HashSet::new());

        assert!(acceleration(&world, axon).x < 0.0);
        assert_eq!(acceleration(&world, dendrite), Vec3::ZERO);
    }

    #[test]
    fn smoothing_an_axon_into_its_ending_leaves_the_dendrite_alone() {
        let mut world = World::new();
        let start = compartment(&mut world, 0.0, 0.0);
        let terminal = compartment(&mut world, 2.0, 0.0);
        let dendrite = compartment(&mut world, 2.0, 2.0);
        world.insert_one(dendrite, Dendrite).unwrap();
        cable(&mut world, start, terminal);
        axon_ending(&mut world, terminal, dendrite);

        apply_spatial_forces(&mut world, &HashSet::new());

        assert_ne!(acceleration(&world, terminal), Vec3::ZERO);
        assert_eq!(acceleration(&world, dendrite), Vec3::ZERO);
    }

    #[test]
    fn an_axon_ending_pulls_the_axon_but_not_the_dendrite_it_ends_on() {
        let mut world = World::new();
        let axon = compartment(&mut world, 0.0, 0.0);
        let dendrite = compartment(&mut world, 6.0, 0.0);
        axon_ending(&mut world, axon, dendrite);

        apply_spatial_forces(&mut world, &HashSet::new());

        assert!(acceleration(&world, axon).x > 0.0);
        assert_eq!(acceleration(&world, dendrite), Vec3::ZERO);
    }

    #[test]
    fn crowded_compartments_stay_above_the_floor() {
        let mut world = World::new();
        let lower = compartment(&mut world, 0.0, 0.0);
        let upper = compartment(&mut world, 0.0, 0.0);
        world.get::<&mut Position>(upper).unwrap().position.y = 0.5;

        for _ in 0..200 {
            apply_spatial_forces(&mut world, &HashSet::new());
            integrate_motion(&mut world, PHYSICS_DT, &HashSet::new());
            for compartment in [lower, upper] {
                let height = position(&world, compartment).y;
                assert!(height >= 0.0, "sank to {height}");
            }
        }
    }

    #[test]
    fn held_compartments_stay_put_while_the_rest_moves() {
        let mut world = World::new();
        let held = compartment(&mut world, 0.0, 0.0);
        let free = compartment(&mut world, 0.5, 0.0);
        let before = [held, free].map(|entity| position(&world, entity));
        for _ in 0..500 {
            apply_spatial_forces(&mut world, &HashSet::from([held]));
            integrate_motion(&mut world, PHYSICS_DT, &HashSet::from([held]));
        }
        assert_eq!(position(&world, held), before[0]);
        assert!(position(&world, free).distance(before[1]) > 0.1);
    }

    #[test]
    fn stretched_cables_pull_compartments_together() {
        let mut world = World::new();
        let first = compartment(&mut world, 0.0, 0.0);
        let second = compartment(&mut world, 0.0, 5.0);
        cable(&mut world, first, second);

        apply_spatial_forces(&mut world, &HashSet::new());

        assert!(acceleration(&world, first).z > 0.0);
        assert!(acceleration(&world, second).z < 0.0);
    }

    #[test]
    fn cables_to_despawned_entities_are_ignored() {
        let mut world = World::new();
        let erased = neuron(&mut world, 0.0, 0.0);
        let attached = compartment(&mut world, 100.0, 0.0);
        cable(&mut world, erased, attached);
        world.despawn(erased).unwrap();
        let reusing_erased_id = neuron(&mut world, 50.0, 5.0);
        assert_eq!(reusing_erased_id.id(), erased.id());

        apply_spatial_forces(&mut world, &HashSet::new());

        assert_eq!(acceleration(&world, attached), Vec3::ZERO);
    }

    #[test]
    fn non_finite_positions_do_not_affect_others() {
        let crowd = |world: &mut World| -> Vec<Entity> {
            [0.3, -0.4, 1.2]
                .into_iter()
                .map(|x| compartment(world, x, 0.5))
                .collect()
        };
        let accelerations = |world: &World, entities: &[Entity]| -> Vec<Vec3> {
            entities
                .iter()
                .map(|&entity| acceleration(world, entity))
                .collect()
        };

        let mut expected = World::new();
        let expected_crowd = crowd(&mut expected);
        apply_spatial_forces(&mut expected, &HashSet::new());

        let mut actual = World::new();
        let actual_crowd = crowd(&mut actual);
        for position in [Vec3::NAN, Vec3::new(f32::INFINITY, 0.0, 0.0)] {
            actual.spawn((Position { position },));
            actual.spawn((Position { position }, SpatialDynamics::default()));
        }
        let far_away = compartment(&mut actual, 3e9, 3e9);
        apply_spatial_forces(&mut actual, &HashSet::new());

        let expected = accelerations(&expected, &expected_crowd);
        assert!(expected.iter().all(|a| a.is_finite() && *a != Vec3::ZERO));
        assert_eq!(accelerations(&actual, &actual_crowd), expected);
        assert_eq!(acceleration(&actual, far_away), Vec3::ZERO);
    }
}
