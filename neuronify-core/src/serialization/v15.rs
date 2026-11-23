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
use hecs::{ComponentError, World};
use neuronify_engine::units::{Milliseconds, Millivolts, Nanofarads, Seconds};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakyNeuron {
    pub capacitance: Nanofarads,
    pub resting_potential: Millivolts,
    pub threshold: Millivolts,
    pub initial_potential: Millivolts,
    pub voltage_clamped: bool,
    pub minimum_voltage: Millivolts,
    pub maximum_voltage: Millivolts,
    pub refractory_period: Milliseconds,
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
);

pub fn upgrade_from_v14(world: &mut World) -> Result<(), ComponentError> {
    for series in world.query_mut::<&mut VoltageSeries>() {
        for measurement in series.measurements.data.iter_mut() {
            measurement.time = from_seconds(measurement.time);
        }
        for time in &mut series.spike_times {
            *time = from_seconds(*time);
        }
    }
    for detector in world.query_mut::<&mut SpikeDetector>() {
        detector.time_range = from_seconds(detector.time_range);
        detector.time = from_seconds(detector.time);
    }
    for train in world.query_mut::<&mut SpikeTrain>() {
        for time in &mut train.times {
            *time = from_seconds(*time);
        }
    }
    for plot in world.query_mut::<&mut RatePlot>() {
        plot.window = from_seconds(plot.window);
        plot.kernel_width = from_seconds(plot.kernel_width);
    }
    for series in world.query_mut::<&mut RateSeries>() {
        for (time, _) in &mut series.samples {
            *time = from_seconds(*time);
        }
    }
    Ok(())
}

fn from_seconds(time: Milliseconds) -> Milliseconds {
    Seconds(time.0).to_milliseconds()
}
