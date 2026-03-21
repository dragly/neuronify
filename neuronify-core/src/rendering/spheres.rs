use glam::Vec3;

use crate::components::*;
use crate::constants::*;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::Sphere;

pub fn collect_spheres(world: &hecs::World) -> Vec<Sphere> {
    let mut spheres = Vec::new();

    let lif_neuron_spheres: Vec<Sphere> = world
        .query::<(&LeakyNeuron, &LeakyDynamics, &Position)>()
        .iter()
        .map(|(entity, (neuron, dynamics, position))| {
            let value = ((dynamics.voltage - neuron.resting_potential)
                / (neuron.threshold - neuron.resting_potential))
                .clamp(0.0, 1.0) as f32;
            let is_inhibitory = world.get::<&Inhibitory>(entity).is_ok();
            let color = if is_inhibitory {
                value * mantle() + (1.0 - value) * red()
            } else {
                value * base() + (1.0 - value) * blue()
            };
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
        .map(|(_, (compartment, position, neuron_type))| {
            let value = ((compartment.voltage + 50.0) / 200.0) as f32;
            Sphere {
                position: position.position,
                color: neurocolor(neuron_type, value),
                radius: COMPARTMENT_SPHERE_SCALE * NODE_RADIUS,
                _padding: Default::default(),
            }
        })
        .collect();

    let trigger_spheres: Vec<Sphere> = world
        .query::<(&CurrentSynapse, &Connection)>()
        .iter()
        .flat_map(|(_, (synapse, connection))| {
            let start = world
                .get::<&Position>(connection.from)
                .map(|p| p.position)
                .unwrap_or(Vec3::ZERO);
            let end = world
                .get::<&Position>(connection.to)
                .map(|p| p.position)
                .unwrap_or(Vec3::ZERO);
            let diff = end - start;
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
                        position: start + diff * progress,
                        color: crust(),
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
    spheres.extend(trigger_spheres.iter());

    spheres
}
