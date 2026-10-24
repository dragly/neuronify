use std::collections::{BTreeMap, HashMap};

use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;

/// How long a soma keeps glowing after it fires, in simulated seconds.
const FLASH_DURATION: f64 = 0.01;
/// How much brighter than white a flash starts, so that only flashes bloom.
const FLASH_BRIGHTNESS: f32 = 3.0;

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
        .map(|(entity, (neuron, dynamics, position))| {
            let value = ((dynamics.voltage - neuron.resting_potential)
                / (neuron.threshold - neuron.resting_potential))
                .clamp(0.0, 1.0) as f32;
            let neuron_type = if world.get::<&Inhibitory>(entity).is_ok() {
                NeuronType::Inhibitory
            } else {
                NeuronType::Excitatory
            };
            let color = palette.soma(&neuron_type, value);
            let remaining = (1.0 - dynamics.time_since_fire / FLASH_DURATION).max(0.0) as f32;
            let color = color.lerp(palette.flash * FLASH_BRIGHTNESS, remaining * remaining);
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS,
                _padding: Default::default(),
            }
        })
        .collect();

    let current_clamp_spheres: Vec<Sphere> = world
        .query::<&Position>()
        .with::<&CurrentClamp>()
        .iter()
        .map(|(_, position)| Sphere {
            position: position.position,
            color: yellow(),
            radius: NODE_RADIUS,
            _padding: Default::default(),
        })
        .collect();

    let generator_spheres: Vec<Sphere> = world
        .query::<(&Position, &GeneratorDynamics)>()
        .iter()
        .map(|(_, (position, _))| Sphere {
            position: position.position,
            color: orange(),
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

    let synapse_spheres: Vec<Sphere> = paths
        .values()
        .filter(|path| path.directional)
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
    spheres.extend(current_clamp_spheres.iter());
    spheres.extend(generator_spheres.iter());
    spheres.extend(compartment_spheres.iter());
    spheres.extend(joint_spheres.iter());
    spheres.extend(synapse_spheres.iter());
    spheres.extend(trigger_spheres.iter());

    spheres
}
