use std::collections::{BTreeMap, HashMap, HashSet};

use glam::{Vec2, Vec3};
use hecs::{Entity, EntityRef};

use crate::components::*;
use crate::constants::RESTING_POTENTIAL;
use crate::editor::Bead;
use crate::rendering::colors::{terminal_color, Palette};
use crate::rendering::gpu_types::{CylinderData, Sphere};
use crate::rendering::paths::{ending_path, ending_path_to, ConnectionPath};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{shown_voltage, soma_color};
use crate::rendering::spheres::{path_spheres, soma_radius};
use crate::simulation::model::{
    dendrite_diameter, drawn_radius, soma_end_radius, INTERNODE_DIAMETER, MYELIN_G_RATIO,
};

const MIN_GROWN_LENGTH: f32 = 1e-4;
const FILLET_SEGMENTS: usize = 8;
const NARROWEST_FILLET: f32 = 0.15;
const WIDEST_FILLET: f32 = 1.6;
const FILLET_LENGTH_SCALE: Micrometres = Micrometres(400.0);
const FILLET_REACH: f32 = 0.9;
const SHEATH_RADIUS: f32 = 0.26;
pub const NODE_GAP: f32 = 0.5;

pub struct Sheath {
    pub start: Vec3,
    pub end: Vec3,
    pub start_color: Vec3,
    pub end_color: Vec3,
}

pub fn axon_radius(display: &NeuronDisplay, preparation: Preparation) -> f32 {
    drawn_radius(preparation.axon_diameter(), display)
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
    voltage: Millivolts,
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
        .query::<Entity>()
        .with::<&Compartment>()
        .iter()
        .map(|entity| (entity, (0.0, 0)))
        .collect();
    for (connection, cable) in world.query::<(&Connection, &Cable)>().iter() {
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
            0 => (entity, axon_radius(&display, Preparation::of(world))),
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
    for (connection, segment, state) in world
        .query::<(&Connection, &Cable, Option<&CableState>)>()
        .iter()
        .filter(|(connection, ..)| !connection.directional)
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
    cylinders.extend(soma_fillets(world, radii, &display, palette));
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
        cylinders.extend(path_cylinders(path));
    }
    cylinders
}

fn fillet_profile(soma: f32, branch: f32, rounding: f32) -> Vec<Vec2> {
    let center = Vec2::new(
        ((soma + rounding).powi(2) - (branch + rounding).powi(2)).sqrt(),
        branch + rounding,
    );
    let start = (-center.y).atan2(-center.x);
    let end = -std::f32::consts::FRAC_PI_2;
    (0..=FILLET_SEGMENTS)
        .map(|k| {
            let angle = start + (end - start) * k as f32 / FILLET_SEGMENTS as f32;
            center + rounding * Vec2::new(angle.cos(), angle.sin())
        })
        .collect()
}

fn fillet_rounding(soma: f32, branch: f32, length: Micrometres, room: f32) -> Option<f32> {
    if branch >= soma {
        return None;
    }
    let grown = 1.0 - (-(length / FILLET_LENGTH_SCALE)).exp() as f32;
    let wanted = soma * (NARROWEST_FILLET + (WIDEST_FILLET - NARROWEST_FILLET) * grown);
    let reach = FILLET_REACH * room;
    let most = (reach * reach - soma * soma + branch * branch) / (2.0 * (soma - branch));
    (most > 0.0).then(|| wanted.min(most))
}

fn branch_length(
    world: &hecs::World,
    neighbours: &HashMap<Entity, Vec<(Entity, Micrometres)>>,
    soma: Entity,
    first: Entity,
) -> Micrometres {
    let mut seen = HashSet::from([soma, first]);
    let mut stack = vec![first];
    let mut length = Micrometres::ZERO;
    while let Some(node) = stack.pop() {
        for &(next, cable) in neighbours.get(&node).into_iter().flatten() {
            if !world.satisfies::<&LeakyNeuron>(next) && seen.insert(next) {
                length += cable;
                stack.push(next);
            }
        }
    }
    length
}

fn soma_fillets(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Vec<CylinderData> {
    if world.query::<&CompartmentSoma>().iter().next().is_none() {
        return Vec::new();
    }
    let mut neighbours: HashMap<Entity, Vec<(Entity, Micrometres)>> = HashMap::new();
    let mut branches = Vec::new();
    for (connection, cable) in world
        .query::<(&Connection, &Cable)>()
        .iter()
        .filter(|(connection, _)| !connection.directional)
    {
        for (from, to) in [
            (connection.from, connection.to),
            (connection.to, connection.from),
        ] {
            neighbours.entry(from).or_default().push((to, cable.length));
            if world.satisfies::<&CompartmentSoma>(from) && world.satisfies::<&Compartment>(to) {
                branches.push((from, to, cable.length, cable.diameter));
            }
        }
    }
    let mut fillets = Vec::new();
    for (soma, first, first_length, diameter) in branches {
        let (Ok(soma_ref), Ok(first_ref)) = (world.entity(soma), world.entity(first)) else {
            continue;
        };
        let (Some(neuron), Some(state), Some(center), Some(end)) = (
            soma_ref.get::<&LeakyNeuron>(),
            soma_ref.get::<&NodeState>(),
            soma_ref.get::<&Position>(),
            first_ref.get::<&Position>(),
        ) else {
            continue;
        };
        let (center, end) = (center.position, end.position);
        let room = center.distance(end);
        let direction = (end - center).normalize_or_zero();
        let soma_size = soma_radius(world, soma, display);
        let root = soma_end_radius(diameter, display);
        let tip = radii.get(&first).copied().unwrap_or(root);
        let length = first_length + branch_length(world, &neighbours, soma, first);
        let along = |x: f32| (x / room).clamp(0.0, 1.0);
        let Some(rounding) = fillet_rounding(soma_size, root, length, room) else {
            continue;
        };
        let reached = fillet_profile(soma_size, root, rounding)[FILLET_SEGMENTS].x;
        let branch = root + (tip - root) * along(reached);
        let Some(rounding) = fillet_rounding(soma_size, branch, length, room) else {
            continue;
        };
        let profile = fillet_profile(soma_size, branch, rounding);
        let soma_end = soma_end_color(soma_ref, first_ref, display, palette);
        let far_end = compartment_color(first_ref, display, palette);
        let (Some(soma_end), Some(far_end)) = (soma_end, far_end) else {
            continue;
        };
        let body = soma_color(world, soma, &neuron, &state, display, palette);
        let joined = soma_end.lerp(far_end, along(profile[FILLET_SEGMENTS].x));
        let color = |k: usize| body.lerp(joined, k as f32 / FILLET_SEGMENTS as f32);
        fillets.extend(
            profile
                .windows(2)
                .enumerate()
                .map(|(k, pair)| CylinderData {
                    start: center + direction * pair[0].x,
                    start_radius: pair[0].y,
                    end: center + direction * pair[1].x,
                    end_radius: pair[1].y,
                    start_color: color(k),
                    end_color: color(k + 1),
                    _padding: Default::default(),
                }),
        );
    }
    fillets
}

pub fn path_cylinders(path: &ConnectionPath) -> Vec<CylinderData> {
    let segments = path.points.len().saturating_sub(1);
    let color_at = |i: usize| {
        path.start_color
            .lerp(path.end_color, i as f32 / segments as f32)
    };
    let mut cylinders: Vec<CylinderData> = path
        .points
        .windows(2)
        .enumerate()
        .map(|(i, w)| CylinderData {
            start: w[0],
            start_radius: path.radius,
            end: w[1],
            end_radius: path.radius,
            start_color: color_at(i),
            end_color: color_at(i + 1),
            _padding: Default::default(),
        })
        .collect();
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
    cylinders
}

pub fn sheaths(world: &hecs::World, palette: &Palette) -> Vec<Sheath> {
    let display = NeuronDisplay::of(world);
    world
        .query::<(Entity, &Connection)>()
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
    growing(world, from, (start, end), false, palette)
}

pub fn growing_ending(
    world: &hecs::World,
    from: Entity,
    to: Entity,
    palette: &Palette,
) -> Option<Scene> {
    let path = ending_path(world, from, to, palette)?;
    Some(Scene {
        spheres: path_spheres(&path),
        cylinders: path_cylinders(&path),
        rings: Vec::new(),
        transmitter: Vec::new(),
    })
}

pub fn bead_ending(world: &hecs::World, bead: &Bead, palette: &Palette) -> Option<Scene> {
    let path = ending_path_to(world, bead.from, bead.onto, (bead.at, bead.radius), palette)?;
    Some(Scene {
        spheres: path_spheres(&path),
        cylinders: path_cylinders(&path),
        rings: Vec::new(),
        transmitter: Vec::new(),
    })
}

pub fn growing_dendrite(
    world: &hecs::World,
    from: Entity,
    start: Vec3,
    end: Vec3,
    palette: &Palette,
) -> Option<Scene> {
    growing(world, from, (start, end), true, palette)
}

fn growing(
    world: &hecs::World,
    from: Entity,
    (start, end): (Vec3, Vec3),
    dendrite: bool,
    palette: &Palette,
) -> Option<Scene> {
    if start.distance(end) < MIN_GROWN_LENGTH {
        return None;
    }
    let source = world.entity(from).ok()?;
    let neuron_type = *source.get::<&NeuronType>()?;
    let display = NeuronDisplay::of(world);
    let (tip, start_radius, end_radius) = match dendrite {
        true => {
            let diameter = dendrite_diameter(world, [Some(start), Some(end)]);
            let end_radius = drawn_radius(diameter, &display);
            let start_radius = match source.has::<Compartment>() {
                true => joint_radius(world, from, end_radius, &display),
                false => soma_end_radius(diameter, &display),
            };
            (
                palette.dendrite(&neuron_type, RESTING_POTENTIAL),
                start_radius,
                end_radius,
            )
        }
        false => {
            let preparation = Preparation::of(world);
            let end_radius = axon_radius(&display, preparation);
            let start_radius = match source.has::<Compartment>() {
                true => end_radius,
                false => soma_end_radius(preparation.axon_diameter(), &display),
            };
            (
                palette.compartment(&neuron_type, RESTING_POTENTIAL),
                start_radius,
                end_radius,
            )
        }
    };
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
                start_radius,
                compartment_color(source, &display, palette)
                    .or(Some(palette.compartment(&neuron_type, RESTING_POTENTIAL))),
            ),
            (end, end_radius, Some(tip)),
        )],
        rings: Vec::new(),
        transmitter: Vec::new(),
    })
}

fn joint_radius(world: &hecs::World, joint: Entity, grown: f32, display: &NeuronDisplay) -> f32 {
    let radii: Vec<f32> = world
        .query::<(&Connection, &Cable)>()
        .iter()
        .filter(|(connection, _)| !connection.directional)
        .filter(|(connection, _)| connection.from == joint || connection.to == joint)
        .map(|(_, cable)| drawn_radius(cable.diameter, display))
        .chain([grown])
        .collect();
    radii.iter().sum::<f32>() / radii.len() as f32
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
        grows_like_it_lays(growing_axon, false);
    }

    #[test]
    fn a_growing_dendrite_looks_like_the_dendrite_it_lays() {
        grows_like_it_lays(growing_dendrite, true);
    }

    fn grows_like_it_lays(
        grow: fn(&hecs::World, Entity, Vec3, Vec3, &Palette) -> Option<Scene>,
        dendrite: bool,
    ) {
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
            let growing = grow(&world, from, start, tip, &palette).unwrap();
            let compartment = spawn_compartment(&mut world, tip, NeuronType::Inhibitory);
            if dendrite {
                world.insert_one(compartment, Dendrite).unwrap();
            }
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
            let spheres = collect_spheres(&world, &radii, &paths, &palette);
            let joint = spheres
                .iter()
                .find(|sphere| sphere.position == tip)
                .unwrap();
            let bead = growing.spheres[0];
            assert_eq!((bead.radius, bead.color), (joint.radius, joint.color));
            world.get::<&mut NodeState>(compartment).unwrap().voltage =
                Millivolts(-65.0 + 25.0 * step as f64);
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
        let axon_radius = axon_radius(&NeuronDisplay::default(), Preparation::Squid);
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
        let diameter = |entity| real[&entity] * 2.0 * MICROMETRES_PER_UNIT.0 as f32;
        assert!((diameter(axon[1]) - Preparation::Squid.axon_diameter().0 as f32).abs() < 1e-6);
        assert!(diameter(dendrite[0]) > 2.0 && diameter(dendrite[0]) < 3.5);
        assert!(real[&dendrite[0]] < shown[&dendrite[0]] / 5.0);
        let palette = Theme::Light.palette();
        let paths = connection_paths(&world, &real, &palette);
        let somas: Vec<f32> = collect_spheres(&world, &real, &paths, &palette)
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
            collect_spheres(&world, &radii, &paths, &palette).len()
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
        world.get::<&mut NodeState>(second).unwrap().voltage = Millivolts(30.0);
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
        let caps = collect_spheres(&world, &radii, &paths, &palette)
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
        let diameter = Micrometres(4.0);
        world.get::<&mut Cable>(thick).unwrap().diameter = diameter;
        let display = NeuronDisplay::default();
        let radii = compartment_radii(&world);
        assert_eq!(radii[&first], drawn_radius(diameter, &display));
        assert_eq!(radii[&last], axon_radius(&display, Preparation::Squid));
        let mean =
            (drawn_radius(diameter, &display) + axon_radius(&display, Preparation::Squid)) / 2.0;
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
                    voltage: vec![RESTING_POTENTIAL, Millivolts(30.0)],
                    gates: Vec::new(),
                },
            )
            .unwrap();
        let pieces = drawn(&world);
        let lit = palette.compartment(&NeuronType::Excitatory, Millivolts(30.0));
        let rest = palette.compartment(&NeuronType::Excitatory, RESTING_POTENTIAL);
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[1].start, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!((pieces[1].start_color, pieces[1].end_color), (rest, lit));
        assert_eq!((pieces[2].start_color, pieces[2].end_color), (lit, rest));
    }

    #[test]
    fn a_fillet_meets_the_soma_and_the_branch_without_a_crease() {
        let profile = fillet_profile(1.0, 0.3, 0.8);
        let (first, second) = (profile[0], profile[1]);
        let (last, before) = (profile[FILLET_SEGMENTS], profile[FILLET_SEGMENTS - 1]);
        assert!((first.length() - 1.0).abs() < 1e-5);
        assert!((second - first).normalize().dot(first.normalize()).abs() < 0.2);
        assert!((last.y - 0.3).abs() < 1e-5);
        assert!((last - before).normalize().y.abs() < 0.2);
        assert!(profile.windows(2).all(|pair| pair[1].y < pair[0].y));
    }

    #[test]
    fn a_longer_branch_leaves_its_soma_from_a_wider_base() {
        let base = |length: f64| {
            let rounding = fillet_rounding(1.0, 0.3, Micrometres(length), 2.0).unwrap();
            fillet_profile(1.0, 0.3, rounding)[0].y
        };
        assert!(base(100.0) < base(300.0) && base(300.0) < base(1000.0));
    }

    #[test]
    fn a_fillet_ends_before_the_first_compartment() {
        for room in [0.8, 1.5, 2.0, 4.0] {
            let Some(rounding) = fillet_rounding(1.0, 0.3, Micrometres(1e6), room) else {
                continue;
            };
            let end = fillet_profile(1.0, 0.3, rounding)[FILLET_SEGMENTS].x;
            assert!(end <= FILLET_REACH * room + 1e-4, "{end} in {room}");
        }
    }

    #[test]
    fn only_a_compartment_soma_blends_into_its_dendrites_and_axon() {
        let mut world = hecs::World::new();
        let (soma, _, _) = neuron_with_a_dendrite_and_an_axon(&mut world, NeuronType::Excitatory);
        world.insert_one(soma, NodeState::default()).unwrap();
        let point = drawn(&world).len();
        world.insert_one(soma, CompartmentSoma::default()).unwrap();
        assert_eq!(drawn(&world).len(), point + 2 * FILLET_SEGMENTS);
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
