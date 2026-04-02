// Game constants - world
pub const PETRI_DISH_RADIUS: f32 = 120.0;
pub const PETRI_DISH_SEGMENTS: usize = 128;

// Game constants - blood vessels
pub const BLOOD_VESSEL_GLUCOSE_RATE: f64 = 5.0;
pub const BLOOD_VESSEL_BLOCK_RATE: f64 = 5.0;
pub const BLOOD_VESSEL_SUPPLY_RADIUS: f32 = 20.0;
pub const BLOOD_VESSEL_HEIGHT: f32 = 8.0;
pub const BLOOD_VESSEL_VISUAL_RADIUS: f32 = 3.0;
/// Snap distance for connecting to blood vessels (larger than neurons).
pub const BLOOD_VESSEL_SNAP_RADIUS: f32 = BLOOD_VESSEL_VISUAL_RADIUS + 1.0;

// Game constants - building blocks
pub const MAX_BUILDING_BLOCKS: f64 = 1000.0;
pub const INITIAL_BUILDING_BLOCKS: f64 = 600.0;

// Game constants - metabolism
pub const RESTING_METABOLIC_COST: f64 = 0.5;
pub const FIRING_METABOLIC_COST: f64 = 1.0;
pub const CONNECTION_MAINTENANCE_COST: f64 = 0.1;
pub const COMPARTMENT_METABOLIC_COST: f64 = 0.15;
pub const RESOURCE_FLOW_RATE: f64 = 3.0;
pub const ORIGIN_MIN_ENERGY: f64 = 50.0;
pub const DEFAULT_NEURON_ENERGY: f64 = 80.0;
pub const MAX_NEURON_ENERGY: f64 = 150.0;

// Game constants - glial cells (astrocytes)
pub const GLIAL_GATHER_RADIUS: f32 = 15.0;
pub const GLIAL_DISTRIBUTE_RADIUS: f32 = 12.0;
pub const GLIAL_MAX_GLUCOSE: f64 = 100.0;
pub const GLIAL_MAX_BLOCKS: f64 = 50.0;
pub const GLIAL_BLOCK_TRANSFER_RATE: f64 = 10.0;
pub const GLIAL_COST: f64 = 8.0;

// Game constants - glucose transport (blood vessel → glial)
pub const GLUCOSE_PACKET_SPEED: f32 = 60.0;
pub const GLUCOSE_PACKET_INTERVAL: f64 = 0.15;

// Game constants - lactate transport (glial → neuron)
pub const LACTATE_PACKET_SPEED: f32 = 50.0;
pub const LACTATE_PACKET_INTERVAL: f64 = 0.25;

// Game constants - building costs
pub const NEURON_SPAWN_COST: f64 = 25.0;
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
pub const TCELL_HEALTH: f32 = 20.0;
pub const TCELL_SPEED: f32 = 14.0;
pub const TCELL_BURST_DAMAGE: f32 = 50.0;
pub const TCELL_BURST_RANGE: f32 = 1.8;
pub const TCELL_BURST_COOLDOWN: f32 = 3.0;

// Game constants - combat: area control (Reactive Astrocyte / Firewall Node / Tumor Barrier)
pub const REACTIVE_ASTROCYTE_HEALTH: f32 = 80.0;
pub const REACTIVE_ASTROCYTE_ABSORB_RADIUS: f32 = 8.0;
pub const REACTIVE_ASTROCYTE_ABSORB_RATE: f32 = 20.0;

// Game constants - axon structural health (added to compartments in combat scenarios)
pub const AXON_COMPARTMENT_HEALTH: f32 = 50.0;

// Game constants - combat unit build costs (building blocks)
pub const REACTIVE_ASTROCYTE_COST: f64 = 20.0;
pub const TCELL_COST: f64 = 18.0;
