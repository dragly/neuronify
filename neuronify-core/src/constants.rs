pub const FHN_TAU: f64 = 60.0;
pub const FHN_A: f64 = 0.7;
pub const FHN_B: f64 = 0.8;
pub const FHN_EPS: f64 = 0.08;
pub const FHN_SCALE: f64 = 50.0;
pub const FHN_OFFSET: f64 = 50.0;
pub const FHN_CDT: f64 = 0.01;

pub const LIF_DT: f64 = 0.0001;
pub const PHYSICS_DT: f64 = 0.001;

pub const NODE_RADIUS: f32 = 1.0;
pub const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;
pub const ATTACHMENT_RANGE: f32 = 1.5 * NODE_RADIUS;
pub const SELECTION_RANGE: f32 = 0.9 * NODE_RADIUS;

pub const VOLTMETER_TIME_WINDOW: f64 = 1.0 / 9.0;
pub const LIF_VOLTAGE_MIN: f64 = -100.0;
pub const LIF_VOLTAGE_MAX: f64 = 50.0;
pub const FHN_VOLTAGE_MIN: f64 = -80.0;
pub const FHN_VOLTAGE_MAX: f64 = 160.0;

pub const REPULSION_STRENGTH: f32 = 5.0;
pub const SPRING_STRENGTH: f32 = 10.0;
pub const ANGLE_ALIGNMENT_STRENGTH: f32 = 1.0;

pub const COUPLING_CAPACITANCE: f64 = 1.0 / 17.0;
pub const BRIDGE_CURRENT_SCALE: f64 = 10e-9;
pub const BRIDGE_VOLTAGE_THRESHOLD: f64 = 50.0;
pub const BRIDGE_VOLTAGE_CLAMP: f64 = 200.0;

pub const COMPARTMENT_SPHERE_SCALE: f32 = 0.3;
pub const TRIGGER_SPHERE_SCALE: f32 = 0.5;

pub const FHN_FIRE_VOLTAGE: f64 = 1.0;

pub const BEZIER_SEGMENTS: usize = 16;
pub const BEZIER_BEND_FRACTION: f32 = 0.2;

pub const MIN_CREATION_DISTANCE_AXON: f32 = 2.0 * NODE_RADIUS;
pub const MIN_CREATION_DISTANCE_DEFAULT: f32 = 6.0 * NODE_RADIUS;

pub const FPS_LOW_PASS_FACTOR: f64 = 0.05;
pub const TARGET_FRAME_MS: i64 = 16;

pub const CAMERA_MIN_DISTANCE: f32 = 5.0;
pub const CAMERA_MAX_DISTANCE: f32 = 400.0;

// Game constants - world
pub const PETRI_DISH_RADIUS: f32 = 120.0;
pub const PETRI_DISH_SEGMENTS: usize = 128;

// Game constants - resources
pub const RESOURCE_NODE_RADIUS: f32 = 8.0;
pub const RESOURCE_EMISSION_RATE: f64 = 8.0;
pub const RESOURCE_NODE_VISUAL_RADIUS: f32 = 2.0;

// Game constants - metabolism
pub const RESTING_METABOLIC_COST: f64 = 0.5;
pub const FIRING_METABOLIC_COST: f64 = 3.0;
pub const CONNECTION_MAINTENANCE_COST: f64 = 0.1;
pub const COMPARTMENT_METABOLIC_COST: f64 = 0.15;
pub const MEMBRANE_METABOLIC_COST: f64 = 0.2;
pub const RESOURCE_FLOW_RATE: f64 = 3.0;
pub const ORIGIN_MIN_ENERGY: f64 = 30.0;
pub const DEFAULT_NEURON_ENERGY: f64 = 80.0;
pub const MAX_NEURON_ENERGY: f64 = 150.0;

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
