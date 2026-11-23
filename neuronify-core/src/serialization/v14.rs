use super::v15::LeakyNeuron;
use super::v16::VisualInput;
use super::v19::{
    AdaptationCurrent, AlternatingCurrent, Annotation, BurstCurrent, Cable, CableState,
    Compartment, ConductanceSynapse, Connection, ConnectionView, CurrentClamp, CurrentSynapse,
    Deletable, Dendrite, GeneratorDynamics, ImmediateFireSynapse, Label, LeakCurrent, Link,
    Loudspeaker, Membrane, MeterSize, Myelin, NeuronDisplay, NeuronType, NodeState, NoteSize,
    PoissonGenerator, Position, Preparation, Randomness, RatePlot, RateSeries,
    RegularSpikeGenerator, RowOrder, Selectable, Selected, SpatialDynamics, SpikeDetector,
    SpikeTrain, StaticConnectionSource, SynapseState, Temperature, TouchSensor, VoltageSeries,
    Voltmeter, VoltmeterRange,
};
use hecs::{ComponentError, Entity, World};

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
    ConnectionView,
    Selected,
    RowOrder,
);

pub fn upgrade_from_v13(world: &mut World) -> Result<(), ComponentError> {
    let detectors: Vec<hecs::Entity> = world
        .query::<Entity>()
        .with::<&SpikeDetector>()
        .iter()
        .collect();
    for detector in detectors {
        world.insert_one(detector, RowOrder::Height)?;
    }
    Ok(())
}
