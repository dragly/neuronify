use std::f64::consts::{FRAC_PI_2, TAU};

use super::v15;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{RowOrder, SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::{ReceptiveField, VisualInput};
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
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
