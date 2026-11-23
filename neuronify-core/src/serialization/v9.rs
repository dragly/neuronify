use super::v11::{
    AdaptationCurrent, Compartment, CompartmentCurrent, CurrentSynapse, GeneratorDynamics,
    ImmediateFireSynapse, LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronDisplay,
};
use super::v19::{
    AlternatingCurrent, Annotation, BurstCurrent, Connection, CurrentClamp, Deletable, Dendrite,
    Label, Link, Loudspeaker, MeterSize, NeuronType, NoteSize, Pattern, PoissonGenerator, Position,
    RatePlot, RateSeries, ReceptiveField, RegularSpikeGenerator, Selectable, SpatialDynamics,
    SpikeDetector, SpikeTrain, StaticConnectionSource, TouchSensor, VoltageSeries, Voltmeter,
    VoltmeterRange,
};
use hecs::{ComponentError, Entity, World};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VisualInput {
    pub field: ReceptiveField,
    pub sensitivity: f64,
    pub show_field: bool,
    pub pattern: Pattern,
    pub pattern_angle: f64,
}

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
);

pub fn upgrade_from_v8(world: &mut World) -> Result<(), ComponentError> {
    let neurons: Vec<hecs::Entity> = world
        .query::<Entity>()
        .with::<&LeakyNeuron>()
        .iter()
        .collect();
    for neuron in neurons {
        world.insert_one(neuron, Label::default())?;
    }
    let voltmeters: Vec<hecs::Entity> = world
        .query::<Entity>()
        .with::<&Voltmeter>()
        .iter()
        .collect();
    for voltmeter in voltmeters {
        let measured = world.remove::<(Connection, VoltageSeries)>(voltmeter);
        world.insert_one(voltmeter, VoltmeterRange::default())?;
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
    Ok(())
}
