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
            let mut color = if is_inhibitory {
                value * mantle() + (1.0 - value) * red()
            } else {
                value * base() + (1.0 - value) * blue()
            };

            // Depolarization block visual - override with flickering dark
            if let Ok(block) = world.get::<&DepolarizationBlock>(entity) {
                if block.blocked {
                    color = srgb(80, 20, 20);
                }
            }

            // Ownership tinting
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let player_color = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.6 + player_color * 0.4;
            }

            // Energy dimming
            if let Ok(metab) = world.get::<&MetabolicState>(entity) {
                let energy_factor = (metab.energy / metab.max_energy).clamp(0.3, 1.0) as f32;
                color *= energy_factor;
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
            // Ownership tinting for compartments too
            if let Ok(ownership) = world.get::<&Ownership>(entity) {
                let player_color = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.7 + player_color * 0.3;
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
                let player_color = match ownership.player {
                    PlayerId::Player1 => player1_color(),
                    PlayerId::Player2 => player2_color(),
                };
                color = color * 0.7 + player_color * 0.3;
            }
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 1.2,
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

    let resource_node_spheres: Vec<Sphere> = world
        .query::<(&ResourceNode, &Position)>()
        .iter()
        .map(|(_, (_, position))| Sphere {
            position: position.position,
            color: resource_color(),
            radius: RESOURCE_NODE_VISUAL_RADIUS,
            _padding: Default::default(),
        })
        .collect();

    spheres.extend(lif_neuron_spheres.iter());
    spheres.extend(current_clamp_spheres.iter());
    spheres.extend(generator_spheres.iter());
    spheres.extend(compartment_spheres.iter());
    spheres.extend(membrane_spheres.iter());
    spheres.extend(trigger_spheres.iter());
    spheres.extend(resource_node_spheres.iter());

    spheres
}

pub fn collect_placement_preview(
    tool: &crate::Tool,
    placement_preview: &Option<Vec3>,
) -> Vec<Sphere> {
    let mut spheres = Vec::new();

    if let Some(position) = placement_preview {
        // Ghost sphere for placement preview - semi-transparent
        let ghost_color = match tool {
            crate::Tool::ExcitatoryNeuron => blue(),
            crate::Tool::InhibitoryNeuron => red(),
            crate::Tool::CurrentSource => yellow(),
            crate::Tool::TouchSensor => orange(),
            crate::Tool::RegularSpikeGenerator => orange(),
            crate::Tool::PoissonGenerator => orange(),
            crate::Tool::MembraneSegment => membrane_color(),
            crate::Tool::StaticConnection => crust(),
            crate::Tool::Axon => crust(),
            crate::Tool::Voltmeter => crust(),
            crate::Tool::Select | crate::Tool::Erase | crate::Tool::Stimulate => {
                // No preview for these tools
                return spheres;
            }
        };

        // Make it semi-transparent by blending with white
        let ghost_color = ghost_color * 0.4 + Vec3::new(1.0, 1.0, 1.0) * 0.6;

        spheres.push(Sphere {
            position: *position,
            color: ghost_color,
            radius: NODE_RADIUS,
            _padding: Default::default(),
        });
    }

    spheres
}
