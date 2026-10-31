use glam::Vec3;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

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
}

impl Default for NeuronDisplay {
    fn default() -> Self {
        Self {
            voltage: true,
            firing_rate: false,
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
}

impl Default for LeakyNeuron {
    fn default() -> Self {
        Self {
            capacitance: 2e-10,
            resting_potential: -0.07,
            threshold: -0.055,
            initial_potential: -0.08,
            voltage_clamped: true,
            minimum_voltage: -0.09,
            maximum_voltage: 0.06,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakyDynamics {
    pub voltage: f64,
    pub received_currents: f64,
    pub fired: bool,
    pub refractory_period: f64,
    pub time_since_fire: f64,
    pub enabled: bool,
}

impl Default for LeakyDynamics {
    fn default() -> Self {
        Self {
            voltage: -0.07,
            received_currents: 0.0,
            fired: false,
            refractory_period: 0.002,
            time_since_fire: f64::INFINITY,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub resistance: f64,
    pub current: f64,
}

impl Default for LeakCurrent {
    fn default() -> Self {
        Self {
            resistance: 1e8,
            current: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdaptationCurrent {
    pub adaptation: f64,
    pub conductance: f64,
    pub time_constant: f64,
    pub current: f64,
}

impl Default for AdaptationCurrent {
    fn default() -> Self {
        Self {
            adaptation: 1e-8,
            conductance: 0.0,
            time_constant: 0.5,
            current: 0.0,
        }
    }
}

/// Neuronify 1's burst neuron. A spike after a quiet spell starts a boost current
/// that drains linearly and keeps the neuron firing until it runs out.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BurstCurrent {
    /// The current at the start of a burst
    pub peak: f64,
    /// How fast the current drains, in amperes per second
    pub drain: f64,
    pub boost: f64,
}

impl Default for BurstCurrent {
    fn default() -> Self {
        Self {
            peak: 1.2e-9,
            drain: 6e-9,
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
            current_output: 2e-9,
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
            amplitude: 300e-12,
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
    pub time: f64,
    pub current_output: f64,
}

impl Default for CurrentSynapse {
    fn default() -> Self {
        Self {
            tau: 0.002,
            maximum_current: 6e-9,
            delay: 0.005,
            alpha_function: false,
            exponential: 0.0,
            linear: 0.0,
            triggers: Vec::new(),
            time: 0.0,
            current_output: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImmediateFireSynapse {
    pub current_output: f64,
}

impl Default for ImmediateFireSynapse {
    fn default() -> Self {
        Self {
            current_output: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TouchSensor;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltmeterSize {
    pub width: f32,
    pub height: f32,
}

impl Default for VoltmeterSize {
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
    pub fired: bool,
    pub time_since_fire: f64,
}

impl Default for GeneratorDynamics {
    fn default() -> Self {
        Self {
            fired: false,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompartmentCurrent {
    pub capacitance: f64,
}

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Compartment {
    pub voltage: f64,
    pub m: f64,
    pub h: f64,
    pub n: f64,
    pub influence: f64,
    pub capacitance: f64,
    pub injected_current: f64,
    #[serde(default)]
    pub fire_impulse: f64,
}

impl Default for Compartment {
    fn default() -> Self {
        Self {
            voltage: -10.0,
            m: -0.625,
            h: 0.0,
            n: 0.0,
            influence: 0.0,
            capacitance: 1.0,
            injected_current: 0.0,
            fire_impulse: 0.0,
        }
    }
}
