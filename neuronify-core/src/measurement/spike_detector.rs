use std::collections::VecDeque;

use hecs::{Entity, World};
use neuronify_engine::units::Milliseconds;
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::firing_rate::RatePlot;
use crate::simulation::Spike;

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
    pub times: VecDeque<Milliseconds>,
}

pub fn record_spikes(world: &mut World, spikes: &[Spike], time: Milliseconds) {
    for detector in world.query_mut::<&mut SpikeDetector>() {
        detector.time = time;
    }
    let wires: Vec<(Entity, Entity, Milliseconds)> = world
        .query::<(Entity, &Connection)>()
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
            train.times.push_back(spike.time);
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
        let detector = spawn_spike_detector(&mut world, wired, Vec3::ZERO);
        let wire = crate::measurement::voltmeter::measured(&world, detector)[0].0;
        let fired = Milliseconds(1000.0);
        let spikes_at = |time| [wired, unwired].map(|entity| Spike { entity, time });
        record_spikes(&mut world, &spikes_at(fired), fired);
        assert_eq!(spikes(&world, wire), [fired]);
        assert_eq!(world.get::<&SpikeDetector>(detector).unwrap().time, fired);

        record_spikes(&mut world, &[], Milliseconds(1050.0));
        assert_eq!(spikes(&world, wire), [fired]);
        record_spikes(
            &mut world,
            &[],
            fired + SpikeDetector::default().time_range + DT,
        );
        assert!(spikes(&world, wire).is_empty());
    }
}
