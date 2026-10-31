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
);

pub fn upgrade_from_v3(world: &mut World) {
    let inhibitory: Vec<_> = world
        .query::<&Inhibitory>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in inhibitory {
        world.remove_one::<Inhibitory>(entity).unwrap();
        world.insert_one(entity, NeuronType::Inhibitory).unwrap();
    }
}
