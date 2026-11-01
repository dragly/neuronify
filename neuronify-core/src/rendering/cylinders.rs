use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::{Entity, EntityRef};

use crate::components::*;
use crate::rendering::colors::{terminal_color, Palette};
use crate::rendering::gpu_types::{CylinderData, Sphere};
use crate::rendering::paths::ConnectionPath;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::shown_voltage;

/// Radius where a process leaves the soma.
const ROOT_RADIUS: f32 = 0.30;
/// Factor the radius shrinks by for every three units away from the nearest soma.
const TAPER: f32 = 0.85;
/// Floor so that long processes never disappear.
const MIN_RADIUS: f32 = 0.16;
const FULLY_TAPERED_DISTANCE: f32 = 12.0;

const MIN_GROWN_LENGTH: f32 = 1e-4;
const AXON_RADIUS: f32 = 0.11;

fn radius_from_soma_distance(distance: f32) -> f32 {
    (ROOT_RADIUS * TAPER.powf(distance / 3.0)).max(MIN_RADIUS)
}

fn compartment_color(
    entity: EntityRef,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Option<Vec3> {
    let neuron_type = &*entity.get::<&NeuronType>()?;
    let voltage = shown_voltage(&*entity.get::<&Compartment>()?, display);
    Some(match entity.has::<Dendrite>() {
        true => palette.dendrite(neuron_type, voltage),
        false => palette.compartment(neuron_type, voltage),
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
        shown_voltage(&*compartment.get::<&Compartment>()?, display),
    ))
}

fn radius_at(world: &hecs::World, position: Vec3) -> f32 {
    let distance = world
        .query::<&Position>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|(_, soma)| soma.position.distance(position))
        .fold(f32::MAX, f32::min);
    radius_from_soma_distance(distance)
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

/// Radius of every compartment, thick next to a soma and tapering away from it.
///
/// Spheres and cylinders both use these, so that a chain of compartments
/// bends smoothly at each joint.
pub fn compartment_radii(world: &hecs::World) -> HashMap<Entity, f32> {
    let cell_of = |position: Vec3| {
        let coordinate = |x: f32| (x / FULLY_TAPERED_DISTANCE).floor() as i32;
        (coordinate(position.x), coordinate(position.z))
    };
    let mut somas: HashMap<(i32, i32), Vec<Vec3>> = HashMap::new();
    for (_, position) in world.query::<&Position>().with::<&LeakyNeuron>().iter() {
        somas
            .entry(cell_of(position.position))
            .or_default()
            .push(position.position);
    }
    world
        .query::<(&Position, Option<&Dendrite>)>()
        .with::<&Compartment>()
        .iter()
        .map(|(entity, (position, dendrite))| {
            if dendrite.is_none() {
                return (entity, AXON_RADIUS);
            }
            let (x, z) = cell_of(position.position);
            let mut distance = f32::MAX;
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let cell = (x.saturating_add(dx), z.saturating_add(dz));
                    for soma in somas.get(&cell).into_iter().flatten() {
                        distance = distance.min(soma.distance(position.position));
                    }
                }
            }
            (entity, radius_from_soma_distance(distance))
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
    let radius = |entity: Entity| radii.get(&entity).copied().unwrap_or(ROOT_RADIUS);
    let display = NeuronDisplay::of(world);
    let mut cylinders: Vec<CylinderData> = world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .filter(|(_, connection)| !connection.directional)
        .filter_map(|(_, connection)| {
            let from = world.entity(connection.from).ok()?;
            let to = world.entity(connection.to).ok()?;
            let start = from.get::<&Position>()?.position;
            let end = to.get::<&Position>()?.position;
            Some(cable(
                palette,
                (
                    start,
                    radius(connection.from),
                    compartment_color(from, &display, palette)
                        .or_else(|| soma_end_color(from, to, &display, palette)),
                ),
                (
                    end,
                    radius(connection.to),
                    compartment_color(to, &display, palette),
                ),
            ))
        })
        .collect();
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
    let tip = palette.compartment(&neuron_type, Compartment::default().voltage);
    let end_radius = AXON_RADIUS;
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
                if source.has::<Compartment>() {
                    AXON_RADIUS
                } else {
                    radius_at(world, start)
                },
                compartment_color(source, &NeuronDisplay::of(world), palette),
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
    use crate::elements::{spawn_cable, spawn_compartment};
    use crate::rendering::colors::Theme;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::spheres::{collect_spheres, SPINES_PER_CABLE};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

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
            let spheres = collect_spheres(&world, &radii, &paths, &palette);
            let joint = spheres
                .iter()
                .find(|sphere| sphere.position == tip)
                .unwrap();
            let bead = growing.spheres[0];
            assert_eq!((bead.radius, bead.color), (joint.radius, joint.color));
            world.get::<&mut Compartment>(compartment).unwrap().voltage = 30.0 * step as f64;
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

    #[test]
    fn compartments_beyond_the_taper_are_thinnest() {
        assert_eq!(
            radius_from_soma_distance(FULLY_TAPERED_DISTANCE),
            MIN_RADIUS
        );
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
        assert!(axon
            .iter()
            .all(|compartment| radii[compartment] == AXON_RADIUS));
        assert!(radii[&dendrite[0]] > radii[&dendrite[3]]);
        assert!(radii[&dendrite[3]] > AXON_RADIUS);
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
        let rest = Compartment::default().voltage;
        let excitatory = NeuronType::Excitatory;
        assert_eq!(root.start_color, palette.compartment(&excitatory, rest));
        assert_eq!(root.end_color, palette.dendrite(&excitatory, rest));
    }

    #[test]
    fn radii_follow_the_nearest_soma() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut world = hecs::World::new();
        let mut random_position =
            || Vec3::new(rng.gen_range(-60.0..60.0), 0.0, rng.gen_range(-60.0..60.0));
        let somas: Vec<Vec3> = (0..40).map(|_| random_position()).collect();
        for &position in &somas {
            world.spawn((Position { position }, LeakyNeuron::default()));
        }
        for _ in 0..500 {
            let position = random_position();
            world.spawn((Position { position }, Compartment::default(), Dendrite));
        }

        let radii = compartment_radii(&world);

        for (entity, position) in world.query::<&Position>().with::<&Compartment>().iter() {
            let nearest = somas
                .iter()
                .map(|soma| soma.distance(position.position))
                .fold(f32::MAX, f32::min);
            assert_eq!(radii[&entity], radius_from_soma_distance(nearest));
        }
        assert!(radii.values().any(|&radius| radius > MIN_RADIUS + 0.05));
        assert!(radii.values().any(|&radius| radius == MIN_RADIUS));
    }
}
