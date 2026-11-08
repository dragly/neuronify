use glam::Vec3;
use hecs::{Entity, EntityBuilder, World};

use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{RowOrder, SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{RollingWindow, VoltageSeries, Voltmeter, VoltmeterRange};
use crate::simulation::model::cable_between;

const VOLTMETER_MEASUREMENTS: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    ExcitatoryNeuron,
    InhibitoryNeuron,
    AdaptiveNeuron,
    BurstNeuron,
    CurrentSource,
    AlternatingCurrentSource,
    TouchSensor,
    RegularSpikeGenerator,
    PoissonGenerator,
    VisualInput,
    Note,
}

impl Element {
    pub fn is_neuron(self) -> bool {
        matches!(
            self,
            Element::ExcitatoryNeuron
                | Element::InhibitoryNeuron
                | Element::AdaptiveNeuron
                | Element::BurstNeuron
        )
    }

    pub fn is_input(self) -> bool {
        matches!(
            self,
            Element::CurrentSource
                | Element::AlternatingCurrentSource
                | Element::TouchSensor
                | Element::RegularSpikeGenerator
                | Element::PoissonGenerator
                | Element::VisualInput
        )
    }

    pub fn spawn(self, world: &mut World, position: Vec3) -> Entity {
        let position = Position { position };
        match self {
            Element::ExcitatoryNeuron => {
                world.spawn(neuron(position, NeuronType::Excitatory).build())
            }
            Element::InhibitoryNeuron => {
                world.spawn(neuron(position, NeuronType::Inhibitory).build())
            }
            Element::AdaptiveNeuron => world.spawn(
                neuron(position, NeuronType::Excitatory)
                    .add(AdaptationCurrent::default())
                    .build(),
            ),
            Element::BurstNeuron => world.spawn(
                neuron(position, NeuronType::Excitatory)
                    .add(BurstCurrent::default())
                    .build(),
            ),
            Element::CurrentSource => {
                world.spawn((position, CurrentClamp::default(), Deletable {}))
            }
            Element::AlternatingCurrentSource => world.spawn((
                position,
                CurrentClamp {
                    current_output: Nanoamperes::ZERO,
                },
                AlternatingCurrent::default(),
                Deletable {},
            )),
            Element::TouchSensor => world.spawn((
                position,
                TouchSensor,
                GeneratorDynamics::default(),
                Deletable {},
            )),
            Element::RegularSpikeGenerator => world.spawn((
                position,
                RegularSpikeGenerator::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
                Deletable {},
            )),
            Element::PoissonGenerator => world.spawn((
                position,
                PoissonGenerator::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
                Deletable {},
            )),
            Element::VisualInput => world.spawn((
                position,
                VisualInput::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
                Deletable {},
            )),
            Element::Note => world.spawn((
                position,
                Annotation {
                    text: "New note".to_string(),
                },
                NoteSize::default(),
                Deletable {},
            )),
        }
    }
}

fn neuron(position: Position, neuron_type: NeuronType) -> EntityBuilder {
    let mut neuron = EntityBuilder::new();
    neuron.add_bundle((
        position,
        LeakyNeuron::default(),
        NodeState::at(LeakyNeuron::default().resting_potential),
        LeakCurrent::default(),
        neuron_type,
        Label::default(),
        Deletable {},
    ));
    neuron
}

fn compartment(world: &World, position: Vec3, neuron_type: NeuronType) -> EntityBuilder {
    let mut compartment = EntityBuilder::new();
    compartment.add_bundle((
        Position { position },
        neuron_type,
        Compartment,
        NodeState::at(Preparation::of(world).resting_potential()),
        StaticConnectionSource {},
        Deletable {},
        Selectable { selected: false },
        SpatialDynamics::default(),
    ));
    compartment
}

pub fn spawn_compartment(world: &mut World, position: Vec3, neuron_type: NeuronType) -> Entity {
    world.spawn(compartment(world, position, neuron_type).build())
}

pub fn spawn_dendrite_compartment(
    world: &mut World,
    position: Vec3,
    neuron_type: NeuronType,
) -> Entity {
    world.spawn(
        compartment(world, position, neuron_type)
            .add(Dendrite)
            .build(),
    )
}

pub fn spawn_synapse(world: &mut World, from: Entity, to: Entity) -> Entity {
    world.spawn((
        Connection {
            from,
            to,
            strength: 1.0,
            directional: true,
        },
        CurrentSynapse::default(),
        Deletable {},
    ))
}

pub fn spawn_connection(world: &mut World, from: Entity, to: Entity) -> Entity {
    let connection = Connection {
        from,
        to,
        strength: 1.0,
        directional: true,
    };
    if world.get::<&CurrentClamp>(from).is_ok() {
        world.spawn((connection, Deletable {}))
    } else if world.get::<&TouchSensor>(from).is_ok() {
        world.spawn((connection, Deletable {}, ImmediateFireSynapse))
    } else {
        spawn_synapse(world, from, to)
    }
}

fn spawn_link(world: &mut World, from: Entity, to: Entity, myelinated: bool) -> Entity {
    let (cable, membrane) = cable_between(world, from, to, myelinated);
    let mut link = EntityBuilder::new();
    link.add_bundle((
        Connection {
            from,
            to,
            strength: 1.0,
            directional: false,
        },
        Deletable {},
        cable,
        membrane,
    ));
    if myelinated {
        link.add(Myelin);
    }
    world.spawn(link.build())
}

pub fn spawn_cable(world: &mut World, from: Entity, to: Entity, directional: bool) -> Entity {
    if !directional {
        return spawn_link(world, from, to, false);
    }
    world.spawn((
        Connection {
            from,
            to,
            strength: 1.0,
            directional,
        },
        Deletable {},
        ConductanceSynapse::default(),
        SynapseState::default(),
    ))
}

pub fn axon_ending(world: &World, terminal: Entity) -> Option<Entity> {
    world
        .query::<(Entity, &Connection)>()
        .with::<&ConductanceSynapse>()
        .iter()
        .find(|(_, connection)| connection.from == terminal)
        .map(|(ending, _)| ending)
}

pub fn spawn_internode(world: &mut World, from: Entity, to: Entity) -> Entity {
    spawn_link(world, from, to, true)
}

pub fn spawn_axon_cable(world: &mut World, from: Entity, to: Entity, myelinated: bool) -> Entity {
    match myelinated {
        true => spawn_internode(world, from, to),
        false => spawn_cable(world, from, to, false),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meter {
    Voltmeter,
    SpikeDetector,
    RatePlot,
    Loudspeaker,
}

impl Meter {
    pub fn of(world: &World, entity: Entity) -> Option<Meter> {
        let has = |meter: Meter| match meter {
            Meter::Voltmeter => world.satisfies::<&Voltmeter>(entity),
            Meter::SpikeDetector => world.satisfies::<&SpikeDetector>(entity),
            Meter::RatePlot => world.satisfies::<&RatePlot>(entity),
            Meter::Loudspeaker => world.satisfies::<&Loudspeaker>(entity),
        };
        [
            Meter::Voltmeter,
            Meter::SpikeDetector,
            Meter::RatePlot,
            Meter::Loudspeaker,
        ]
        .into_iter()
        .find(|&meter| has(meter))
    }

    pub fn size(self) -> Option<MeterSize> {
        match self {
            Meter::Voltmeter => Some(MeterSize::default()),
            Meter::SpikeDetector => Some(SpikeDetector::size()),
            Meter::RatePlot => Some(RatePlot::size()),
            Meter::Loudspeaker => None,
        }
    }

    pub fn spawn(self, world: &mut World, position: Vec3) -> Entity {
        let mut meter = EntityBuilder::new();
        meter.add_bundle((Position { position }, Deletable {}));
        if let Some(size) = self.size() {
            meter.add(size);
        }
        match self {
            Meter::Voltmeter => meter.add_bundle((Voltmeter {}, VoltmeterRange::default())),
            Meter::SpikeDetector => {
                meter.add_bundle((SpikeDetector::default(), RowOrder::default()))
            }
            Meter::RatePlot => meter.add_bundle((RatePlot::default(), RateSeries::default())),
            Meter::Loudspeaker => meter.add(Loudspeaker::default()),
        };
        world.spawn(meter.build())
    }

    pub fn measures(self, world: &World, target: Entity) -> bool {
        let neuron = world.satisfies::<&LeakyNeuron>(target);
        let compartment = world.satisfies::<&Compartment>(target);
        neuron || (self == Meter::Voltmeter && compartment)
    }

    pub fn wire(self, world: &mut World, target: Entity, meter: Entity) -> Entity {
        match self {
            Meter::Voltmeter => spawn_voltmeter_wire(world, target, meter),
            Meter::SpikeDetector => spawn_spike_detector_wire(world, target, meter),
            Meter::RatePlot => spawn_rate_plot_wire(world, target, meter),
            Meter::Loudspeaker => spawn_loudspeaker_wire(world, target, meter),
        }
    }
}

pub fn spawn_voltmeter(world: &mut World, target: Entity, position: Vec3) -> Entity {
    let voltmeter = Meter::Voltmeter.spawn(world, position);
    spawn_voltmeter_wire(world, target, voltmeter);
    voltmeter
}

pub fn spawn_spike_detector(world: &mut World, target: Entity, position: Vec3) -> Entity {
    let detector = Meter::SpikeDetector.spawn(world, position);
    spawn_spike_detector_wire(world, target, detector);
    detector
}

pub fn spawn_spike_detector_wire(world: &mut World, target: Entity, detector: Entity) -> Entity {
    world.spawn((
        Connection {
            from: target,
            to: detector,
            strength: 1.0,
            directional: true,
        },
        SpikeTrain::default(),
        Deletable {},
    ))
}

pub fn spawn_rate_plot(world: &mut World, target: Entity, position: Vec3) -> Entity {
    let plot = Meter::RatePlot.spawn(world, position);
    spawn_rate_plot_wire(world, target, plot);
    plot
}

pub fn spawn_rate_plot_wire(world: &mut World, target: Entity, plot: Entity) -> Entity {
    world.spawn((
        Connection {
            from: target,
            to: plot,
            strength: 1.0,
            directional: true,
        },
        SpikeTrain::default(),
        Deletable {},
    ))
}

pub fn spawn_loudspeaker(world: &mut World, target: Entity, position: Vec3) -> Entity {
    let speaker = Meter::Loudspeaker.spawn(world, position);
    spawn_loudspeaker_wire(world, target, speaker);
    speaker
}

pub fn spawn_loudspeaker_wire(world: &mut World, target: Entity, speaker: Entity) -> Entity {
    world.spawn((
        Connection {
            from: target,
            to: speaker,
            strength: 1.0,
            directional: true,
        },
        Deletable {},
    ))
}

pub fn spawn_voltmeter_wire(world: &mut World, target: Entity, voltmeter: Entity) -> Entity {
    world.spawn((
        Connection {
            from: target,
            to: voltmeter,
            strength: 1.0,
            directional: true,
        },
        VoltageSeries {
            measurements: RollingWindow::new(VOLTMETER_MEASUREMENTS),
            spike_times: Vec::new(),
        },
        Deletable {},
    ))
}
