// Game constants - world
pub const PETRI_DISH_RADIUS: f32 = 120.0;
pub const PETRI_DISH_SEGMENTS: usize = 128;

// Game constants - blood vessels
pub const BLOOD_VESSEL_ATP_RATE: f64 = 5.0;
pub const BLOOD_VESSEL_BLOCK_RATE: f64 = 5.0;
pub const BLOOD_VESSEL_SUPPLY_RADIUS: f32 = 20.0;
pub const BLOOD_VESSEL_HEIGHT: f32 = 8.0;
pub const BLOOD_VESSEL_VISUAL_RADIUS: f32 = 1.5;

// Game constants - building blocks
pub const MAX_BUILDING_BLOCKS: f64 = 200.0;
pub const INITIAL_BUILDING_BLOCKS: f64 = 100.0;

// Game constants - metabolism
pub const RESTING_METABOLIC_COST: f64 = 0.5;
pub const FIRING_METABOLIC_COST: f64 = 1.0;
pub const CONNECTION_MAINTENANCE_COST: f64 = 0.1;
pub const COMPARTMENT_METABOLIC_COST: f64 = 0.15;
pub const MEMBRANE_METABOLIC_COST: f64 = 0.2;
pub const RESOURCE_FLOW_RATE: f64 = 3.0;
pub const ORIGIN_MIN_ENERGY: f64 = 50.0;
pub const DEFAULT_NEURON_ENERGY: f64 = 80.0;
pub const MAX_NEURON_ENERGY: f64 = 150.0;

// Game constants - sensors
pub const SENSOR_COST: f64 = 20.0;
pub const SENSOR_SENSITIVITY: f32 = 15.0;
pub const SENSOR_GAIN: f64 = 5e-9;

// Game constants - glial cells
pub const GLIAL_GATHER_RADIUS: f32 = 15.0;
pub const GLIAL_DISTRIBUTE_RADIUS: f32 = 12.0;
pub const GLIAL_MAX_ATP: f64 = 100.0;
pub const GLIAL_MAX_BLOCKS: f64 = 50.0;
pub const GLIAL_BLOCK_TRANSFER_RATE: f64 = 10.0;
pub const GLIAL_COST: f64 = 8.0;

// Game constants - building costs
pub const NEURON_SPAWN_COST: f64 = 25.0;
pub const COMPARTMENT_SPAWN_COST: f64 = 3.0;
pub const MEMBRANE_SPAWN_COST: f64 = 15.0;
pub const BUILD_RANGE: f32 = 15.0;

// Game constants - depolarization block
pub const DEPOL_BLOCK_THRESHOLD: f64 = -0.02;
pub const DEPOL_BLOCK_DURATION: f64 = 0.01;
pub const DEPOL_BLOCK_RECOVERY: f64 = 0.05;

// Game constants - AI (slow, deliberate - like Red Alert early game)
pub const AI_TICK_INTERVAL: u32 = 600;
pub const AI_EXPAND_CHANCE: f64 = 0.7;
pub const AI_DEFEND_CHANCE: f64 = 0.2;

// Game constants - environment
pub const SUBSTRATE_ZONE_RADIUS: f32 = 12.0;
pub const HIGH_K_THRESHOLD_SHIFT: f64 = 0.01;
pub const HIGH_MG_WEIGHT_FACTOR: f64 = 0.5;
pub const NOISE_ZONE_CURRENT: f64 = 1e-9;
pub const DAMAGE_ZONE_DRAIN: f64 = 2.0;

// Game constants - motor
pub const MOTOR_CILIA_COST: f64 = 20.0;
pub const MOTOR_THRUST_STRENGTH: f32 = 2.0;
pub const MOTOR_METABOLIC_COST: f64 = 1.0;

// Game constants - generators
pub const GENERATOR_SPAWN_COST: f64 = 15.0;
