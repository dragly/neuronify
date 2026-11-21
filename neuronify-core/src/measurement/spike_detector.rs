pub mod drawing;
pub mod gui;

use std::ops::RangeInclusive;

use glam::Vec3;
use hecs::World;
use neuronify_engine::units::Milliseconds;
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::edited::{edited, edited_settings, Settings};
use crate::measurement::frame::RowOrder;
use crate::measurement::About;
use crate::serialization::bounds::within;

pub const TIME_RANGES: RangeInclusive<Milliseconds> = Milliseconds(1.0)..=Milliseconds(1e6);

pub const ABOUT: About = About {
    name: "Spike detector",
    description: "Press a neuron to draw a tick each time it fires. \
                  Connect more neurons to give each a row.",
    size: Some(MeterSize {
        width: 8.0,
        height: 6.0,
    }),
    offset: Vec3::X,
    edited: &[
        edited_settings::<SpikeDetector, TimeRange>(),
        edited::<RowOrder>(),
    ],
};

pub fn bound(world: &mut World) {
    for detector in world.query_mut::<&mut SpikeDetector>() {
        let default = SpikeDetector::default();
        within(&mut detector.time_range, TIME_RANGES, default.time_range);
        if !detector.time.is_finite() {
            detector.time = Milliseconds::ZERO;
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpikeDetector {
    pub time_range: Milliseconds,
    pub time: Milliseconds,
}

impl Default for SpikeDetector {
    fn default() -> Self {
        Self {
            time_range: Milliseconds(100.0),
            time: Milliseconds::ZERO,
        }
    }
}

struct TimeRange(Milliseconds);

impl Settings<SpikeDetector> for TimeRange {
    fn of(detector: &SpikeDetector) -> Self {
        TimeRange(detector.time_range)
    }

    fn swap(&mut self, detector: &mut SpikeDetector) {
        std::mem::swap(&mut self.0, &mut detector.time_range);
    }
}

pub fn advance(world: &mut World, time: Milliseconds) {
    for detector in world.query_mut::<&mut SpikeDetector>() {
        detector.time = time;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DT;
    use crate::measurement::{record, Meter, SpikeTrain};
    use crate::simulation::Spike;
    use glam::Vec3;
    use hecs::Entity;

    fn spikes(world: &World, wire: Entity) -> Vec<Milliseconds> {
        world
            .get::<&SpikeTrain>(wire)
            .unwrap()
            .times
            .iter()
            .copied()
            .collect()
    }

    #[test]
    fn records_spikes_of_wired_neurons_within_the_time_range() {
        let mut world = World::new();
        let [wired, unwired] = [(); 2].map(|_| world.spawn((NodeState::default(),)));
        let detector = Meter::SpikeDetector.spawn_measuring(&mut world, wired, Vec3::ZERO);
        let wire = crate::measurement::frame::measured(&world, detector)[0].0;
        let fired = Milliseconds(1000.0);
        let spikes_at = |time| [wired, unwired].map(|entity| Spike { entity, time });
        record(&mut world, &spikes_at(fired), &[], fired, DT);
        assert_eq!(spikes(&world, wire), [fired]);
        assert_eq!(world.get::<&SpikeDetector>(detector).unwrap().time, fired);

        record(&mut world, &[], &[], Milliseconds(1050.0), DT);
        assert_eq!(spikes(&world, wire), [fired]);
        record(
            &mut world,
            &[],
            &[],
            fired + SpikeDetector::default().time_range + DT,
            DT,
        );
        assert!(spikes(&world, wire).is_empty());
    }
}
