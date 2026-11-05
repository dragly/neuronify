use hecs::{Entity, World};
use neuronify_engine::{ChannelId, Density, Membrane};

use crate::components::*;
use crate::constants::*;
use crate::rendering::cylinders::radius_from_soma_distance;

pub const AXON_DIAMETER: f64 = 0.15;
pub const INTERNODE_DIAMETER: f64 = 1.0;
pub const SOMA_DIAMETER: f64 = 20.0;
pub const SOMA_MEMBRANE_RESISTANCE: f64 = 20_000.0;
pub const DENDRITE_AXIAL_RESISTIVITY: f64 = 250.0;
pub const DENDRITE_CAPACITANCE: f64 = 1.0;
pub const DENDRITE_SODIUM: f64 = 0.06;
pub const DENDRITE_POTASSIUM: f64 = 0.018;
pub const MYELIN_CAPACITANCE: f64 = 0.02;
pub const MYELIN_LEAK: f64 = 1e-6;
pub const RANVIER_LENGTH: f64 = 1.0;
pub const RANVIER_SODIUM: f64 = 1.2;
pub const RANVIER_POTASSIUM: f64 = 0.36;
pub const RANVIER_LEAK: f64 = 0.003;
pub const ENDING_CONDUCTANCE: f64 = 0.0055;
pub const SOMA_ENDING_SCALE: f64 = 8.0;
pub const INHIBITORY_ENDING_SCALE: f64 = 40.0;
pub const ENDING_DELAY: f64 = 0.5;
pub const EXCITATORY_TAU: f64 = 2.0;
pub const INHIBITORY_TAU: f64 = 6.0;
pub const EXCITATORY_REVERSAL: f64 = 0.0;
pub const INHIBITORY_REVERSAL: f64 = -70.0;
pub const PULSE_CURRENT: f64 = 2.0;
pub const PULSE_DURATION: f64 = 0.25;
pub const AXON_PULSE_FACTOR: f64 = 3.0;
pub const SHORTEST_PULSE_INTERVAL: f64 = 4.0;
pub const RANVIER_PULSE_SCALE: f64 = 0.1;
pub const DRAWN_RADIUS_PER_ROOT_MICROMETRE: f32 = 0.173;

impl Default for ConductanceSynapse {
    fn default() -> Self {
        ConductanceSynapse {
            conductance: ENDING_CONDUCTANCE,
            delay: ENDING_DELAY,
        }
    }
}

fn sodium_and_potassium(
    [sodium, potassium]: [ChannelId; 2],
    sodium_density: f64,
    potassium_density: f64,
) -> Vec<Density> {
    vec![
        Density {
            channel: sodium,
            conductance: sodium_density,
            reversal: 50.0,
        },
        Density {
            channel: potassium,
            conductance: potassium_density,
            reversal: -77.0,
        },
    ]
}

const SQUID: [ChannelId; 2] = [ChannelId::SquidSodium, ChannelId::SquidPotassium];
const SLOW: [ChannelId; 2] = [ChannelId::SlowSodium, ChannelId::SlowPotassium];

pub fn cable_conductance(diameter: f64) -> f64 {
    diameter.powf(1.5) / 2.0
}

pub fn axon_membrane() -> Membrane {
    Membrane::squid()
}

pub fn dendrite_membrane() -> Membrane {
    Membrane {
        capacitance: DENDRITE_CAPACITANCE,
        axial_resistivity: DENDRITE_AXIAL_RESISTIVITY,
        leak: 1.0 / SOMA_MEMBRANE_RESISTANCE,
        leak_reversal: -70.0,
        channels: sodium_and_potassium(SLOW, DENDRITE_SODIUM, DENDRITE_POTASSIUM),
    }
}

pub fn myelin_membrane() -> Membrane {
    Membrane {
        capacitance: MYELIN_CAPACITANCE,
        axial_resistivity: 35.4,
        leak: MYELIN_LEAK,
        leak_reversal: RESTING_POTENTIAL,
        channels: Vec::new(),
    }
}

pub fn ranvier_membrane() -> Membrane {
    Membrane {
        capacitance: 1.0,
        axial_resistivity: 35.4,
        leak: RANVIER_LEAK,
        leak_reversal: -54.3,
        channels: sodium_and_potassium(SQUID, RANVIER_SODIUM, RANVIER_POTASSIUM),
    }
}

pub fn soma_membrane(resting_potential: f64) -> Membrane {
    Membrane::passive(
        SOMA_MEMBRANE_RESISTANCE,
        resting_potential,
        DENDRITE_AXIAL_RESISTIVITY,
    )
}

pub fn diameter_of_drawn_radius(radius: f32) -> f64 {
    (radius / DRAWN_RADIUS_PER_ROOT_MICROMETRE).powi(2) as f64
}

fn position(world: &World, entity: Entity) -> Option<glam::Vec3> {
    world.get::<&Position>(entity).ok().map(|p| p.position)
}

fn dendritic(world: &World, entity: Entity) -> bool {
    world.satisfies::<&Dendrite>(entity).unwrap_or(false)
}

fn soma_distance(world: &World, at: glam::Vec3) -> f32 {
    world
        .query::<&Position>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|(_, soma)| soma.position.distance(at))
        .fold(f32::MAX, f32::min)
}

pub fn cable_between(
    world: &World,
    from: Entity,
    to: Entity,
    myelinated: bool,
) -> (Cable, Membrane) {
    if myelinated {
        let cable = Cable {
            length: INTERNODE_LENGTH as f64 * MICROMETRES_PER_UNIT,
            diameter: INTERNODE_DIAMETER,
        };
        return (cable, myelin_membrane());
    }
    let length = SEGMENT_LENGTH as f64 * MICROMETRES_PER_UNIT;
    if dendritic(world, from) || dendritic(world, to) {
        let radius = |end: Entity| {
            position(world, end)
                .map(|at| radius_from_soma_distance(soma_distance(world, at)))
                .unwrap_or(0.3)
        };
        let drawn = (radius(from) + radius(to)) / 2.0;
        let cable = Cable {
            length,
            diameter: diameter_of_drawn_radius(drawn),
        };
        return (cable, dendrite_membrane());
    }
    let cable = Cable {
        length,
        diameter: AXON_DIAMETER,
    };
    (cable, axon_membrane())
}

pub fn pulse_size(world: &World, node: Entity) -> f64 {
    world
        .query::<(&Connection, &Cable, Option<&Myelin>)>()
        .iter()
        .filter(|(_, (cable, ..))| cable.from == node || cable.to == node)
        .map(|(_, (_, cable, myelin))| {
            let scale = myelin.map_or(1.0, |_| RANVIER_PULSE_SCALE);
            scale * cable_conductance(cable.diameter)
        })
        .sum()
}

pub fn node_size(world: &World, node: Entity) -> f64 {
    world
        .query::<(&Connection, &Cable)>()
        .iter()
        .filter(|(_, (cable, _))| cable.from == node || cable.to == node)
        .map(|(_, (_, cable))| cable_conductance(cable.diameter))
        .sum()
}

pub fn ending_site_scale(world: &World, connection: &Connection) -> f64 {
    let site = match world
        .satisfies::<&LeakyNeuron>(connection.to)
        .unwrap_or(false)
    {
        true => SOMA_ENDING_SCALE,
        false => node_size(world, connection.to),
    };
    site * ending_kinetics(world, connection.from).scale
}

pub struct Kinetics {
    pub tau: f64,
    pub reversal: f64,
    pub scale: f64,
}

pub fn ending_kinetics(world: &World, from: Entity) -> Kinetics {
    match neuron_type(world, from) {
        NeuronType::Excitatory => Kinetics {
            tau: EXCITATORY_TAU,
            reversal: EXCITATORY_REVERSAL,
            scale: 1.0,
        },
        NeuronType::Inhibitory => Kinetics {
            tau: INHIBITORY_TAU,
            reversal: INHIBITORY_REVERSAL,
            scale: INHIBITORY_ENDING_SCALE,
        },
    }
}
