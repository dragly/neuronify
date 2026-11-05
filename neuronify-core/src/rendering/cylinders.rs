use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::{Entity, EntityRef};

use crate::components::*;
use crate::constants::RESTING_POTENTIAL;
use crate::rendering::colors::{terminal_color, Palette};
use crate::rendering::gpu_types::{CylinderData, Sphere};
use crate::rendering::paths::ConnectionPath;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::shown_voltage;
use crate::simulation::model::{
    drawn_radius, soma_end_radius, AXON_DIAMETER, INTERNODE_DIAMETER, MYELIN_G_RATIO,
};

const MIN_GROWN_LENGTH: f32 = 1e-4;
const SHEATH_RADIUS: f32 = 0.26;
pub const NODE_GAP: f32 = 0.5;

pub struct Sheath {
    pub start: Vec3,
    pub end: Vec3,
    pub start_color: Vec3,
    pub end_color: Vec3,
}

pub fn axon_radius(display: &NeuronDisplay) -> f32 {
    drawn_radius(AXON_DIAMETER, display)
}

pub fn sheath_radius(display: &NeuronDisplay) -> f32 {
    match display.true_proportions {
        true => drawn_radius(INTERNODE_DIAMETER / MYELIN_G_RATIO, display),
        false => SHEATH_RADIUS,
    }
}

fn compartment_color(
    entity: EntityRef,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Option<Vec3> {
    let neuron_type = &*entity.get::<&NeuronType>()?;
    entity.get::<&Compartment>()?;
    let voltage = shown_voltage(&*entity.get::<&NodeState>()?, display);
    Some(match entity.has::<Dendrite>() {
        true => palette.dendrite(neuron_type, voltage),
        false => palette.compartment(neuron_type, voltage),
    })
}

fn inside_color(
    from: EntityRef,
    to: EntityRef,
    voltage: f64,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Option<Vec3> {
    let neuron_type = *from
        .get::<&NeuronType>()
        .or_else(|| to.get::<&NeuronType>())?;
    let voltage = match display.voltage {
        true => voltage,
        false => RESTING_POTENTIAL,
    };
    Some(match from.has::<Dendrite>() || to.has::<Dendrite>() {
        true => palette.dendrite(&neuron_type, voltage),
        false => palette.compartment(&neuron_type, voltage),
    })
}

fn soma_end_color(
    soma: EntityRef,
    compartment: EntityRef,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Option<Vec3> {
    soma.get::<&LeakyNeuron>()?;
    Some(palette.compartment(
        &*compartment.get::<&NeuronType>()?,
        shown_voltage(&*compartment.get::<&NodeState>()?, display),
    ))
}

fn cable(
    palette: &Palette,
    (start, start_radius, from): (Vec3, f32, Option<Vec3>),
    (end, end_radius, to): (Vec3, f32, Option<Vec3>),
) -> CylinderData {
    CylinderData {
        start,
        start_radius,
        end,
        end_radius,
        start_color: from.or(to).unwrap_or(palette.neutral),
        end_color: to.or(from).unwrap_or(palette.neutral),
        _padding: Default::default(),
    }
}

/// Radius of every compartment, the mean of the cables that meet there.
///
/// Spheres and cylinders both use these, so that a chain of compartments
/// bends smoothly at each joint.
pub fn compartment_radii(world: &hecs::World) -> HashMap<Entity, f32> {
    let display = NeuronDisplay::of(world);
    let mut sums: HashMap<Entity, (f32, u32)> = world
        .query::<()>()
        .with::<&Compartment>()
        .iter()
        .map(|(entity, _)| (entity, (0.0, 0)))
        .collect();
    for (_, (connection, cable)) in world.query::<(&Connection, &Cable)>().iter() {
        if connection.directional {
            continue;
        }
        let radius = drawn_radius(cable.diameter, &display);
        for end in [connection.from, connection.to] {
            if let Some((sum, count)) = sums.get_mut(&end) {
                *sum += radius;
                *count += 1;
            }
        }
    }
    sums.into_iter()
        .map(|(entity, (sum, count))| match count {
            0 => (entity, axon_radius(&display)),
            _ => (entity, sum / count as f32),
        })
        .collect()
}

/// Tapered tubes for the cable segments between compartments, from a soma to
/// its first compartment, and along the paths of all other connections.
pub fn collect_cylinders(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
) -> Vec<CylinderData> {
    let display = NeuronDisplay::of(world);
    let mut cylinders = Vec::new();
    for (_, (connection, segment, state)) in world
        .query::<(&Connection, &Cable, Option<&CableState>)>()
        .iter()
        .filter(|(_, (connection, ..))| !connection.directional)
    {
        let root = soma_end_radius(segment.diameter, &display);
        let radius = |entity: Entity| radii.get(&entity).copied().unwrap_or(root);
        let (Ok(from), Ok(to)) = (world.entity(connection.from), world.entity(connection.to))
        else {
            continue;
        };
        let (Some(start), Some(end)) = (from.get::<&Position>(), to.get::<&Position>()) else {
            continue;
        };
        let laid = cable(
            palette,
            (
                start.position,
                radius(connection.from),
                compartment_color(from, &display, palette)
                    .or_else(|| soma_end_color(from, to, &display, palette)),
            ),
            (
                end.position,
                radius(connection.to),
                compartment_color(to, &display, palette),
            ),
        );
        let inner = state.map_or(&[][..], |state| &state.voltage[..]);
        if inner.is_empty() {
            cylinders.push(laid);
            continue;
        }
        let pieces = inner.len() + 1;
        let joint = |k: usize| {
            let along = k as f32 / pieces as f32;
            let color = match k {
                0 => laid.start_color,
                k if k == pieces => laid.end_color,
                k => inside_color(from, to, inner[k - 1], &display, palette)
                    .unwrap_or(laid.start_color.lerp(laid.end_color, along)),
            };
            let radius = laid.start_radius + (laid.end_radius - laid.start_radius) * along;
            (laid.start.lerp(laid.end, along), radius, color)
        };
        for k in 0..pieces {
            let ((start, start_radius, start_color), (end, end_radius, end_color)) =
                (joint(k), joint(k + 1));
            cylinders.push(CylinderData {
                start,
                start_radius,
                end,
                end_radius,
                start_color,
                end_color,
                _padding: Default::default(),
            });
        }
    }
    let sheath_radius = sheath_radius(&display);
    cylinders.extend(
        sheaths(world, palette)
            .into_iter()
            .map(|sheath| CylinderData {
                start: sheath.start,
                start_radius: sheath_radius,
                end: sheath.end,
                end_radius: sheath_radius,
                start_color: sheath.start_color,
                end_color: sheath.end_color,
                _padding: Default::default(),
            }),
    );
    cylinders.reserve(paths.values().map(|path| path.points.len()).sum());
    for path in paths.values() {
        let segments = path.points.len().saturating_sub(1);
        let color_at = |i: usize| {
            path.start_color
                .lerp(path.end_color, i as f32 / segments as f32)
        };
        for (i, w) in path.points.windows(2).enumerate() {
            cylinders.push(CylinderData {
                start: w[0],
                start_radius: path.radius,
                end: w[1],
                end_radius: path.radius,
                start_color: color_at(i),
                end_color: color_at(i + 1),
                _padding: Default::default(),
            });
        }
        // The bar across the end of an inhibitory synapse, as in ⊣
        if let (true, Some(&end)) = (path.directional && path.inhibitory, path.points.last()) {
            let direction = path.terminal_direction;
            cylinders.push(CylinderData {
                start: end - direction * 0.12,
                start_radius: 1.2 * path.terminal_radius,
                end: end + direction * 0.12,
                end_radius: 1.2 * path.terminal_radius,
                start_color: terminal_color(),
                end_color: terminal_color(),
                _padding: Default::default(),
            });
        }
    }
    cylinders
}

pub fn sheaths(world: &hecs::World, palette: &Palette) -> Vec<Sheath> {
    let display = NeuronDisplay::of(world);
    world
        .query::<&Connection>()
        .with::<&Myelin>()
        .iter()
        .filter_map(|(_, connection)| {
            let from = world.entity(connection.from).ok()?;
            let to = world.entity(connection.to).ok()?;
            let start = from.get::<&Position>()?.position;
            let end = to.get::<&Position>()?.position;
            let along = (end - start).normalize_or_zero();
            let to_color = compartment_color(to, &display, palette).unwrap_or(palette.neutral);
            let from_color = compartment_color(from, &display, palette)
                .or_else(|| soma_end_color(from, to, &display, palette))
                .unwrap_or(to_color);
            (start.distance(end) > 2.0 * NODE_GAP).then(|| Sheath {
                start: start + along * NODE_GAP,
                end: end - along * NODE_GAP,
                start_color: palette.sheath(from_color),
                end_color: palette.sheath(to_color),
            })
        })
        .collect()
}

pub fn growing_axon(
    world: &hecs::World,
    from: Entity,
    start: Vec3,
    end: Vec3,
    palette: &Palette,
) -> Option<Scene> {
    if start.distance(end) < MIN_GROWN_LENGTH {
        return None;
    }
    let source = world.entity(from).ok()?;
    let neuron_type = *source.get::<&NeuronType>()?;
    let tip = palette.compartment(&neuron_type, RESTING_POTENTIAL);
    let display = NeuronDisplay::of(world);
    let end_radius = axon_radius(&display);
    Some(Scene {
        spheres: vec![Sphere {
            position: end,
            color: tip,
            radius: end_radius,
            _padding: Default::default(),
        }],
        cylinders: vec![cable(
            palette,
            (
                start,
                match source.has::<Compartment>() {
                    true => end_radius,
                    false => soma_end_radius(AXON_DIAMETER, &display),
                },
                compartment_color(source, &display, palette),
            ),
            (end, end_radius, Some(tip)),
        )],
        rings: Vec::new(),
        transmitter: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::MICROMETRES_PER_UNIT;
    use crate::elements::{spawn_cable, spawn_compartment, spawn_internode};
    use crate::rendering::colors::Theme;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::spheres::{collect_spheres, SPINES_PER_CABLE};

    #[test]
    fn a_growing_axon_looks_like_the_cable_it_lays() {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::new(2.0, 0.0, 9.0),
            },
            LeakyNeuron::default(),
        ));
        let soma = Vec3::ZERO;
        let source = world.spawn((
            Position { position: soma },
            LeakyNeuron::default(),
            NeuronType::Inhibitory,
        ));
        let mut from = source;
        let mut start = soma;
        for step in 1..=4 {
            let tip = Vec3::new(0.5, 0.0, 2.5) * step as f32;
            let growing = growing_axon(&world, from, start, tip, &palette).unwrap();
            let compartment = spawn_compartment(&mut world, tip, NeuronType::Inhibitory);
            spawn_cable(&mut world, from, compartment, false);
            let radii = compartment_radii(&world);
            let paths = connection_paths(&world, &radii, &palette);
            let cylinders = collect_cylinders(&world, &radii, &paths, &palette);
            let laid = cylinders
                .iter()
                .find(|cylinder| cylinder.end == tip)
                .unwrap();
            let [preview] = growing.cylinders[..] else {
                panic!("expected one cylinder, got {}", growing.cylinders.len());
            };
            assert_eq!(
                (preview.start, preview.start_color, preview.start_radius),
                (laid.start, laid.start_color, laid.start_radius)
            );
            assert_eq!(
                (preview.end, preview.end_color, preview.end_radius),
                (laid.end, laid.end_color, laid.end_radius)
            );
            let spheres = collect_spheres(&world, &radii, &paths, &palette, 0.0);
            let joint = spheres
                .iter()
                .find(|sphere| sphere.position == tip)
                .unwrap();
            let bead = growing.spheres[0];
            assert_eq!((bead.radius, bead.color), (joint.radius, joint.color));
            world.get::<&mut NodeState>(compartment).unwrap().voltage = -65.0 + 25.0 * step as f64;
            from = compartment;
            start = tip;
        }
    }

    fn bar_of_inhibitory_ending(distance_to_soma: f32) -> Option<CylinderData> {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        let soma = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            LeakyNeuron::default(),
        ));
        let tip = Vec3::new(distance_to_soma, 0.0, 0.0);
        let tip = spawn_compartment(&mut world, tip, NeuronType::Inhibitory);
        spawn_cable(&mut world, tip, soma, true);
        let radii = compartment_radii(&world);
        let paths = connection_paths(&world, &radii, &palette);
        collect_cylinders(&world, &radii, &paths, &palette)
            .into_iter()
            .find(|cylinder| cylinder.start_color == terminal_color())
    }

    #[test]
    fn an_inhibitory_axon_ending_keeps_its_bar_as_its_tip_sways() {
        for distance in [1.9, 1.99, 2.01, 2.1] {
            let bar = bar_of_inhibitory_ending(distance)
                .unwrap_or_else(|| panic!("no bar with the tip {distance} from the soma"));
            let middle = (bar.start + bar.end) / 2.0;
            assert!((middle.x - 2.0).abs() < 0.11, "bar at {middle}");
            assert!((bar.end - bar.start)
                .normalize()
                .abs_diff_eq(-Vec3::X, 1e-5));
        }
    }

    #[test]
    fn an_axon_that_has_not_grown_yet_is_not_drawn() {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        let source = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            LeakyNeuron::default(),
            NeuronType::Excitatory,
        ));
        assert!(growing_axon(&world, source, Vec3::ZERO, Vec3::ZERO, &palette).is_none());
    }

    fn neuron_with_a_dendrite_and_an_axon(
        world: &mut hecs::World,
        neuron_type: NeuronType,
    ) -> (Entity, Vec<Entity>, Vec<Entity>) {
        let soma = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            LeakyNeuron::default(),
            neuron_type,
        ));
        let mut grow = |direction: Vec3, dendrite: bool| {
            let mut previous = soma;
            (1..=4)
                .map(|step| {
                    let compartment =
                        spawn_compartment(world, direction * 2.0 * step as f32, neuron_type);
                    if dendrite {
                        world.insert_one(compartment, Dendrite).unwrap();
                    }
                    spawn_cable(world, previous, compartment, false);
                    previous = compartment;
                    compartment
                })
                .collect::<Vec<Entity>>()
        };
        let dendrite = grow(Vec3::X, true);
        let axon = grow(-Vec3::X, false);
        (soma, dendrite, axon)
    }

    fn drawn(world: &hecs::World) -> Vec<CylinderData> {
        let palette = Theme::Light.palette();
        let radii = compartment_radii(world);
        let paths = connection_paths(world, &radii, &palette);
        collect_cylinders(world, &radii, &paths, &palette)
    }

    #[test]
    fn axons_are_thin_and_even_while_dendrites_taper() {
        let mut world = hecs::World::new();
        let (_, dendrite, axon) =
            neuron_with_a_dendrite_and_an_axon(&mut world, NeuronType::Excitatory);
        let radii = compartment_radii(&world);
        let axon_radius = axon_radius(&NeuronDisplay::default());
        assert!(axon
            .iter()
            .all(|compartment| radii[compartment] == axon_radius));
        assert!(radii[&dendrite[0]] > radii[&dendrite[3]]);
        assert!(radii[&dendrite[3]] > axon_radius);
    }

    #[test]
    fn true_proportions_draw_cables_at_their_real_width() {
        let mut world = hecs::World::new();
        let (soma, dendrite, axon) =
            neuron_with_a_dendrite_and_an_axon(&mut world, NeuronType::Excitatory);
        let shown = compartment_radii(&world);
        NeuronDisplay {
            true_proportions: true,
            ..Default::default()
        }
        .set_in(&mut world);
        let real = compartment_radii(&world);
        world
            .insert_one(soma, NodeState::at(RESTING_POTENTIAL))
            .unwrap();
        let diameter = |entity| real[&entity] * 2.0 * MICROMETRES_PER_UNIT as f32;
        assert!((diameter(axon[1]) - AXON_DIAMETER as f32).abs() < 1e-6);
        assert!(diameter(dendrite[0]) > 2.0 && diameter(dendrite[0]) < 3.5);
        assert!(real[&dendrite[0]] < shown[&dendrite[0]] / 5.0);
        let palette = Theme::Light.palette();
        let paths = connection_paths(&world, &real, &palette);
        let somas: Vec<f32> = collect_spheres(&world, &real, &paths, &palette, 0.0)
            .into_iter()
            .filter(|sphere| sphere.position == Vec3::ZERO)
            .map(|sphere| sphere.radius)
            .collect();
        assert_eq!(somas, [0.2]);
    }

    #[test]
    fn only_the_dendrites_of_excitatory_neurons_have_spines() {
        let palette = Theme::Light.palette();
        let spheres = |neuron_type| {
            let mut world = hecs::World::new();
            neuron_with_a_dendrite_and_an_axon(&mut world, neuron_type);
            let radii = compartment_radii(&world);
            let paths = connection_paths(&world, &radii, &palette);
            collect_spheres(&world, &radii, &paths, &palette, 0.0).len()
        };
        assert_eq!(
            spheres(NeuronType::Excitatory),
            spheres(NeuronType::Inhibitory) + 3 * SPINES_PER_CABLE
        );
    }

    #[test]
    fn a_dendrite_fades_from_the_colour_of_its_soma() {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        let (_, dendrite, _) =
            neuron_with_a_dendrite_and_an_axon(&mut world, NeuronType::Excitatory);
        let first = world.get::<&Position>(dendrite[0]).unwrap().position;
        let root = drawn(&world)
            .into_iter()
            .find(|cylinder| cylinder.start == Vec3::ZERO && cylinder.end == first)
            .unwrap();
        let rest = RESTING_POTENTIAL;
        let excitatory = NeuronType::Excitatory;
        assert_eq!(root.start_color, palette.compartment(&excitatory, rest));
        assert_eq!(root.end_color, palette.dendrite(&excitatory, rest));
    }

    #[test]
    fn myelinated_cables_are_sheathed_but_bare_at_the_nodes_and_light_up() {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        let node = |world: &mut hecs::World, z: f32| {
            spawn_compartment(world, Vec3::new(0.0, 0.0, z), NeuronType::Excitatory)
        };
        let (first, second, third) = (
            node(&mut world, 0.0),
            node(&mut world, 8.0),
            node(&mut world, 10.0),
        );
        spawn_internode(&mut world, first, second);
        spawn_cable(&mut world, second, third, false);
        world.get::<&mut NodeState>(second).unwrap().voltage = 30.0;
        let radii = compartment_radii(&world);
        let paths = connection_paths(&world, &radii, &palette);
        let display = NeuronDisplay::default();
        let sheaths: Vec<CylinderData> = collect_cylinders(&world, &radii, &paths, &palette)
            .into_iter()
            .filter(|cylinder| cylinder.start_radius == sheath_radius(&display))
            .collect();
        let [sheath] = sheaths[..] else {
            panic!("expected one sheath, got {}", sheaths.len());
        };
        assert_eq!(sheath.start, Vec3::new(0.0, 0.0, NODE_GAP));
        assert_eq!(sheath.end, Vec3::new(0.0, 0.0, 8.0 - NODE_GAP));
        assert!(sheath.start_radius > radii[&second]);
        assert!(sheath.end_color.max_element() > sheath.start_color.max_element());
        let caps = collect_spheres(&world, &radii, &paths, &palette, 0.0)
            .into_iter()
            .filter(|sphere| sphere.radius == sheath_radius(&display))
            .count();
        assert_eq!(caps, 2);
    }

    #[test]
    fn a_joint_is_as_wide_as_the_mean_of_its_cables() {
        let mut world = hecs::World::new();
        let node = |world: &mut hecs::World, x: f32| {
            spawn_compartment(world, Vec3::new(x, 0.0, 0.0), NeuronType::Excitatory)
        };
        let (first, middle, last) = (
            node(&mut world, 0.0),
            node(&mut world, 2.0),
            node(&mut world, 4.0),
        );
        let thick = spawn_cable(&mut world, first, middle, false);
        spawn_cable(&mut world, middle, last, false);
        world.get::<&mut Cable>(thick).unwrap().diameter = 4.0;
        let display = NeuronDisplay::default();
        let radii = compartment_radii(&world);
        assert_eq!(radii[&first], drawn_radius(4.0, &display));
        assert_eq!(radii[&last], axon_radius(&display));
        let mean = (drawn_radius(4.0, &display) + axon_radius(&display)) / 2.0;
        assert!((radii[&middle] - mean).abs() < 1e-6);
    }

    #[test]
    fn a_cable_lights_up_inside_where_its_voltage_is_high() {
        let palette = Theme::Light.palette();
        let mut world = hecs::World::new();
        let node = |world: &mut hecs::World, x: f32| {
            spawn_compartment(world, Vec3::new(x, 0.0, 0.0), NeuronType::Excitatory)
        };
        let (first, second) = (node(&mut world, 0.0), node(&mut world, 3.0));
        let laid = spawn_cable(&mut world, first, second, false);
        world
            .insert_one(
                laid,
                CableState {
                    voltage: vec![RESTING_POTENTIAL, 30.0],
                    gates: Vec::new(),
                },
            )
            .unwrap();
        let pieces = drawn(&world);
        let lit = palette.compartment(&NeuronType::Excitatory, 30.0);
        let rest = palette.compartment(&NeuronType::Excitatory, RESTING_POTENTIAL);
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[1].start, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!((pieces[1].start_color, pieces[1].end_color), (rest, lit));
        assert_eq!((pieces[2].start_color, pieces[2].end_color), (lit, rest));
    }

    #[test]
    fn an_axon_leaves_its_soma_thick_and_narrows_to_its_drawn_width() {
        let mut world = hecs::World::new();
        let (_, _, axon) = neuron_with_a_dendrite_and_an_axon(&mut world, NeuronType::Excitatory);
        let first = world.get::<&Position>(axon[0]).unwrap().position;
        let root = drawn(&world)
            .into_iter()
            .find(|cylinder| cylinder.start == Vec3::ZERO && cylinder.end == first)
            .unwrap();
        assert_eq!((root.start_radius, root.end_radius), (0.30, 0.11));
    }
}
