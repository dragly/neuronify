use glam::Vec3;

use crate::components::*;
use crate::measurement::visual_input::{
    grid, visual_input_frame, VisualInput, RESOLUTION, VISUAL_INPUT_SIZE,
};
use crate::measurement::voltmeter::Corner;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{ConnectionData, CylinderData};

const TILE_HEIGHT: f32 = 0.05;
const TILE_FILL: f32 = 0.42;
const FIELD_TINT: f32 = 0.35;
const BAR_GAP: f32 = 0.6;
const BAR_STRENGTH: f32 = 1.2;

fn line(position_a: Vec3, position_b: Vec3, strength: f32, color: Vec3) -> ConnectionData {
    ConnectionData {
        position_a,
        position_b,
        strength,
        directional: 0.0,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    }
}

pub fn tile_position(center: Vec3, column: usize, row: usize) -> Vec3 {
    center + Vec3::Z * grid(column) as f32 * VISUAL_INPUT_SIZE
        - Vec3::X * grid(row) as f32 * VISUAL_INPUT_SIZE
        + Vec3::Y * TILE_HEIGHT
}

pub fn collect_visual_input_tiles(world: &hecs::World) -> Vec<CylinderData> {
    let tile_radius = TILE_FILL * VISUAL_INPUT_SIZE / RESOLUTION as f32;
    let mut tiles = Vec::new();
    for (_, (input, position)) in world.query::<(&VisualInput, &Position)>().iter() {
        let picture = input.picture(input.seen_at.unwrap_or(0.0));
        let weights = input.field_weights();
        for (index, (&brightness, &weight)) in picture.iter().zip(&weights).enumerate() {
            let gray = Vec3::splat(0.04 + 0.8 * brightness as f32);
            let tint = if weight > 0.0 { rate_color() } else { blue() };
            let strength = if input.show_field {
                FIELD_TINT * (weight.abs() as f32).min(1.0)
            } else {
                0.0
            };
            let center = tile_position(position.position, index % RESOLUTION, index / RESOLUTION);
            let color = gray.lerp(tint, strength);
            tiles.push(CylinderData {
                start: center - Vec3::Y * TILE_HEIGHT,
                start_radius: tile_radius,
                end: center,
                end_radius: tile_radius,
                start_color: color,
                end_color: color,
                _padding: Default::default(),
            });
        }
    }
    tiles
}

pub fn collect_visual_input_lines(world: &hecs::World) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for (_, (input, position)) in world.query::<(&VisualInput, &Position)>().iter() {
        let frame = visual_input_frame(position.position);
        let corners = [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomRight,
            Corner::BottomLeft,
        ]
        .map(|corner| frame.corner(corner));
        for (index, &corner) in corners.iter().enumerate() {
            lines.push(line(
                corner,
                corners[(index + 1) % corners.len()],
                0.3,
                orange(),
            ));
        }
        let below = frame.bottom_left - Vec3::X * BAR_GAP;
        let middle = below + Vec3::Z * VISUAL_INPUT_SIZE * 0.5;
        lines.push(line(
            below,
            below + Vec3::Z * VISUAL_INPUT_SIZE,
            BAR_STRENGTH,
            frame_color(),
        ));
        let reach = (input.response.abs() as f32).min(0.49) * VISUAL_INPUT_SIZE;
        if reach > 0.0 {
            let (end, color) = if input.response > 0.0 {
                (middle + Vec3::Z * reach, orange())
            } else {
                (middle - Vec3::Z * reach, Vec3::splat(0.5))
            };
            lines.push(line(
                middle + Vec3::Y * 0.01,
                end + Vec3::Y * 0.01,
                BAR_STRENGTH,
                color,
            ));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::visual_input::{Pattern, ReceptiveField};
    use hecs::World;

    #[test]
    fn the_picture_is_a_mosaic_of_tiles_inside_the_frame() {
        let mut world = World::new();
        let center = Vec3::new(3.0, 0.0, -2.0);
        world.spawn((Position { position: center }, VisualInput::default()));
        let tiles = collect_visual_input_tiles(&world);
        assert_eq!(tiles.len(), RESOLUTION * RESOLUTION);
        let frame = visual_input_frame(center);
        for tile in &tiles {
            assert!(frame.contains(tile.end));
        }
        assert!(tiles[0].end.x > tiles[RESOLUTION * (RESOLUTION - 1)].end.x);
        assert!(tiles[0].end.z < tiles[RESOLUTION - 1].end.z);
    }

    #[test]
    fn showing_the_receptive_field_tints_the_tiles() {
        let mut world = World::new();
        let input = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            VisualInput {
                pattern: Pattern::Dot,
                field: ReceptiveField::CenterSurround {
                    off_center: false,
                    x: 0.0,
                    y: 0.0,
                },
                ..Default::default()
            },
        ));
        let shown = collect_visual_input_tiles(&world);
        world.get::<&mut VisualInput>(input).unwrap().show_field = false;
        let hidden = collect_visual_input_tiles(&world);
        let center = RESOLUTION / 2 * RESOLUTION + RESOLUTION / 2;
        assert_ne!(shown[center].start_color, hidden[center].start_color);
        assert!(hidden
            .iter()
            .all(|tile| tile.start_color.x == tile.start_color.z));
    }

    #[test]
    fn the_match_bar_grows_right_for_a_match_and_left_for_the_opposite() {
        let mut world = World::new();
        let input = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            VisualInput::default(),
        ));
        for (response, rightwards) in [(0.2, true), (-0.2, false)] {
            world.get::<&mut VisualInput>(input).unwrap().response = response;
            let bar = *collect_visual_input_lines(&world).last().unwrap();
            assert_eq!(bar.position_b.z > bar.position_a.z, rightwards);
            assert!(
                ((bar.position_b.z - bar.position_a.z).abs() - 0.2 * VISUAL_INPUT_SIZE).abs()
                    < 1e-4
            );
        }
    }
}
