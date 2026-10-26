use glam::Vec3;

use crate::components::*;
use crate::measurement::voltmeter::Voltmeter;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;

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

pub fn connection_being_drawn(start: Vec3, end: Vec3, palette: &Palette) -> ConnectionData {
    ConnectionData {
        position_a: start,
        position_b: end,
        strength: 1.0,
        directional: 1.0,
        start_color: palette.neutral,
        end_color: palette.neutral,
        _padding: Default::default(),
    }
}
