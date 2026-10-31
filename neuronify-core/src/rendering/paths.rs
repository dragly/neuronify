use std::collections::{BTreeMap, HashMap, HashSet};

use glam::Vec3;
use hecs::{Entity, EntityRef};

use crate::components::*;
use crate::constants::*;
use crate::measurement::is_meter;
use crate::rendering::colors::*;
use crate::rendering::shapes::{shown_voltage, voltage_fraction};

/// Spacing between the joints along a connection, the same as between compartments.
const JOINT_SPACING: f32 = 2.0 * NODE_RADIUS;
const MAX_SEGMENTS: usize = 12;
/// Largest sideways bend of a connection, as a fraction of its length.
const MAX_BEND: f32 = 0.1;
/// Radius of a connection with strength 1, the same as the thinnest axon.
const CONNECTION_RADIUS: f32 = 0.16;
/// Distance from the synapse marker to the centre of the target.
const SYNAPSE_GAP: f32 = 2.0 * NODE_RADIUS;
const SOMA_TERMINAL_RADIUS: f32 = 0.5 * NODE_RADIUS;
const COMPARTMENT_TERMINAL_RADIUS: f32 = 0.3 * NODE_RADIUS;
const MIN_TERMINAL_SCALE: f32 = 0.4;

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
    pub terminal_radius: f32,
    pub terminal_direction: Vec3,
}

impl ConnectionPath {
    pub fn length(&self) -> f32 {
        self.points.windows(2).map(|w| w[0].distance(w[1])).sum()
    }

    /// The point a fraction `t` of the way along the path.
    pub fn point_at(&self, t: f32) -> Vec3 {
        let mut remaining = t.clamp(0.0, 1.0) * self.length();
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

fn connection_colors(
    from: EntityRef,
    to: EntityRef,
    display: &NeuronDisplay,
    palette: &Palette,
) -> (Vec3, Vec3) {
    let value = |target: EntityRef| -> f32 {
        if let Some(compartment) = target.get::<&Compartment>() {
            ((shown_voltage(&compartment, display) + 50.0) / 200.0) as f32
        } else if let Some(dynamics) = target.get::<&LeakyDynamics>() {
            let neuron = target.get::<&LeakyNeuron>();
            if let Some(neuron) = neuron {
                voltage_fraction(&neuron, &dynamics, display)
            } else {
                0.5
            }
        } else {
            1.0
        }
    };
    let start_value = value(to);
    let end_value = value(from);
    if let Some(clamp) = from.get::<&CurrentClamp>() {
        let color = current_color(clamp.current_output);
        (color, color)
    } else if from.has::<GeneratorDynamics>() {
        let color = palette.generator(
            from.get::<&NeuronType>()
                .map_or_else(Default::default, |t| *t),
        );
        (color, color)
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
    let steps = ((length / JOINT_SPACING).ceil() as usize).clamp(1, MAX_SEGMENTS);
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
    let display = NeuronDisplay::of(world);
    let pairs: HashSet<(Entity, Entity)> = world
        .query::<&Connection>()
        .iter()
        .map(|(_, c)| (c.from, c.to))
        .collect();
    world
        .query::<(&Connection, Option<&CompartmentCurrent>)>()
        .iter()
        .filter(|(_, (connection, _))| !is_meter(world, connection.to))
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
            let target_radius = radii.get(&connection.to).copied();
            let terminal_radius = match target_radius {
                Some(_) => COMPARTMENT_TERMINAL_RADIUS,
                None => SOMA_TERMINAL_RADIUS,
            };
            let terminal_radius = if from_compartment {
                terminal_radius * (connection.strength as f32).sqrt().max(MIN_TERMINAL_SCALE)
            } else {
                terminal_radius
            };
            let trim = match (connection.directional, target_radius) {
                (false, _) => 0.0,
                (true, Some(radius)) => radius + terminal_radius,
                (true, None) => SYNAPSE_GAP,
            };
            let radius = match radii.get(&connection.from) {
                Some(&radius) if from_compartment => radius,
                _ => (CONNECTION_RADIUS * connection.strength as f32).max(0.05),
            };
            let (start_color, end_color) = connection_colors(from, to, &display, palette);
            let points = bent_line(start, end, bend, trim);
            let terminal_direction = match &points[..] {
                [.., before, last] => *last - *before,
                _ => end - start,
            }
            .normalize_or_zero();
            Some((
                entity,
                ConnectionPath {
                    points,
                    radius,
                    start_color,
                    end_color,
                    directional: connection.directional,
                    inhibitory: from
                        .get::<&NeuronType>()
                        .is_some_and(|neuron_type| *neuron_type == NeuronType::Inhibitory),
                    terminal_radius,
                    terminal_direction,
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
            terminal_radius: SOMA_TERMINAL_RADIUS,
            terminal_direction: Vec3::X,
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
    fn long_connections_have_a_limited_number_of_joints() {
        let target = Vec3::new(500.0, 0.0, 0.0);
        let points = bent_line(Vec3::ZERO, target, MAX_BEND, SYNAPSE_GAP);
        assert_eq!(points.len(), MAX_SEGMENTS + 1);
        let end = *points.last().unwrap();
        assert!((end.distance(target) - SYNAPSE_GAP).abs() < 0.1);
    }

    #[test]
    fn an_axon_ending_on_a_compartment_touches_it() {
        let mut world = hecs::World::new();
        let compartment_at = |world: &mut hecs::World, x: f32| {
            world.spawn((
                Position {
                    position: Vec3::new(x, 0.0, 0.0),
                },
                Compartment::default(),
                NeuronType::Excitatory,
            ))
        };
        let tip = compartment_at(&mut world, 0.0);
        let dendrite = compartment_at(&mut world, 2.0);
        let ending = crate::elements::spawn_cable(&mut world, tip, dendrite, true);
        let radii = crate::rendering::compartment_radii(&world);

        let paths = connection_paths(&world, &radii, &Theme::default().palette());

        let path = &paths[&ending];
        let gap = path
            .points
            .last()
            .unwrap()
            .distance(Vec3::new(2.0, 0.0, 0.0));
        assert!((gap - radii[&dendrite] - path.terminal_radius).abs() < 1e-4);
        assert_eq!(path.terminal_radius, COMPARTMENT_TERMINAL_RADIUS);
    }

    #[test]
    fn a_stronger_axon_ending_has_a_bigger_terminal() {
        let terminal_radius = |strength: f64| {
            let mut world = hecs::World::new();
            let compartment_at = |world: &mut hecs::World, x: f32| {
                world.spawn((
                    Position {
                        position: Vec3::new(x, 0.0, 0.0),
                    },
                    Compartment::default(),
                    NeuronType::Excitatory,
                ))
            };
            let tip = compartment_at(&mut world, 0.0);
            let dendrite = compartment_at(&mut world, 2.0);
            let ending = crate::elements::spawn_cable(&mut world, tip, dendrite, true);
            world.get::<&mut Connection>(ending).unwrap().strength = strength;
            let radii = crate::rendering::compartment_radii(&world);
            connection_paths(&world, &radii, &Theme::default().palette())[&ending].terminal_radius
        };
        assert_eq!(terminal_radius(4.0), 2.0 * COMPARTMENT_TERMINAL_RADIUS);
        assert_eq!(
            terminal_radius(0.0),
            MIN_TERMINAL_SCALE * COMPARTMENT_TERMINAL_RADIUS
        );
    }

    #[test]
    fn a_connection_shorter_than_the_gap_has_no_tube() {
        let points = bent_line(Vec3::ZERO, Vec3::X, 0.0, SYNAPSE_GAP);
        assert_eq!(points, vec![Vec3::ZERO]);
    }
}
