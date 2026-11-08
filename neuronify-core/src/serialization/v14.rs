use super::v15::LeakyNeuron;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{RowOrder, SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
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
