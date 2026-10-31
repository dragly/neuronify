use glam::Vec3;

use crate::components::*;
use crate::measurement::voltmeter::Voltmeter;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;

const MIN_SEGMENT_LENGTH: f32 = 1e-4;

pub fn collect_voltmeter_wires(world: &hecs::World, palette: &Palette) -> Vec<ConnectionData> {
    world
        .query::<&Connection>()
        .with::<&Voltmeter>()
        .iter()
        .filter_map(|(_, connection)| {
            let start = world.get::<&Position>(connection.from).ok()?.position;
            let end = world.get::<&Position>(connection.to).ok()?.position;
            Some(ConnectionData {
                position_a: start,
                position_b: end,
                strength: connection.strength as f32,
                directional: 0.0,
                start_color: palette.neutral,
                end_color: palette.neutral,
                _padding: Default::default(),
            })
        })
        .collect()
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
