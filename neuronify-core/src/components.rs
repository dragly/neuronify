use std::collections::HashSet;

use glam::Vec3;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

pub use neuronify_engine::units::*;

use crate::measurement::voltmeter::VoltageSeries;

pub trait Setting: hecs::Component + Copy + Default {
    fn of(world: &World) -> Self {
        world
            .query::<(Entity, &Self)>()
            .iter()
            .next()
            .map(|(_, setting)| *setting)
            .unwrap_or_default()
    }

    fn set_in(self, world: &mut World) {
        let existing = world.query_mut::<(Entity, &mut Self)>().into_iter().next();
        match existing {
            Some((_, setting)) => *setting = self,
            None => {
                world.spawn((self,));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeuronDisplay {
    pub voltage: bool,
    pub firing_rate: bool,
    pub true_proportions: bool,
}

impl Default for NeuronDisplay {
    fn default() -> Self {
        Self {
            voltage: true,
            firing_rate: false,
            true_proportions: false,
        }
    }
}

impl Setting for NeuronDisplay {}

#[derive(Clone, Debug, Default)]
pub struct FiringRate {
    pub rate: Hertz,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakyNeuron {
    pub capacitance: Nanofarads,
    pub resting_potential: Millivolts,
    pub threshold: Millivolts,
    pub initial_potential: Millivolts,
    pub voltage_clamped: bool,
    pub minimum_voltage: Millivolts,
    pub maximum_voltage: Millivolts,
    pub refractory_period: Milliseconds,
}

impl Default for LeakyNeuron {
    fn default() -> Self {
        Self {
            capacitance: Nanofarads(0.2),
            resting_potential: Millivolts(-70.0),
            threshold: Millivolts(-55.0),
            initial_potential: Millivolts(-80.0),
            voltage_clamped: true,
            minimum_voltage: Millivolts(-90.0),
            maximum_voltage: Millivolts(60.0),
            refractory_period: Milliseconds(2.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NodeState {
    pub voltage: Millivolts,
    pub gates: Vec<f64>,
    pub since_spike: Milliseconds,
}

impl NodeState {
    pub fn at(voltage: Millivolts) -> NodeState {
        NodeState {
            voltage,
            gates: Vec::new(),
            since_spike: Milliseconds::INFINITY,
        }
    }

    pub fn fired_within(&self, duration: Milliseconds) -> bool {
        self.since_spike < duration
    }
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState::at(crate::constants::RESTING_POTENTIAL)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct CableState {
    pub voltage: Vec<Millivolts>,
    pub gates: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub resistance: Megaohms,
}

impl Default for LeakCurrent {
    fn default() -> Self {
        Self {
            resistance: Megaohms(100.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdaptationCurrent {
    pub adaptation: Microsiemens,
    pub conductance: Microsiemens,
    pub time_constant: Milliseconds,
}

impl Default for AdaptationCurrent {
    fn default() -> Self {
        Self {
            adaptation: Microsiemens(0.01),
            conductance: Microsiemens::ZERO,
            time_constant: Milliseconds(500.0),
        }
    }
}

/// Neuronify 1's burst neuron. A spike after a quiet spell starts a boost current
/// that drains linearly and keeps the neuron firing until it runs out.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BurstCurrent {
    /// The current at the start of a burst
    pub peak: Nanoamperes,
    pub drain: NanoamperesPerMillisecond,
    pub boost: Nanoamperes,
}

impl Default for BurstCurrent {
    fn default() -> Self {
        Self {
            peak: Nanoamperes(1.2),
            drain: NanoamperesPerMillisecond(0.006),
            boost: Nanoamperes::ZERO,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentClamp {
    pub current_output: Nanoamperes,
}

impl Default for CurrentClamp {
    fn default() -> Self {
        Self {
            current_output: Nanoamperes(2.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlternatingCurrent {
    pub amplitude: Nanoamperes,
    pub frequency: Hertz,
}

impl Default for AlternatingCurrent {
    fn default() -> Self {
        Self {
            amplitude: Nanoamperes(0.3),
            frequency: Hertz(50.0),
        }
    }
}

impl AlternatingCurrent {
    pub fn at(&self, time: Milliseconds) -> Nanoamperes {
        self.amplitude * (std::f64::consts::TAU * self.frequency.cycles_in(time)).sin()
    }

    pub fn charge_until(&self, time: Milliseconds) -> Picocoulombs {
        let angular_frequency = std::f64::consts::TAU * self.frequency.to_kilohertz();
        if angular_frequency == Kilohertz::ZERO {
            return Picocoulombs::ZERO;
        }
        self.amplitude * (1.0 - (angular_frequency * time).cos()) * (1.0 / angular_frequency)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentSynapse {
    pub tau: Milliseconds,
    pub maximum_current: Nanoamperes,
    pub delay: Milliseconds,
    pub alpha_function: bool,
    pub exponential: f64,
    pub linear: f64,
    pub triggers: Vec<Milliseconds>,
}

impl Default for CurrentSynapse {
    fn default() -> Self {
        Self {
            tau: Milliseconds(2.0),
            maximum_current: Nanoamperes(6.0),
            delay: Milliseconds(5.0),
            alpha_function: false,
            exponential: 0.0,
            linear: 0.0,
            triggers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ImmediateFireSynapse;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConductanceSynapse {
    pub conductance: Microsiemens,
    pub delay: Milliseconds,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SynapseState {
    pub open: Microsiemens,
    pub pending: Vec<Milliseconds>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TouchSensor;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MeterSize {
    pub width: f32,
    pub height: f32,
}

impl Default for MeterSize {
    fn default() -> Self {
        Self {
            width: 8.0,
            height: 4.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Annotation {
    pub text: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Label {
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoteSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub example: String,
}

impl Default for NoteSize {
    fn default() -> Self {
        Self {
            width: 7.2,
            height: 4.8,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratorDynamics {
    pub time_since_fire: Milliseconds,
}

impl Default for GeneratorDynamics {
    fn default() -> Self {
        Self {
            time_since_fire: Milliseconds::INFINITY,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegularSpikeGenerator {
    pub frequency: Hertz,
}

impl Default for RegularSpikeGenerator {
    fn default() -> Self {
        Self {
            frequency: Hertz(20.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoissonGenerator {
    pub rate: Hertz,
}

impl Default for PoissonGenerator {
    fn default() -> Self {
        Self { rate: Hertz(20.0) }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum NeuronType {
    #[default]
    Excitatory,
    Inhibitory,
}

impl NeuronType {
    pub fn sign(self) -> f64 {
        match self {
            NeuronType::Excitatory => 1.0,
            NeuronType::Inhibitory => -1.0,
        }
    }
}

pub fn neuron_type(world: &hecs::World, entity: Entity) -> NeuronType {
    world
        .get::<&NeuronType>(entity)
        .map(|neuron_type| *neuron_type)
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Cable {
    pub length: Micrometres,
    pub diameter: Micrometres,
}

pub use neuronify_engine::Membrane;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Selectable {
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticConnectionSource {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deletable {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dendrite;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Myelin;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gliding {
    pub from: Vec3,
    pub since: Seconds,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Vec3,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpatialDynamics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub strength: f64,
    pub directional: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Selected;

impl Selected {
    pub fn entity(world: &World) -> Option<Entity> {
        world.query::<Entity>().with::<&Selected>().iter().next()
    }

    pub fn set(world: &mut World, entity: Option<Entity>) {
        let previous: Vec<Entity> = world
            .query_mut::<Entity>()
            .with::<&Selected>()
            .into_iter()
            .collect();
        for previous in previous {
            world.remove_one::<Selected>(previous).unwrap();
        }
        if let Some(entity) = entity {
            let _ = world.insert_one(entity, Selected);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ConnectionView {
    #[default]
    All,
    LastSelected,
    None,
}

impl Setting for ConnectionView {}

impl ConnectionView {
    pub fn shows(&self, connection: &Connection, selected: Option<Entity>) -> bool {
        match self {
            ConnectionView::All => true,
            ConnectionView::LastSelected => {
                selected.is_some_and(|entity| connection.from == entity || connection.to == entity)
            }
            ConnectionView::None => false,
        }
    }
}

pub fn hidden_connections(world: &World) -> HashSet<Entity> {
    let view = ConnectionView::of(world);
    let selected = Selected::entity(world);
    world
        .query::<(Entity, &Connection)>()
        .without::<hecs::Or<&Cable, &ConductanceSynapse>>()
        .without::<&VoltageSeries>()
        .iter()
        .filter(|(entity, _)| Some(*entity) != selected)
        .filter(|(_, connection)| !view.shows(connection, selected))
        .map(|(entity, _)| entity)
        .collect()
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Compartment;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Temperature {
    pub celsius: Celsius,
}

impl Default for Temperature {
    fn default() -> Self {
        Temperature {
            celsius: crate::constants::DEFAULT_TEMPERATURE,
        }
    }
}

impl Setting for Temperature {}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum Preparation {
    #[default]
    Squid,
    Mammal,
}

impl Setting for Preparation {}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Randomness {
    pub state: u64,
}

impl Default for Randomness {
    fn default() -> Self {
        Randomness {
            state: 0x5eed_0fce_1100,
        }
    }
}

impl Setting for Randomness {}
