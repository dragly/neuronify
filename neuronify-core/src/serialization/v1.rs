//! The first version of the save file format.

use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron,
};
use crate::components::*;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Inhibitory;

schema!(
    Position,
    LeakyNeuron,
    LeakyDynamics,
    LeakCurrent,
    AdaptationCurrent,
    CurrentClamp,
    CurrentSynapse,
    ImmediateFireSynapse,
    Inhibitory,
    TouchSensor,
    GeneratorDynamics,
    RegularSpikeGenerator,
    PoissonGenerator,
    MeterSize,
    Annotation,
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
);
