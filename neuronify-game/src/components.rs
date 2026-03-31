use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PlayerId {
    Player1,
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
    pub glucose_rate: f64,
    pub block_rate: f64,
    pub supply_radius: f32,
}

impl Default for BloodVessel {
    fn default() -> Self {
        Self {
            glucose_rate: crate::constants::BLOOD_VESSEL_GLUCOSE_RATE,
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

/// Marks an entity as immovable.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Anchored;

/// Marker for dendrite compartments (visual distinction from axon compartments).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dendrite;

/// Marks a blood vessel as a valid endpoint for axon/dendrite connections.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VesselAnchor;

/// Marker for compartments that belong to a glial cell's process network.
/// Used to distinguish glial processes from neuronal axons/dendrites.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlialProcess;

/// Glial cell (astrocyte) — gathers glucose from blood vessels via process connections,
/// converts it to lactate, and distributes lactate to nearby neurons for energy.
/// Also contributes building blocks to the player economy.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlialCell {
    pub gather_radius: f32,
    pub distribute_radius: f32,
    pub glucose_stored: f64,
    pub max_glucose: f64,
    pub blocks_stored: f64,
    pub max_blocks: f64,
    pub packet_timer: f64,
    pub lactate_timer: f64,
}

impl Default for GlialCell {
    fn default() -> Self {
        Self {
            gather_radius: crate::constants::GLIAL_GATHER_RADIUS,
            distribute_radius: crate::constants::GLIAL_DISTRIBUTE_RADIUS,
            glucose_stored: 0.0,
            max_glucose: crate::constants::GLIAL_MAX_GLUCOSE,
            blocks_stored: 0.0,
            max_blocks: crate::constants::GLIAL_MAX_BLOCKS,
            packet_timer: 0.0,
            lactate_timer: 0.0,
        }
    }
}

/// A lactate packet traveling from a glial cell to a neuron along the glial
/// process chain.  The path mirrors the structure of GlucosePacket: an ordered
/// list of entities from the glial soma to the neuron soma.
pub struct LactatePacket {
    /// Ordered entities from glial soma → process compartments → bridge → neuron soma.
    pub path: Vec<hecs::Entity>,
    /// Current segment: traveling from path[path_index] to path[path_index + 1].
    pub path_index: usize,
    /// Progress along the current segment, 0.0 to 1.0.
    pub progress: f32,
    /// Movement speed in world units per second.
    pub speed: f32,
    /// Energy deposited into the target neuron on arrival.
    pub energy_amount: f64,
}

/// A glucose packet traveling along glial process connections from a blood vessel
/// to a glial cell. Physical entity that can be destroyed to disrupt resource flow.
pub struct GlucosePacket {
    /// Ordered list of entities from blood vessel to glial cell.
    pub path: Vec<hecs::Entity>,
    /// Current segment: traveling from path[path_index] to path[path_index + 1].
    pub path_index: usize,
    /// Progress along the current segment, 0.0 to 1.0.
    pub progress: f32,
    /// Movement speed in world units per second.
    pub speed: f32,
    /// Glucose carried by this packet.
    pub glucose_amount: f64,
    /// Building blocks carried by this packet.
    pub block_amount: f64,
}
