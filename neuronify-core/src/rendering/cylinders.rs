use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::CylinderData;
use crate::rendering::paths::ConnectionPath;

/// Radius where a process leaves the soma.
const ROOT_RADIUS: f32 = 0.30;
/// Factor the radius shrinks by for every three units away from the nearest soma.
const TAPER: f32 = 0.85;
/// Floor so that long processes never disappear.
const MIN_RADIUS: f32 = 0.16;

fn radius_from_soma_distance(distance: f32) -> f32 {
    (ROOT_RADIUS * TAPER.powf(distance / 3.0)).max(MIN_RADIUS)
}

/// Radius of every compartment, thick next to a soma and tapering away from it.
///
/// Spheres and cylinders both use these, so that a chain of compartments
/// bends smoothly at each joint.
pub fn compartment_radii(world: &hecs::World) -> HashMap<Entity, f32> {
    let somas: Vec<Vec3> = world
        .query::<&Position>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|(_, position)| position.position)
        .collect();
    world
        .query::<&Position>()
        .with::<&Compartment>()
        .iter()
        .map(|(entity, position)| {
            let distance = somas
                .iter()
                .map(|soma| soma.distance(position.position))
                .fold(f32::MAX, f32::min);
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
            let start = world.get::<&Position>(connection.from).ok()?.position;
            let end = world.get::<&Position>(connection.to).ok()?.position;
            let colored_by = if world.get::<&Compartment>(connection.to).is_ok() {
                connection.to
            } else {
                connection.from
            };
            let color = match (
                world.get::<&Compartment>(colored_by),
                world.get::<&NeuronType>(colored_by),
            ) {
                (Ok(compartment), Ok(neuron_type)) => palette.neurocolor(
                    &neuron_type,
                    ((compartment.voltage + 50.0) / 200.0).clamp(0.0, 1.0) as f32,
                ),
                _ => palette.neutral,
            };
            Some(CylinderData {
                start,
                start_radius: radius(connection.from),
                end,
                end_radius: radius(connection.to),
                color,
                _padding: Default::default(),
            })
        })
        .collect();
    for path in paths.values() {
        let segments = path.points.len().saturating_sub(1);
        for (i, w) in path.points.windows(2).enumerate() {
            cylinders.push(CylinderData {
                start: w[0],
                start_radius: path.radius,
                end: w[1],
                end_radius: path.radius,
                color: path
                    .start_color
                    .lerp(path.end_color, (i as f32 + 0.5) / segments as f32),
                _padding: Default::default(),
            });
        }
    }
    cylinders
}
