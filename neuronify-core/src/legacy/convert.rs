use glam::Vec3;
use hecs::{DynamicBundle, Entity, EntityBuilder, World};

use crate::components::*;
use crate::elements::{
    spawn_loudspeaker_wire, spawn_rate_plot_wire, spawn_spike_detector_wire, spawn_voltmeter_wire,
};
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::is_meter;
use crate::measurement::loudspeaker::{Loudspeaker, Sound};
use crate::measurement::spike_detector::{RowOrder, SpikeDetector};
use crate::measurement::visual_input::{ReceptiveField, VisualInput};
use crate::measurement::voltmeter::{Voltmeter, VoltmeterRange};
use crate::{Connection, Deletable, NeuronType, Position};

const MILLI: f64 = 1e3;
const MICRO: f64 = 1e6;
const NANO: f64 = 1e9;

use super::{LegacyEdge, LegacyNode, LegacySimulation};

const PIXELS_PER_UNIT: f32 = 50.0 / 2.0;
const NEXT_TUTORIAL: &str = "Next tutorial ›";
const NEXT_TUTORIAL_PIXELS: f64 = 196.0;
const LINK_SIZE: NoteSize = NoteSize {
    width: 7.2,
    height: 2.0,
};
const LEGACY_GENERATOR_RATE: Hertz = Hertz(500.0);
const LEGACY_METER_WIDTH: f64 = 320.0;
const LEGACY_METER_HEIGHT: f64 = 240.0;
const LEGACY_RETINA_SIZE: f64 = 240.0;

fn from_seconds(seconds: f64) -> Milliseconds {
    Seconds(seconds).to_milliseconds()
}

fn convert_position(x: f64, y: f64) -> Vec3 {
    Vec3::new(
        -(y as f32 - 540.0) / PIXELS_PER_UNIT,
        0.0,
        (x as f32 - 960.0) / PIXELS_PER_UNIT,
    )
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
    let mut element = element(node);
    if !node.label.is_empty() || element.has::<LeakyNeuron>() {
        element.add(Label {
            text: node.label.trim().to_string(),
        });
    }
    world.spawn(element.build())
}

fn builder(components: impl DynamicBundle) -> EntityBuilder {
    let mut builder = EntityBuilder::new();
    builder.add_bundle(components);
    builder
}

fn element(node: &LegacyNode) -> EntityBuilder {
    let pos = Position {
        position: convert_position(node.x, node.y),
    };

    match node.filename.as_str() {
        "neurons/LeakyNeuron.qml" => {
            let negative_output = node.engine.fire_output.is_some_and(|output| output < 0.0);
            leaky_neuron(node, pos, node.inhibitory || negative_output)
        }
        "neurons/LeakyInhibitoryNeuron.qml" => leaky_neuron(node, pos, true),
        "neurons/AdaptationNeuron.qml" => adaptation_neuron(node, pos, node.inhibitory),
        "neurons/AdaptationInhibitoryNeuron.qml" => adaptation_neuron(node, pos, true),
        "neurons/BurstNeuron.qml" => burst_neuron(node, pos, node.inhibitory),
        "neurons/BurstInhibitoryNeuron.qml" => burst_neuron(node, pos, true),
        "generators/CurrentClamp.qml" => {
            let current = node
                .engine
                .current_output
                .map_or(Nanoamperes(2.0), |current| Nanoamperes(current * NANO));
            builder((
                pos,
                CurrentClamp {
                    current_output: current,
                },
                Deletable {},
            ))
        }
        "generators/ACClamp.qml" => {
            let default = AlternatingCurrent::default();
            let alternating = AlternatingCurrent {
                amplitude: node
                    .engine
                    .amplitude
                    .map_or(default.amplitude, |amplitude| Nanoamperes(amplitude * NANO)),
                frequency: node.engine.frequency.map_or(default.frequency, Hertz),
            };
            let clamp = CurrentClamp {
                current_output: Nanoamperes::ZERO,
            };
            builder((pos, clamp, alternating, Deletable {}))
        }
        "generators/RegularSpikeGenerator.qml" => spike_generator(
            node,
            pos,
            RegularSpikeGenerator {
                frequency: node.engine.rate.map_or(LEGACY_GENERATOR_RATE, Hertz),
            },
        ),
        "generators/IrregularSpikeGenerator.qml" => spike_generator(
            node,
            pos,
            PoissonGenerator {
                rate: node.engine.rate.map_or(LEGACY_GENERATOR_RATE, Hertz),
            },
        ),
        "sensors/TouchSensor.qml" => {
            builder((pos, TouchSensor, GeneratorDynamics::default(), Deletable {}))
        }
        "meters/SpikeDetector.qml" => {
            let (left_middle, size) = meter_frame(node);
            let detector = SpikeDetector {
                time_range: node
                    .time_range
                    .map_or(SpikeDetector::default().time_range, from_seconds),
                ..Default::default()
            };
            builder((
                left_middle,
                detector,
                RowOrder::default(),
                size,
                Deletable {},
            ))
        }
        "sensors/Retina.qml" => {
            let center = Position {
                position: convert_position(
                    node.x + LEGACY_RETINA_SIZE / 2.0,
                    node.y + LEGACY_RETINA_SIZE / 2.0,
                ),
            };
            let default = VisualInput::default();
            let input = VisualInput {
                field: receptive_field(node),
                sensitivity: node.sensitivity.map_or(default.sensitivity, Hertz),
                show_field: node.plot_kernel.unwrap_or(default.show_field),
                ..default
            };
            builder((
                center,
                input,
                GeneratorDynamics::default(),
                legacy_type(node.inhibitory),
                Deletable {},
            ))
        }
        "meters/Loudspeaker.qml" => {
            let speaker = Loudspeaker {
                sound: Sound::from_legacy(node.source.as_deref().unwrap_or_default()),
                ..Default::default()
            };
            builder((pos, speaker, Deletable {}))
        }
        "meters/RatePlot.qml" => {
            let (left_middle, size) = meter_frame(node);
            let default = RatePlot::default();
            let plot = RatePlot {
                window: node.window_duration.map_or(default.window, from_seconds),
                kernel_width: node
                    .temporal_resolution
                    .map_or(default.kernel_width, from_seconds),
                minimum: node.minimum_value.map_or(default.minimum, Hertz),
                maximum: node.maximum_value.map_or(default.maximum, Hertz),
            };
            builder((left_middle, plot, RateSeries::default(), size, Deletable {}))
        }
        "meters/Voltmeter.qml" => {
            let (left_middle, size) = meter_frame(node);
            let default_range = VoltmeterRange::default();
            let range = VoltmeterRange {
                minimum: node.minimum_value.map_or(default_range.minimum, Millivolts),
                maximum: node.maximum_value.map_or(default_range.maximum, Millivolts),
            };
            builder((left_middle, Voltmeter {}, size, range, Deletable {}))
        }
        "annotations/NextTutorial.qml" => builder((
            Position {
                position: convert_position(
                    node.x + NEXT_TUTORIAL_PIXELS / 2.0,
                    node.y + NEXT_TUTORIAL_PIXELS / 2.0,
                ),
            },
            Annotation {
                text: NEXT_TUTORIAL.to_string(),
            },
            LINK_SIZE,
            Link {
                example: node.target.clone().unwrap_or_default(),
            },
            Deletable {},
        )),
        "annotations/Note.qml" => {
            let text = node.text.clone().unwrap_or_default();
            let default_size = NoteSize::default();
            let width = node
                .width
                .unwrap_or((default_size.width * PIXELS_PER_UNIT).into());
            let height = node
                .height
                .unwrap_or((default_size.height * PIXELS_PER_UNIT).into());
            // QML places an item by its top left corner, but a note is drawn around its position
            let center = Position {
                position: convert_position(node.x + width / 2.0, node.y + height / 2.0),
            };
            let size = NoteSize {
                width: width as f32 / PIXELS_PER_UNIT,
                height: height as f32 / PIXELS_PER_UNIT,
            };
            builder((center, Annotation { text }, size, Deletable {}))
        }
        _ => builder((pos,)),
    }
}

fn wire_meter(world: &mut World, meter: Entity, neuron: Entity) {
    if world.get::<&SpikeDetector>(meter).is_ok() {
        spawn_spike_detector_wire(world, neuron, meter);
    } else if world.get::<&RatePlot>(meter).is_ok() {
        spawn_rate_plot_wire(world, neuron, meter);
    } else if world.get::<&Loudspeaker>(meter).is_ok() {
        spawn_loudspeaker_wire(world, neuron, meter);
    } else {
        spawn_voltmeter_wire(world, neuron, meter);
    }
}

fn receptive_field(node: &LegacyNode) -> ReceptiveField {
    let property = |name: &str| {
        node.kernel_properties
            .as_ref()
            .and_then(|properties| properties.get(name))
    };
    let number = |name: &str| property(name).and_then(|v| v.as_f64()).unwrap_or(0.0);
    match node.kernel_type.as_deref() {
        Some("kernels/DogKernel.qml") => ReceptiveField::CenterSurround {
            off_center: property("isOffCenter")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            x: number("xOffset"),
            y: number("yOffset"),
        },
        Some("kernels/RectangularKernel.qml") => ReceptiveField::Edge {
            angle: number("orientation"),
        },
        _ => ReceptiveField::Orientation {
            angle: number("theta"),
            x: number("xOffset"),
            y: number("yOffset"),
        },
    }
}

fn meter_frame(node: &LegacyNode) -> (Position, MeterSize) {
    let width = node.width.unwrap_or(LEGACY_METER_WIDTH);
    let height = node.height.unwrap_or(LEGACY_METER_HEIGHT);
    let left_middle = Position {
        position: convert_position(node.x, node.y + height / 2.0),
    };
    let size = MeterSize {
        width: width as f32 / PIXELS_PER_UNIT,
        height: height as f32 / PIXELS_PER_UNIT,
    };
    (left_middle, size)
}

fn spike_generator(
    node: &LegacyNode,
    pos: Position,
    generator: impl hecs::Component,
) -> EntityBuilder {
    builder((
        pos,
        generator,
        GeneratorDynamics::default(),
        legacy_type(node.inhibitory),
        Deletable {},
    ))
}

fn legacy_type(inhibitory: bool) -> NeuronType {
    if inhibitory {
        NeuronType::Inhibitory
    } else {
        NeuronType::Excitatory
    }
}

fn leaky_neuron(node: &LegacyNode, pos: Position, inhibitory: bool) -> EntityBuilder {
    let e = &node.engine;

    let neuron = LeakyNeuron {
        capacitance: Nanofarads(e.capacitance.unwrap_or(2e-10) * NANO),
        resting_potential: Millivolts(e.resting_potential.unwrap_or(-0.07) * MILLI),
        threshold: Millivolts(e.threshold.unwrap_or(-0.055) * MILLI),
        initial_potential: Millivolts(e.initial_potential.unwrap_or(-0.08) * MILLI),
        voltage_clamped: e.voltage_clamped.unwrap_or(true),
        minimum_voltage: Millivolts(e.minimum_voltage.unwrap_or(-0.09) * MILLI),
        maximum_voltage: Millivolts(e.maximum_voltage.unwrap_or(0.06) * MILLI),
        refractory_period: Milliseconds(e.refractory_period.unwrap_or(0.002) * MILLI),
    };

    let state = NodeState::at(
        e.voltage
            .map_or(neuron.resting_potential, |v| Millivolts(v * MILLI)),
    );

    let leak = LeakCurrent {
        resistance: Megaohms(e.resistance.unwrap_or(1e8) / MICRO),
    };

    builder((
        pos,
        neuron,
        state,
        leak,
        legacy_type(inhibitory),
        Deletable {},
    ))
}

fn adaptation_neuron(node: &LegacyNode, pos: Position, inhibitory: bool) -> EntityBuilder {
    let mut neuron = leaky_neuron(node, pos, inhibitory);

    let e = &node.engine;
    let adapt = AdaptationCurrent {
        adaptation: Microsiemens(e.adaptation.unwrap_or(1e-8) * MICRO),
        conductance: Microsiemens(e.conductance.unwrap_or(0.0) * MICRO),
        time_constant: Milliseconds(e.time_constant.unwrap_or(0.5) * MILLI),
    };

    neuron.add(adapt);
    neuron
}

fn burst_neuron(node: &LegacyNode, pos: Position, inhibitory: bool) -> EntityBuilder {
    let mut neuron = leaky_neuron(node, pos, inhibitory);
    neuron.add(BurstCurrent::default());
    neuron
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

    let meter_and_neuron = if is_meter(world, from) {
        Some((from, to))
    } else if is_meter(world, to) {
        Some((to, from))
    } else {
        None
    };
    match edge.filename.as_str() {
        "edges/CurrentSynapse.qml" => {
            let e = &edge.engine;
            let synapse = CurrentSynapse {
                tau: Milliseconds(e.tau.unwrap_or(0.002) * MILLI),
                maximum_current: Nanoamperes(e.maximum_current.unwrap_or(3e-9) * NANO),
                delay: Milliseconds(e.delay.unwrap_or(0.005) * MILLI),
                alpha_function: e.alpha_function.unwrap_or(false),
                exponential: e.exponential.unwrap_or(0.0),
                linear: e.linear.unwrap_or(0.0),
                triggers: Vec::new(),
            };
            world.spawn((connection, synapse, Deletable {}));
        }
        "edges/ImmediateFireSynapse.qml" => {
            world.spawn((connection, ImmediateFireSynapse, Deletable {}));
        }
        "edges/MeterEdge.qml" | "Edge.qml" if meter_and_neuron.is_some() => {
            if let Some((meter, neuron)) = meter_and_neuron {
                wire_meter(world, meter, neuron);
            }
        }
        "edges/MeterEdge.qml" => {
            world.spawn((connection,));
        }
        "Edge.qml" => {
            if world.get::<&LeakyNeuron>(to).is_err() {
                return;
            }
            if world.get::<&CurrentClamp>(from).is_ok() {
                world.spawn((connection, Deletable {}));
                return;
            }
            let e = &edge.engine;
            let synapse = CurrentSynapse {
                tau: Milliseconds(e.tau.unwrap_or(0.002) * MILLI),
                maximum_current: Nanoamperes(e.maximum_current.unwrap_or(3e-9) * NANO),
                delay: Milliseconds(e.delay.unwrap_or(0.005) * MILLI),
                alpha_function: e.alpha_function.unwrap_or(false),
                exponential: e.exponential.unwrap_or(0.0),
                linear: e.linear.unwrap_or(0.0),
                triggers: Vec::new(),
            };
            world.spawn((connection, synapse, Deletable {}));
        }
        _ => {
            world.spawn((connection, Deletable {}));
        }
    }
}
