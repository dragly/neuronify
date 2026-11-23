use std::collections::VecDeque;

use glam::Vec3;
use hecs::{ComponentError, Entity, World};
use neuronify_engine::units::{
    Celsius, Hertz, Megaohms, MicrofaradsPerSquareCentimetre, Micrometres, Microsiemens,
    Milliseconds, Millivolts, Nanoamperes, NanoamperesPerMillisecond, Nanofarads, OhmCentimetres,
    SiemensPerSquareCentimetre,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
schema!(
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

#[derive(Clone, Deserialize, Serialize)]
pub struct Position {
    pub position: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LeakyNeuron {
    pub capacitance: Nanofarads,
    pub resting_potential: Millivolts,
    pub threshold: Millivolts,
    pub initial_potential: Millivolts,
    pub voltage_clamped: bool,
    pub minimum_voltage: Millivolts,
    pub maximum_voltage: Millivolts,
    pub refractory_period: Milliseconds,
    pub soma_diameter: Micrometres,
}

impl Default for LeakyNeuron {
    fn default() -> Self {
        LeakyNeuron {
            capacitance: Nanofarads(0.2),
            resting_potential: Millivolts(-70.0),
            threshold: Millivolts(-55.0),
            initial_potential: Millivolts(-80.0),
            voltage_clamped: true,
            minimum_voltage: Millivolts(-90.0),
            maximum_voltage: Millivolts(60.0),
            refractory_period: Milliseconds(2.0),
            soma_diameter: Micrometres(20.0),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeState {
    pub voltage: Millivolts,
    pub gates: Vec<f64>,
    pub since_spike: Milliseconds,
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState {
            voltage: Millivolts(-65.0),
            gates: Vec::new(),
            since_spike: Milliseconds::INFINITY,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub resistance: Megaohms,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AdaptationCurrent {
    pub adaptation: Microsiemens,
    pub conductance: Microsiemens,
    pub time_constant: Milliseconds,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CurrentClamp {
    pub current_output: Nanoamperes,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CurrentSynapse {
    pub tau: Milliseconds,
    pub maximum_current: Nanoamperes,
    pub delay: Milliseconds,
    pub alpha_function: bool,
    pub exponential: f64,
    pub linear: f64,
    pub triggers: Vec<Milliseconds>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ImmediateFireSynapse;

#[derive(Clone, Deserialize, Serialize)]
pub struct TouchSensor;

#[derive(Clone, Deserialize, Serialize)]
pub struct GeneratorDynamics {
    pub time_since_fire: Milliseconds,
}

impl Default for GeneratorDynamics {
    fn default() -> Self {
        GeneratorDynamics {
            time_since_fire: Milliseconds::INFINITY,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RegularSpikeGenerator {
    pub frequency: Hertz,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PoissonGenerator {
    pub rate: Hertz,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MeterSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Annotation {
    pub text: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NoteSize {
    pub width: f32,
    pub height: f32,
}

impl Default for NoteSize {
    fn default() -> Self {
        NoteSize {
            width: 7.2,
            height: 4.8,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StaticConnectionSource {}

#[derive(Clone, Deserialize, Serialize)]
pub struct Deletable {}

#[derive(Clone, Deserialize, Serialize)]
pub struct Selectable {
    pub selected: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Voltmeter {}

#[derive(Clone, Deserialize, Serialize)]
pub struct VoltageSeries {
    pub measurements: RollingWindow<VoltageMeasurement>,
    #[serde(default)]
    pub spike_times: Vec<Milliseconds>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RollingWindow<T> {
    pub data: VecDeque<T>,
    pub max_capacity: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct VoltageMeasurement {
    pub voltage: Millivolts,
    pub time: Milliseconds,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub strength: f64,
    pub directional: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Compartment;

#[derive(Clone, Deserialize, Serialize)]
pub struct SpatialDynamics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

#[derive(Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub enum NeuronType {
    #[default]
    Excitatory,
    Inhibitory,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Dendrite;

#[derive(Clone, Deserialize, Serialize)]
pub struct BurstCurrent {
    pub peak: Nanoamperes,
    pub drain: NanoamperesPerMillisecond,
    pub boost: Nanoamperes,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Link {
    pub example: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NeuronDisplay {
    pub voltage: bool,
    pub firing_rate: bool,
    pub true_proportions: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct VoltmeterRange {
    pub minimum: Millivolts,
    pub maximum: Millivolts,
}

impl Default for VoltmeterRange {
    fn default() -> Self {
        VoltmeterRange {
            minimum: Millivolts(-100.0),
            maximum: Millivolts(50.0),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Label {
    pub text: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AlternatingCurrent {
    pub amplitude: Nanoamperes,
    pub frequency: Hertz,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpikeDetector {
    pub time_range: Milliseconds,
    pub time: Milliseconds,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpikeTrain {
    pub times: VecDeque<Milliseconds>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RatePlot {
    pub window: Milliseconds,
    pub kernel_width: Milliseconds,
    pub minimum: Hertz,
    pub maximum: Hertz,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RateSeries {
    pub samples: VecDeque<(Milliseconds, Hertz)>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Loudspeaker {
    pub sound: Sound,
    pub volume: f64,
    pub muted: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Sound {
    Click,
    Pop,
    Tone,
    Drop,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct VisualInput {
    pub pattern: Pattern,
    pub pattern_angle: f64,
    pub speed: Hertz,
    pub phase: f64,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Pattern {
    Bar,
    Grating,
    Dot,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ReceptiveField {
    Orientation { angle: f64, x: f64, y: f64 },
    CenterSurround { off_center: bool, x: f64, y: f64 },
    Edge { angle: f64 },
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Myelin;

#[derive(Clone, Deserialize, Serialize)]
pub struct Cable {
    pub length: Micrometres,
    pub diameter: Micrometres,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Membrane {
    pub capacitance: MicrofaradsPerSquareCentimetre,
    pub axial_resistivity: OhmCentimetres,
    pub leak: SiemensPerSquareCentimetre,
    pub leak_reversal: Millivolts,
    pub channels: Vec<Density>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Density {
    pub channel: ChannelId,
    pub conductance: SiemensPerSquareCentimetre,
    pub reversal: Millivolts,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ChannelId {
    SquidSodium,
    SquidPotassium,
    SlowSodium,
    SlowPotassium,
    CorticalSodium,
    CorticalPotassium,
    ATypePotassium,
    Hcn,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CableState {
    pub voltage: Vec<Millivolts>,
    pub gates: Vec<f64>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ConductanceSynapse {
    pub conductance: Microsiemens,
    pub delay: Milliseconds,
}

impl Default for ConductanceSynapse {
    fn default() -> Self {
        ConductanceSynapse {
            conductance: Microsiemens(0.0055),
            delay: Milliseconds(1.5),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SynapseState {
    pub open: Microsiemens,
    pub pending: Vec<Milliseconds>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Temperature {
    pub celsius: Celsius,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Randomness {
    pub state: u64,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Preparation {
    Squid,
    Mammal,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ConnectionView {
    All,
    LastSelected,
    None,
    Rings,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Selected;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum RowOrder {
    Height,
    Connection,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PendingEvents {
    pub adaptation: Vec<Milliseconds>,
    pub burst: Vec<Milliseconds>,
    pub fire: Vec<Milliseconds>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CompartmentSoma {
    pub threshold: Millivolts,
}

impl Default for CompartmentSoma {
    fn default() -> Self {
        CompartmentSoma {
            threshold: Millivolts(-20.0),
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ConnectionMode {
    One,
    Chain,
    FanOut,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ReceptiveFieldCell {
    pub field: ReceptiveField,
    pub sensitivity: Hertz,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ConnectionDirection {
    Both,
    Inputs,
    Outputs,
}

const CELL_BESIDE_PICTURE: f32 = (8.0 + 4.0 + 2.0 * 1.5) * 0.5;

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
                position: center + Vec3::Z * CELL_BESIDE_PICTURE,
            },
            ReceptiveFieldCell {
                field: old.field,
                sensitivity: old.sensitivity,
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
        world.spawn((
            Connection {
                from: picture,
                to: cell,
                strength: 1.0,
                directional: false,
            },
            Deletable {},
        ));
    }
    Ok(())
}
