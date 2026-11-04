use std::collections::VecDeque;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::firing_rate::RatePlot;
use crate::simulation::Spike;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpikeDetector {
    pub time_range: f64,
    pub time: f64,
}

impl Default for SpikeDetector {
    fn default() -> Self {
        Self {
            time_range: 0.1,
            time: 0.0,
        }
    }
}

impl SpikeDetector {
    pub fn size() -> MeterSize {
        MeterSize {
            width: 8.0,
            height: 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum RowOrder {
    #[default]
    Height,
    Connection,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpikeTrain {
    pub times: VecDeque<f64>,
}

pub fn record_spikes(world: &mut World, spikes: &[Spike], time: f64) {
    for (_, detector) in world.query_mut::<&mut SpikeDetector>() {
        detector.time = time;
    }
    let wires: Vec<(Entity, Entity, f64)> = world
        .query::<&Connection>()
        .with::<&SpikeTrain>()
        .iter()
        .filter_map(|(wire, connection)| {
            let range = match world.get::<&SpikeDetector>(connection.to) {
                Ok(detector) => detector.time_range,
                Err(_) => world.get::<&RatePlot>(connection.to).ok()?.window,
            };
            Some((wire, connection.from, range))
        })
        .collect();
    for (wire, from, range) in wires {
        let Ok(mut train) = world.get::<&mut SpikeTrain>(wire) else {
            continue;
        };
        for spike in spikes.iter().filter(|spike| spike.entity == from) {
            train.times.push_back(spike.time * 1e-3);
        }
        while train
            .times
            .front()
            .is_some_and(|&spike| spike < time - range)
        {
            train.times.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DT;
    use crate::elements::spawn_spike_detector;
    use glam::Vec3;

    const STEP: f64 = DT * 1e-3;

    fn spikes(world: &World, wire: Entity) -> Vec<f64> {
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
        let detector = spawn_spike_detector(&mut world, wired, Vec3::ZERO);
        let wire = crate::measurement::voltmeter::measured(&world, detector)[0].0;
        let spikes_at = |time: f64| {
            [wired, unwired].map(|entity| Spike {
                entity,
                time: time * 1e3,
            })
        };
        record_spikes(&mut world, &spikes_at(1.0), 1.0);
        assert_eq!(spikes(&world, wire), [1.0]);
        assert_eq!(world.get::<&SpikeDetector>(detector).unwrap().time, 1.0);

        record_spikes(&mut world, &[], 1.05);
        assert_eq!(spikes(&world, wire), [1.0]);
        record_spikes(
            &mut world,
            &[],
            1.0 + SpikeDetector::default().time_range + STEP,
        );
        assert!(spikes(&world, wire).is_empty());
    }
}
