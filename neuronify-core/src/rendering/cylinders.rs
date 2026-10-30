use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::{Entity, EntityRef};

use crate::components::*;
use crate::rendering::colors::{terminal_color, Palette};
use crate::rendering::gpu_types::CylinderData;
use crate::rendering::paths::ConnectionPath;

/// Radius where a process leaves the soma.
const ROOT_RADIUS: f32 = 0.30;
/// Factor the radius shrinks by for every three units away from the nearest soma.
const TAPER: f32 = 0.85;
/// Floor so that long processes never disappear.
const MIN_RADIUS: f32 = 0.16;
const FULLY_TAPERED_DISTANCE: f32 = 12.0;
const PIECES_PER_CABLE: usize = 8;

fn radius_from_soma_distance(distance: f32) -> f32 {
    (ROOT_RADIUS * TAPER.powf(distance / 3.0)).max(MIN_RADIUS)
}

fn compartment_state(entity: EntityRef) -> Option<(NeuronType, f64)> {
    Some((
        *entity.get::<&NeuronType>()?,
        entity.get::<&Compartment>()?.voltage,
    ))
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
        .query::<&Position>()
        .with::<&Compartment>()
        .iter()
        .map(|(entity, position)| {
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
            let (from, to) = (compartment_state(from), compartment_state(to));
            let (from, to) = (from.or(to), to.or(from));
            let (start_radius, end_radius) = (radius(connection.from), radius(connection.to));
            Some((0..PIECES_PER_CABLE).map(move |piece| {
                let fraction = |piece: usize| piece as f32 / PIECES_PER_CABLE as f32;
                let (near, far) = (fraction(piece), fraction(piece + 1));
                let color = match (from, to) {
                    (Some((neuron_type, from_voltage)), Some((_, to_voltage))) => {
                        let middle = f64::from(0.5 * (near + far));
                        let voltage = from_voltage + (to_voltage - from_voltage) * middle;
                        palette.compartment(&neuron_type, voltage)
                    }
                    _ => palette.neutral,
                };
                CylinderData {
                    start: start.lerp(end, near),
                    start_radius: start_radius + (end_radius - start_radius) * near,
                    end: start.lerp(end, far),
                    end_radius: start_radius + (end_radius - start_radius) * far,
                    start_color: color,
                    end_color: color,
                    _padding: Default::default(),
                }
            }))
        })
        .flatten()
        .collect();
    cylinders.reserve(paths.values().map(|path| path.points.len()).sum());
    for path in paths.values() {
        let segments = path.points.len().saturating_sub(1);
        for (i, w) in path.points.windows(2).enumerate() {
            let color = path
                .start_color
                .lerp(path.end_color, (i as f32 + 0.5) / segments as f32);
            cylinders.push(CylinderData {
                start: w[0],
                start_radius: path.radius,
                end: w[1],
                end_radius: path.radius,
                start_color: color,
                end_color: color,
                _padding: Default::default(),
            });
        }
        // The bar across the end of an inhibitory synapse, as in ⊣
        if let (true, [.., before, end]) = (path.directional && path.inhibitory, &path.points[..]) {
            let direction = (*end - *before).normalize_or_zero();
            cylinders.push(CylinderData {
                start: *end - direction * 0.12,
                start_radius: 1.2 * path.terminal_radius,
                end: *end + direction * 0.12,
                end_radius: 1.2 * path.terminal_radius,
                start_color: terminal_color(),
                end_color: terminal_color(),
                _padding: Default::default(),
            });
        }
    }
    cylinders
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn compartments_beyond_the_taper_are_thinnest() {
        assert_eq!(
            radius_from_soma_distance(FULLY_TAPERED_DISTANCE),
            MIN_RADIUS
        );
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
            world.spawn((Position { position }, Compartment::default()));
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
