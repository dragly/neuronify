use glam::Vec3;
use hecs::{Entity, EntityBuilder, World};

use crate::components::*;
use crate::note;
use crate::simulation::model::cable_between;
use crate::stimuli::Stimulus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    ExcitatoryNeuron,
    InhibitoryNeuron,
    AdaptiveNeuron,
    BurstNeuron,
    Stimulus(Stimulus),
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

    pub fn is_stimulus(self) -> bool {
        matches!(self, Element::Stimulus(_))
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
            Element::Stimulus(stimulus) => stimulus.spawn(world, position.position),
            Element::Note => note::spawn(world, position.position),
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
    match Stimulus::of(world, from) {
        Some(Stimulus::CurrentSource | Stimulus::AlternatingCurrentSource) => {
            world.spawn((connection, Deletable {}))
        }
        Some(Stimulus::TouchSensor) => {
            world.spawn((connection, Deletable {}, ImmediateFireSynapse))
        }
        Some(
            Stimulus::RegularSpikeGenerator
            | Stimulus::PoissonGenerator
            | Stimulus::VisualInput
            | Stimulus::ReceptiveField,
        )
        | None => spawn_synapse(world, from, to),
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
    let ending = ConductanceSynapse {
        delay: Preparation::of(world).ending_delay(),
        ..Default::default()
    };
    world.spawn((
        Connection {
            from,
            to,
            strength: 1.0,
            directional,
        },
        Deletable {},
        ending,
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
