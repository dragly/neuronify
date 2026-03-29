use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PlayerId {
    Player1,
    Player2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ownership {
    pub player: PlayerId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OriginNeuron {
    pub player: PlayerId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetabolicState {
    pub energy: f64,
    pub max_energy: f64,
}

impl Default for MetabolicState {
    fn default() -> Self {
        Self {
            energy: crate::constants::DEFAULT_NEURON_ENERGY,
            max_energy: crate::constants::MAX_NEURON_ENERGY,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BloodVessel {
    pub atp_rate: f64,
    pub block_rate: f64,
    pub supply_radius: f32,
}

impl Default for BloodVessel {
    fn default() -> Self {
        Self {
            atp_rate: crate::constants::BLOOD_VESSEL_ATP_RATE,
            block_rate: crate::constants::BLOOD_VESSEL_BLOCK_RATE,
            supply_radius: crate::constants::BLOOD_VESSEL_SUPPLY_RADIUS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayerEconomy {
    pub building_blocks: f64,
    pub max_building_blocks: f64,
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self {
            building_blocks: crate::constants::INITIAL_BUILDING_BLOCKS,
            max_building_blocks: crate::constants::MAX_BUILDING_BLOCKS,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MembraneSegment;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepolarizationBlock {
    pub time_above_threshold: f64,
    pub blocked: bool,
    pub recovery_timer: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SubstrateZoneType {
    HighPotassium,
    HighMagnesium,
    Noise,
    Damage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubstrateZone {
    pub zone_type: SubstrateZoneType,
    pub radius: f32,
}

/// Marks an entity as immovable (origin neurons, base structures).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Anchored;

/// Motor cilia attached to a neuron — converts firing into propulsion.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MotorCilia {
    pub direction: Vec3,
    pub strength: f32,
}

/// Marker for dendrite compartments (visual distinction from axon compartments).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dendrite;

/// Sensor neuron type — determines what the sensor detects.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SensorType {
    Activity, // detects nearby neural firing
    Chemical, // detects substrate zone proximity
    Touch,    // detects proximity to other-player entities
}

/// Sensor neuron — detects environmental signals and injects current.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SensorNeuron {
    pub sensor_type: SensorType,
    pub sensitivity: f32,
    pub gain: f64,
}

/// Glial cell — gathers ATP and building blocks from nearby blood vessels,
/// distributes ATP to nearby neurons, and contributes blocks to the player economy.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlialCell {
    pub gather_radius: f32,
    pub distribute_radius: f32,
    pub atp_stored: f64,
    pub max_atp: f64,
    pub blocks_stored: f64,
    pub max_blocks: f64,
}

impl Default for GlialCell {
    fn default() -> Self {
        Self {
            gather_radius: crate::constants::GLIAL_GATHER_RADIUS,
            distribute_radius: crate::constants::GLIAL_DISTRIBUTE_RADIUS,
            atp_stored: 0.0,
            max_atp: crate::constants::GLIAL_MAX_ATP,
            blocks_stored: 0.0,
            max_blocks: crate::constants::GLIAL_MAX_BLOCKS,
        }
    }
}
