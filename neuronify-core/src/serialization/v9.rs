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
    Label,
    AlternatingCurrent,
);

pub fn upgrade_from_v8(world: &mut World) {
    let neurons: Vec<hecs::Entity> = world
        .query::<()>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|(neuron, _)| neuron)
        .collect();
    for neuron in neurons {
        world.insert_one(neuron, Label::default()).unwrap();
    }
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
