pub mod fhn;
pub mod firing_rate;
pub mod lif;
pub mod spatial;
pub mod stimulation;

pub use fhn::fhn_step;
pub use firing_rate::update_firing_rates;
pub use lif::{lif_step, run_headless, SpikeRecord};
pub use spatial::{apply_spatial_forces, integrate_motion};
pub use stimulation::{fire_nearby, stimulate_nearby, touch_sensor_near};

use std::collections::HashSet;

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::LeakyDynamics;
use crate::constants::*;
use crate::measurement::firing_rate::record_rates;
use crate::measurement::loudspeaker::listen;
use crate::measurement::spike_detector::record_spikes;

pub fn step_simulation(world: &mut World, time: &mut f64, steps: u32, touch: Option<Vec3>) {
    for _ in 0..steps {
        if let Some(position) = touch {
            stimulate_nearby(world, position);
        }
        lif_step(world, LIF_DT, *time);
        update_firing_rates(world, LIF_DT);
        record_spikes(world, *time);
        record_rates(world, *time);
        listen(world, *time, LIF_DT);
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
