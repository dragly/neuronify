use std::collections::VecDeque;

use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Milliseconds};
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::spike_detector::SpikeTrain;
use crate::measurement::voltmeter::measured;

pub const RATE_SAMPLE_INTERVAL: Milliseconds = Milliseconds(1.0);
pub const RATE_HISTORY: Milliseconds = Milliseconds(1000.0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RatePlot {
    pub window: Milliseconds,
    pub kernel_width: Milliseconds,
    pub minimum: Hertz,
    pub maximum: Hertz,
}

impl Default for RatePlot {
    fn default() -> Self {
        Self {
            window: Milliseconds(300.0),
            kernel_width: Milliseconds(200.0),
            minimum: Hertz(0.0),
            maximum: Hertz(100.0),
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
    pub samples: VecDeque<(Milliseconds, Hertz)>,
}

pub fn firing_rate(
    spike_times: impl Iterator<Item = Milliseconds>,
    now: Milliseconds,
    window: Milliseconds,
    kernel_width: Milliseconds,
) -> Hertz {
    let sigma = kernel_width.max(Milliseconds(1e-3));
    let area = sigma
        * (std::f64::consts::PI / 2.0).sqrt()
        * erf(window / (sigma * std::f64::consts::SQRT_2));
    let weight: f64 = spike_times
        .map(|time| now - time)
        .filter(|age| (Milliseconds::ZERO..window).contains(age))
        .map(|age| {
            let spread = age / sigma;
            (-spread * spread / 2.0).exp()
        })
        .sum();
    (weight / area.max(Milliseconds(1e-9))).to_hertz()
}

fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let polynomial = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    (1.0 - polynomial * (-x * x).exp()).copysign(x)
}

pub fn record_rates(world: &mut World, time: Milliseconds) {
    let plots: Vec<(Entity, RatePlot)> = world
        .query::<(Entity, (&RatePlot, &RateSeries))>()
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
        let rates: Vec<Hertz> = measured(world, plot)
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
            Hertz::ZERO
        } else {
            rates.iter().sum::<Hertz>() / rates.len() as f64
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
        for (frequency, window, kernel_width) in [
            (40.0, 300.0, 200.0),
            (40.0, 400.0, 300.0),
            (10.0, 1000.0, 100.0),
        ] {
            let frequency = Hertz(frequency);
            let (window, kernel_width) = (Milliseconds(window), Milliseconds(kernel_width));
            let interval = frequency.period();
            let spikes: Vec<Milliseconds> = (0..200).map(|i| i as f64 * interval).collect();
            let samples = 100;
            let mean = (0..samples)
                .map(|step| {
                    let now = 100.0 * interval + interval * step as f64 / samples as f64;
                    firing_rate(spikes.iter().copied(), now, window, kernel_width)
                })
                .sum::<Hertz>()
                / samples as f64;
            assert!(
                (mean - frequency).abs() < 0.03 * frequency,
                "{mean} for {frequency}"
            );
        }
    }

    #[test]
    fn only_spikes_within_the_window_count() {
        let rate = |spikes: &[f64]| {
            firing_rate(
                spikes.iter().copied().map(Milliseconds),
                Milliseconds(1000.0),
                Milliseconds(300.0),
                Milliseconds(200.0),
            )
        };
        assert_eq!(rate(&[0.0, 500.0]), Hertz::ZERO);
        assert!(rate(&[800.0]) > Hertz::ZERO);
    }

    #[test]
    fn a_rate_plot_follows_the_rate_of_the_neurons_wired_to_it() {
        use crate::elements::{spawn_rate_plot, Element};
        use crate::simulation::{step_simulation, Engine};
        let mut world = World::new();
        let source = Element::RegularSpikeGenerator.spawn(&mut world, glam::Vec3::ZERO);
        world
            .get::<&mut crate::components::RegularSpikeGenerator>(source)
            .unwrap()
            .frequency = Hertz(40.0);
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, glam::Vec3::Z * 5.0);
        crate::elements::spawn_synapse(&mut world, source, neuron);
        let plot = spawn_rate_plot(&mut world, neuron, glam::Vec3::X * 5.0);
        let (mut engine, mut time) = (Engine::default(), Milliseconds::ZERO);
        let mut spikes = 0;
        while time < Milliseconds(3000.0) {
            step_simulation(&mut world, &mut engine, &mut time, 40, None);
            if time > Milliseconds(2000.0) {
                spikes += engine
                    .spikes()
                    .iter()
                    .filter(|spike| spike.entity == neuron)
                    .count();
            }
        }
        let series = world.get::<&RateSeries>(plot).unwrap();
        let recent: Vec<Hertz> = series.samples.iter().map(|&(_, rate)| rate).collect();
        let mean = (recent.iter().sum::<Hertz>() / recent.len() as f64).0;
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
