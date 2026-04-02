use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PlayerId {
    Player1,
}

/// Faction affiliation for combat units. All three factions are playable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Faction {
    Biological,
    Tech,
    Tumor,
}

/// Health for combat units (microglia, macrophages, astrocytes, etc.).
/// Neurons use MetabolicState instead.
#[derive(Clone, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Structural health of an axon compartment.
/// When drained to zero the compartment is despawned, severing connections.
#[derive(Clone, Debug)]
pub struct AxonHealth {
    pub current: f32,
    pub max: f32,
}

impl AxonHealth {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Shared movement state for all mobile combat units.
#[derive(Clone, Debug)]
pub struct MobileUnit {
    pub speed: f32,
    pub target: Option<hecs::Entity>,
    pub faction: Faction,
}

// ── Unit-specific behavior components ────────────────────────────────────────

/// Targets Compartment entities and fires projectiles to drain AxonHealth (connection cutter role).
#[derive(Clone, Debug)]
pub struct AxonCutter {
    pub shot_damage: f32,
    pub shoot_cooldown: f32,
    pub shoot_timer: f32,
}

/// Targets LeakyNeuron somas and fires projectiles to drain MetabolicState.energy (neuron destroyer role).
#[derive(Clone, Debug)]
pub struct NeuronEngulfment {
    pub shot_damage: f32,
    pub target: Option<hecs::Entity>,
    pub shoot_cooldown: f32,
    pub shoot_timer: f32,
    /// Unit must be within this distance of its target to open fire (prevents long-range sniping).
    pub fire_range: f32,
}

/// A visible projectile fired by a combat unit toward its target.
/// Travels through space; applies damage and despawns on arrival.
#[derive(Clone, Debug)]
pub struct AttackProjectile {
    pub target: hecs::Entity,
    pub speed: f32,
    /// Structural damage applied to Health on arrival (macrophage attacks).
    pub health_damage: f32,
    /// Axon damage applied to AxonHealth on arrival (microglia attacks).
    pub axon_damage: f32,
    pub color: glam::Vec3,
    pub radius: f32,
}

/// Delivers a burst of damage on contact, then waits for cooldown (fast raider role).
#[derive(Clone, Debug)]
pub struct BurstAttack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub cooldown_timer: f32,
}

/// Stationary area control: drains Health of all enemy MobileUnits within radius.
#[derive(Clone, Debug)]
pub struct GlialAbsorption {
    pub absorb_radius: f32,
    pub absorb_rate: f32,
}

// ── Unit type markers ─────────────────────────────────────────────────────────
// Each marker drives rendering (mesh shape/color) and target-selection behavior.

/// Biological connection cutter — spiky cyan star mesh.
#[derive(Clone, Debug)]
pub struct MicroglialCell;
/// Biological neuron destroyer — large purple sphere.
#[derive(Clone, Debug)]
pub struct MacrophageUnit;
/// Biological fast raider — small lime-green sphere.
#[derive(Clone, Debug)]
pub struct TCellUnit;
/// Biological area control — gold six-pointed star mesh, stationary.
#[derive(Clone, Debug)]
pub struct ReactiveAstrocyte;

/// Tech connection cutter.
#[derive(Clone, Debug)]
pub struct DisruptorDrone;
/// Tech neuron destroyer.
#[derive(Clone, Debug)]
pub struct SiegeSynapse;
/// Tech fast raider.
#[derive(Clone, Debug)]
pub struct NanoProbe;
/// Tech area control.
#[derive(Clone, Debug)]
pub struct FirewallNode;

/// Tumor connection cutter.
#[derive(Clone, Debug)]
pub struct SeveringClaw;
/// Tumor neuron destroyer.
#[derive(Clone, Debug)]
pub struct MetastaticBud;
/// Tumor fast raider.
#[derive(Clone, Debug)]
pub struct Invadopod;
/// Tumor area control.
#[derive(Clone, Debug)]
pub struct TumorBarrier;

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
