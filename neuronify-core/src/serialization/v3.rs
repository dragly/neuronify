//! Dendrites are marked, so they keep their look after loading.

use crate::components::*;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
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
    Inhibitory,
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
);

/// Version 3 only adds components, so a version 2 world is already a version 3 world.
pub fn upgrade_from_v2(_world: &mut World) {}
