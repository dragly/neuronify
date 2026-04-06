use hecs::Entity;
use neuronify_core::rendering::colors::{
    base, blue, mantle, neurocolor, orange, red, srgb, yellow,
};
use neuronify_core::rendering::gpu_types::Sphere;
use neuronify_core::{
    Compartment, CurrentClamp, GeneratorDynamics, Inhibitory, LeakyDynamics, LeakyNeuron,
    NeuronType, Position, COMPARTMENT_SPHERE_SCALE, NODE_RADIUS,
};

use crate::components::{
    AttackProjectile, GlialCell, GlialProcess, GlucosePacket, GrowthCone, LactatePacket,
    MaturingNeuron, MetabolicState, Neuroblast, OriginNeuron, Ownership, ProducibleCell, TCellUnit,
};
use crate::rendering::colors::{glial_color, player1_color};
use crate::tools::GameTool;

pub fn collect_game_spheres(world: &hecs::World, funds_blocked_entity: Option<Entity>) -> Vec<Sphere> {
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

            // Origin neuron: gold/amber so the player can spot their base.
            if world.get::<&OriginNeuron>(entity).is_ok() {
                color = srgb(220, 160, 30);
            } else if world.get::<&Ownership>(entity).is_ok() {
                // Ownership tinting for other player neurons.
                color = color * 0.6 + player1_color() * 0.4;
            }

            // Energy dimming / dormancy + red stress tint when under attack
            if let Ok(metab) = world.get::<&MetabolicState>(entity) {
                let effectiveness = (metab.energy / metab.max_energy).clamp(0.0, 1.0) as f32;
                if effectiveness < 0.05 {
                    color = srgb(40, 40, 50);
                } else {
                    color *= effectiveness.max(0.3);
                    // Below 60% energy: shift toward red so the player can see the neuron is dying
                    if effectiveness < 0.6 {
                        let stress = (0.6 - effectiveness) / 0.6; // 0 at 60%, 1.0 at 0%
                        color = color * (1.0 - stress * 0.6)
                            + glam::Vec3::new(0.8, 0.05, 0.05) * (stress * 0.6);
                    }
                }
            }

            // Action potential flash: bright yellow-white burst lasting 0.2 sim-seconds.
            const SOMA_FLASH_DURATION: f64 = 0.2;
            if dynamics.time_since_fire < SOMA_FLASH_DURATION {
                let flash = (1.0 - dynamics.time_since_fire as f32 / SOMA_FLASH_DURATION as f32)
                    .max(0.0);
                color = color * (1.0 - flash * 0.85) + glam::Vec3::new(1.0, 1.0, 0.55) * (flash * 0.85);
            }

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
        .filter(|(entity, _)| world.get::<&GlialProcess>(*entity).is_err())
        .map(|(entity, (compartment, position, neuron_type))| {
            let color = if funds_blocked_entity == Some(entity) {
                red()
            } else {
                let value = ((compartment.voltage + 50.0) / 200.0) as f32;
                let mut c = neurocolor(neuron_type, value);
                if world.get::<&Ownership>(entity).is_ok() {
                    c = c * 0.7 + player1_color() * 0.3;
                }
                c
            };
            Sphere {
                position: position.position,
                color,
                radius: COMPARTMENT_SPHERE_SCALE * NODE_RADIUS,
                _padding: Default::default(),
            }
        })
        .collect();

    // Glial process compartments — tinted green to distinguish from neuronal compartments
    let glial_process_spheres: Vec<Sphere> = world
        .query::<(&Compartment, &Position, &GlialProcess)>()
        .iter()
        .map(|(entity, (_, position, _))| {
            let color = if funds_blocked_entity == Some(entity) {
                red()
            } else {
                let mut c = glial_color() * 0.6;
                if world.get::<&Ownership>(entity).is_ok() {
                    c = c * 0.7 + player1_color() * 0.3;
                }
                c
            };
            Sphere {
                position: position.position,
                color,
                radius: COMPARTMENT_SPHERE_SCALE * NODE_RADIUS,
                _padding: Default::default(),
            }
        })
        .collect();

    let glial_spheres: Vec<Sphere> = world
        .query::<(&GlialCell, &Position)>()
        .iter()
        .map(|(entity, (glial, position))| {
            let glucose_factor = (glial.glucose_stored / glial.max_glucose).clamp(0.3, 1.0) as f32;
            let mut color = glial_color() * glucose_factor;
            if world.get::<&Ownership>(entity).is_ok() {
                color = color * 0.6 + player1_color() * 0.4;
            }
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 1.3,
                _padding: Default::default(),
            }
        })
        .collect();

    // Glucose packets — small amber spheres traveling along glial processes (vessel → glial)
    let glucose_packet_spheres: Vec<Sphere> = world
        .query::<(&GlucosePacket, &Position)>()
        .iter()
        .map(|(_, (_, position))| {
            let color = srgb(220, 160, 30);
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 0.4,
                _padding: Default::default(),
            }
        })
        .collect();

    // Lactate packets — small green spheres flying from glial cells to neurons
    let lactate_packet_spheres: Vec<Sphere> = world
        .query::<(&LactatePacket, &Position)>()
        .iter()
        .map(|(_, (_, position))| {
            let color = srgb(130, 220, 90);
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 0.35,
                _padding: Default::default(),
            }
        })
        .collect();

    // AttackProjectile — small fast-moving spheres, color from component
    let attack_projectile_spheres: Vec<Sphere> = world
        .query::<(&AttackProjectile, &Position)>()
        .iter()
        .map(|(_, (proj, position))| Sphere {
            position: position.position,
            color: proj.color,
            radius: proj.radius,
            _padding: Default::default(),
        })
        .collect();

    // TCellUnit — lime green sphere, radius 1.2
    let tcell_spheres: Vec<Sphere> = world
        .query::<(&TCellUnit, &Position)>()
        .iter()
        .map(|(_, (_, position))| Sphere {
            position: position.position,
            color: srgb(80, 255, 60),
            radius: 1.2,
            _padding: Default::default(),
        })
        .collect();

    // Neuroblasts — small dim spheres migrating through the soup
    let neuroblast_spheres: Vec<Sphere> = world
        .query::<(&Neuroblast, &Position)>()
        .iter()
        .map(|(_, (nb, position))| {
            let color = match nb.cell_type {
                ProducibleCell::ExcitatoryNeuroblast => srgb(80, 130, 255),
                ProducibleCell::InhibitoryNeuroblast => srgb(255, 80, 80),
            };
            Sphere {
                position: position.position,
                color,
                radius: NODE_RADIUS * 0.8,
                _padding: Default::default(),
            }
        })
        .collect();

    // Maturing neurons — grow from neuroblast size to full size over maturation time
    let maturing_spheres: Vec<Sphere> = world
        .query::<(&MaturingNeuron, &Position)>()
        .iter()
        .map(|(_, (maturing, position))| {
            let t = (maturing.timer / maturing.duration).clamp(0.0, 1.0);
            let radius = NODE_RADIUS * (0.6 + 0.4 * t);
            let base_color = match maturing.cell_type {
                ProducibleCell::ExcitatoryNeuroblast => blue(),
                ProducibleCell::InhibitoryNeuroblast => red(),
            };
            let color = base_color * (0.4 + 0.6 * t);
            Sphere {
                position: position.position,
                color,
                radius,
                _padding: Default::default(),
            }
        })
        .collect();

    // Growth cones — bright white-yellow sphere indicating an axon under construction.
    let growth_cone_spheres: Vec<Sphere> = world
        .query::<(&GrowthCone, &Position)>()
        .iter()
        .map(|(_, (_, position))| Sphere {
            position: position.position,
            color: srgb(255, 240, 80),
            radius: NODE_RADIUS * 0.5,
            _padding: Default::default(),
        })
        .collect();

    spheres.extend(lif_neuron_spheres.iter());
    spheres.extend(current_clamp_spheres.iter());
    spheres.extend(generator_spheres.iter());
    spheres.extend(compartment_spheres.iter());
    spheres.extend(glial_process_spheres.iter());
    spheres.extend(glial_spheres.iter());
    spheres.extend(glucose_packet_spheres.iter());
    spheres.extend(lactate_packet_spheres.iter());
    spheres.extend(attack_projectile_spheres.iter());
    spheres.extend(tcell_spheres.iter());
    spheres.extend(neuroblast_spheres.iter());
    spheres.extend(maturing_spheres.iter());
    spheres.extend(growth_cone_spheres.iter());

    spheres
}

pub fn collect_placement_preview(
    tool: &GameTool,
    placement_preview: &Option<glam::Vec3>,
    connection_preview_end: Option<glam::Vec3>,
) -> Vec<Sphere> {
    let mut spheres = Vec::new();

    // Ghost sphere for connection tools — sits exactly at the mouse (future node position).
    if let Some(pos) = connection_preview_end {
        let color = match tool {
            GameTool::GlialProcess => glial_color() * 0.5,
            _ => glam::Vec3::new(0.6, 0.6, 0.6),
        };
        spheres.push(Sphere {
            position: pos,
            color,
            radius: COMPARTMENT_SPHERE_SCALE * NODE_RADIUS,
            _padding: Default::default(),
        });
        return spheres;
    }

    if let Some(_position) = placement_preview {
        // No direct placement tools remain — all cells are produced via the origin neuron.
        // Ghost sphere is only shown for connection tools (handled above via connection_preview_end).
    }

    spheres
}
