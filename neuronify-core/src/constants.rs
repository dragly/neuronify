use neuronify_engine::units::{
    Celsius, Hertz, Micrometres, Milliseconds, Millivolts, Nanoamperes, Seconds,
};

pub const DT: Milliseconds = Milliseconds(0.025);
pub const DEFAULT_TEMPERATURE: Celsius = Celsius(16.3);
pub const RESTING_POTENTIAL: Millivolts = Millivolts(-65.0);
pub const MICROMETRES_PER_UNIT: Micrometres = Micrometres(50.0);
pub const FIRING_RATE_TIME_CONSTANT: Milliseconds = Milliseconds(50.0);
pub const PHYSICS_DT: Seconds = Seconds(0.001);

pub const SIMULATION_STEPS_PER_SECOND: Hertz = Hertz(250.0);
pub const DEFAULT_SPEED: f64 = 2.0;
pub const SPEEDS: [f64; 5] = [0.5, 1.0, 2.0, 4.0, 8.0];
pub const PHYSICS_STEPS_PER_SECOND: Hertz = Hertz(250.0);

pub const STIMULATION_CURRENT: Nanoamperes = Nanoamperes(0.3);

pub const NODE_RADIUS: f32 = 1.0;
pub const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;
pub const ATTACHMENT_RANGE: f32 = 1.5 * NODE_RADIUS;
pub const SELECTION_RANGE: f32 = 0.9 * NODE_RADIUS;
pub const SEGMENT_LENGTH: f32 = 2.0 * NODE_RADIUS;
pub const INTERNODE_LENGTH: f32 = 2.8 * SEGMENT_LENGTH;
pub const TERMINAL_INSET: f32 = 2.0 * NODE_RADIUS;
pub const CONNECTION_WIDTH_PER_STRENGTH: f32 = 0.3;

pub const VOLTMETER_TIME_WINDOW: Milliseconds = Milliseconds(1000.0 / 9.0);
pub const PROBE_EVERY: u64 = 4;
pub const VOLTMETER_MEASUREMENTS: usize =
    (VOLTMETER_TIME_WINDOW.0 / (DT.0 * PROBE_EVERY as f64)) as usize + 2;

pub const REPULSION_STRENGTH: f32 = 5.0;
pub const SPRING_STRENGTH: f32 = 10.0;
pub const ANGLE_ALIGNMENT_STRENGTH: f32 = 1.0;
pub const AXON_SMOOTHING_STRENGTH: f32 = 20.0;

pub const TERMINAL_THRESHOLD: Millivolts = Millivolts(-20.0);

pub const COMPARTMENT_SPHERE_SCALE: f32 = 0.3;

pub const MAX_ENDING_STRENGTH: f64 = 2.0;

pub const BEZIER_BEND_FRACTION: f32 = 0.2;

pub const MIN_CREATION_DISTANCE_AXON: f32 = 2.0 * NODE_RADIUS;
pub const MIN_CREATION_DISTANCE_DEFAULT: f32 = 6.0 * NODE_RADIUS;

pub const FLOOR_HEIGHT: f32 = -NODE_RADIUS;
pub const FLOOR_SIZE: f32 = 100_000.0;
pub const FLOOR_RINGS: i32 = 17;
