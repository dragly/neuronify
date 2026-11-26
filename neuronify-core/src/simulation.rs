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
use neuronify_engine::units::Milliseconds;

use crate::constants::*;
use crate::measurement;

pub fn step_simulation(
    world: &mut World,
    engine: &mut Engine,
    time: &mut Milliseconds,
    steps: u32,
    touch: Option<Touch>,
) {
    engine.sync(world);
    engine.touch(world, touched(world, touch));
    let duration = steps as f64 * DT;
    crate::stimuli::advance(world, engine, *time, duration);
    engine.run(steps);
    *time = engine.time();
    engine.reflect(world);
    update_firing_rates(world, engine.spikes(), duration, *time);
    let probes = engine.samples();
    measurement::record(world, engine.spikes(), &probes, *time, duration);
}

pub fn step_physics(world: &mut World, steps: u32, held: &HashSet<Entity>) {
    for _ in 0..steps {
        apply_spatial_forces(world, held);
        integrate_motion(world, PHYSICS_DT, held);
    }
}
