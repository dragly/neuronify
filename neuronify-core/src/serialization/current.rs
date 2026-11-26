use hecs::{Component, Entity, EntityBuilder, EntityRef, NoSuchEntity, World};
use neuronify_engine::{ChannelId, Density};

use super::v19 as copies;
use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::frame::RowOrder;
use crate::measurement::loudspeaker::{Loudspeaker, Sound};
use crate::measurement::spike_detector::SpikeDetector;
use crate::measurement::voltmeter::{
    RollingWindow, VoltageMeasurement, VoltageSeries, Voltmeter, VoltmeterRange,
};
use crate::measurement::SpikeTrain;
use crate::stimuli::alternating_current::AlternatingCurrent;
use crate::stimuli::poisson_generator::PoissonGenerator;
use crate::stimuli::receptive_field::{ReceptiveField, ReceptiveFieldCell};
use crate::stimuli::regular_spike_generator::RegularSpikeGenerator;
use crate::stimuli::touch_sensor::TouchSensor;
use crate::stimuli::visual_input::{Pattern, VisualInput};
use crate::stimuli::{CurrentClamp, GeneratorDynamics};
use crate::tools::ConnectionMode;

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

pub fn from_copies(copied: &World) -> Result<World, NoSuchEntity> {
    let mut world = empty_entities_in_id_order(copied);
    for entity in copied.iter() {
        let mut components = EntityBuilder::new();
        for id in ComponentId::ALL {
            let convert: fn(&EntityRef, &mut EntityBuilder) = match id {
                ComponentId::Position => convert::<copies::Position, Position>,
                ComponentId::LeakyNeuron => convert::<copies::LeakyNeuron, LeakyNeuron>,
                ComponentId::NodeState => convert::<copies::NodeState, NodeState>,
                ComponentId::LeakCurrent => convert::<copies::LeakCurrent, LeakCurrent>,
                ComponentId::AdaptationCurrent => {
                    convert::<copies::AdaptationCurrent, AdaptationCurrent>
                }
                ComponentId::CurrentClamp => convert::<copies::CurrentClamp, CurrentClamp>,
                ComponentId::CurrentSynapse => convert::<copies::CurrentSynapse, CurrentSynapse>,
                ComponentId::ImmediateFireSynapse => {
                    convert::<copies::ImmediateFireSynapse, ImmediateFireSynapse>
                }
                ComponentId::TouchSensor => convert::<copies::TouchSensor, TouchSensor>,
                ComponentId::GeneratorDynamics => {
                    convert::<copies::GeneratorDynamics, GeneratorDynamics>
                }
                ComponentId::RegularSpikeGenerator => {
                    convert::<copies::RegularSpikeGenerator, RegularSpikeGenerator>
                }
                ComponentId::PoissonGenerator => {
                    convert::<copies::PoissonGenerator, PoissonGenerator>
                }
                ComponentId::MeterSize => convert::<copies::MeterSize, MeterSize>,
                ComponentId::Annotation => convert::<copies::Annotation, Annotation>,
                ComponentId::NoteSize => convert::<copies::NoteSize, NoteSize>,
                ComponentId::StaticConnectionSource => {
                    convert::<copies::StaticConnectionSource, StaticConnectionSource>
                }
                ComponentId::Deletable => convert::<copies::Deletable, Deletable>,
                ComponentId::Selectable => convert::<copies::Selectable, Selectable>,
                ComponentId::Voltmeter => convert::<copies::Voltmeter, Voltmeter>,
                ComponentId::VoltageSeries => convert::<copies::VoltageSeries, VoltageSeries>,
                ComponentId::Connection => convert::<copies::Connection, Connection>,
                ComponentId::Compartment => convert::<copies::Compartment, Compartment>,
                ComponentId::SpatialDynamics => convert::<copies::SpatialDynamics, SpatialDynamics>,
                ComponentId::NeuronType => convert::<copies::NeuronType, NeuronType>,
                ComponentId::Dendrite => convert::<copies::Dendrite, Dendrite>,
                ComponentId::BurstCurrent => convert::<copies::BurstCurrent, BurstCurrent>,
                ComponentId::Link => convert::<copies::Link, Link>,
                ComponentId::NeuronDisplay => convert::<copies::NeuronDisplay, NeuronDisplay>,
                ComponentId::VoltmeterRange => convert::<copies::VoltmeterRange, VoltmeterRange>,
                ComponentId::Label => convert::<copies::Label, Label>,
                ComponentId::AlternatingCurrent => {
                    convert::<copies::AlternatingCurrent, AlternatingCurrent>
                }
                ComponentId::SpikeDetector => convert::<copies::SpikeDetector, SpikeDetector>,
                ComponentId::SpikeTrain => convert::<copies::SpikeTrain, SpikeTrain>,
                ComponentId::RatePlot => convert::<copies::RatePlot, RatePlot>,
                ComponentId::RateSeries => convert::<copies::RateSeries, RateSeries>,
                ComponentId::Loudspeaker => convert::<copies::Loudspeaker, Loudspeaker>,
                ComponentId::VisualInput => convert::<copies::VisualInput, VisualInput>,
                ComponentId::Myelin => convert::<copies::Myelin, Myelin>,
                ComponentId::Cable => convert::<copies::Cable, Cable>,
                ComponentId::Membrane => convert::<copies::Membrane, Membrane>,
                ComponentId::CableState => convert::<copies::CableState, CableState>,
                ComponentId::ConductanceSynapse => {
                    convert::<copies::ConductanceSynapse, ConductanceSynapse>
                }
                ComponentId::SynapseState => convert::<copies::SynapseState, SynapseState>,
                ComponentId::Temperature => convert::<copies::Temperature, Temperature>,
                ComponentId::Randomness => convert::<copies::Randomness, Randomness>,
                ComponentId::Preparation => convert::<copies::Preparation, Preparation>,
                ComponentId::ConnectionView => convert::<copies::ConnectionView, ConnectionView>,
                ComponentId::Selected => convert::<copies::Selected, Selected>,
                ComponentId::RowOrder => convert::<copies::RowOrder, RowOrder>,
                ComponentId::PendingEvents => convert::<copies::PendingEvents, PendingEvents>,
                ComponentId::CompartmentSoma => convert::<copies::CompartmentSoma, CompartmentSoma>,
                ComponentId::ConnectionMode => convert::<copies::ConnectionMode, ConnectionMode>,
                ComponentId::ReceptiveFieldCell => {
                    convert::<copies::ReceptiveFieldCell, ReceptiveFieldCell>
                }
                ComponentId::ConnectionDirection => {
                    convert::<copies::ConnectionDirection, ConnectionDirection>
                }
            };
            convert(&entity, &mut components);
        }
        world.insert(entity.entity(), components.build())?;
    }
    Ok(world)
}

fn empty_entities_in_id_order(from: &World) -> World {
    let mut handles: Vec<Entity> = from.iter().map(|entity| entity.entity()).collect();
    handles.sort_by_key(|handle| handle.id());
    let mut world = World::new();
    for handle in handles {
        world.spawn_at(handle, ());
    }
    world
}

fn convert<Copy, App>(entity: &EntityRef, components: &mut EntityBuilder)
where
    Copy: Component + Clone,
    App: Component + From<Copy>,
{
    if let Some(copy) = entity.get::<&Copy>() {
        components.add(App::from((*copy).clone()));
    }
}

macro_rules! same_fields {
    ($($name:ident { $($field:ident),* })*) => {
        $(
            impl From<copies::$name> for $name {
                fn from(copies::$name { $($field),* }: copies::$name) -> Self {
                    $name { $($field),* }
                }
            }
        )*
    };
}

macro_rules! same_variants {
    ($($name:ident { $($variant:ident),* })*) => {
        $(
            impl From<copies::$name> for $name {
                fn from(copy: copies::$name) -> Self {
                    match copy {
                        $(copies::$name::$variant => $name::$variant),*
                    }
                }
            }
        )*
    };
}

same_fields! {
    Position { position }
    LeakyNeuron {
        capacitance, resting_potential, threshold, initial_potential, voltage_clamped,
        minimum_voltage, maximum_voltage, refractory_period, soma_diameter
    }
    NodeState { voltage, gates, since_spike }
    LeakCurrent { resistance }
    AdaptationCurrent { adaptation, conductance, time_constant }
    CurrentClamp { current_output }
    CurrentSynapse { tau, maximum_current, delay, alpha_function, exponential, linear, triggers }
    ImmediateFireSynapse {}
    TouchSensor {}
    GeneratorDynamics { time_since_fire }
    RegularSpikeGenerator { frequency }
    PoissonGenerator { rate }
    MeterSize { width, height }
    Annotation { text }
    NoteSize { width, height }
    StaticConnectionSource {}
    Deletable {}
    Selectable { selected }
    Voltmeter {}
    VoltageMeasurement { voltage, time }
    Connection { from, to, strength, directional }
    Compartment {}
    SpatialDynamics { velocity, acceleration }
    Dendrite {}
    BurstCurrent { peak, drain, boost }
    Link { example }
    NeuronDisplay { voltage, firing_rate, true_proportions }
    VoltmeterRange { minimum, maximum }
    Label { text }
    AlternatingCurrent { amplitude, frequency }
    SpikeDetector { time_range, time }
    SpikeTrain { times }
    RatePlot { window, kernel_width, minimum, maximum }
    RateSeries { samples }
    Myelin {}
    Cable { length, diameter }
    CableState { voltage, gates }
    ConductanceSynapse { conductance, delay }
    SynapseState { open, pending }
    Temperature { celsius }
    Randomness { state }
    Selected {}
    PendingEvents { adaptation, burst, fire }
    CompartmentSoma { threshold }
}

same_variants! {
    NeuronType { Excitatory, Inhibitory }
    Sound { Click, Pop, Tone, Drop }
    Pattern { Bar, Grating, Dot }
    ChannelId {
        SquidSodium, SquidPotassium, SlowSodium, SlowPotassium, CorticalSodium,
        CorticalPotassium, ATypePotassium, Hcn
    }
    Preparation { Squid, Mammal }
    ConnectionView { All, LastSelected, None, Rings }
    RowOrder { Height, Connection }
    ConnectionMode { One, Chain, FanOut }
    ConnectionDirection { Both, Inputs, Outputs }
}

impl From<copies::VoltageSeries> for VoltageSeries {
    fn from(copy: copies::VoltageSeries) -> Self {
        let copies::VoltageSeries {
            measurements: copies::RollingWindow { data, max_capacity },
            spike_times,
        } = copy;
        VoltageSeries {
            measurements: RollingWindow::from_parts(
                data.into_iter().map(VoltageMeasurement::from).collect(),
                max_capacity,
            ),
            spike_times,
        }
    }
}

impl From<copies::Loudspeaker> for Loudspeaker {
    fn from(
        copies::Loudspeaker {
            sound,
            volume,
            muted,
        }: copies::Loudspeaker,
    ) -> Self {
        Loudspeaker {
            sound: sound.into(),
            volume,
            muted,
            ..Loudspeaker::default()
        }
    }
}

impl From<copies::VisualInput> for VisualInput {
    fn from(copy: copies::VisualInput) -> Self {
        let copies::VisualInput {
            pattern,
            pattern_angle,
            speed,
            phase,
        } = copy;
        VisualInput {
            pattern: pattern.into(),
            pattern_angle,
            speed,
            phase,
        }
    }
}

impl From<copies::ReceptiveFieldCell> for ReceptiveFieldCell {
    fn from(copies::ReceptiveFieldCell { field, sensitivity }: copies::ReceptiveFieldCell) -> Self {
        ReceptiveFieldCell {
            field: field.into(),
            sensitivity,
            ..ReceptiveFieldCell::default()
        }
    }
}

impl From<copies::ReceptiveField> for ReceptiveField {
    fn from(copy: copies::ReceptiveField) -> Self {
        match copy {
            copies::ReceptiveField::Orientation { angle, x, y } => {
                ReceptiveField::Orientation { angle, x, y }
            }
            copies::ReceptiveField::CenterSurround { off_center, x, y } => {
                ReceptiveField::CenterSurround { off_center, x, y }
            }
            copies::ReceptiveField::Edge { angle } => ReceptiveField::Edge { angle },
        }
    }
}

impl From<copies::Membrane> for Membrane {
    fn from(copy: copies::Membrane) -> Self {
        let copies::Membrane {
            capacitance,
            axial_resistivity,
            leak,
            leak_reversal,
            channels,
        } = copy;
        Membrane {
            capacitance,
            axial_resistivity,
            leak,
            leak_reversal,
            channels: channels.into_iter().map(Density::from).collect(),
        }
    }
}

impl From<copies::Density> for Density {
    fn from(
        copies::Density {
            channel,
            conductance,
            reversal,
        }: copies::Density,
    ) -> Self {
        Density {
            channel: channel.into(),
            conductance,
            reversal,
        }
    }
}
