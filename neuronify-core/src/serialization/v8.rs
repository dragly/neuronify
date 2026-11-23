use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronDisplay,
};
use super::v19::{
    Annotation, BurstCurrent, Connection, CurrentClamp, Deletable, Dendrite, Link, MeterSize,
    NeuronType, NoteSize, PoissonGenerator, Position, RegularSpikeGenerator, Selectable,
    SpatialDynamics, StaticConnectionSource, TouchSensor, VoltageSeries, Voltmeter,
};
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
);

pub fn upgrade_from_v7(_world: &mut World) -> Result<(), ComponentError> {
    Ok(())
}
