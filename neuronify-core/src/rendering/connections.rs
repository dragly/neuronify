use crate::components::*;
use crate::measurement::voltmeter::Voltmeter;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;
use crate::ConnectionTool;
use crate::Tool;

/// Flat lines for the voltmeter wires and for the connection being dragged out.
///
/// Every other connection is drawn as a tube along its `ConnectionPath`.
pub fn collect_connections(
    world: &hecs::World,
    tool: &Tool,
    connection_tool: &Option<ConnectionTool>,
    palette: &Palette,
) -> Vec<ConnectionData> {
    let mut connections: Vec<ConnectionData> = world
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
        .collect();

    if *tool == Tool::StaticConnection {
        if let Some(connection) = connection_tool {
            connections.push(ConnectionData {
                position_a: connection.start,
                position_b: connection.end,
                strength: 1.0,
                directional: 1.0,
                start_color: palette.neutral,
                end_color: palette.neutral,
                _padding: Default::default(),
            });
        }
    }

    connections
}
