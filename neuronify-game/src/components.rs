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
    /// Player-assigned attack target. Takes priority over AI-selected target when set.
    pub manual_target: Option<hecs::Entity>,
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
    /// If > 0, applies a SlowEffect of this duration (seconds) to the target on arrival.
    pub slow_duration: f32,
    pub color: glam::Vec3,
    pub radius: f32,
}

/// Temporary speed debuff applied by absorption stagger bolts.
/// While timer > 0 the unit moves at reduced speed.
#[derive(Clone, Debug)]
pub struct SlowEffect {
    pub timer: f32,
    /// Speed multiplier while slowed (e.g. 0.25 = 25% of normal speed).
    pub factor: f32,
}

/// Delivers a burst of damage on contact, then waits for cooldown (fast raider role).
#[derive(Clone, Debug)]
pub struct BurstAttack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub cooldown_timer: f32,
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

/// Marks a unit currently playing its death animation before final despawn.
/// Added by `despawn_dead` when a combat unit's health reaches zero.
#[derive(Clone, Debug)]
pub struct Dying {
    pub timer: f32,    // seconds elapsed since death
    pub duration: f32, // total animation duration
    pub seed: f32,     // per-unit scatter variety (from entity ID)
}

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

// ── Production / migration / maturation ──────────────────────────────────────

/// What type of neuron a Neuroblast will become on maturation.
/// Used by the Neuroblast component and the maturation system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducibleCell {
    ExcitatoryNeuroblast,
    InhibitoryNeuroblast,
}

/// Everything the player can queue for production at the origin neuron.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducibleItem {
    // Neurons — spawns a Neuroblast that migrates and matures
    ExcitatoryNeuron,
    InhibitoryNeuron,
    // Combat units — direct spawn near origin on completion
    MicroglialCell,
    Macrophage,
    TCell,
}

impl ProducibleItem {
    pub fn label(self) -> &'static str {
        match self {
            ProducibleItem::ExcitatoryNeuron   => "Excitatory",
            ProducibleItem::InhibitoryNeuron   => "Inhibitory",
            ProducibleItem::MicroglialCell     => "Microglia",
            ProducibleItem::Macrophage         => "Macrophage",
            ProducibleItem::TCell              => "T-Cell",
        }
    }

    pub fn cost(self) -> f64 {
        match self {
            ProducibleItem::ExcitatoryNeuron   => crate::constants::NEURON_PRODUCE_COST,
            ProducibleItem::InhibitoryNeuron   => crate::constants::NEURON_PRODUCE_COST,
            ProducibleItem::MicroglialCell     => crate::constants::MICROGLIA_PRODUCE_COST,
            ProducibleItem::Macrophage         => crate::constants::MACROPHAGE_PRODUCE_COST,
            ProducibleItem::TCell              => crate::constants::TCELL_PRODUCE_COST,
        }
    }

    pub fn build_duration(self) -> f32 {
        match self {
            ProducibleItem::ExcitatoryNeuron   => crate::constants::NEURON_BUILD_DURATION,
            ProducibleItem::InhibitoryNeuron   => crate::constants::NEURON_BUILD_DURATION,
            ProducibleItem::MicroglialCell     => crate::constants::MICROGLIA_BUILD_DURATION,
            ProducibleItem::Macrophage         => crate::constants::MACROPHAGE_BUILD_DURATION,
            ProducibleItem::TCell              => crate::constants::TCELL_BUILD_DURATION,
        }
    }

    /// Returns the ProducibleCell variant for neuron items; None for combat units.
    pub fn to_producible_cell(self) -> Option<ProducibleCell> {
        match self {
            ProducibleItem::ExcitatoryNeuron  => Some(ProducibleCell::ExcitatoryNeuroblast),
            ProducibleItem::InhibitoryNeuron  => Some(ProducibleCell::InhibitoryNeuroblast),
            _ => None,
        }
    }
}

/// One item in the production queue.
#[derive(Clone, Debug)]
pub struct QueuedItem {
    pub item: ProducibleItem,
    pub timer: f32,
    pub duration: f32,
}

/// Build queue attached to the origin neuron.
/// Items are processed front-to-back; clicking a produce button appends to back.
#[derive(Clone, Debug, Default)]
pub struct ProductionQueue {
    pub items: std::collections::VecDeque<QueuedItem>,
}

/// Attached to the origin neuron while it is producing a cell.
/// Removed and replaced by a spawned Neuroblast when `timer >= duration`.
#[derive(Clone, Debug)]
pub struct Production {
    pub cell_type: ProducibleCell,
    pub timer: f32,
    pub duration: f32,
}

/// An immature cell traveling through the soup toward a destination.
/// Produced at the radial glial cell; becomes a neuron when it arrives.
/// No MovePath means the unit is idle at spawn.
#[derive(Clone, Debug)]
pub struct Neuroblast {
    pub cell_type: ProducibleCell,
    pub speed: f32,
}

/// Ordered list of world-space waypoints for a moving entity.
/// The entity moves toward `waypoints[0]`, pops it on arrival, repeats.
#[derive(Clone, Debug)]
pub struct MovePath {
    /// Remaining world-space waypoints, front = next target.
    pub waypoints: Vec<glam::Vec3>,
    /// Countdown until A* is re-run to account for new obstacles.
    pub replan_timer: f32,
}

/// A neuroblast that has arrived at its destination and is maturing into a full neuron.
#[derive(Clone, Debug)]
pub struct MaturingNeuron {
    pub timer: f32,
    pub duration: f32,
    pub cell_type: ProducibleCell,
}


/// An in-flight growth cone that lays axon compartments as it advances.
/// Spawned when the player designates an axon destination; self-destructs on arrival.
#[derive(Clone, Debug)]
pub struct GrowthCone {
    /// Entity of the last compartment placed (starts as the source neuron).
    pub last_comp: hecs::Entity,
    /// World position of `last_comp` (cached to avoid repeated lookups).
    pub last_comp_pos: glam::Vec3,
    /// Final destination world position.
    pub target: glam::Vec3,
    /// If the axon should terminate at a specific neuron soma, its entity.
    pub target_entity: Option<hecs::Entity>,
    /// NeuronType of the source neuron (determines axon color/behavior).
    pub neuron_type: neuronify_core::NeuronType,
    /// Advance speed in world units per second.
    pub speed: f32,
}

/// Which unit type a NeuronSpawner produces when the neuron fires.
#[derive(Clone, Debug)]
pub enum NeuronSpawnType {
    MicroglialCell,
    TCell,
    Macrophage,
}

/// A standalone spawn point that emits combat units on a timer, independent of
/// neural firing.  Suitable for enemy spawn points in scenarios where the neural
/// simulation is not running.
#[derive(Clone, Debug)]
pub struct EnemySpawnPoint {
    pub faction: Faction,
    pub spawn_type: NeuronSpawnType,
    /// Seconds between spawns.
    pub cooldown: f32,
    /// Countdown; zero means ready to spawn immediately.
    pub timer: f32,
    pub spawn_offset: glam::Vec3,
}

/// Attached to a neuron soma. Each time that neuron fires AND the spawn cooldown
/// has elapsed, a combat unit is spawned at `spawn_offset` from the soma.
///
/// Fire detection uses `LeakyDynamics.time_since_fire` or `GeneratorDynamics.time_since_fire`:
/// a value below `combat_dt × 1.5` means the neuron fired this combat frame.
#[derive(Clone, Debug)]
pub struct NeuronSpawner {
    pub faction: Faction,
    pub spawn_type: NeuronSpawnType,
    /// Minimum wall-clock seconds between successive spawns.
    pub cooldown: f32,
    /// Countdown timer; zero means ready to spawn.
    pub timer: f32,
    /// World-space offset from soma where the new unit appears.
    pub spawn_offset: glam::Vec3,
}
