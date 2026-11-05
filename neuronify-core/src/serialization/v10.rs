use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronDisplay,
};
use super::v9::VisualInput;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use hecs::World;

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

pub fn upgrade_from_v9(_world: &mut World) {}
