pub mod engine;
pub mod firing_rate;
pub mod mammal;
pub mod model;
pub mod spatial;
pub mod stimulation;

pub use engine::{run_headless, Engine, Spike};
pub use firing_rate::update_firing_rates;
pub use spatial::{apply_spatial_forces, integrate_motion};
pub use stimulation::{fire_nearby, touch_sensor_near, touched, Touch};

use std::collections::HashSet;

use hecs::{Entity, World};

use crate::constants::*;
use crate::measurement::firing_rate::record_rates;
use crate::measurement::loudspeaker::listen;
use crate::measurement::spike_detector::record_spikes;
use crate::measurement::voltmeter::record_voltages;

pub fn step_simulation(
    world: &mut World,
    engine: &mut Engine,
    time: &mut f64,
    steps: u32,
    touch: Option<Touch>,
) {
    engine.sync(world);
    engine.touch(world, touched(world, touch));
    let duration = steps as f64 * DT * 1e-3;
    engine::visual_inputs(world, engine, *time, duration);
    engine.run(steps);
    *time = engine.time() * 1e-3;
    engine.reflect(world);
    let spikes = engine.spikes();
    update_firing_rates(world, spikes, duration, *time);
    record_spikes(world, spikes, *time);
    record_rates(world, *time);
    listen(world, spikes, *time, duration);
    record_voltages(world, engine);
}

pub fn step_physics(world: &mut World, steps: u32, held: &HashSet<Entity>) {
    for _ in 0..steps {
        apply_spatial_forces(world, held);
        integrate_motion(world, PHYSICS_DT, held);
    }
}
