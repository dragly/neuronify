use glam::Vec3;
use hecs::Entity;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, ConnectionColor, Deletable, Inhibitory,
    LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronType, Position, Selectable, SpatialDynamics,
    StaticConnectionSource, VisualRadius, COUPLING_CAPACITANCE, NODE_RADIUS,
};
use crate::rendering::colors::glial_color;

use crate::components::*;

const DENDRITE_LENGTH: f32 = 2.5;
const DENDRITE_COMPARTMENTS: usize = 2;

/// Spawn a glial cell (astrocyte) with processes arranged radially.
pub fn spawn_glial(
    world: &mut hecs::World,
    position: Vec3,
    player: PlayerId,
    num_processes: usize,
) -> Entity {
    let soma = world.spawn((
        Position { position },
        GlialCell::default(),
        NeuronType::Excitatory,
        StaticConnectionSource {},
        Ownership { player },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.3 },
        ConnectionColor(glial_color()),
    ));

    for i in 0..num_processes {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / num_processes as f32;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin());

        let mut prev_entity = soma;
        for seg in 0..DENDRITE_COMPARTMENTS {
            let offset = direction * DENDRITE_LENGTH * (seg + 1) as f32;
            let comp_pos = position + offset;

            let compartment = world.spawn((
                Position { position: comp_pos },
                NeuronType::Excitatory,
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                GlialProcess,
                Ownership { player },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
                ConnectionColor(glial_color()),
            ));

            world.spawn((
                Connection {
                    from: prev_entity,
                    to: compartment,
                    strength: 1.0,
                    directional: false,
                },
                Deletable {},
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));

            prev_entity = compartment;
        }
    }

    soma
}

/// Spawn a neuron with dendrites arranged radially around the soma.
pub fn spawn_neuron(
    world: &mut hecs::World,
    position: Vec3,
    neuron_type: NeuronType,
    player: PlayerId,
) -> Entity {
    spawn_neuron_with_dendrites(world, position, neuron_type, player, 5)
}

/// Spawn a neuron soma with `num_dendrites` dendrite branches.
pub fn spawn_neuron_with_dendrites(
    world: &mut hecs::World,
    position: Vec3,
    neuron_type: NeuronType,
    player: PlayerId,
    num_dendrites: usize,
) -> Entity {
    let soma = world.spawn((
        Position { position },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        neuron_type.clone(),
        MetabolicState::default(),
        Ownership { player },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
    ));

    if matches!(neuron_type, NeuronType::Inhibitory) {
        let _ = world.insert_one(soma, Inhibitory);
    }

    for i in 0..num_dendrites {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / num_dendrites as f32;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin());

        let mut prev_entity = soma;
        for seg in 0..DENDRITE_COMPARTMENTS {
            let offset = direction * DENDRITE_LENGTH * (seg + 1) as f32;
            let comp_pos = position + offset;

            let compartment = world.spawn((
                Position { position: comp_pos },
                neuron_type.clone(),
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                Dendrite,
                Ownership { player },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
            ));

            world.spawn((
                Connection {
                    from: prev_entity,
                    to: compartment,
                    strength: 1.0,
                    directional: false,
                },
                Deletable {},
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));

            prev_entity = compartment;
        }
    }

    soma
}
