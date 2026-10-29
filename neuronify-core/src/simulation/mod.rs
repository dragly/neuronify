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

pub fn step_simulation(world: &mut World, time: &mut f64, steps: u32) {
    for _ in 0..steps {
        lif_step(world, LIF_DT, *time);
        *time += LIF_DT;

        let recently_fired: HashSet<Entity> = world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < FHN_FIRE_DURATION)
            .map(|(e, _)| e)
            .collect();
        fhn_step(world, FHN_CDT, &recently_fired);
    }
}

pub fn step_physics(world: &mut World, steps: u32) {
    for _ in 0..steps {
        apply_spatial_forces(world);
        integrate_motion(world, PHYSICS_DT);
    }
}
