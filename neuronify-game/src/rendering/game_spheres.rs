use neuronify_core::rendering::colors::{
    base, blue, mantle, neurocolor, orange, red, srgb, yellow,
};
use neuronify_core::rendering::gpu_types::Sphere;
use neuronify_core::{
    Compartment, CurrentClamp, GeneratorDynamics, Inhibitory, LeakyDynamics, LeakyNeuron,
    NeuronType, Position, COMPARTMENT_SPHERE_SCALE, NODE_RADIUS,
};

use crate::components::*;
use crate::rendering::colors::{
    activity_sensor_color, chemical_sensor_color, glial_color, membrane_color, player1_color,
    player2_color, touch_sensor_color,
};

pub fn collect_game_spheres(world: &hecs::World) -> Vec<Sphere> {
    let mut spheres = Vec::new();

    let lif_neuron_spheres: Vec<Sphere> = world
        .query::<(&LeakyNeuron, &LeakyDynamics, &Position)>()
        .iter()
        .map(|(entity, (neuron, dynamics, position))| {
            let value = ((dynamics.voltage - neuron.resting_potential)
                / (neuron.threshold - neuron.resting_potential))
                .clamp(0.0, 1.0) as f32;
            let is_inhibitory = world.get::<&Inhibitory>(entity).is_ok();
            let mut color = if let Ok(sensor) = world.get::<&SensorNeuron>(entity) {
                let base_color = match sensor.sensor_type {
                    SensorType::Activity => activity_sensor_color(),
                    SensorType::Chemical => chemical_sensor_color(),
                    SensorType::Touch => touch_sensor_color(),
                };
                value * base_color + (1.0 - value) * (base_color * 0.4)
            } else if is_inhibitory {
                value * mantle() + (1.0 - value) * red()
            } else {
                value * base() + (1.0 - value) * blue()
            };

            // Depolarization block visual
            if let Ok(block) = world.get::<&DepolarizationBlock>(entity) {
                if block.blocked {
                    color = srgb(80, 20, 20);
                }
            }

            // Ownership tinting
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let pc = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.6 + pc * 0.4;
            }

            // Energy dimming / dormancy
            if let Ok(metab) = world.get::<&MetabolicState>(entity) {
                let effectiveness = (metab.energy / metab.max_energy).clamp(0.0, 1.0) as f32;
                if effectiveness < 0.05 {
                    // Fully dormant — dark gray
                    color = srgb(40, 40, 50);
                } else {
                    color *= effectiveness.max(0.3);
                }
            }

            // Origin neuron is slightly larger
            let radius = if world.get::<&OriginNeuron>(entity).is_ok() {
                NODE_RADIUS * 1.5
            } else {
                NODE_RADIUS
            };

            Sphere {
                position: position.position,
                color,
                radius,
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
            let mut color = neurocolor(neuron_type, value);
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let pc = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.7 + pc * 0.3;
            }
            Sphere {
                position: position.position,
                color,
                radius: COMPARTMENT_SPHERE_SCALE * NODE_RADIUS,
                _padding: Default::default(),
            }
        })
        .collect();

    let membrane_spheres: Vec<Sphere> = world
        .query::<(&MembraneSegment, &Position)>()
        .iter()
        .map(|(entity, (_, position))| {
            let mut color = membrane_color();
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let pc = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.7 + pc * 0.3;
            }
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 1.2,
                _padding: Default::default(),
            }
        })
        .collect();

    let glial_spheres: Vec<Sphere> = world
        .query::<(&GlialCell, &Position)>()
        .iter()
        .map(|(entity, (glial, position))| {
            let atp_factor = (glial.atp_stored / glial.max_atp).clamp(0.3, 1.0) as f32;
            let mut color = glial_color() * atp_factor;
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let pc = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.6 + pc * 0.4;
            }
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 1.3,
                _padding: Default::default(),
            }
        })
        .collect();

    spheres.extend(lif_neuron_spheres.iter());
    spheres.extend(current_clamp_spheres.iter());
    spheres.extend(generator_spheres.iter());
    spheres.extend(compartment_spheres.iter());
    spheres.extend(membrane_spheres.iter());
    spheres.extend(glial_spheres.iter());

    spheres
}
