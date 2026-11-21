use std::collections::VecDeque;

use hecs::World;
use neuronify_engine::units::{Milliseconds, Millivolts};
use serde::{Deserialize, Serialize};

use crate::constants::VOLTMETER_MEASUREMENTS;
use crate::simulation::Engine;

const SMALLEST_GROWTH: usize = 16;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollingWindow<T> {
    data: VecDeque<T>,
    max_capacity: usize,
}

impl<T> RollingWindow<T> {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            data: VecDeque::new(),
            max_capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        while !self.data.is_empty() && self.data.len() >= self.max_capacity {
            self.data.pop_front();
        }
        if self.data.len() < self.max_capacity {
            if self.data.len() == self.data.capacity() {
                let growth = self.data.len().max(SMALLEST_GROWTH);
                self.data
                    .reserve_exact(growth.min(self.max_capacity - self.data.len()));
            }
            self.data.push_back(value);
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator + '_ {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.data.iter_mut()
    }
    pub fn last(&self) -> Option<&T> {
        self.data.back()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn retain(&mut self, keep: impl FnMut(&T) -> bool) {
        self.data.retain(keep);
    }

    pub fn keep_latest(&mut self, capacity: usize) {
        let surplus = self.data.len().saturating_sub(capacity);
        self.data.drain(..surplus);
        self.data.shrink_to(capacity);
        self.max_capacity = capacity;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageMeasurement {
    pub voltage: Millivolts,
    pub time: Milliseconds,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageSeries {
    pub measurements: RollingWindow<VoltageMeasurement>,
    #[serde(default)]
    pub spike_times: Vec<Milliseconds>,
}

impl VoltageSeries {
    pub fn new() -> VoltageSeries {
        VoltageSeries {
            measurements: RollingWindow::new(VOLTMETER_MEASUREMENTS),
            spike_times: Vec::new(),
        }
    }

    pub fn bound(&mut self, capacity: usize) {
        self.measurements
            .retain(|m| m.time.is_finite() && m.voltage.is_finite());
        self.measurements.keep_latest(capacity);
        self.spike_times.retain(|time| time.is_finite());
        self.forget_spikes_before_measurements();
    }

    fn forget_spikes_before_measurements(&mut self) {
        let oldest = self.measurements.iter().next().map(|m| m.time);
        self.spike_times
            .retain(|&time| oldest.is_none_or(|oldest| time >= oldest));
    }
}

impl Default for VoltageSeries {
    fn default() -> Self {
        VoltageSeries::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Voltmeter {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct VoltmeterRange {
    pub minimum: Millivolts,
    pub maximum: Millivolts,
}

impl Default for VoltmeterRange {
    fn default() -> Self {
        Self {
            minimum: Millivolts(-100.0),
            maximum: Millivolts(50.0),
        }
    }
}

pub fn record_voltages(world: &mut World, engine: &mut Engine) {
    let samples = engine.samples();
    let spikes = engine.spikes();
    for probe in samples {
        let Ok(mut series) = world.get::<&mut VoltageSeries>(probe.wire) else {
            continue;
        };
        for (time, voltage) in probe.samples {
            series
                .measurements
                .push(VoltageMeasurement { voltage, time });
        }
        for spike in spikes.iter().filter(|spike| spike.entity == probe.target) {
            series.spike_times.push(spike.time);
        }
        series.forget_spikes_before_measurements();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::frame::measured;
    use glam::Vec3;

    #[test]
    fn a_rolling_window_takes_memory_as_it_fills_and_never_beyond_its_capacity() {
        let mut window = RollingWindow::new(VOLTMETER_MEASUREMENTS);
        assert_eq!(window.data.capacity(), 0);
        for sample in 0..VOLTMETER_MEASUREMENTS {
            window.push(sample);
            assert!(window.data.capacity() <= VOLTMETER_MEASUREMENTS);
            assert!(window.data.capacity() <= 2 * window.len().max(16));
        }
        for sample in 0..VOLTMETER_MEASUREMENTS {
            window.push(sample);
        }
        assert_eq!(window.len(), VOLTMETER_MEASUREMENTS);
        assert_eq!(window.data.capacity(), VOLTMETER_MEASUREMENTS);
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct RollingWindowBackedByVec {
        data: Vec<u32>,
        max_capacity: usize,
    }

    #[test]
    fn rolling_window_keeps_vec_behaviour_and_format() {
        let mut window = RollingWindow::new(5);
        let mut reference = RollingWindowBackedByVec {
            data: Vec::new(),
            max_capacity: 5,
        };
        for value in 0..12 {
            window.push(value);
            if reference.data.len() == reference.max_capacity {
                reference.data.remove(0);
            }
            reference.data.push(value);

            assert_eq!(window.iter().copied().collect::<Vec<_>>(), reference.data);
            assert_eq!(window.last(), reference.data.last());

            let bytes = postcard::to_stdvec(&window).unwrap();
            assert_eq!(bytes, postcard::to_stdvec(&reference).unwrap());
            let loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(loaded.iter().copied().collect::<Vec<_>>(), reference.data);
        }
    }

    #[test]
    fn rolling_window_stays_within_capacity() {
        let mut empty = RollingWindow::new(0);
        empty.push(1);
        empty.push(2);
        assert!(empty.is_empty());

        let corrupt = RollingWindowBackedByVec {
            data: vec![0, 1],
            max_capacity: 0,
        };
        let bytes = postcard::to_stdvec(&corrupt).unwrap();
        let mut loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
        loaded.push(2);
        assert!(loaded.is_empty());

        let oversized = RollingWindowBackedByVec {
            data: (0..8).collect(),
            max_capacity: 3,
        };
        let bytes = postcard::to_stdvec(&oversized).unwrap();
        let mut loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
        loaded.push(8);
        assert_eq!(loaded.iter().copied().collect::<Vec<_>>(), [6, 7, 8]);
    }

    #[test]
    fn a_firing_neuron_keeps_only_the_voltages_and_spikes_its_plot_shows() {
        use crate::constants::VOLTMETER_TIME_WINDOW;
        use crate::elements::{spawn_connection, spawn_voltmeter, Element};
        use crate::simulation::run_headless;

        let mut world = hecs::World::new();
        let source = Element::CurrentSource.spawn(&mut world, Vec3::ZERO);
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(0.0, 0.0, 5.0));
        spawn_connection(&mut world, source, neuron);
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::new(10.0, 0.0, 5.0));
        let spikes = run_headless(&mut world, Milliseconds(1000.0));
        assert!(spikes.len() > 20, "{}", spikes.len());

        let (wire, _) = measured(&world, voltmeter)[0];
        let series = world.get::<&VoltageSeries>(wire).unwrap();
        let oldest = series.measurements.iter().next().unwrap().time;
        let latest = series.measurements.last().unwrap().time;
        assert_eq!(series.measurements.len(), VOLTMETER_MEASUREMENTS);
        assert!(latest - oldest >= VOLTMETER_TIME_WINDOW);
        assert!(!series.spike_times.is_empty());
        assert!(series.spike_times.iter().all(|&time| time >= oldest));
    }
}
