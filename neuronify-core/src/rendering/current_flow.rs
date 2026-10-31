use std::collections::BTreeMap;

use glam::Vec3;
use hecs::Entity;

use crate::components::{Connection, CurrentClamp};
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::current_color;
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;

const PARTICLE_SPACING: f32 = 0.7 * NODE_RADIUS;
const PARTICLE_RADIUS_PER_PATH_RADIUS: f32 = 1.6;
const PARTICLE_WHITENESS: f32 = 0.55;
const DISTANCE_PER_SIMULATED_SECOND_AND_NANOAMPERE: f64 = 240.0;
const FASTEST_SHOWN_CURRENT: f64 = 2e-9;

pub fn collect_current_flow(
    world: &hecs::World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    time: f64,
) -> Vec<Sphere> {
    let mut particles = Vec::new();
    for (entity, connection) in world.query::<&Connection>().iter() {
        let Ok(clamp) = world.get::<&CurrentClamp>(connection.from) else {
            continue;
        };
        let Some(path) = paths.get(&entity) else {
            continue;
        };
        particles.extend(flow_along(path, clamp.current_output, time));
    }
    particles
}

fn flow_along(path: &ConnectionPath, current: f64, time: f64) -> Vec<Sphere> {
    let length = path.length();
    if current <= 0.0 || length <= 0.0 {
        return Vec::new();
    }
    let speed =
        DISTANCE_PER_SIMULATED_SECOND_AND_NANOAMPERE * current.min(FASTEST_SHOWN_CURRENT) * 1e9;
    let offset = (time * speed).rem_euclid(PARTICLE_SPACING as f64) as f32;
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

    fn source_and_paths(current: f64) -> (hecs::World, BTreeMap<Entity, ConnectionPath>) {
        let mut world = hecs::World::new();
        let source = Element::CurrentSource.spawn(&mut world, Vec3::ZERO);
        world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = current;
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(0.0, 0.0, 10.0));
        spawn_synapse(&mut world, source, neuron);
        let paths = connection_paths(&world, &HashMap::new(), &Theme::Dark.palette());
        (world, paths)
    }

    fn first_particle(current: f64, time: f64) -> Vec3 {
        let (world, paths) = source_and_paths(current);
        collect_current_flow(&world, &paths, time)[0].position
    }

    #[test]
    fn particles_line_the_connection_of_a_current_source() {
        let (world, paths) = source_and_paths(0.5e-9);
        let length = paths.values().next().unwrap().length();
        let particles = collect_current_flow(&world, &paths, 0.0);
        assert_eq!(particles.len(), (length / PARTICLE_SPACING).ceil() as usize);

        let (world, paths) = source_and_paths(0.0);
        assert!(collect_current_flow(&world, &paths, 0.0).is_empty());
    }

    #[test]
    fn a_stronger_current_flows_faster() {
        let moved = |current: f64| {
            let start = first_particle(current, 0.0);
            start.distance(first_particle(current, 1e-4))
        };
        let weak = moved(0.3e-9);
        let strong = moved(0.6e-9);
        assert!(weak > 0.0);
        assert!((strong / weak - 2.0).abs() < 0.05, "{weak} {strong}");
        assert!((moved(5e-9) - moved(FASTEST_SHOWN_CURRENT)).abs() < 1e-4);
    }
}
