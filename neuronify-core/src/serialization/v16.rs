use std::f64::consts::{FRAC_PI_2, TAU};

use super::v15;
use super::v19::{
    AdaptationCurrent, AlternatingCurrent, Annotation, BurstCurrent, Cable, CableState,
    Compartment, ConductanceSynapse, Connection, ConnectionView, CurrentClamp, CurrentSynapse,
    Deletable, Dendrite, GeneratorDynamics, ImmediateFireSynapse, Label, LeakCurrent, LeakyNeuron,
    Link, Loudspeaker, Membrane, MeterSize, Myelin, NeuronDisplay, NeuronType, NodeState, NoteSize,
    Pattern, PendingEvents, PoissonGenerator, Position, Preparation, Randomness, RatePlot,
    RateSeries, ReceptiveField, RegularSpikeGenerator, RowOrder, Selectable, Selected,
    SpatialDynamics, SpikeDetector, SpikeTrain, StaticConnectionSource, SynapseState, Temperature,
    TouchSensor, VoltageSeries, Voltmeter, VoltmeterRange,
};
use hecs::{ComponentError, Entity, World};
use neuronify_engine::units::Hertz;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VisualInput {
    pub field: ReceptiveField,
    pub sensitivity: Hertz,
    pub _show_field: bool,
    pub pattern: Pattern,
    pub pattern_angle: f64,
    pub speed: Hertz,
    pub phase: f64,
}

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
);

pub fn upgrade_from_v15(world: &mut World) -> Result<(), ComponentError> {
    let neurons: Vec<Entity> = world
        .query::<Entity>()
        .with::<&v15::LeakyNeuron>()
        .iter()
        .collect();
    for neuron in neurons {
        let old = world.remove_one::<v15::LeakyNeuron>(neuron)?;
        let upgraded = LeakyNeuron {
            capacitance: old.capacitance,
            resting_potential: old.resting_potential,
            threshold: old.threshold,
            initial_potential: old.initial_potential,
            voltage_clamped: old.voltage_clamped,
            minimum_voltage: old.minimum_voltage,
            maximum_voltage: old.maximum_voltage,
            refractory_period: old.refractory_period,
            ..LeakyNeuron::default()
        };
        world.insert_one(neuron, upgraded)?;
    }
    for input in world.query_mut::<&mut VisualInput>() {
        if let ReceptiveField::Edge { angle } = &mut input.field {
            *angle = (FRAC_PI_2 - *angle).rem_euclid(TAU);
        }
    }
    Ok(())
}
