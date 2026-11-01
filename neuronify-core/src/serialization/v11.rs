use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use hecs::World;

schema!(
    save,
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

pub fn upgrade_from_v10(world: &mut World) {
    let inputs: Vec<hecs::Entity> = world
        .query::<()>()
        .with::<&super::v9::VisualInput>()
        .iter()
        .map(|(input, _)| input)
        .collect();
    for input in inputs {
        let old = world.remove_one::<super::v9::VisualInput>(input).unwrap();
        world
            .insert_one(
                input,
                VisualInput {
                    field: old.field,
                    sensitivity: old.sensitivity,
                    show_field: old.show_field,
                    pattern: old.pattern,
                    pattern_angle: old.pattern_angle,
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
