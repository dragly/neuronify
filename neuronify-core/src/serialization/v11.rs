use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use hecs::{ComponentError, Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct NeuronDisplay {
    pub voltage: bool,
    pub firing_rate: bool,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakyDynamics {
    pub voltage: f64,
    pub received_currents: f64,
    pub fired: bool,
    pub refractory_period: f64,
    pub time_since_fire: f64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub resistance: f64,
    pub current: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdaptationCurrent {
    pub adaptation: f64,
    pub conductance: f64,
    pub time_constant: f64,
    pub current: f64,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImmediateFireSynapse {
    pub current_output: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratorDynamics {
    pub fired: bool,
    pub time_since_fire: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Compartment {
    pub voltage: f64,
    pub m: f64,
    pub h: f64,
    pub n: f64,
    pub inhibition: f64,
    pub capacitance: f64,
    pub injected_current: f64,
    #[serde(default)]
    pub fire_impulse: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompartmentCurrent {
    pub capacitance: f64,
}

schema!(
    Position,
    LeakyNeuron,
    LeakyDynamics,
    LeakCurrent,
    AdaptationCurrent,
    CurrentClamp,
    CurrentSynapse,
    ImmediateFireSynapse,
    TouchSensor,
    GeneratorDynamics,
    RegularSpikeGenerator,
    PoissonGenerator,
    MeterSize,
    Annotation,
    NoteSize,
    StaticConnectionSource,
    Deletable,
    Selectable,
    Voltmeter,
    VoltageSeries,
    Connection,
    CompartmentCurrent,
    Compartment,
    SpatialDynamics,
    NeuronType,
    Dendrite,
    BurstCurrent,
    Link,
    NeuronDisplay,
    VoltmeterRange,
    Label,
    AlternatingCurrent,
    SpikeDetector,
    SpikeTrain,
    RatePlot,
    RateSeries,
    Loudspeaker,
    VisualInput,
    Myelin,
);

pub fn upgrade_from_v10(world: &mut World) -> Result<(), ComponentError> {
    let inputs: Vec<hecs::Entity> = world
        .query::<Entity>()
        .with::<&super::v9::VisualInput>()
        .iter()
        .collect();
    for input in inputs {
        let old = world.remove_one::<super::v9::VisualInput>(input)?;
        world.insert_one(
            input,
            VisualInput {
                field: old.field,
                sensitivity: Hertz(old.sensitivity),
                show_field: old.show_field,
                pattern: old.pattern,
                pattern_angle: old.pattern_angle,
                ..Default::default()
            },
        )?;
    }
    Ok(())
}
