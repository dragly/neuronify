use std::collections::HashSet;

use glam::Vec3;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::measurement::voltmeter::VoltageSeries;

pub trait Setting: hecs::Component + Copy + Default {
    fn of(world: &World) -> Self {
        world
            .query::<&Self>()
            .iter()
            .next()
            .map(|(_, setting)| *setting)
            .unwrap_or_default()
    }

    fn set_in(self, world: &mut World) {
        let existing = world.query_mut::<&mut Self>().into_iter().next();
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
    pub rate: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakyNeuron {
    pub capacitance: f64,
    pub resting_potential: f64,
    pub threshold: f64,
    pub initial_potential: f64,
    pub voltage_clamped: bool,
    pub minimum_voltage: f64,
    pub maximum_voltage: f64,
    pub refractory_period: f64,
}

impl Default for LeakyNeuron {
    fn default() -> Self {
        Self {
            capacitance: 0.2,
            resting_potential: -70.0,
            threshold: -55.0,
            initial_potential: -80.0,
            voltage_clamped: true,
            minimum_voltage: -90.0,
            maximum_voltage: 60.0,
            refractory_period: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NodeState {
    pub voltage: f64,
    pub gates: Vec<f64>,
    pub since_spike: f64,
}

impl NodeState {
    pub fn at(voltage: f64) -> NodeState {
        NodeState {
            voltage,
            gates: Vec::new(),
            since_spike: f64::INFINITY,
        }
    }

    pub fn fired_within(&self, duration: f64) -> bool {
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
    pub voltage: Vec<f64>,
    pub gates: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub resistance: f64,
}

impl Default for LeakCurrent {
    fn default() -> Self {
        Self { resistance: 100.0 }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdaptationCurrent {
    pub adaptation: f64,
    pub conductance: f64,
    pub time_constant: f64,
}

impl Default for AdaptationCurrent {
    fn default() -> Self {
        Self {
            adaptation: 0.01,
            conductance: 0.0,
            time_constant: 500.0,
        }
    }
}

/// Neuronify 1's burst neuron. A spike after a quiet spell starts a boost current
/// that drains linearly and keeps the neuron firing until it runs out.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BurstCurrent {
    /// The current at the start of a burst
    pub peak: f64,
    /// How fast the current drains, in nanoamperes per millisecond
    pub drain: f64,
    pub boost: f64,
}

impl Default for BurstCurrent {
    fn default() -> Self {
        Self {
            peak: 1.2,
            drain: 0.006,
            boost: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentClamp {
    pub current_output: f64,
}

impl Default for CurrentClamp {
    fn default() -> Self {
        Self {
            current_output: 2.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlternatingCurrent {
    pub amplitude: f64,
    pub frequency: f64,
}

impl Default for AlternatingCurrent {
    fn default() -> Self {
        Self {
            amplitude: 0.3,
            frequency: 50.0,
        }
    }
}

impl AlternatingCurrent {
    pub fn at(&self, time: f64) -> f64 {
        self.amplitude * (std::f64::consts::TAU * self.frequency * time).sin()
    }

    pub fn charge_until(&self, time: f64) -> f64 {
        let angular_frequency = std::f64::consts::TAU * self.frequency;
        if angular_frequency == 0.0 {
            return 0.0;
        }
        self.amplitude * (1.0 - (angular_frequency * time).cos()) / angular_frequency
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentSynapse {
    pub tau: f64,
    pub maximum_current: f64,
    pub delay: f64,
    pub alpha_function: bool,
    pub exponential: f64,
    pub linear: f64,
    pub triggers: Vec<f64>,
}

impl Default for CurrentSynapse {
    fn default() -> Self {
        Self {
            tau: 2.0,
            maximum_current: 6.0,
            delay: 5.0,
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
    pub conductance: f64,
    pub delay: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SynapseState {
    pub open: f64,
    pub pending: Vec<f64>,
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
    pub time_since_fire: f64,
}

impl Default for GeneratorDynamics {
    fn default() -> Self {
        Self {
            time_since_fire: f64::INFINITY,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegularSpikeGenerator {
    pub frequency: f64,
}

impl Default for RegularSpikeGenerator {
    fn default() -> Self {
        Self { frequency: 20.0 }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoissonGenerator {
    pub rate: f64,
}

impl Default for PoissonGenerator {
    fn default() -> Self {
        Self { rate: 20.0 }
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
    pub length: f64,
    pub diameter: f64,
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
        world
            .query::<()>()
            .with::<&Selected>()
            .iter()
            .next()
            .map(|(entity, _)| entity)
    }

    pub fn set(world: &mut World, entity: Option<Entity>) {
        let previous: Vec<Entity> = world
            .query_mut::<()>()
            .with::<&Selected>()
            .into_iter()
            .map(|(entity, _)| entity)
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
        .query::<&Connection>()
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
    pub celsius: f64,
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
