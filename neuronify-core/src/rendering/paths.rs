use std::collections::{BTreeMap, HashMap, HashSet};

use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::Voltmeter;
use crate::rendering::colors::*;

/// Spacing between the joints along a connection, the same as between compartments.
const JOINT_SPACING: f32 = 2.0 * NODE_RADIUS;
/// Largest sideways bend of a connection, as a fraction of its length.
const MAX_BEND: f32 = 0.1;
/// Radius of a connection with strength 1, the same as the thinnest axon.
const CONNECTION_RADIUS: f32 = 0.16;
/// Distance from the synapse marker to the centre of the target.
const SYNAPSE_GAP: f32 = 2.0 * NODE_RADIUS;

/// The tube a connection is drawn as, from the source towards the target.
///
/// Directional connections stop short of the target, where the synapse marker goes.
pub struct ConnectionPath {
    pub points: Vec<Vec3>,
    pub radius: f32,
    pub start_color: Vec3,
    pub end_color: Vec3,
    pub directional: bool,
}

impl ConnectionPath {
    /// The point a fraction `t` of the way along the path.
    pub fn point_at(&self, t: f32) -> Vec3 {
        let total: f32 = self.points.windows(2).map(|w| w[0].distance(w[1])).sum();
        let mut remaining = t.clamp(0.0, 1.0) * total;
        for w in self.points.windows(2) {
            let length = w[0].distance(w[1]);
            if remaining <= length && length > 0.0 {
                return w[0].lerp(w[1], remaining / length);
            }
            remaining -= length;
        }
        *self.points.last().unwrap_or(&Vec3::ZERO)
    }

    /// Distance from `point` to the nearest segment it lies alongside, if any.
    pub fn distance_to(&self, point: Vec3) -> Option<f32> {
        self.points
            .windows(2)
            .filter_map(|w| {
                let ab = w[1] - w[0];
                let t = (point - w[0]).dot(ab) / ab.length_squared();
                (0.0..=1.0)
                    .contains(&t)
                    .then(|| point.distance(w[0] + t * ab))
            })
            .reduce(f32::min)
    }
}

/// Cable segments between compartments and voltmeter wires are drawn in other ways.
pub(crate) fn is_drawn_as_path(
    world: &hecs::World,
    entity: Entity,
    connection: &Connection,
) -> bool {
    world.get::<&Voltmeter>(entity).is_err()
        && (connection.directional || world.get::<&CompartmentCurrent>(entity).is_err())
}

pub(crate) fn connection_colors(world: &hecs::World, from: Entity, to: Entity) -> (Vec3, Vec3) {
    let value = |target: Entity| -> f32 {
        if let Ok(compartment) = world.get::<&Compartment>(target) {
            ((compartment.voltage + 50.0) / 200.0) as f32
        } else if let Ok(dynamics) = world.get::<&LeakyDynamics>(target) {
            let neuron = world.get::<&LeakyNeuron>(target).ok();
            if let Some(neuron) = neuron {
                ((dynamics.voltage - neuron.resting_potential)
                    / (neuron.threshold - neuron.resting_potential))
                    .clamp(0.0, 1.0) as f32
            } else {
                0.5
            }
        } else {
            1.0
        }
    };
    let start_value = value(to);
    let end_value = value(from);
    if world.get::<&CurrentClamp>(from).is_ok() {
        (yellow(), yellow())
    } else if world.get::<&GeneratorDynamics>(from).is_ok() {
        (orange(), orange())
    } else if let Ok(neuron_type) = world.get::<&NeuronType>(from) {
        (
            neurocolor(&neuron_type, start_value),
            neurocolor(&neuron_type, end_value),
        )
    } else {
        (crust(), crust())
    }
}

fn quadratic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

/// Samples a bent line from `start` to `end`, cut `trim` short of `end`.
fn bent_line(start: Vec3, end: Vec3, bend: f32, trim: f32) -> Vec<Vec3> {
    let diff = end - start;
    let length = diff.length();
    let side = diff.cross(Vec3::Y).normalize_or_zero();
    let control = (start + end) * 0.5 + side * bend * length;
    let steps = ((length / JOINT_SPACING).ceil() as usize).max(1);
    let curve: Vec<Vec3> = (0..=steps)
        .map(|i| quadratic_bezier(start, control, end, i as f32 / steps as f32))
        .collect();
    let mut keep = curve.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>() - trim;
    let mut points = vec![start];
    for w in curve.windows(2) {
        let segment = w[0].distance(w[1]);
        if keep <= segment {
            if keep > 0.0 {
                points.push(w[0].lerp(w[1], keep / segment));
            }
            break;
        }
        keep -= segment;
        points.push(w[1]);
    }
    points
}

/// Paths for the connections drawn as tubes, keyed by connection entity.
pub fn connection_paths(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
) -> BTreeMap<Entity, ConnectionPath> {
    let pairs: HashSet<(Entity, Entity)> = world
        .query::<&Connection>()
        .iter()
        .map(|(_, c)| (c.from, c.to))
        .collect();
    world
        .query::<&Connection>()
        .iter()
        .filter(|(entity, connection)| is_drawn_as_path(world, *entity, connection))
        .filter_map(|(entity, connection)| {
            let start = world.get::<&Position>(connection.from).ok()?.position;
            let end = world.get::<&Position>(connection.to).ok()?.position;
            let from_compartment = world.get::<&Compartment>(connection.from).is_ok();
            // Reciprocal pairs bend to opposite sides so both stay visible.
            // An axon terminal continues its chain of compartments straight.
            let bend = if pairs.contains(&(connection.to, connection.from)) {
                BEZIER_BEND_FRACTION
            } else if from_compartment {
                0.0
            } else {
                ((entity.id() as f32 * 0.618_034).fract() * 2.0 - 1.0) * MAX_BEND
            };
            let trim = if connection.directional {
                SYNAPSE_GAP
            } else {
                0.0
            };
            let radius = match radii.get(&connection.from) {
                Some(&radius) if from_compartment => radius,
                _ => (CONNECTION_RADIUS * connection.strength as f32).max(0.05),
            };
            let (start_color, end_color) = connection_colors(world, connection.from, connection.to);
            Some((
                entity,
                ConnectionPath {
                    points: bent_line(start, end, bend, trim),
                    radius,
                    start_color,
                    end_color,
                    directional: connection.directional,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(bend: f32, trim: f32) -> ConnectionPath {
        ConnectionPath {
            points: bent_line(Vec3::ZERO, Vec3::new(20.0, 0.0, 0.0), bend, trim),
            radius: CONNECTION_RADIUS,
            start_color: Vec3::ONE,
            end_color: Vec3::ONE,
            directional: trim > 0.0,
        }
    }

    #[test]
    fn stops_short_of_the_target() {
        let path = path(MAX_BEND, SYNAPSE_GAP);
        let end = *path.points.last().unwrap();
        assert!((end.distance(Vec3::new(20.0, 0.0, 0.0)) - SYNAPSE_GAP).abs() < 0.1);
        assert!(path
            .points
            .windows(2)
            .all(|w| w[0].distance(w[1]) <= JOINT_SPACING * 1.1));
    }

    #[test]
    fn spikes_and_erasing_follow_the_bend() {
        let path = path(MAX_BEND, SYNAPSE_GAP);
        assert_eq!(path.point_at(0.0), Vec3::ZERO);
        assert_eq!(path.point_at(1.0), *path.points.last().unwrap());
        let middle = path.point_at(0.5);
        assert!(
            middle.z.abs() > 0.5,
            "the middle should be bent, got {middle}"
        );
        assert!(path.distance_to(middle).unwrap() < 1e-4);
        assert!(path.distance_to(Vec3::new(10.0, 0.0, 0.0)).unwrap() > 0.5);
    }

    #[test]
    fn a_connection_shorter_than_the_gap_has_no_tube() {
        let points = bent_line(Vec3::ZERO, Vec3::X, 0.0, SYNAPSE_GAP);
        assert_eq!(points, vec![Vec3::ZERO]);
    }
}
