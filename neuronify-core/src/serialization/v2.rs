//! Notes get a size and can be deleted.

use super::v1::Inhibitory;
use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron,
};
use crate::components::*;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use hecs::{ComponentError, Entity, World};

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
pub fn upgrade_from_v1(world: &mut World) -> Result<(), ComponentError> {
    let notes: Vec<_> = world
        .query::<(Entity, &Annotation)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in notes {
        world.insert(entity, (NoteSize::default(), Deletable {}))?;
    }
    Ok(())
}
