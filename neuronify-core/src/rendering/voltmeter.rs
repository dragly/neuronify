use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::{
    measured, Corner, Frame, VoltageSeries, Voltmeter, VoltmeterRange,
};
use crate::rendering::colors;
use crate::rendering::gpu_types::ConnectionData;

const LINE_STRENGTH: f32 = 0.3;

fn visible_tail<T>(
    oldest_first: impl DoubleEndedIterator<Item = T>,
    is_visible: impl Fn(&T) -> bool,
) -> Vec<T> {
    let mut tail: Vec<T> = oldest_first.rev().take_while(is_visible).collect();
    tail.reverse();
    tail
}

fn line(position_a: glam::Vec3, position_b: glam::Vec3, color: glam::Vec3) -> ConnectionData {
    ConnectionData {
        position_a,
        position_b,
        strength: LINE_STRENGTH,
        directional: 0.0,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    }
}

pub fn collect_voltmeter_traces(world: &hecs::World) -> Vec<ConnectionData> {
    let mut result = Vec::new();
    for (voltmeter, (position, size, range)) in world
        .query::<(&Position, &VoltmeterSize, Option<&VoltmeterRange>)>()
        .with::<&Voltmeter>()
        .iter()
    {
        let frame = Frame::new(position.position, size);
        let corners = [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomRight,
            Corner::BottomLeft,
        ]
        .map(|corner| frame.corner(corner));
        for (index, &corner) in corners.iter().enumerate() {
            result.push(line(
                corner,
                corners[(index + 1) % corners.len()],
                colors::frame_color(),
            ));
        }
        let range = range.cloned().unwrap_or_default();
        let measured = measured(world, voltmeter);
        for (index, (wire, target)) in measured.iter().enumerate() {
            let row = frame.row(index, measured.len());
            if index > 0 {
                result.push(line(
                    row.corner(Corner::TopLeft),
                    row.corner(Corner::TopRight),
                    colors::frame_color(),
                ));
            }
            let Ok(series) = world.get::<&VoltageSeries>(*wire) else {
                continue;
            };
            let (low, high) = if world.get::<&Compartment>(*target).is_ok() {
                (FHN_VOLTAGE_MIN, FHN_VOLTAGE_MAX)
            } else {
                (range.minimum, range.maximum)
            };
            result.extend(trace(&row, &series, low, high));
        }
    }
    result
}

fn trace(row: &Frame, series: &VoltageSeries, low: f64, high: f64) -> Vec<ConnectionData> {
    let mut result = Vec::new();
    let Some(latest_time) = series.measurements.last().map(|m| m.time) else {
        return result;
    };
    let start_time = latest_time - VOLTMETER_TIME_WINDOW;
    let is_visible = |time: f64| (start_time..=latest_time).contains(&time);
    let visible = visible_tail(
        series.measurements.iter().map(|m| (m.time, m.voltage)),
        |&(time, _)| is_visible(time),
    );
    let at = |time: f64, voltage: f64| {
        row.point(
            ((time - start_time) / VOLTMETER_TIME_WINDOW) as f32,
            ((voltage - low) / (high - low)).clamp(0.0, 1.0) as f32,
        )
    };
    let color = colors::green();
    for pair in visible.windows(2) {
        let (t0, v0) = pair[0];
        let (t1, v1) = pair[1];
        result.push(line(at(t0, v0), at(t1, v1), color));
    }
    for spike_time in visible_tail(series.spike_times.iter().copied(), |&time| is_visible(time)) {
        let right = ((spike_time - start_time) / VOLTMETER_TIME_WINDOW) as f32;
        result.push(line(row.point(right, 1.0), row.point(right, 0.0), color));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_voltmeter, spawn_voltmeter_wire};
    use crate::measurement::voltmeter::{RollingWindow, VoltageMeasurement};
    use glam::Vec3;
    use hecs::World;

    fn trace_from_full_copy(
        row: &Frame,
        series: &VoltageSeries,
        low: f64,
        high: f64,
    ) -> Vec<ConnectionData> {
        let measurements: Vec<(f64, f64)> = series
            .measurements
            .iter()
            .map(|m| (m.time, m.voltage))
            .collect();
        let Some(&(latest_time, _)) = measurements.last() else {
            return Vec::new();
        };
        let start_time = latest_time - VOLTMETER_TIME_WINDOW;
        let at = |time: f64, voltage: f64| {
            row.point(
                ((time - start_time) / VOLTMETER_TIME_WINDOW) as f32,
                ((voltage - low) / (high - low)).clamp(0.0, 1.0) as f32,
            )
        };
        let visible: Vec<_> = measurements
            .iter()
            .filter(|(time, _)| *time >= start_time)
            .collect();
        let mut result: Vec<ConnectionData> = visible
            .windows(2)
            .map(|pair| {
                line(
                    at(pair[0].0, pair[0].1),
                    at(pair[1].0, pair[1].1),
                    colors::green(),
                )
            })
            .collect();
        for &spike_time in &series.spike_times {
            if spike_time < start_time || spike_time > latest_time {
                continue;
            }
            let right = ((spike_time - start_time) / VOLTMETER_TIME_WINDOW) as f32;
            result.push(line(
                row.point(right, 1.0),
                row.point(right, 0.0),
                colors::green(),
            ));
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

    fn bytes(data: &[ConnectionData]) -> &[u8] {
        bytemuck::cast_slice(data)
    }

    fn row() -> Frame {
        Frame::new(
            Vec3::new(1.0, 0.0, 0.0),
            &VoltmeterSize {
                width: 6.0,
                height: 3.0,
            },
        )
    }

    #[test]
    fn a_trace_matches_one_drawn_from_a_full_copy() {
        let more_samples_than_capacity =
            || (0..5000).map(|i| (i as f64 * LIF_DT, -70.0 + (i as f64 * 0.01).sin()));
        let spikes: Vec<f64> = (0..5000).step_by(37).map(|i| i as f64 * LIF_DT).collect();
        let full = series(more_samples_than_capacity(), spikes);
        let drawn = trace(&row(), &full, -100.0, 50.0);
        assert!(drawn.len() > 1000);
        assert_eq!(
            bytes(&drawn),
            bytes(&trace_from_full_copy(&row(), &full, -100.0, 50.0))
        );
    }

    #[test]
    fn a_trace_ignores_history_newer_than_the_latest_measurement() {
        let saved_until_5_s = (0..2000).map(|i| (5.0 - (2000 - i) as f64 * LIF_DT, -60.0));
        let recorded_after_loading_at_0 =
            || (0..500).map(|i| (i as f64 * LIF_DT, -65.0 + i as f64 * 0.01));
        let new_spikes = vec![0.01, 0.02];
        let loaded = series(
            saved_until_5_s.chain(recorded_after_loading_at_0()),
            [vec![4.95, 4.99], new_spikes.clone()].concat(),
        );
        let fresh = series(recorded_after_loading_at_0(), new_spikes);
        let expected = trace(&row(), &fresh, -100.0, 50.0);
        assert_eq!(
            bytes(&trace(&row(), &loaded, -100.0, 50.0)),
            bytes(&expected)
        );
        assert_ne!(
            bytes(&trace_from_full_copy(&row(), &loaded, -100.0, 50.0)),
            bytes(&expected)
        );
    }

    #[test]
    fn each_measured_neuron_gets_a_row_from_the_top_in_the_order_it_was_added() {
        let mut world = World::new();
        let neurons = [0.0, 5.0, 10.0].map(|z| {
            world.spawn((
                Position {
                    position: Vec3::new(-10.0, 0.0, z),
                },
                LeakyDynamics::default(),
            ))
        });
        let voltmeter = spawn_voltmeter(&mut world, neurons[0], Vec3::ZERO);
        for &neuron in &neurons[1..] {
            spawn_voltmeter_wire(&mut world, neuron, voltmeter);
        }
        let frame = Frame::of(&world, voltmeter).unwrap();
        for (index, (wire, neuron)) in measured(&world, voltmeter).into_iter().enumerate() {
            assert_eq!(neuron, neurons[index]);
            let rising = (0..10).map(|i| (i as f64 * LIF_DT, -100.0 + i as f64 * 150.0 / 9.0));
            *world.get::<&mut VoltageSeries>(wire).unwrap() = series(rising, vec![]);
        }
        let traces = collect_voltmeter_traces(&world);
        let separators = 2;
        let segments = 9;
        assert_eq!(traces.len(), 4 + separators + 3 * segments);
        let rising: Vec<ConnectionData> = traces
            .into_iter()
            .filter(|line| line.start_color == colors::green())
            .collect();
        for (index, row_trace) in rising.chunks(segments).enumerate() {
            let row = frame.row(index, 3);
            let lowest = row_trace.first().unwrap().position_a;
            let highest = row_trace.last().unwrap().position_b;
            assert!((lowest.x - row.bottom_left.x).abs() < 1e-4, "row {index}");
            assert!(
                (highest.x - (row.bottom_left.x + row.height)).abs() < 1e-4,
                "row {index}"
            );
        }
    }

    #[test]
    fn the_range_sets_the_bottom_and_top_of_each_chart() {
        let mut world = World::new();
        let neuron = world.spawn((Position {
            position: Vec3::new(-10.0, 0.0, 0.0),
        },));
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::ZERO);
        *world.get::<&mut VoltmeterRange>(voltmeter).unwrap() = VoltmeterRange {
            minimum: -80.0,
            maximum: -40.0,
        };
        let (wire, _) = measured(&world, voltmeter)[0];
        *world.get::<&mut VoltageSeries>(wire).unwrap() = series(
            [(0.0, -80.0), (LIF_DT, -60.0), (2.0 * LIF_DT, 0.0)].into_iter(),
            vec![],
        );
        let frame = Frame::of(&world, voltmeter).unwrap();
        let traces = collect_voltmeter_traces(&world);
        let [bottom, middle] = [traces[4].position_a, traces[4].position_b];
        let top = traces[5].position_b;
        assert!((bottom.x - frame.bottom_left.x).abs() < 1e-4);
        assert!((middle.x - (frame.bottom_left.x + frame.height * 0.5)).abs() < 1e-4);
        assert!((top.x - (frame.bottom_left.x + frame.height)).abs() < 1e-4);
    }
}
