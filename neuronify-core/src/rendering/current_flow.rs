use std::collections::BTreeMap;

use glam::Vec3;
use hecs::Entity;

use crate::components::{
    AlternatingCurrent, Connection, CurrentClamp, Milliseconds, Nanoamperes, Picocoulombs,
};
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::current_color;
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;

const PARTICLE_SPACING: f32 = 0.7 * NODE_RADIUS;
const PARTICLE_RADIUS_PER_PATH_RADIUS: f32 = 1.6;
const PARTICLE_WHITENESS: f32 = 0.55;
const DISTANCE_PER_PICOCOULOMB: f64 = 0.24;
const FASTEST_SHOWN_CURRENT: Nanoamperes = Nanoamperes(2.0);

pub fn collect_current_flow(
    world: &hecs::World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    time: Milliseconds,
) -> Vec<Sphere> {
    let mut particles = Vec::new();
    for (entity, connection) in world.query::<(Entity, &Connection)>().iter() {
        let Ok(clamp) = world.get::<&CurrentClamp>(connection.from) else {
            continue;
        };
        let Some(path) = paths.get(&entity) else {
            continue;
        };
        let charge = shown_charge(world, connection.from, clamp.current_output, time);
        particles.extend(flow_along(path, clamp.current_output, charge));
    }
    particles
}

fn shown_charge(
    world: &hecs::World,
    source: Entity,
    current: Nanoamperes,
    time: Milliseconds,
) -> Picocoulombs {
    match world.get::<&AlternatingCurrent>(source) {
        Ok(alternating) => {
            let slowdown = (FASTEST_SHOWN_CURRENT / alternating.amplitude.abs()).min(1.0);
            alternating.charge_until(time) * slowdown
        }
        Err(_) => current.clamp(-FASTEST_SHOWN_CURRENT, FASTEST_SHOWN_CURRENT) * time,
    }
}

fn flow_along(path: &ConnectionPath, current: Nanoamperes, charge: Picocoulombs) -> Vec<Sphere> {
    let length = path.length();
    if current == Nanoamperes::ZERO || length <= 0.0 {
        return Vec::new();
    }
    let travelled = DISTANCE_PER_PICOCOULOMB * charge.0;
    let offset = travelled.rem_euclid(PARTICLE_SPACING as f64) as f32;
    let color = current_color(current).lerp(Vec3::ONE, PARTICLE_WHITENESS);
    let radius = path.radius * PARTICLE_RADIUS_PER_PATH_RADIUS;
    (0..)
        .map(|index| offset + index as f32 * PARTICLE_SPACING)
        .take_while(|&distance| distance < length)
        .map(|distance| Sphere {
            position: path.point_at(distance / length),
            color,
            radius,
            _padding: Default::default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_synapse, Element};
    use crate::rendering::paths::connection_paths;
    use crate::rendering::Theme;
    use std::collections::HashMap;

    const STEP: Milliseconds = Milliseconds(0.1);

    fn source_and_paths(current: f64) -> (hecs::World, BTreeMap<Entity, ConnectionPath>) {
        let mut world = hecs::World::new();
        let source = Element::CurrentSource.spawn(&mut world, Vec3::ZERO);
        world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = Nanoamperes(current);
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(0.0, 0.0, 10.0));
        spawn_synapse(&mut world, source, neuron);
        let paths = connection_paths(&world, &HashMap::new(), &Theme::Dark.palette());
        (world, paths)
    }

    fn first_particle(current: f64, time: Milliseconds) -> Vec3 {
        let (world, paths) = source_and_paths(current);
        collect_current_flow(&world, &paths, time)[0].position
    }

    #[test]
    fn particles_line_the_connection_of_a_current_source() {
        let (world, paths) = source_and_paths(0.5);
        let length = paths.values().next().unwrap().length();
        let particles = collect_current_flow(&world, &paths, Milliseconds::ZERO);
        assert_eq!(particles.len(), (length / PARTICLE_SPACING).ceil() as usize);

        let (world, paths) = source_and_paths(0.0);
        assert!(collect_current_flow(&world, &paths, Milliseconds::ZERO).is_empty());
    }

    #[test]
    fn a_stronger_current_flows_faster() {
        let moved = |current: f64| {
            let start = first_particle(current, Milliseconds::ZERO);
            start.distance(first_particle(current, Milliseconds(0.1)))
        };
        let weak = moved(0.3);
        let strong = moved(0.6);
        assert!(weak > 0.0);
        assert!((strong / weak - 2.0).abs() < 0.05, "{weak} {strong}");
        assert!((moved(5.0) - moved(FASTEST_SHOWN_CURRENT.0)).abs() < 1e-4);
    }

    #[test]
    fn particles_of_an_alternating_source_follow_the_charge_it_has_delivered() {
        let mut world = hecs::World::new();
        let source = Element::AlternatingCurrentSource.spawn(&mut world, Vec3::ZERO);
        let alternating = (*world.get::<&AlternatingCurrent>(source).unwrap()).clone();
        let angular_frequency = std::f64::consts::TAU * alternating.frequency.to_kilohertz();
        let scale = alternating.amplitude * (1.0 / angular_frequency);
        let mut time = Milliseconds::ZERO;
        let mut delivered = Picocoulombs::ZERO;
        let mut previous = Picocoulombs::ZERO;
        let mut went_back = false;
        while time < 2.0 * alternating.frequency.period() {
            let current = alternating.at(time);
            time += STEP;
            delivered += current * STEP;
            let shown = shown_charge(&world, source, current, time);
            assert!(
                (shown - delivered).abs() < 0.02 * scale,
                "{shown} {delivered}"
            );
            went_back |= shown < previous;
            previous = shown;
        }
        assert!(went_back);
    }
}
