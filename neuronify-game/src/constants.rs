// Game constants - world
pub const PETRI_DISH_RADIUS: f32 = 120.0;
pub const PETRI_DISH_SEGMENTS: usize = 128;

// Game constants - vessel terrain harvesting
/// Glucose gained per adjacent vessel hex per second.
pub const VESSEL_GLUCOSE_RATE: f64 = 5.0;
/// Building blocks gained per adjacent vessel hex per second.
pub const VESSEL_BLOCK_RATE: f64 = 5.0;

// Game constants - building blocks
pub const MAX_BUILDING_BLOCKS: f64 = 1000.0;
pub const INITIAL_BUILDING_BLOCKS: f64 = 600.0;

// Game constants - metabolism
pub const RESTING_METABOLIC_COST: f64 = 0.5;
pub const FIRING_METABOLIC_COST: f64 = 3.0;
pub const CONNECTION_MAINTENANCE_COST: f64 = 0.3;
pub const COMPARTMENT_METABOLIC_COST: f64 = 0.4;
pub const RESOURCE_FLOW_RATE: f64 = 1.5;
pub const ORIGIN_MIN_ENERGY: f64 = 50.0;
pub const DEFAULT_NEURON_ENERGY: f64 = 80.0;
pub const MAX_NEURON_ENERGY: f64 = 150.0;

// Game constants - glial cells (astrocytes)
pub const GLIAL_DISTRIBUTE_RADIUS: f32 = 12.0;
pub const GLIAL_MAX_GLUCOSE: f64 = 100.0;
pub const GLIAL_MAX_BLOCKS: f64 = 50.0;
pub const GLIAL_BLOCK_TRANSFER_RATE: f64 = 10.0;

/// Fixed timestep for all combat / movement / production systems.
/// Using a fixed dt means simulation speed is identical on every machine.
pub const COMBAT_DT: f32 = 1.0 / 60.0;

/// Global game speed multiplier applied to the dt passed to every combat /
/// movement / production system.  1.0 = real time, 0.5 = half speed.
pub const GAME_SPEED: f32 = 1.0;

// Game constants - lactate transport (glial → neuron)
pub const LACTATE_PACKET_SPEED: f32 = 50.0;
pub const LACTATE_PACKET_INTERVAL: f64 = 0.25;

// Game constants - building costs
pub const COMPARTMENT_SPAWN_COST: f64 = 3.0;

// Game constants - neurons and glial cells (structural health under attack)
pub const NEURON_HEALTH: f32 = 100.0;
pub const GLIAL_HEALTH: f32 = 80.0;

// Game constants - combat: connection cutters (Microglial Cell / Disruptor Drone / Severing Claw)
pub const MICROGLIA_HEALTH: f32 = 40.0;
pub const MICROGLIA_SPEED: f32 = 6.0;
/// Axon HP dealt per projectile hit.
pub const MICROGLIA_SHOT_DAMAGE: f32 = 2.0;
/// Seconds between projectile shots.
pub const MICROGLIA_SHOOT_COOLDOWN: f32 = 1.0;

// Game constants - mast cells and cytokines
/// World-unit radius within which cytokine particles activate macrophages.
pub const MAST_CELL_CYTOKINE_RADIUS: f32 = 28.0;
/// Seconds between cytokine bursts emitted by a mast cell while its driver is firing.
pub const MAST_CELL_EMIT_INTERVAL: f32 = 0.25;
/// Number of cytokine particles per burst.
pub const MAST_CELL_PARTICLES_PER_BURST: usize = 8;
/// Seconds a cytokine particle lives before fading out.
pub const CYTOKINE_LIFETIME: f32 = 3.5;
/// World-units per second a cytokine particle drifts outward from its origin.
pub const CYTOKINE_SPEED: f32 = 5.0;

// Game constants - combat: neuron destroyers (Macrophage / Siege Synapse / Metastatic Bud)
pub const MACROPHAGE_HEALTH: f32 = 60.0;
pub const MACROPHAGE_SPEED: f32 = 3.0;
/// Energy and health drained from target neuron per projectile hit.
pub const MACROPHAGE_SHOT_DAMAGE: f32 = 4.5;
/// Seconds between projectile shots.
pub const MACROPHAGE_SHOOT_COOLDOWN: f32 = 1.5;
/// Macrophage must be within this distance of its target soma to open fire.
/// Also used as the standoff distance: unit stops moving at this range so
/// projectiles have a visible flight path rather than spawning at distance 0.
pub const MACROPHAGE_FIRE_RANGE: f32 = 5.0;

// Game constants - combat: projectiles (shared)
/// World-units per second for attack projectiles.
pub const ATTACK_PROJECTILE_SPEED: f32 = 14.0;

// Game constants - combat: fast raiders (T-Cell / Nano-Probe / Invadopod)
pub const TCELL_HEALTH: f32 = 25.0;
pub const TCELL_SPEED: f32 = 10.0;
pub const TCELL_BURST_DAMAGE: f32 = 15.0;
pub const TCELL_BURST_RANGE: f32 = 5.0;
pub const TCELL_BURST_COOLDOWN: f32 = 2.5;

/// Enemy AI will attack nearby player combat units (within this range) before targeting neurons.
pub const COMBAT_PRIORITY_RANGE: f32 = 15.0;
/// Duration (seconds) of the death break-apart animation before final despawn.
pub const DEATH_DURATION: f32 = 1.5;
/// Speed multiplier applied to a unit while its SlowEffect is active.
pub const ASTROCYTE_STAGGER_SLOW_FACTOR: f32 = 0.28;

// Game constants - axon structural health (added to compartments in combat scenarios)
#[allow(dead_code)]
pub const AXON_COMPARTMENT_HEALTH: f32 = 50.0;

// Game constants - production queue
pub const QUEUE_MAX_SIZE: usize = 5;

// Production costs and build durations for all producible items.
// "PRODUCE" costs are what the player pays; separate from old constants kept for compatibility.
pub const NEURON_PRODUCE_COST: f64 = 25.0;
pub const MICROGLIA_PRODUCE_COST: f64 = 15.0;
pub const MACROPHAGE_PRODUCE_COST: f64 = 20.0;
pub const TCELL_PRODUCE_COST: f64 = 12.0;
pub const GLIAL_PRODUCE_COST: f64 = 20.0;

pub const NEURON_BUILD_DURATION: f32 = 3.0;
pub const MICROGLIA_BUILD_DURATION: f32 = 2.0;
pub const MACROPHAGE_BUILD_DURATION: f32 = 3.0;
pub const TCELL_BUILD_DURATION: f32 = 1.5;
pub const GLIAL_BUILD_DURATION: f32 = 4.0;

// Game constants - neuroblast production and migration
/// World-units per second for a migrating neuroblast.
pub const NEUROBLAST_SPEED: f32 = 25.0;
/// Health of a neuroblast while migrating.
pub const NEUROBLAST_HEALTH: f32 = 30.0;
/// Side length of each hex cell in the pathfinding grid.
pub const HEX_GRID_CELL_SIZE: f32 = 4.0;

// Game constants - growth cone (axon building)
/// World-units per second a growth cone advances.
pub const GROWTH_CONE_SPEED: f32 = 15.0;
/// Distance (world units) between successive compartments laid by a growth cone.
/// Must equal the spring rest length (2.0 * NODE_RADIUS = 2.0) so painted compartments
/// sit at the physics equilibrium and don't contract after building.
pub const GROWTH_CONE_COMP_SPACING: f32 = 2.0;
/// How close (world units) a growth cone must be to its target before it completes.
pub const GROWTH_CONE_SNAP_RADIUS: f32 = 3.0;
