use std::collections::HashSet;

use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::Voltmeter;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::ConnectionData;
use crate::ConnectionTool;
use crate::Tool;

fn quadratic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

pub fn collect_connections(
    world: &hecs::World,
    tool: &Tool,
    connection_tool: &Option<ConnectionTool>,
) -> Vec<ConnectionData> {
    let connection_info: Vec<(Entity, Entity, Entity, f32, bool)> = world
        .query::<&Connection>()
        .iter()
        .map(|(e, c)| {
            let is_voltmeter = world.get::<&Voltmeter>(e).is_ok();
            (
                e,
                c.from,
                c.to,
                c.strength as f32,
                if is_voltmeter { false } else { c.directional },
            )
        })
        .collect();

    let connection_pairs: HashSet<(Entity, Entity)> = connection_info
        .iter()
        .map(|(_, from, to, _, _)| (*from, *to))
        .collect();

    let mut connections: Vec<ConnectionData> = Vec::new();

    for &(_edge_entity, from, to, strength, directional) in &connection_info {
        let start = world
            .get::<&Position>(from)
            .expect("Connection from broken")
            .position;
        let end = world
            .get::<&Position>(to)
            .expect("Connection to broken")
            .position;
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
        let (start_color, end_color) = if world.get::<&CurrentClamp>(from).is_ok() {
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
        };

        let is_reciprocal = connection_pairs.contains(&(to, from));
        let dir_val = if directional { 1.0 } else { 0.0 };

        if is_reciprocal {
            let segments = BEZIER_SEGMENTS;
            let diff = end - start;
            let up = Vec3::new(0.0, 1.0, 0.0);
            let right = diff.cross(up);
            let bend_amount = BEZIER_BEND_FRACTION * diff.length();
            let control = (start + end) * 0.5 + right.normalize_or_zero() * bend_amount;

            for i in 0..segments {
                let t0 = i as f32 / segments as f32;
                let t1 = (i + 1) as f32 / segments as f32;
                let p0 = quadratic_bezier(start, control, end, t0);
                let p1 = quadratic_bezier(start, control, end, t1);
                let c0 = start_color.lerp(end_color, t0);
                let c1 = start_color.lerp(end_color, t1);
                let is_last = i == segments - 1;
                connections.push(ConnectionData {
                    position_a: p0,
                    position_b: p1,
                    strength,
                    directional: if is_last { dir_val } else { 0.0 },
                    start_color: c0,
                    end_color: c1,
                    _padding: Default::default(),
                });
            }
        } else {
            connections.push(ConnectionData {
                position_a: start,
                position_b: end,
                strength,
                directional: dir_val,
                start_color,
                end_color,
                _padding: Default::default(),
            });
        }
    }

    if *tool == Tool::StaticConnection {
        if let Some(connection) = connection_tool {
            connections.push(ConnectionData {
                position_a: connection.start,
                position_b: connection.end,
                strength: 1.0,
                directional: 1.0,
                start_color: Vec3::new(0.8, 0.8, 0.8),
                end_color: Vec3::new(0.8, 0.8, 0.8),
                _padding: Default::default(),
            });
        }
    }

    connections
}
