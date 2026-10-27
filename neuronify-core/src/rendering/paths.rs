use std::collections::{BTreeMap, HashMap, HashSet};

use glam::Vec3;
use hecs::{Entity, EntityRef};

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
    /// The source inhibits its target, so the path ends in a bar rather than a knob
    pub inhibitory: bool,
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
            .filter_map(|w| distance_to_segment(w[0], w[1], point))
            .reduce(f32::min)
    }
}

pub fn distance_to_segment(start: Vec3, end: Vec3, point: Vec3) -> Option<f32> {
    let along = end - start;
    let t = (point - start).dot(along) / along.length_squared();
    (0.0..=1.0)
        .contains(&t)
        .then(|| point.distance(start + t * along))
}

fn connection_colors(from: EntityRef, to: EntityRef, palette: &Palette) -> (Vec3, Vec3) {
    let value = |target: EntityRef| -> f32 {
        if let Some(compartment) = target.get::<&Compartment>() {
            ((compartment.voltage + 50.0) / 200.0) as f32
        } else if let Some(dynamics) = target.get::<&LeakyDynamics>() {
            let neuron = target.get::<&LeakyNeuron>();
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
    if from.has::<CurrentClamp>() {
        (yellow(), yellow())
    } else if from.has::<GeneratorDynamics>() {
        (orange(), orange())
    } else if let Some(neuron_type) = from.get::<&NeuronType>() {
        (
            palette.neurocolor(&neuron_type, start_value),
            palette.neurocolor(&neuron_type, end_value),
        )
    } else {
        (palette.neutral, palette.neutral)
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
    let mut points: Vec<Vec3> = (0..=steps)
        .map(|i| quadratic_bezier(start, control, end, i as f32 / steps as f32))
        .collect();
    let mut keep = points.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>() - trim;
    for i in 1..points.len() {
        let segment = points[i - 1].distance(points[i]);
        if keep <= segment {
            if keep > 0.0 {
                points[i] = points[i - 1].lerp(points[i], keep / segment);
                points.truncate(i + 1);
            } else {
                points.truncate(i);
            }
            break;
        }
        keep -= segment;
    }
    points
}

/// Paths for the connections drawn as tubes, keyed by connection entity.
pub fn connection_paths(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    palette: &Palette,
) -> BTreeMap<Entity, ConnectionPath> {
    let pairs: HashSet<(Entity, Entity)> = world
        .query::<&Connection>()
        .iter()
        .map(|(_, c)| (c.from, c.to))
        .collect();
    world
        .query::<(&Connection, Option<&CompartmentCurrent>)>()
        .without::<&Voltmeter>()
        .iter()
        .filter(|(_, (connection, cable))| connection.directional || cable.is_none())
        .filter_map(|(entity, (connection, _))| {
            let from = world.entity(connection.from).ok()?;
            let to = world.entity(connection.to).ok()?;
            let start = from.get::<&Position>()?.position;
            let end = to.get::<&Position>()?.position;
            let from_compartment = from.has::<Compartment>();
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
            let (start_color, end_color) = connection_colors(from, to, palette);
            Some((
                entity,
                ConnectionPath {
                    points: bent_line(start, end, bend, trim),
                    radius,
                    start_color,
                    end_color,
                    directional: connection.directional,
                    inhibitory: from
                        .get::<&NeuronType>()
                        .is_some_and(|neuron_type| *neuron_type == NeuronType::Inhibitory),
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
            inhibitory: false,
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
