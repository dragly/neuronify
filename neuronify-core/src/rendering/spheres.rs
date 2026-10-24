use std::collections::{BTreeMap, HashMap};

use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;
use crate::rendering::shapes::{generator_color, soma_color};

/// `radii` comes from `compartment_radii`, so the joints match the cylinders.
pub fn collect_spheres(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
) -> Vec<Sphere> {
    let mut spheres = Vec::new();

    let lif_neuron_spheres: Vec<Sphere> = world
        .query::<(&LeakyNeuron, &LeakyDynamics, &Position)>()
        .iter()
        .map(|(entity, (neuron, dynamics, position))| Sphere {
            position: position.position,
            color: soma_color(world, entity, neuron, dynamics, palette),
            radius: NODE_RADIUS,
            _padding: Default::default(),
        })
        .collect();

    // Touch sensors and current sources are flat pads, see `collect_shapes`
    let generator_spheres: Vec<Sphere> = world
        .query::<(&Position, &GeneratorDynamics)>()
        .without::<&TouchSensor>()
        .iter()
        .map(|(_, (position, dynamics))| Sphere {
            position: position.position,
            color: generator_color(dynamics, palette),
            radius: NODE_RADIUS,
            _padding: Default::default(),
        })
        .collect();

    let compartment_spheres: Vec<Sphere> = world
        .query::<(&Compartment, &Position, &NeuronType)>()
        .iter()
        .map(|(entity, (compartment, position, neuron_type))| {
            let value = ((compartment.voltage + 50.0) / 200.0) as f32;
            Sphere {
                position: position.position,
                color: palette.neurocolor(neuron_type, value),
                radius: radii
                    .get(&entity)
                    .copied()
                    .unwrap_or(COMPARTMENT_SPHERE_SCALE * NODE_RADIUS),
                _padding: Default::default(),
            }
        })
        .collect();

    let joint_spheres: Vec<Sphere> = paths
        .values()
        .flat_map(|path| {
            let segments = path.points.len().saturating_sub(1);
            let joints = path
                .points
                .iter()
                .enumerate()
                .skip(1)
                .take(segments.saturating_sub(1));
            joints.map(move |(i, &position)| Sphere {
                position,
                color: path
                    .start_color
                    .lerp(path.end_color, i as f32 / segments as f32),
                radius: path.radius,
                _padding: Default::default(),
            })
        })
        .collect();

    // Inhibitory synapses end in a bar instead, see `collect_cylinders`
    let synapse_spheres: Vec<Sphere> = paths
        .values()
        .filter(|path| path.directional && !path.inhibitory)
        .filter_map(|path| {
            Some(Sphere {
                position: *path.points.last()?,
                color: terminal_color(),
                radius: 0.5 * NODE_RADIUS,
                _padding: Default::default(),
            })
        })
        .collect();

    let trigger_spheres: Vec<Sphere> = world
        .query::<&CurrentSynapse>()
        .iter()
        .filter_map(|(entity, synapse)| Some((paths.get(&entity)?, synapse)))
        .flat_map(|(path, synapse)| {
            synapse
                .triggers
                .iter()
                .map(move |&trigger_time| {
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
                })
                .collect::<Vec<_>>()
        })
        .collect();

    spheres.extend(lif_neuron_spheres.iter());
    spheres.extend(generator_spheres.iter());
    spheres.extend(compartment_spheres.iter());
    spheres.extend(joint_spheres.iter());
    spheres.extend(synapse_spheres.iter());
    spheres.extend(trigger_spheres.iter());

    spheres
}
