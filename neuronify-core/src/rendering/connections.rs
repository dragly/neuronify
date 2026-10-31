use glam::Vec3;

use crate::components::*;
use crate::measurement::voltmeter::{measured, Frame};
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;

const MIN_SEGMENT_LENGTH: f32 = 1e-4;

pub fn collect_meter_wires(world: &hecs::World, palette: &Palette) -> Vec<ConnectionData> {
    let mut wires = Vec::new();
    for (meter, _) in world.query::<()>().with::<&MeterSize>().iter() {
        let Some(frame) = Frame::of(world, meter) else {
            continue;
        };
        let measured = measured(world, meter);
        for (row, (_, neuron)) in measured.iter().enumerate() {
            let Ok(start) = world.get::<&Position>(*neuron).map(|p| p.position) else {
                continue;
            };
            wires.push(ConnectionData {
                position_a: start,
                position_b: frame.row(row, measured.len()).point(0.0, 0.5),
                strength: 1.0,
                directional: 0.0,
                start_color: palette.neutral,
                end_color: palette.neutral,
                _padding: Default::default(),
            });
        }
    }
    wires
}

pub fn connection_being_drawn(points: &[Vec3], palette: &Palette) -> Vec<ConnectionData> {
    let last = points.len().saturating_sub(2);
    points
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[0].distance(pair[1]) > MIN_SEGMENT_LENGTH)
        .map(|(index, pair)| ConnectionData {
            position_a: pair[0],
            position_b: pair[1],
            strength: 1.0,
            directional: if index == last { 1.0 } else { 0.0 },
            start_color: palette.neutral,
            end_color: palette.neutral,
            _padding: Default::default(),
        })
        .collect()
}
