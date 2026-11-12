use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::{
    Cable, Connection, CurrentClamp, GeneratorDynamics, LeakyNeuron, Position,
};
use crate::constants::NODE_RADIUS;
use crate::measurement::is_meter;

pub const TIP_LENGTH: f32 = 2.6 * NODE_RADIUS;
const GRAB_REACH: f32 = 1.2 * NODE_RADIUS;
const ONWARD: Vec3 = Vec3::Z;
const ONE_SIDED: f32 = 0.5;
const SIDES: usize = 16;

type Source =
    hecs::Or<&'static LeakyNeuron, hecs::Or<&'static CurrentClamp, &'static GeneratorDynamics>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tip {
    pub from: Entity,
    pub center: Vec3,
    pub root: Vec3,
    pub knob: Vec3,
}

pub fn tip(world: &World, from: Entity) -> Option<Tip> {
    if !world.satisfies::<Source>(from) || cabled(world, from) {
        return None;
    }
    let center = world.get::<&Position>(from).ok()?.position;
    let onward = onward(world, from);
    Some(Tip {
        from,
        center,
        root: center + onward * NODE_RADIUS,
        knob: center + onward * TIP_LENGTH,
    })
}

pub fn onward(world: &World, from: Entity) -> Vec3 {
    let Ok(center) = world
        .get::<&Position>(from)
        .map(|position| position.position)
    else {
        return ONWARD;
    };
    let neighbours: Vec<Vec3> = world
        .query::<&Connection>()
        .iter()
        .filter_map(|connection| {
            if connection.from == from {
                Some(connection.to)
            } else if connection.to == from {
                Some(connection.from)
            } else {
                None
            }
        })
        .filter(|&other| !is_meter(world, other))
        .filter_map(|other| {
            let at = world.get::<&Position>(other).ok()?.position;
            Some((at - center).with_y(0.0).normalize_or_zero())
        })
        .collect();
    free_side(&neighbours)
}

fn cabled(world: &World, entity: Entity) -> bool {
    world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .any(|cable| cable.from == entity || cable.to == entity)
}

fn free_side(neighbours: &[Vec3]) -> Vec3 {
    if neighbours.is_empty() {
        return ONWARD;
    }
    let away = -neighbours.iter().sum::<Vec3>();
    if away.length() >= ONE_SIDED {
        return away.normalize();
    }
    let clearance = |direction: Vec3| {
        neighbours
            .iter()
            .map(|neighbour| -neighbour.dot(direction))
            .fold(f32::INFINITY, f32::min)
    };
    (0..SIDES)
        .map(|side| {
            let angle = TAU * side as f32 / SIDES as f32;
            Vec3::new(angle.sin(), 0.0, angle.cos())
        })
        .max_by(|a, b| clearance(*a).total_cmp(&clearance(*b)))
        .unwrap_or(ONWARD)
}

pub fn tip_at(world: &World, selected: Option<Entity>, position: Vec3, reach: f32) -> Option<Tip> {
    let tip = tip(world, selected?)?;
    let distance = tip.knob.distance(position);
    let near = distance < reach.max(GRAB_REACH) && distance < tip.center.distance(position);
    near.then_some(tip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_connection, Element, Meter};

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    fn side(tip: Tip) -> Vec3 {
        (tip.knob - tip.center).normalize()
    }

    #[test]
    fn a_source_with_nowhere_to_send_its_spikes_has_a_tip_pointing_on() {
        let mut world = World::new();
        let sensor = Element::TouchSensor.spawn(&mut world, at(0.0, -8.0));
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
        spawn_connection(&mut world, sensor, neuron);
        let onward = tip(&world, neuron).unwrap();
        assert!(onward.knob.abs_diff_eq(at(0.0, TIP_LENGTH), 1e-4));
        assert!(onward.root.abs_diff_eq(at(0.0, NODE_RADIUS), 1e-4));
    }

    #[test]
    fn meter_wires_leave_the_tip_and_dendrites_take_it_away() {
        let mut world = World::new();
        let measured = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
        let meter = Meter::Voltmeter.spawn(&mut world, at(10.0, 0.0));
        Meter::Voltmeter.wire(&mut world, measured, meter);
        let grown = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 20.0));
        crate::dendrites::spawn_dendrites(&mut world, grown);
        assert!(side(tip(&world, measured).unwrap()).abs_diff_eq(ONWARD, 1e-4));
        assert_eq!(tip(&world, grown), None);
        assert_eq!(tip(&world, meter), None);
    }

    #[test]
    fn only_the_selected_tip_is_picked_and_only_near_its_knob() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
        let knob = at(0.3, TIP_LENGTH - 0.2);
        assert_eq!(tip_at(&world, None, knob, 0.5), None);
        let picked = tip_at(&world, Some(neuron), knob, 0.5).map(|tip| tip.from);
        assert_eq!(picked, Some(neuron));
        assert_eq!(tip_at(&world, Some(neuron), at(0.0, 1.0), 2.0), None);
        assert_eq!(
            tip_at(&world, Some(neuron), at(0.0, -TIP_LENGTH), 2.0),
            None
        );
    }

    #[test]
    fn a_connected_source_shows_its_tip_on_its_free_side() {
        let mut world = World::new();
        let sensor = Element::TouchSensor.spawn(&mut world, at(0.0, -8.0));
        let middle = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
        let last = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 8.0));
        spawn_connection(&mut world, sensor, middle);
        spawn_connection(&mut world, middle, last);
        let across = side(tip(&world, middle).unwrap());
        assert!(across.dot(Vec3::Z).abs() < 0.2, "{across}");
        assert!(side(tip(&world, sensor).unwrap()).abs_diff_eq(-Vec3::Z, 1e-4));
    }
}
