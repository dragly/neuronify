use serde::{Deserialize, Serialize};

/// Leaky integrate-and-fire neuron parameters (SI units).
/// Matches C++ NeuronEngine defaults.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicNeuron {
    pub capacitance: f64,
    pub resting_potential: f64,
    pub threshold: f64,
    pub initial_potential: f64,
    pub voltage_clamped: bool,
    pub minimum_voltage: f64,
    pub maximum_voltage: f64,
}

impl Default for ClassicNeuron {
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

/// Dynamic state of a classic neuron.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicNeuronDynamics {
    pub voltage: f64,
    pub received_currents: f64,
    pub fired: bool,
    pub refractory_period: f64,
    pub time_since_fire: f64,
    pub enabled: bool,
}

impl Default for ClassicNeuronDynamics {
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

/// Leak current: I = -(V - E_rest) / R
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicLeakCurrent {
    pub resistance: f64,
    pub current: f64,
}

impl Default for ClassicLeakCurrent {
    fn default() -> Self {
        Self {
            resistance: 1e8,
            current: 0.0,
        }
    }
}

/// Adaptation current with conductance-based dynamics.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicAdaptationCurrent {
    pub adaptation: f64,
    pub conductance: f64,
    pub time_constant: f64,
    pub current: f64,
}

impl Default for ClassicAdaptationCurrent {
    fn default() -> Self {
        Self {
            adaptation: 1e-8,
            conductance: 0.0,
            time_constant: 0.5,
            current: 0.0,
        }
    }
}

/// Constant DC current source.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicCurrentClamp {
    pub current_output: f64,
}

impl Default for ClassicCurrentClamp {
    fn default() -> Self {
        Self {
            current_output: 2e-9,
        }
    }
}

/// Current synapse with exponential or alpha-function decay.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicCurrentSynapse {
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

impl Default for ClassicCurrentSynapse {
    fn default() -> Self {
        Self {
            tau: 0.002,
            maximum_current: 3e-9,
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

/// Immediate fire synapse: delivers a huge instantaneous current on fire.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicImmediateFireSynapse {
    pub current_output: f64,
}

impl Default for ClassicImmediateFireSynapse {
    fn default() -> Self {
        Self { current_output: 0.0 }
    }
}

/// Marker for inhibitory neurons.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicInhibitory;

/// Touch sensor (no auto-fire in headless mode).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicTouchSensor;

/// Size of a classic voltmeter trace display.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicVoltmeterSize {
    pub width: f32,
    pub height: f32,
}

impl Default for ClassicVoltmeterSize {
    fn default() -> Self {
        Self {
            width: 8.0,
            height: 4.0,
        }
    }
}

/// Annotation/note for display.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassicAnnotation {
    pub text: String,
}
