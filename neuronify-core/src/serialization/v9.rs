use crate::components::*;
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
    VoltmeterSize,
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
);

pub fn upgrade_from_v8(world: &mut World) {
    let voltmeters: Vec<hecs::Entity> = world
        .query::<()>()
        .with::<&Voltmeter>()
        .iter()
        .map(|(voltmeter, _)| voltmeter)
        .collect();
    for voltmeter in voltmeters {
        let measured = world.remove::<(Connection, VoltageSeries)>(voltmeter);
        world
            .insert_one(voltmeter, VoltmeterRange::default())
            .unwrap();
        if let Ok((connection, series)) = measured {
            world.spawn((
                Connection {
                    to: voltmeter,
                    ..connection
                },
                series,
                Deletable {},
            ));
        }
    }
}
