pub mod firing_rate;
pub mod loudspeaker;
pub mod spike_detector;
pub mod voltmeter;

use hecs::{Entity, World};

use crate::components::MeterSize;
use loudspeaker::Loudspeaker;

pub fn is_meter(world: &World, entity: Entity) -> bool {
    world
        .satisfies::<hecs::Or<&MeterSize, &Loudspeaker>>(entity)
        .unwrap_or(false)
}
