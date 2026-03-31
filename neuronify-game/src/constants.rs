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
