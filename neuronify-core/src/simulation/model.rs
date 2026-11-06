use std::ops::RangeInclusive;

use hecs::{Entity, World};
use neuronify_engine::{ChannelId, Density, Membrane};

use super::mammal;
use crate::components::*;
use crate::constants::*;

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
pub const MYELIN_G_RATIO: f64 = 0.7;
const ROOT_DRAWN_RADIUS: f32 = 0.30;
const TAPER: f32 = 0.85;
const TIP_DRAWN_RADIUS: f32 = 0.16;
const THINNEST_DRAWN_RADIUS: f32 = 0.11;

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

fn axon_membrane() -> Membrane {
    Membrane::squid()
}

fn dendrite_membrane() -> Membrane {
    Membrane {
        capacitance: DENDRITE_CAPACITANCE,
        axial_resistivity: DENDRITE_AXIAL_RESISTIVITY,
        leak: 1.0 / SOMA_MEMBRANE_RESISTANCE,
        leak_reversal: -70.0,
        channels: sodium_and_potassium(SLOW, DENDRITE_SODIUM, DENDRITE_POTASSIUM),
    }
}

fn myelin_membrane() -> Membrane {
    Membrane {
        capacitance: MYELIN_CAPACITANCE,
        axial_resistivity: 35.4,
        leak: MYELIN_LEAK,
        leak_reversal: RESTING_POTENTIAL,
        channels: Vec::new(),
    }
}

fn ranvier_membrane() -> Membrane {
    Membrane {
        capacitance: 1.0,
        axial_resistivity: 35.4,
        leak: RANVIER_LEAK,
        leak_reversal: -54.3,
        channels: sodium_and_potassium(SQUID, RANVIER_SODIUM, RANVIER_POTASSIUM),
    }
}

impl Preparation {
    pub fn celsius(self) -> f64 {
        match self {
            Preparation::Squid => DEFAULT_TEMPERATURE,
            Preparation::Mammal => mammal::CELSIUS,
        }
    }

    pub fn temperatures(self) -> RangeInclusive<f64> {
        match self {
            Preparation::Squid => 0.0..=35.0,
            Preparation::Mammal => 20.0..=42.0,
        }
    }

    pub fn resting_potential(self) -> f64 {
        match self {
            Preparation::Squid => RESTING_POTENTIAL,
            Preparation::Mammal => mammal::RESTING_POTENTIAL,
        }
    }

    pub fn axon_diameter(self) -> f64 {
        match self {
            Preparation::Squid => AXON_DIAMETER,
            Preparation::Mammal => mammal::AXON_DIAMETER,
        }
    }

    fn membrane(self, role: Role) -> Membrane {
        match (self, role) {
            (Preparation::Squid, Role::Axon) => axon_membrane(),
            (Preparation::Squid, Role::Dendrite) => dendrite_membrane(),
            (Preparation::Squid, Role::Myelin) => myelin_membrane(),
            (Preparation::Mammal, Role::Axon) => mammal::axon_membrane(),
            (Preparation::Mammal, Role::Dendrite) => mammal::dendrite_membrane(),
            (Preparation::Mammal, Role::Myelin) => mammal::myelin_membrane(),
        }
    }

    pub fn ranvier_membrane(self) -> Membrane {
        match self {
            Preparation::Squid => ranvier_membrane(),
            Preparation::Mammal => mammal::ranvier_membrane(),
        }
    }

    pub fn cable_ending_scale(self) -> f64 {
        match self {
            Preparation::Squid => 1.0,
            Preparation::Mammal => mammal::ENDING_CONDUCTANCE / ENDING_CONDUCTANCE,
        }
    }

    pub fn shortest_dendrite_pulse_interval(self) -> f64 {
        match self {
            Preparation::Squid => 0.0,
            Preparation::Mammal => mammal::SHORTEST_DENDRITE_PULSE_INTERVAL,
        }
    }
}

pub fn prepare(world: &mut World, preparation: Preparation) {
    preparation.set_in(world);
    Temperature {
        celsius: preparation.celsius(),
    }
    .set_in(world);
    let links: Vec<(Entity, Entity, Entity, bool)> = world
        .query::<(&Connection, Option<&Myelin>)>()
        .with::<&Cable>()
        .iter()
        .map(|(link, (connection, myelin))| {
            (link, connection.from, connection.to, myelin.is_some())
        })
        .collect();
    for (link, from, to, myelinated) in links {
        let role = role(world, from, to, myelinated);
        world.insert_one(link, preparation.membrane(role)).unwrap();
        if let Role::Axon = role {
            world.get::<&mut Cable>(link).unwrap().diameter = preparation.axon_diameter();
        }
    }
    rest(world, preparation);
}

pub fn rest(world: &mut World, preparation: Preparation) {
    let cables: Vec<Entity> = world
        .query::<()>()
        .with::<&CableState>()
        .iter()
        .map(|(cable, _)| cable)
        .collect();
    for cable in cables {
        world.remove_one::<CableState>(cable).unwrap();
    }
    for (_, (state, soma)) in world.query_mut::<(&mut NodeState, Option<&LeakyNeuron>)>() {
        match soma {
            Some(_) => state.gates.clear(),
            None => *state = NodeState::at(preparation.resting_potential()),
        }
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

fn dendrite_drawn_radius(soma_distance: f32) -> f32 {
    (ROOT_DRAWN_RADIUS * TAPER.powf(soma_distance / 3.0)).max(TIP_DRAWN_RADIUS)
}

pub fn drawn_radius(diameter: f64, display: &NeuronDisplay) -> f32 {
    match display.true_proportions {
        true => (diameter / 2.0 / MICROMETRES_PER_UNIT) as f32,
        false => {
            (DRAWN_RADIUS_PER_ROOT_MICROMETRE * (diameter as f32).sqrt()).max(THINNEST_DRAWN_RADIUS)
        }
    }
}

pub fn soma_end_radius(diameter: f64, display: &NeuronDisplay) -> f32 {
    match display.true_proportions {
        true => drawn_radius(diameter, display),
        false => ROOT_DRAWN_RADIUS,
    }
}

pub fn soma_diameter(world: &World, soma: Entity) -> f64 {
    let dendritic = world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .filter(|(_, cable)| cable.from == soma || cable.to == soma)
        .any(|(_, cable)| dendritic(world, cable.from) || dendritic(world, cable.to));
    if dendritic {
        return SOMA_DIAMETER;
    }
    let capacitance = world
        .get::<&LeakyNeuron>(soma)
        .map_or(LeakyNeuron::default().capacitance, |neuron| {
            neuron.capacitance
        });
    let area = capacitance * 1e5 / DENDRITE_CAPACITANCE;
    (area / std::f64::consts::PI).sqrt()
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

pub fn dendrite_diameter(world: &World, ends: [Option<glam::Vec3>; 2]) -> f64 {
    let [start, end] = ends.map(|end| {
        end.map_or(ROOT_DRAWN_RADIUS, |at| {
            dendrite_drawn_radius(soma_distance(world, at))
        })
    });
    diameter_of_drawn_radius((start + end) / 2.0)
}

#[derive(Clone, Copy)]
enum Role {
    Axon,
    Dendrite,
    Myelin,
}

fn role(world: &World, from: Entity, to: Entity, myelinated: bool) -> Role {
    if myelinated {
        Role::Myelin
    } else if dendritic(world, from) || dendritic(world, to) {
        Role::Dendrite
    } else {
        Role::Axon
    }
}

pub fn cable_between(
    world: &World,
    from: Entity,
    to: Entity,
    myelinated: bool,
) -> (Cable, Membrane) {
    let preparation = Preparation::of(world);
    let role = role(world, from, to, myelinated);
    let length = SEGMENT_LENGTH as f64 * MICROMETRES_PER_UNIT;
    let cable = match role {
        Role::Myelin => Cable {
            length: INTERNODE_LENGTH as f64 * MICROMETRES_PER_UNIT,
            diameter: INTERNODE_DIAMETER,
        },
        Role::Dendrite => Cable {
            length,
            diameter: dendrite_diameter(world, [from, to].map(|end| position(world, end))),
        },
        Role::Axon => Cable {
            length,
            diameter: preparation.axon_diameter(),
        },
    };
    (cable, preparation.membrane(role))
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
        false => node_size(world, connection.to) * Preparation::of(world).cable_ending_scale(),
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
