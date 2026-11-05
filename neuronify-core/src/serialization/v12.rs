use super::v11;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
use crate::simulation::model::cable_between;
use hecs::{Component, Entity, World};

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
);

const MILLI: f64 = 1e3;
const NANO: f64 = 1e9;
const MICRO: f64 = 1e6;

fn replace<Old: Component, New: Component>(world: &mut World, convert: impl Fn(Old) -> New) {
    let entities: Vec<Entity> = world
        .query::<()>()
        .with::<&Old>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in entities {
        let old = world.remove_one::<Old>(entity).unwrap();
        world.insert_one(entity, convert(old)).unwrap();
    }
}

fn scale<T: Component>(world: &mut World, convert: impl Fn(&mut T)) {
    for (_, component) in world.query_mut::<&mut T>() {
        convert(component);
    }
}

pub fn upgrade_from_v11(world: &mut World) {
    let neurons: Vec<Entity> = world
        .query::<()>()
        .with::<&v11::LeakyNeuron>()
        .iter()
        .map(|(neuron, _)| neuron)
        .collect();
    for neuron in neurons {
        let old = world.remove_one::<v11::LeakyNeuron>(neuron).unwrap();
        let dynamics = world.remove_one::<v11::LeakyDynamics>(neuron).ok();
        let refractory_period = dynamics
            .as_ref()
            .map_or(LeakyNeuron::default().refractory_period, |dynamics| {
                dynamics.refractory_period * MILLI
            });
        let state = NodeState {
            voltage: dynamics
                .as_ref()
                .map_or(old.resting_potential, |dynamics| dynamics.voltage)
                * MILLI,
            gates: Vec::new(),
            since_spike: dynamics
                .map_or(f64::INFINITY, |dynamics| dynamics.time_since_fire * MILLI),
        };
        let neuron_parameters = LeakyNeuron {
            capacitance: old.capacitance * NANO,
            resting_potential: old.resting_potential * MILLI,
            threshold: old.threshold * MILLI,
            initial_potential: old.initial_potential * MILLI,
            voltage_clamped: old.voltage_clamped,
            minimum_voltage: old.minimum_voltage * MILLI,
            maximum_voltage: old.maximum_voltage * MILLI,
            refractory_period,
        };
        world.insert(neuron, (neuron_parameters, state)).unwrap();
    }
    replace(world, |_: v11::LeakyDynamics| NodeState::default());
    replace(world, |old: v11::LeakCurrent| LeakCurrent {
        resistance: old.resistance / MICRO,
    });
    replace(world, |old: v11::AdaptationCurrent| AdaptationCurrent {
        adaptation: old.adaptation * MICRO,
        conductance: old.conductance * MICRO,
        time_constant: old.time_constant * MILLI,
    });
    scale(world, |burst: &mut BurstCurrent| {
        burst.peak *= NANO;
        burst.drain *= NANO / MILLI;
        burst.boost *= NANO;
    });
    scale(world, |clamp: &mut CurrentClamp| {
        clamp.current_output *= NANO;
    });
    scale(world, |alternating: &mut AlternatingCurrent| {
        alternating.amplitude *= NANO;
    });
    replace(world, |old: v11::CurrentSynapse| CurrentSynapse {
        tau: old.tau * MILLI,
        maximum_current: old.maximum_current * NANO,
        delay: old.delay * MILLI,
        alpha_function: old.alpha_function,
        exponential: 0.0,
        linear: 0.0,
        triggers: Vec::new(),
    });
    replace(world, |_: v11::ImmediateFireSynapse| ImmediateFireSynapse);
    replace(world, |old: v11::GeneratorDynamics| GeneratorDynamics {
        time_since_fire: old.time_since_fire * MILLI,
    });
    let compartments: Vec<Entity> = world
        .query::<()>()
        .with::<&v11::Compartment>()
        .iter()
        .map(|(compartment, _)| compartment)
        .collect();
    for compartment in compartments {
        world.remove_one::<v11::Compartment>(compartment).unwrap();
        world
            .insert(compartment, (Compartment, NodeState::default()))
            .unwrap();
    }
    let links: Vec<(Entity, Connection, bool)> = world
        .query::<(&Connection, Option<&Myelin>)>()
        .with::<&v11::CompartmentCurrent>()
        .iter()
        .map(|(link, (connection, myelin))| (link, connection.clone(), myelin.is_some()))
        .collect();
    for (link, connection, myelinated) in links {
        world.remove_one::<v11::CompartmentCurrent>(link).unwrap();
        if connection.directional {
            world
                .insert(
                    link,
                    (ConductanceSynapse::default(), SynapseState::default()),
                )
                .unwrap();
        } else {
            let (cable, membrane) =
                cable_between(world, connection.from, connection.to, myelinated);
            world.insert(link, (cable, membrane)).unwrap();
        }
    }
}
