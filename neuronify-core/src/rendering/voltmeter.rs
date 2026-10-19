use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::{Corner, Frame, VoltageSeries, Voltmeter};
use crate::rendering::colors;
use crate::rendering::gpu_types::ConnectionData;

fn visible_tail<T>(
    oldest_first: impl DoubleEndedIterator<Item = T>,
    is_visible: impl Fn(&T) -> bool,
) -> Vec<T> {
    let mut tail: Vec<T> = oldest_first.rev().take_while(is_visible).collect();
    tail.reverse();
    tail
}

pub fn collect_voltmeter_traces(world: &hecs::World) -> Vec<ConnectionData> {
    let mut result = Vec::new();

    let time_window = VOLTMETER_TIME_WINDOW;

    for (voltmeter_id, _) in world.query::<&Voltmeter>().iter() {
        let (visible, spike_times, start_time, frame, is_compartment) = {
            let Ok(series) = world.get::<&VoltageSeries>(voltmeter_id) else {
                continue;
            };
            if series.measurements.len() < 2 {
                continue;
            }
            let Ok(pos) = world.get::<&Position>(voltmeter_id) else {
                continue;
            };
            let size = world
                .get::<&VoltmeterSize>(voltmeter_id)
                .map(|size| (*size).clone())
                .unwrap_or_default();
            let is_comp = world
                .get::<&Connection>(voltmeter_id)
                .ok()
                .map(|conn| world.get::<&Compartment>(conn.from).is_ok())
                .unwrap_or(false);
            let latest_time = series.measurements.last().map(|m| m.time).unwrap_or(0.0);
            let start_time = latest_time - time_window;
            let is_visible = |time: f64| (start_time..=latest_time).contains(&time);
            let visible = visible_tail(
                series.measurements.iter().map(|m| (m.time, m.voltage)),
                |&(time, _)| is_visible(time),
            );
            let spikes = visible_tail(series.spike_times.iter().copied(), |&time| is_visible(time));
            (
                visible,
                spikes,
                start_time,
                Frame::new(pos.position, &size),
                is_comp,
            )
        };

        let (v_min, v_max) = if is_compartment {
            (FHN_VOLTAGE_MIN, FHN_VOLTAGE_MAX)
        } else {
            (LIF_VOLTAGE_MIN, LIF_VOLTAGE_MAX)
        };

        let trace_color = colors::green();
        let fc = colors::frame_color();

        let bottom_left = frame.corner(Corner::BottomLeft);
        let bottom_right = frame.corner(Corner::BottomRight);
        let top_left = frame.corner(Corner::TopLeft);
        let top_right = frame.corner(Corner::TopRight);
        for (a, b) in [
            (top_left, top_right),
            (top_right, bottom_right),
            (bottom_right, bottom_left),
            (bottom_left, top_left),
        ] {
            result.push(ConnectionData {
                position_a: a,
                position_b: b,
                strength: 0.3,
                directional: 0.0,
                start_color: fc,
                end_color: fc,
                _padding: Default::default(),
            });
        }

        for window in visible.windows(2) {
            let (t0, v0) = window[0];
            let (t1, v1) = window[1];

            let p0 = frame.point(
                ((t0 - start_time) / time_window) as f32,
                ((v0 - v_min) / (v_max - v_min)) as f32,
            );
            let p1 = frame.point(
                ((t1 - start_time) / time_window) as f32,
                ((v1 - v_min) / (v_max - v_min)) as f32,
            );

            result.push(ConnectionData {
                position_a: p0,
                position_b: p1,
                strength: 0.3,
                directional: 0.0,
                start_color: trace_color,
                end_color: trace_color,
                _padding: Default::default(),
            });
        }

        for spike_time in &spike_times {
            let right = ((spike_time - start_time) / time_window) as f32;
            let top = frame.point(right, 1.0);
            let bottom = frame.point(right, 0.0);
            result.push(ConnectionData {
                position_a: top,
                position_b: bottom,
                strength: 0.3,
                directional: 0.0,
                start_color: trace_color,
                end_color: trace_color,
                _padding: Default::default(),
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::voltmeter::{RollingWindow, VoltageMeasurement};
    use glam::Vec3;
    use hecs::{Entity, World};

    fn collect_voltmeter_traces_from_full_copy(world: &hecs::World) -> Vec<ConnectionData> {
        let mut result = Vec::new();

        for (voltmeter_id, _) in world.query::<&Voltmeter>().iter() {
            let (series, spike_times, frame, is_compartment) = {
                let Ok(series) = world.get::<&VoltageSeries>(voltmeter_id) else {
                    continue;
                };
                let Ok(pos) = world.get::<&Position>(voltmeter_id) else {
                    continue;
                };
                let size = world
                    .get::<&VoltmeterSize>(voltmeter_id)
                    .map(|size| (*size).clone())
                    .unwrap_or_default();
                let is_comp = world
                    .get::<&Connection>(voltmeter_id)
                    .ok()
                    .map(|conn| world.get::<&Compartment>(conn.from).is_ok())
                    .unwrap_or(false);
                let measurements: Vec<_> = series
                    .measurements
                    .iter()
                    .map(|m| (m.time, m.voltage))
                    .collect();
                let spikes = series.spike_times.clone();
                (
                    measurements,
                    spikes,
                    Frame::new(pos.position, &size),
                    is_comp,
                )
            };

            if series.len() < 2 {
                continue;
            }

            let time_window = VOLTMETER_TIME_WINDOW;
            let (v_min, v_max) = if is_compartment {
                (FHN_VOLTAGE_MIN, FHN_VOLTAGE_MAX)
            } else {
                (LIF_VOLTAGE_MIN, LIF_VOLTAGE_MAX)
            };

            let latest_time = series.last().map(|(t, _)| *t).unwrap_or(0.0);
            let start_time = latest_time - time_window;

            let trace_color = colors::green();
            let fc = colors::frame_color();

            let bottom_left = frame.corner(Corner::BottomLeft);
            let bottom_right = frame.corner(Corner::BottomRight);
            let top_left = frame.corner(Corner::TopLeft);
            let top_right = frame.corner(Corner::TopRight);
            for (a, b) in [
                (top_left, top_right),
                (top_right, bottom_right),
                (bottom_right, bottom_left),
                (bottom_left, top_left),
            ] {
                result.push(ConnectionData {
                    position_a: a,
                    position_b: b,
                    strength: 0.3,
                    directional: 0.0,
                    start_color: fc,
                    end_color: fc,
                    _padding: Default::default(),
                });
            }

            let visible: Vec<_> = series.iter().filter(|(t, _)| *t >= start_time).collect();

            for window in visible.windows(2) {
                let (t0, v0) = window[0];
                let (t1, v1) = window[1];

                let p0 = frame.point(
                    ((t0 - start_time) / time_window) as f32,
                    ((v0 - v_min) / (v_max - v_min)) as f32,
                );
                let p1 = frame.point(
                    ((t1 - start_time) / time_window) as f32,
                    ((v1 - v_min) / (v_max - v_min)) as f32,
                );

                result.push(ConnectionData {
                    position_a: p0,
                    position_b: p1,
                    strength: 0.3,
                    directional: 0.0,
                    start_color: trace_color,
                    end_color: trace_color,
                    _padding: Default::default(),
                });
            }

            for spike_time in &spike_times {
                if *spike_time < start_time || *spike_time > latest_time {
                    continue;
                }
                let right = ((spike_time - start_time) / time_window) as f32;
                let top = frame.point(right, 1.0);
                let bottom = frame.point(right, 0.0);
                result.push(ConnectionData {
                    position_a: top,
                    position_b: bottom,
                    strength: 0.3,
                    directional: 0.0,
                    start_color: trace_color,
                    end_color: trace_color,
                    _padding: Default::default(),
                });
            }
        }

        result
    }

    fn series(samples: impl Iterator<Item = (f64, f64)>, spike_times: Vec<f64>) -> VoltageSeries {
        let mut measurements = RollingWindow::new(3000);
        for (time, voltage) in samples {
            measurements.push(VoltageMeasurement { voltage, time });
        }
        VoltageSeries {
            measurements,
            spike_times,
        }
    }

    fn spawn_voltmeter(world: &mut World, target: Entity, z: f32, series: VoltageSeries) {
        world.spawn((
            Voltmeter {},
            Position {
                position: Vec3::new(1.0, 0.0, z),
            },
            series,
            Connection {
                from: target,
                to: Entity::DANGLING,
                strength: 1.0,
                directional: true,
            },
            VoltmeterSize {
                width: 6.0,
                height: 3.0,
            },
        ));
    }

    fn bytes(data: &[ConnectionData]) -> &[u8] {
        bytemuck::cast_slice(data)
    }

    #[test]
    fn voltmeter_traces_match_full_copy_implementation() {
        let mut world = World::new();
        let neuron = world.spawn((Position {
            position: Vec3::ZERO,
        },));
        let compartment = world.spawn((Compartment {
            voltage: 0.0,
            m: 0.0,
            h: 0.0,
            n: 0.0,
            influence: 0.0,
            capacitance: 1.0,
            injected_current: 0.0,
            fire_impulse: 0.0,
        },));

        let more_samples_than_capacity =
            || (0..5000).map(|i| (i as f64 * LIF_DT, -70.0 + (i as f64 * 0.01).sin()));
        let spikes: Vec<f64> = (0..5000).step_by(37).map(|i| i as f64 * LIF_DT).collect();
        let full = || series(more_samples_than_capacity(), spikes.clone());
        let too_short_to_draw = series(more_samples_than_capacity().take(1), vec![]);
        spawn_voltmeter(&mut world, neuron, 0.0, full());
        spawn_voltmeter(&mut world, compartment, 10.0, full());
        spawn_voltmeter(&mut world, neuron, 20.0, too_short_to_draw);
        let _without_size_or_connection = world.spawn((
            Voltmeter {},
            Position {
                position: Vec3::new(1.0, 0.0, 30.0),
            },
            series(more_samples_than_capacity().take(100), vec![0.001]),
        ));

        let traces = collect_voltmeter_traces(&world);
        let expected = collect_voltmeter_traces_from_full_copy(&world);
        assert!(traces.len() > 2000);
        assert_eq!(bytes(&traces), bytes(&expected));
    }

    #[test]
    fn voltmeter_traces_ignore_history_newer_than_latest_measurement() {
        let saved_until_5_s = (0..2000).map(|i| (5.0 - (2000 - i) as f64 * LIF_DT, -60.0));
        let recorded_after_loading_at_0 =
            || (0..500).map(|i| (i as f64 * LIF_DT, -65.0 + i as f64 * 0.01));
        let new_spikes = vec![0.01, 0.02];

        let mut loaded = World::new();
        let neuron = loaded.spawn((Position {
            position: Vec3::ZERO,
        },));
        spawn_voltmeter(
            &mut loaded,
            neuron,
            0.0,
            series(
                saved_until_5_s.chain(recorded_after_loading_at_0()),
                [vec![4.95, 4.99], new_spikes.clone()].concat(),
            ),
        );

        let mut fresh = World::new();
        let neuron = fresh.spawn((Position {
            position: Vec3::ZERO,
        },));
        spawn_voltmeter(
            &mut fresh,
            neuron,
            0.0,
            series(recorded_after_loading_at_0(), new_spikes),
        );

        let expected = collect_voltmeter_traces(&fresh);
        assert_eq!(bytes(&collect_voltmeter_traces(&loaded)), bytes(&expected));
        assert_ne!(
            bytes(&collect_voltmeter_traces_from_full_copy(&loaded)),
            bytes(&expected)
        );
    }
}
