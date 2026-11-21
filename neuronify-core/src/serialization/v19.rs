use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::frame::RowOrder;
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::SpikeDetector;
use crate::measurement::visual_input::{
    beside_the_picture, spawn_picture_wire, ReceptiveFieldCell, VisualInput,
};
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use crate::measurement::SpikeTrain;
use crate::tools::ConnectionMode;
use hecs::{ComponentError, Entity, World};

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
    PendingEvents,
    CompartmentSoma,
    ConnectionMode,
    ReceptiveFieldCell,
    ConnectionDirection,
);

pub fn upgrade_from_v18(world: &mut World) -> Result<(), ComponentError> {
    let inputs: Vec<Entity> = world
        .query::<Entity>()
        .with::<&super::v16::VisualInput>()
        .iter()
        .collect();
    for picture in inputs {
        let old = world.remove_one::<super::v16::VisualInput>(picture)?;
        let dynamics = world
            .remove_one::<GeneratorDynamics>(picture)
            .unwrap_or_default();
        let neuron_type = world.remove_one::<NeuronType>(picture).unwrap_or_default();
        let center = world
            .get::<&Position>(picture)
            .map(|position| position.position)
            .unwrap_or_default();
        world.insert_one(
            picture,
            VisualInput {
                pattern: old.pattern,
                pattern_angle: old.pattern_angle,
                speed: old.speed,
                phase: old.phase,
            },
        )?;
        let cell = world.spawn((
            Position {
                position: beside_the_picture(center),
            },
            ReceptiveFieldCell {
                field: old.field,
                sensitivity: old.sensitivity,
                ..Default::default()
            },
            dynamics,
            neuron_type,
            Deletable {},
        ));
        let outgoing: Vec<Entity> = world
            .query::<(Entity, &Connection)>()
            .iter()
            .filter(|(_, connection)| connection.from == picture)
            .map(|(connection, _)| connection)
            .collect();
        for connection in outgoing {
            world.get::<&mut Connection>(connection)?.from = cell;
        }
        spawn_picture_wire(world, picture, cell);
    }
    Ok(())
}
