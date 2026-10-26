pub mod fhn;
pub mod lif;
pub mod spatial;
pub mod stimulation;

pub use fhn::fhn_step;
pub use lif::{lif_step, run_headless, SpikeRecord};
pub use spatial::{apply_spatial_forces, integrate_motion};
pub use stimulation::{stimulate_nearby, touch_sensor_near};

use std::collections::HashSet;

use hecs::{Entity, World};

use crate::components::LeakyDynamics;
use crate::constants::*;

pub fn step_frame(world: &mut World, time: &mut f64, iterations: u32) {
    for _ in 0..iterations {
        lif_step(world, LIF_DT, *time);
        *time += LIF_DT;
    }

    let recently_fired: HashSet<Entity> = world
        .query::<&LeakyDynamics>()
        .iter()
        .filter(|(_, d)| d.time_since_fire < iterations as f64 * LIF_DT)
        .map(|(e, _)| e)
        .collect();

    for _ in 0..iterations {
        fhn_step(world, FHN_CDT, &recently_fired);
        apply_spatial_forces(world);
        integrate_motion(world, PHYSICS_DT);
    }
}
