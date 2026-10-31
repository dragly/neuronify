use std::collections::VecDeque;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::*;

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpikeTrain {
    pub times: VecDeque<f64>,
}

pub fn record_spikes(world: &mut World, time: f64) {
    let mut detectors = world
        .query_mut::<&mut SpikeDetector>()
        .into_iter()
        .peekable();
    if detectors.peek().is_none() {
        return;
    }
    for (_, detector) in detectors {
        detector.time = time;
    }
    let wires: Vec<(Entity, bool, f64)> = world
        .query::<&Connection>()
        .with::<&SpikeTrain>()
        .iter()
        .filter_map(|(wire, connection)| {
            let range = world.get::<&SpikeDetector>(connection.to).ok()?.time_range;
            let fired = world
                .get::<&LeakyDynamics>(connection.from)
                .is_ok_and(|dynamics| dynamics.time_since_fire == 0.0);
            Some((wire, fired, range))
        })
        .collect();
    for (wire, fired, range) in wires {
        let Ok(mut train) = world.get::<&mut SpikeTrain>(wire) else {
            continue;
        };
        if fired {
            train.times.push_back(time);
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
    use crate::constants::LIF_DT;
    use crate::elements::spawn_spike_detector;
    use glam::Vec3;

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
        let [wired, unwired] = [(); 2].map(|_| world.spawn((LeakyDynamics::default(),)));
        let detector = spawn_spike_detector(&mut world, wired, Vec3::ZERO);
        let wire = crate::measurement::voltmeter::measured(&world, detector)[0].0;
        for neuron in [wired, unwired] {
            world
                .get::<&mut LeakyDynamics>(neuron)
                .unwrap()
                .time_since_fire = 0.0;
        }
        record_spikes(&mut world, 1.0);
        assert_eq!(spikes(&world, wire), [1.0]);
        assert_eq!(world.get::<&SpikeDetector>(detector).unwrap().time, 1.0);

        world
            .get::<&mut LeakyDynamics>(wired)
            .unwrap()
            .time_since_fire = LIF_DT;
        record_spikes(&mut world, 1.05);
        assert_eq!(spikes(&world, wire), [1.0]);
        record_spikes(
            &mut world,
            1.0 + SpikeDetector::default().time_range + LIF_DT,
        );
        assert!(spikes(&world, wire).is_empty());
    }
}
