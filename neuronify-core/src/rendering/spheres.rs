use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::measurement::visual_input::VisualInput;
use crate::rendering::colors::*;
use crate::rendering::cylinders::{sheath_radius, sheaths};
use crate::rendering::gpu_types::Sphere;
use crate::rendering::paths::ConnectionPath;
use crate::rendering::shapes::{generator_color, shown_voltage, soma_color};
use crate::simulation::model::{drawn_radius, soma_diameter};

pub const SPINES_PER_CABLE: usize = 4;
const SPINE_HEAD_DIAMETER: Micrometres = Micrometres(0.5);

fn spines(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Vec<Sphere> {
    let mut spines = Vec::new();
    for (cable, connection) in world
        .query::<(Entity, &Connection)>()
        .with::<&Cable>()
        .iter()
        .filter(|(_, connection)| !connection.directional)
    {
        let (Ok(from), Ok(to)) = (world.entity(connection.from), world.entity(connection.to))
        else {
            continue;
        };
        let (Some(neuron_type), true, true) = (
            from.get::<&NeuronType>().map(|neuron_type| *neuron_type),
            from.has::<Dendrite>(),
            to.has::<Dendrite>(),
        ) else {
            continue;
        };
        if neuron_type != NeuronType::Excitatory {
            continue;
        }
        let ends = [from, to].map(|end| {
            let position = end.get::<&Position>().map(|position| position.position);
            let voltage = end
                .get::<&NodeState>()
                .filter(|_| end.has::<Compartment>())
                .map(|state| shown_voltage(&state, display));
            (position, voltage, radii.get(&end.entity()).copied())
        });
        let [(Some(start), Some(start_voltage), Some(start_radius)), (Some(end), Some(end_voltage), Some(end_radius))] =
            ends
        else {
            continue;
        };
        let side = Vec3::Y.cross(end - start).normalize_or_zero();
        let head = drawn_radius(SPINE_HEAD_DIAMETER, display);
        for spine in 0..SPINES_PER_CABLE {
            let fraction = (spine as f32 + 0.5) / SPINES_PER_CABLE as f32;
            let flip = if (cable.id() as usize + spine).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            let thickness = start_radius + (end_radius - start_radius) * fraction;
            let voltage = start_voltage + (end_voltage - start_voltage) * f64::from(fraction);
            spines.push(Sphere {
                position: start.lerp(end, fraction) + side * flip * thickness,
                color: palette.dendrite(&neuron_type, voltage),
                radius: head,
                _padding: Default::default(),
            });
        }
    }
    spines
}

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
            .query::<(Entity, (&LeakyNeuron, &NodeState, &Position))>()
            .iter()
            .map(|(entity, (neuron, state, position))| Sphere {
                position: position.position,
                color: soma_color(world, entity, neuron, state, &display, palette),
                radius: match display.true_proportions {
                    true => drawn_radius(soma_diameter(world, entity), &display),
                    false => NODE_RADIUS,
                },
                _padding: Default::default(),
            }),
    );

    // Touch sensors and current sources are flat pads, see `collect_shapes`
    spheres.extend(
        world
            .query::<(Entity, (&Position, &GeneratorDynamics, Option<&NeuronType>))>()
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
            .query::<(
                Entity,
                (&NodeState, &Position, &NeuronType, Option<&Dendrite>),
            )>()
            .with::<&Compartment>()
            .iter()
            .map(
                |(entity, (compartment, position, neuron_type, dendrite))| Sphere {
                    position: position.position,
                    color: match dendrite {
                        Some(_) => {
                            palette.dendrite(neuron_type, shown_voltage(compartment, &display))
                        }
                        None => {
                            palette.compartment(neuron_type, shown_voltage(compartment, &display))
                        }
                    },
                    radius: radii
                        .get(&entity)
                        .copied()
                        .unwrap_or(COMPARTMENT_SPHERE_SCALE * NODE_RADIUS),
                    _padding: Default::default(),
                },
            ),
    );

    spheres.extend(spines(world, radii, &display, palette));
    let sheath_radius = sheath_radius(&display);
    spheres.extend(sheaths(world, palette).into_iter().flat_map(|sheath| {
        [
            (sheath.start, sheath.start_color),
            (sheath.end, sheath.end_color),
        ]
        .map(|(position, color)| Sphere {
            position,
            color,
            radius: sheath_radius,
            _padding: Default::default(),
        })
    }));
    spheres.reserve(paths.values().map(|path| path.points.len()).sum());
    for path in paths.values() {
        spheres.extend(path_spheres(path));
    }

    spheres
}

pub fn path_spheres(path: &ConnectionPath) -> Vec<Sphere> {
    let segments = path.points.len().saturating_sub(1);
    let joints = path
        .points
        .iter()
        .enumerate()
        .skip(1)
        .take(segments.saturating_sub(1))
        .map(|(i, &position)| Sphere {
            position,
            color: path
                .start_color
                .lerp(path.end_color, i as f32 / segments as f32),
            radius: path.radius,
            _padding: Default::default(),
        });
    let knob = path
        .points
        .last()
        .filter(|_| path.directional && !path.inhibitory)
        .map(|&position| Sphere {
            position,
            color: terminal_color(),
            radius: path.terminal_radius,
            _padding: Default::default(),
        });
    joints.chain(knob).collect()
}
