use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronDisplay,
};
use super::v19::{
    AlternatingCurrent, Annotation, BurstCurrent, Connection, CurrentClamp, Deletable, Dendrite,
    Label, Link, Loudspeaker, MeterSize, Myelin, NeuronType, NoteSize, PoissonGenerator, Position,
    RatePlot, RateSeries, RegularSpikeGenerator, Selectable, SpatialDynamics, SpikeDetector,
    SpikeTrain, StaticConnectionSource, TouchSensor, VoltageSeries, Voltmeter, VoltmeterRange,
};
use super::v9::VisualInput;
use hecs::{ComponentError, World};

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

pub fn upgrade_from_v9(_world: &mut World) -> Result<(), ComponentError> {
    Ok(())
}
