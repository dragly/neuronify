use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{RowOrder, SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use hecs::{Entity, World};

schema!(
    save,
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

pub fn upgrade_from_v14(world: &mut World) {
    for (_, series) in world.query_mut::<(Entity, &mut VoltageSeries)>() {
        for measurement in series.measurements.iter_mut() {
            measurement.time = from_seconds(measurement.time);
        }
        for time in &mut series.spike_times {
            *time = from_seconds(*time);
        }
    }
    for (_, detector) in world.query_mut::<(Entity, &mut SpikeDetector)>() {
        detector.time_range = from_seconds(detector.time_range);
        detector.time = from_seconds(detector.time);
    }
    for (_, train) in world.query_mut::<(Entity, &mut SpikeTrain)>() {
        for time in &mut train.times {
            *time = from_seconds(*time);
        }
    }
    for (_, plot) in world.query_mut::<(Entity, &mut RatePlot)>() {
        plot.window = from_seconds(plot.window);
        plot.kernel_width = from_seconds(plot.kernel_width);
    }
    for (_, series) in world.query_mut::<(Entity, &mut RateSeries)>() {
        for (time, _) in &mut series.samples {
            *time = from_seconds(*time);
        }
    }
}

fn from_seconds(time: Milliseconds) -> Milliseconds {
    Seconds(time.0).to_milliseconds()
}
