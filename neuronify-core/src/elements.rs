use glam::Vec3;
use hecs::{Entity, EntityBuilder, World};

use crate::components::*;
use crate::constants::COUPLING_CAPACITANCE;
use crate::measurement::voltmeter::{RollingWindow, VoltageSeries, Voltmeter};

const VOLTMETER_MEASUREMENTS: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    ExcitatoryNeuron,
    InhibitoryNeuron,
    AdaptiveNeuron,
    BurstNeuron,
    CurrentSource,
    TouchSensor,
    RegularSpikeGenerator,
    PoissonGenerator,
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
                | Element::TouchSensor
                | Element::RegularSpikeGenerator
                | Element::PoissonGenerator
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
        LeakyDynamics::default(),
        LeakCurrent::default(),
        neuron_type,
        Deletable {},
    ));
    neuron
}

pub fn spawn_compartment(world: &mut World, position: Vec3, neuron_type: NeuronType) -> Entity {
    world.spawn((
        Position { position },
        neuron_type,
        Compartment::default(),
        StaticConnectionSource {},
        Deletable {},
        Selectable { selected: false },
        SpatialDynamics::default(),
    ))
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
    if world.get::<&CurrentClamp>(from).is_ok() {
        world.spawn((
            Connection {
                from,
                to,
                strength: 1.0,
                directional: true,
            },
            Deletable {},
        ))
    } else {
        spawn_synapse(world, from, to)
    }
}

pub fn spawn_cable(world: &mut World, from: Entity, to: Entity, directional: bool) -> Entity {
    world.spawn((
        Connection {
            from,
            to,
            strength: 1.0,
            directional,
        },
        Deletable {},
        CompartmentCurrent {
            capacitance: COUPLING_CAPACITANCE,
        },
    ))
}

pub fn axon_ending(world: &World, terminal: Entity) -> Option<Entity> {
    world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .find(|(_, connection)| connection.directional && connection.from == terminal)
        .map(|(ending, _)| ending)
}

pub fn spawn_voltmeter(world: &mut World, target: Entity, position: Vec3) -> Entity {
    let voltmeter = world.spawn((
        Voltmeter {},
        Position { position },
        VoltageSeries {
            measurements: RollingWindow::new(VOLTMETER_MEASUREMENTS),
            spike_times: Vec::new(),
        },
        Connection {
            from: target,
            to: Entity::DANGLING,
            strength: 1.0,
            directional: true,
        },
        VoltmeterSize::default(),
        Deletable {},
    ));
    world.get::<&mut Connection>(voltmeter).unwrap().to = voltmeter;
    voltmeter
}
