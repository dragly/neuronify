use std::collections::VecDeque;

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::spike_detector::SpikeTrain;
use crate::measurement::voltmeter::measured;

pub const RATE_SAMPLE_INTERVAL: f64 = 1e-3;
pub const RATE_HISTORY: f64 = 1.0;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RatePlot {
    pub window: f64,
    pub kernel_width: f64,
    pub minimum: f64,
    pub maximum: f64,
}

impl Default for RatePlot {
    fn default() -> Self {
        Self {
            window: 0.3,
            kernel_width: 0.2,
            minimum: 0.0,
            maximum: 100.0,
        }
    }
}

impl RatePlot {
    pub fn size() -> MeterSize {
        MeterSize {
            width: 8.0,
            height: 5.0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RateSeries {
    pub samples: VecDeque<(f64, f64)>,
}

pub fn firing_rate(
    spike_times: impl Iterator<Item = f64>,
    now: f64,
    window: f64,
    kernel_width: f64,
) -> f64 {
    let sigma = kernel_width.max(1e-6);
    let area = sigma
        * (std::f64::consts::PI / 2.0).sqrt()
        * erf(window / (sigma * std::f64::consts::SQRT_2));
    spike_times
        .map(|time| now - time)
        .filter(|age| (0.0..window).contains(age))
        .map(|age| (-age * age / (2.0 * sigma * sigma)).exp())
        .sum::<f64>()
        / area.max(1e-12)
}

fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let polynomial = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    (1.0 - polynomial * (-x * x).exp()).copysign(x)
}

pub fn record_rates(world: &mut World, time: f64) {
    let plots: Vec<(Entity, RatePlot)> = world
        .query::<(&RatePlot, &RateSeries)>()
        .iter()
        .filter(|(_, (_, series))| {
            series
                .samples
                .back()
                .is_none_or(|&(last, _)| time - last >= RATE_SAMPLE_INTERVAL || time < last)
        })
        .map(|(plot, (settings, _))| (plot, settings.clone()))
        .collect();
    for (plot, settings) in plots {
        let rates: Vec<f64> = measured(world, plot)
            .into_iter()
            .filter_map(|(wire, _)| {
                let train = world.get::<&SpikeTrain>(wire).ok()?;
                Some(firing_rate(
                    train.times.iter().copied(),
                    time,
                    settings.window,
                    settings.kernel_width,
                ))
            })
            .collect();
        let rate = if rates.is_empty() {
            0.0
        } else {
            rates.iter().sum::<f64>() / rates.len() as f64
        };
        let Ok(mut series) = world.get::<&mut RateSeries>(plot) else {
            continue;
        };
        if series.samples.back().is_some_and(|&(last, _)| last > time) {
            series.samples.clear();
        }
        series.samples.push_back((time, rate));
        while series
            .samples
            .front()
            .is_some_and(|&(sampled, _)| sampled < time - RATE_HISTORY)
        {
            series.samples.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_steady_rate_reads_as_itself_on_average() {
        for (frequency, window, kernel_width) in
            [(40.0, 0.3, 0.2), (40.0, 0.4, 0.3), (10.0, 1.0, 0.1)]
        {
            let spikes: Vec<f64> = (0..200).map(|i| i as f64 / frequency).collect();
            let interval = 1.0 / frequency;
            let samples = 100;
            let mean = (0..samples)
                .map(|step| {
                    let now = 100.0 * interval + interval * step as f64 / samples as f64;
                    firing_rate(spikes.iter().copied(), now, window, kernel_width)
                })
                .sum::<f64>()
                / samples as f64;
            assert!(
                (mean - frequency).abs() < 0.03 * frequency,
                "{mean} for {frequency}"
            );
        }
    }

    #[test]
    fn only_spikes_within_the_window_count() {
        let rate = firing_rate([0.0, 0.5].into_iter(), 1.0, 0.3, 0.2);
        assert_eq!(rate, 0.0);
        assert!(firing_rate([0.8].into_iter(), 1.0, 0.3, 0.2) > 0.0);
    }

    #[test]
    fn a_rate_plot_follows_the_rate_of_the_neurons_wired_to_it() {
        use crate::elements::{spawn_rate_plot, Element};
        use crate::simulation::step_simulation;
        let mut world = World::new();
        let source = Element::RegularSpikeGenerator.spawn(&mut world, glam::Vec3::ZERO);
        world
            .get::<&mut crate::components::RegularSpikeGenerator>(source)
            .unwrap()
            .frequency = 40.0;
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, glam::Vec3::Z * 5.0);
        crate::elements::spawn_synapse(&mut world, source, neuron);
        let plot = spawn_rate_plot(&mut world, neuron, glam::Vec3::X * 5.0);
        let mut time = 0.0;
        let mut spikes = 0;
        while time < 3.0 {
            step_simulation(&mut world, &mut time, 1, None);
            if time > 2.0 && world.get::<&LeakyDynamics>(neuron).unwrap().time_since_fire == 0.0 {
                spikes += 1;
            }
        }
        let series = world.get::<&RateSeries>(plot).unwrap();
        let recent: Vec<f64> = series.samples.iter().map(|&(_, rate)| rate).collect();
        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        assert!(spikes > 20, "{spikes} spikes");
        assert!(
            (mean - spikes as f64).abs() < 0.1 * spikes as f64,
            "{mean} vs {spikes}"
        );
        let span = series.samples.back().unwrap().0 - series.samples.front().unwrap().0;
        assert!(
            (span - RATE_HISTORY).abs() < 2.0 * RATE_SAMPLE_INTERVAL,
            "{span}"
        );
    }

    #[test]
    fn erf_matches_known_values() {
        for (x, expected) in [(0.0, 0.0), (0.5, 0.5205), (1.0, 0.8427), (2.0, 0.9953)] {
            assert!((erf(x) - expected).abs() < 1e-3, "erf({x})");
            assert!((erf(-x) + expected).abs() < 1e-3, "erf(-{x})");
        }
    }
}
