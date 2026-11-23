use super::v16::VisualInput;
use super::v19::{
    AdaptationCurrent, AlternatingCurrent, Annotation, BurstCurrent, Cable, CableState,
    Compartment, CompartmentSoma, ConductanceSynapse, Connection, ConnectionView, CurrentClamp,
    CurrentSynapse, Deletable, Dendrite, GeneratorDynamics, ImmediateFireSynapse, Label,
    LeakCurrent, LeakyNeuron, Link, Loudspeaker, Membrane, MeterSize, Myelin, NeuronDisplay,
    NeuronType, NodeState, NoteSize, PendingEvents, PoissonGenerator, Position, Preparation,
    Randomness, RatePlot, RateSeries, RegularSpikeGenerator, RowOrder, Selectable, Selected,
    SpatialDynamics, SpikeDetector, SpikeTrain, StaticConnectionSource, SynapseState, Temperature,
    TouchSensor, VoltageSeries, Voltmeter, VoltmeterRange,
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
    PendingEvents,
    CompartmentSoma,
);

pub fn upgrade_from_v16(world: &mut World) -> Result<(), ComponentError> {
    let branched: Vec<Entity> = world
        .query::<Entity>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|&neuron| has_dendrites(world, neuron))
        .collect();
    for neuron in branched {
        world.insert_one(neuron, CompartmentSoma::default())?;
    }
    Ok(())
}

fn has_dendrites(world: &World, soma: Entity) -> bool {
    let dendritic = |end: Entity| world.satisfies::<&Dendrite>(end);
    world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .filter(|cable| cable.from == soma || cable.to == soma)
        .any(|cable| dendritic(cable.from) || dendritic(cable.to))
}
