//! Notes get a size and can be deleted.

use super::v1::Inhibitory;
use crate::components::*;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use hecs::World;

schema!(
    Position,
    LeakyNeuron,
    LeakyDynamics,
    LeakCurrent,
    AdaptationCurrent,
    CurrentClamp,
    CurrentSynapse,
    ImmediateFireSynapse,
    Inhibitory,
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
);

/// Gives every note the default size and lets it be deleted,
/// like notes made with the note tool.
pub fn upgrade_from_v1(world: &mut World) {
    let notes: Vec<_> = world
        .query::<&Annotation>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in notes {
        world
            .insert(entity, (NoteSize::default(), Deletable {}))
            .unwrap();
    }
}
