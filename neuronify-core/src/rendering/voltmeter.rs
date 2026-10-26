use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::{Corner, Frame, VoltageSeries, Voltmeter};
use crate::rendering::colors;
use crate::rendering::gpu_types::ConnectionData;

pub fn collect_voltmeter_traces(world: &hecs::World) -> Vec<ConnectionData> {
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
