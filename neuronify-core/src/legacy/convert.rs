use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;
use crate::measurement::voltmeter::{RollingWindow, VoltageSeries, Voltmeter};
use crate::{Connection, Deletable, NeuronType, Position};

use super::{LegacyEdge, LegacyNode, LegacySimulation};

fn convert_position(x: f64, y: f64) -> Vec3 {
    let scale = 50.0 / 2.0;
    Vec3::new(-(y as f32 - 540.0) / scale, 0.0, (x as f32 - 960.0) / scale)
}

pub fn spawn_legacy_simulation(world: &mut World, sim: &LegacySimulation) -> Vec<Entity> {
    let mut node_entities = Vec::new();

    for node in &sim.nodes {
        let entity = spawn_node(world, node);
        node_entities.push(entity);
    }

    for edge in &sim.edges {
        if edge.from < node_entities.len() && edge.to < node_entities.len() {
            spawn_edge(world, edge, &node_entities);
        }
    }

    node_entities
}

fn spawn_node(world: &mut World, node: &LegacyNode) -> Entity {
    let pos = Position {
        position: convert_position(node.x, node.y),
    };

    match node.filename.as_str() {
        "neurons/LeakyNeuron.qml" => spawn_leaky_neuron(world, node, pos),
        "neurons/LeakyInhibitoryNeuron.qml" => {
            let entity = spawn_leaky_neuron(world, node, pos);
            world.insert_one(entity, Inhibitory).unwrap();
            entity
        }
        "neurons/AdaptationNeuron.qml" => spawn_adaptation_neuron(world, node, pos),
        "generators/CurrentClamp.qml" => {
            let current = node.engine.current_output.unwrap_or(2e-9);
            world.spawn((
                pos,
                CurrentClamp {
                    current_output: current,
                },
                Deletable {},
            ))
        }
        "sensors/TouchSensor.qml" => {
            world.spawn((pos, TouchSensor, GeneratorDynamics::default(), Deletable {}))
        }
        "meters/Voltmeter.qml" => world.spawn((
            pos,
            Voltmeter {},
            VoltageSeries {
                measurements: RollingWindow::new(10000),
                spike_times: Vec::new(),
            },
            VoltmeterSize::default(),
            Deletable {},
        )),
        "meters/SpikeDetector.qml" => world.spawn((pos,)),
        s if s.starts_with("annotations/") => {
            let text = node.text.clone().unwrap_or_default();
            world.spawn((pos, Annotation { text }))
        }
        _ => world.spawn((pos,)),
    }
}

fn spawn_leaky_neuron(world: &mut World, node: &LegacyNode, pos: Position) -> Entity {
    let e = &node.engine;

    let neuron = LeakyNeuron {
        capacitance: e.capacitance.unwrap_or(2e-10),
        resting_potential: e.resting_potential.unwrap_or(-0.07),
        threshold: e.threshold.unwrap_or(-0.055),
        initial_potential: e.initial_potential.unwrap_or(-0.08),
        voltage_clamped: e.voltage_clamped.unwrap_or(true),
        minimum_voltage: e.minimum_voltage.unwrap_or(-0.09),
        maximum_voltage: e.maximum_voltage.unwrap_or(0.06),
    };

    let dynamics = LeakyDynamics {
        voltage: e.voltage.unwrap_or(neuron.resting_potential),
        received_currents: 0.0,
        fired: false,
        refractory_period: e.refractory_period.unwrap_or(0.002),
        time_since_fire: f64::INFINITY,
        enabled: true,
    };

    let leak = LeakCurrent {
        resistance: e.resistance.unwrap_or(1e8),
        current: 0.0,
    };

    let neuron_type = if node.inhibitory {
        NeuronType::Inhibitory
    } else {
        NeuronType::Excitatory
    };

    let entity = world.spawn((pos, neuron, dynamics, leak, neuron_type, Deletable {}));

    if node.inhibitory {
        world.insert_one(entity, Inhibitory).unwrap();
    }

    entity
}

fn spawn_adaptation_neuron(world: &mut World, node: &LegacyNode, pos: Position) -> Entity {
    let entity = spawn_leaky_neuron(world, node, pos);

    let e = &node.engine;
    let adapt = AdaptationCurrent {
        adaptation: e.adaptation.unwrap_or(1e-8),
        conductance: e.conductance.unwrap_or(0.0),
        time_constant: e.time_constant.unwrap_or(0.5),
        current: 0.0,
    };

    world.insert_one(entity, adapt).unwrap();
    entity
}

fn spawn_edge(world: &mut World, edge: &LegacyEdge, node_entities: &[Entity]) {
    let from = node_entities[edge.from];
    let to = node_entities[edge.to];

    let connection = Connection {
        from,
        to,
        strength: 1.0,
        directional: true,
    };

    match edge.filename.as_str() {
        "edges/CurrentSynapse.qml" => {
            let e = &edge.engine;
            let synapse = CurrentSynapse {
                tau: e.tau.unwrap_or(0.002),
                maximum_current: e.maximum_current.unwrap_or(3e-9),
                delay: e.delay.unwrap_or(0.005),
                alpha_function: e.alpha_function.unwrap_or(false),
                exponential: e.exponential.unwrap_or(0.0),
                linear: e.linear.unwrap_or(0.0),
                triggers: Vec::new(),
                time: 0.0,
                current_output: 0.0,
            };
            world.spawn((connection, synapse, Deletable {}));
        }
        "edges/ImmediateFireSynapse.qml" => {
            world.spawn((connection, ImmediateFireSynapse::default(), Deletable {}));
        }
        "edges/MeterEdge.qml" => {
            let (meter, neuron) = if world.get::<&Voltmeter>(from).is_ok() {
                (from, to)
            } else if world.get::<&Voltmeter>(to).is_ok() {
                (to, from)
            } else {
                world.spawn((connection,));
                return;
            };
            world
                .insert(
                    meter,
                    (Connection {
                        from: neuron,
                        to: meter,
                        strength: 1.0,
                        directional: true,
                    },),
                )
                .unwrap();
        }
        "Edge.qml" => {
            if world.get::<&LeakyDynamics>(to).is_err() {
                return;
            }
            let e = &edge.engine;
            let synapse = CurrentSynapse {
                tau: e.tau.unwrap_or(0.002),
                maximum_current: e.maximum_current.unwrap_or(3e-9),
                delay: e.delay.unwrap_or(0.005),
                alpha_function: e.alpha_function.unwrap_or(false),
                exponential: e.exponential.unwrap_or(0.0),
                linear: e.linear.unwrap_or(0.0),
                triggers: Vec::new(),
                time: 0.0,
                current_output: 0.0,
            };
            world.spawn((connection, synapse, Deletable {}));
        }
        _ => {
            world.spawn((connection, Deletable {}));
        }
    }
}
