use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

/// Optional per-entity color override for connection lines.
/// When present on the `from` entity of a Connection, this color is used
/// instead of the default voltage-based color.
#[derive(Clone, Debug)]
pub struct ConnectionColor(pub Vec3);

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
pub struct Inhibitory;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NeuronType {
    Excitatory,
    Inhibitory,
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

/// Overrides the visual sphere radius used for connection endpoint insets.
/// Attach this to any entity whose rendered radius differs from NODE_RADIUS.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VisualRadius {
    pub radius: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Vec3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

// Game components

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PlayerId {
    Player1,
    Player2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ownership {
    pub player: PlayerId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OriginNeuron {
    pub player: PlayerId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetabolicState {
    pub energy: f64,
    pub max_energy: f64,
}

impl Default for MetabolicState {
    fn default() -> Self {
        Self {
            energy: crate::constants::DEFAULT_NEURON_ENERGY,
            max_energy: crate::constants::MAX_NEURON_ENERGY,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceNode {
    pub emission_rate: f64,
    pub radius: f32,
    pub remaining: f64,
}

impl Default for ResourceNode {
    fn default() -> Self {
        Self {
            emission_rate: crate::constants::RESOURCE_EMISSION_RATE,
            radius: crate::constants::RESOURCE_NODE_RADIUS,
            remaining: f64::INFINITY,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dead;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MembraneSegment;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepolarizationBlock {
    pub time_above_threshold: f64,
    pub blocked: bool,
    pub recovery_timer: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SubstrateZoneType {
    /// High potassium - lowers firing threshold, making neurons more excitable
    HighPotassium,
    /// High magnesium - weakens synaptic transmission (reduces connection strength)
    HighMagnesium,
    /// Electrical noise - injects random current into neurons
    Noise,
    /// Toxic/damage - drains energy from neurons in the zone
    Damage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubstrateZone {
    pub zone_type: SubstrateZoneType,
    pub radius: f32,
}
