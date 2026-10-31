use std::collections::{BTreeMap, HashMap};

use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::measurement::visual_input::VisualInput;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;
use crate::rendering::shapes::{generator_color, shown_voltage, soma_color};

/// `radii` comes from `compartment_radii`, so the joints match the cylinders.
pub fn collect_spheres(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
) -> Vec<Sphere> {
    let mut spheres = Vec::new();
    let display = NeuronDisplay::of(world);

    spheres.extend(
        world
            .query::<(&LeakyNeuron, &LeakyDynamics, &Position)>()
            .iter()
            .map(|(entity, (neuron, dynamics, position))| Sphere {
                position: position.position,
                color: soma_color(world, entity, neuron, dynamics, &display, palette),
                radius: NODE_RADIUS,
                _padding: Default::default(),
            }),
    );

    // Touch sensors and current sources are flat pads, see `collect_shapes`
    spheres.extend(
        world
            .query::<(&Position, &GeneratorDynamics, Option<&NeuronType>)>()
            .without::<hecs::Or<&TouchSensor, &VisualInput>>()
            .iter()
            .map(|(_, (position, dynamics, neuron_type))| Sphere {
                position: position.position,
                color: generator_color(dynamics, neuron_type.copied().unwrap_or_default(), palette),
                radius: NODE_RADIUS,
                _padding: Default::default(),
            }),
    );

    spheres.extend(
        world
            .query::<(&Compartment, &Position, &NeuronType)>()
            .iter()
            .map(|(entity, (compartment, position, neuron_type))| Sphere {
                position: position.position,
                color: palette.compartment(neuron_type, shown_voltage(compartment, &display)),
                radius: radii
                    .get(&entity)
                    .copied()
                    .unwrap_or(COMPARTMENT_SPHERE_SCALE * NODE_RADIUS),
                _padding: Default::default(),
            }),
    );

    spheres.reserve(paths.values().map(|path| path.points.len()).sum());
    for path in paths.values() {
        let segments = path.points.len().saturating_sub(1);
        let joints = path
            .points
            .iter()
            .enumerate()
            .skip(1)
            .take(segments.saturating_sub(1));
        spheres.extend(joints.map(|(i, &position)| {
            Sphere {
                position,
                color: path
                    .start_color
                    .lerp(path.end_color, i as f32 / segments as f32),
                radius: path.radius,
                _padding: Default::default(),
            }
        }));
    }

    // Inhibitory synapses end in a bar instead, see `collect_cylinders`
    spheres.extend(
        paths
            .values()
            .filter(|path| path.directional && !path.inhibitory)
            .filter_map(|path| {
                Some(Sphere {
                    position: *path.points.last()?,
                    color: terminal_color(),
                    radius: path.terminal_radius,
                    _padding: Default::default(),
                })
            }),
    );

    for (entity, synapse) in world.query::<&CurrentSynapse>().iter() {
        if synapse.triggers.is_empty() {
            continue;
        }
        let Some(path) = paths.get(&entity) else {
            continue;
        };
        spheres.extend(synapse.triggers.iter().map(|&trigger_time| {
            let fire_time = trigger_time - synapse.delay;
            let progress = if synapse.delay > 0.0 {
                ((synapse.time - fire_time) / synapse.delay).clamp(0.0, 1.0) as f32
            } else {
                1.0
            };
            Sphere {
                position: path.point_at(progress),
                color: palette.spike,
                radius: NODE_RADIUS * TRIGGER_SPHERE_SCALE,
                _padding: Default::default(),
            }
        }));
    }

    spheres
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::cylinders::compartment_radii;
    use crate::rendering::paths::connection_paths;
    use glam::Vec3;

    #[test]
    fn spikes_in_flight_travel_along_their_synapse() {
        let mut world = hecs::World::new();
        let mut neuron = |x: f32| {
            world.spawn((
                Position {
                    position: Vec3::new(x, 0.0, 0.0),
                },
                LeakyNeuron::default(),
                LeakyDynamics::default(),
                NeuronType::Excitatory,
            ))
        };
        let (first, second) = (neuron(0.0), neuron(20.0));
        let synapse = |from, to, triggers| {
            (
                Connection {
                    from,
                    to,
                    strength: 1.0,
                    directional: true,
                },
                CurrentSynapse {
                    delay: 0.004,
                    time: 0.997,
                    triggers,
                    ..Default::default()
                },
            )
        };
        let firing = world.spawn(synapse(first, second, vec![1.0]));
        world.spawn(synapse(second, first, vec![]));
        let palette = Theme::Dark.palette();
        let radii = compartment_radii(&world);
        let paths = connection_paths(&world, &radii, &palette);

        let spikes: Vec<Sphere> = collect_spheres(&world, &radii, &paths, &palette)
            .into_iter()
            .filter(|sphere| {
                sphere.color == palette.spike && sphere.radius == NODE_RADIUS * TRIGGER_SPHERE_SCALE
            })
            .collect();

        assert_eq!(spikes.len(), 1);
        let a_quarter_along = paths[&firing].point_at(0.25);
        assert!(spikes[0].position.distance(a_quarter_along) < 1e-5);
    }
}
