use super::v15::LeakyNeuron;
use super::v16::VisualInput;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::SpikeDetector;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use crate::measurement::SpikeTrain;
use hecs::{ComponentError, World};

schema!(
    Position,
    LeakyNeuron,
    NodeState,
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
    Cable,
    Membrane,
    CableState,
    ConductanceSynapse,
    SynapseState,
    Temperature,
    Randomness,
    Preparation,
);

pub fn upgrade_from_v12(_world: &mut World) -> Result<(), ComponentError> {
    Ok(())
}
