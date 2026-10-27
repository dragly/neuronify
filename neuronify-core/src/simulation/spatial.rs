use std::collections::HashMap;

use glam::Vec3;

use crate::components::*;
use crate::constants::*;

const REPULSION_RANGE: f32 = 2.0 * NODE_RADIUS;

pub fn apply_spatial_forces(world: &mut hecs::World) {
    let nothing_can_move = world.query::<&SpatialDynamics>().iter().next().is_none();
    if nothing_can_move {
        return;
    }

    let mut slot_of = HashMap::new();
    let mut positions = Vec::new();
    let mut moving = Vec::new();
    for (entity, (position, dynamics)) in world
        .query::<(&Position, Option<&SpatialDynamics>)>()
        .iter()
    {
        slot_of.insert(entity, positions.len());
        positions.push(position.position);
        moving.push(dynamics.is_some());
    }
    let cables: Vec<(usize, usize)> = world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .filter_map(|(_, cable)| Some((*slot_of.get(&cable.from)?, *slot_of.get(&cable.to)?)))
        .collect();

    let mut forces = vec![Vec3::ZERO; positions.len()];
    add_repulsion(&positions, &moving, &mut forces);
    add_cable_bending(&positions, &cables, &mut forces);
    add_cable_springs(&positions, &cables, &mut forces);

    for (entity, dynamics) in world.query_mut::<&mut SpatialDynamics>() {
        if let Some(&slot) = slot_of.get(&entity) {
            dynamics.acceleration += forces[slot];
        }
    }
}

fn repulsion(from: Vec3, to: Vec3) -> Vec3 {
    let overlap = (from.distance_squared(to) - REPULSION_RANGE.powi(2)).min(0.0);
    REPULSION_STRENGTH * overlap * (to - from).normalize_or_zero()
}

fn add_repulsion(positions: &[Vec3], moving: &[bool], forces: &mut [Vec3]) {
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
                    if other != slot {
                        forces[slot] += repulsion(position, positions[other]);
                    }
                }
            }
        }
    }
}

fn add_cable_bending(positions: &[Vec3], cables: &[(usize, usize)], forces: &mut [Vec3]) {
    let mut children = vec![Vec::new(); positions.len()];
    for &(parent, child) in cables {
        children[parent].push(child);
    }
    for &(a, b) in cables {
        for &c in &children[b] {
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
}

fn add_cable_springs(positions: &[Vec3], cables: &[(usize, usize)], forces: &mut [Vec3]) {
    for &(from, to) in cables {
        let stretch = positions[from].distance_squared(positions[to]) - SEGMENT_LENGTH.powi(2);
        let force =
            SPRING_STRENGTH * stretch * (positions[to] - positions[from]).normalize_or_zero();
        forces[from] += force;
        forces[to] -= force;
    }
}

pub fn integrate_motion(world: &mut hecs::World, dt: f64) {
    for (_, (position, dynamics)) in world.query_mut::<(&mut Position, &mut SpatialDynamics)>() {
        let gravity = -position.position.y;
        dynamics.acceleration += Vec3::new(0.0, gravity, 0.0);
        dynamics.velocity += dynamics.acceleration * dt as f32;
        position.position += dynamics.velocity * dt as f32;
        dynamics.acceleration = Vec3::new(0.0, 0.0, 0.0);
        dynamics.velocity -= dynamics.velocity * dt as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        world.spawn((
            Connection {
                from,
                to,
                strength: 1.0,
                directional: false,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));
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

        apply_spatial_forces(&mut world);

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

        add_repulsion(&positions, &moving, &mut forces);

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
            apply_spatial_forces(&mut world);
            integrate_motion(&mut world, PHYSICS_DT);
        }

        assert!(bend(&world) < initial_bend / 2.0, "bend {}", bend(&world));
    }

    #[test]
    fn stretched_cables_pull_compartments_together() {
        let mut world = World::new();
        let first = compartment(&mut world, 0.0, 0.0);
        let second = compartment(&mut world, 0.0, 5.0);
        cable(&mut world, first, second);

        apply_spatial_forces(&mut world);

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

        apply_spatial_forces(&mut world);

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
        apply_spatial_forces(&mut expected);

        let mut actual = World::new();
        let actual_crowd = crowd(&mut actual);
        for position in [Vec3::NAN, Vec3::new(f32::INFINITY, 0.0, 0.0)] {
            actual.spawn((Position { position },));
            actual.spawn((Position { position }, SpatialDynamics::default()));
        }
        let far_away = compartment(&mut actual, 3e9, 3e9);
        apply_spatial_forces(&mut actual);

        let expected = accelerations(&expected, &expected_crowd);
        assert!(expected.iter().all(|a| a.is_finite() && *a != Vec3::ZERO));
        assert_eq!(accelerations(&actual, &actual_crowd), expected);
        assert_eq!(acceleration(&actual, far_away), Vec3::ZERO);
    }
}
