use std::f32::consts::{FRAC_PI_2, PI};

use glam::Vec3;
use hecs::World;

use crate::components::Position;
use crate::measurement::frame::Frame;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{ConnectionData, CylinderData, TileData};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::glyph_color;
use crate::rendering::tiles::TileShape;
use crate::stimuli::drawing::{frame_lines, tile_at, tile_in, DARK, FRAME_LIFT};
use crate::stimuli::visual_input::{
    direction_badge, Motion, VisualInput, DIRECTION_BADGE_RADIUS, RESOLUTION, VISUAL_INPUT_SIZE,
};

const TILE_FILL: f32 = 0.42;
const LIT: f32 = 0.8;
const BADGE_THICKNESS: f32 = 0.15;
const GLYPH_LIFT: f32 = 0.04;
const ARROW_HALF_LENGTH: f32 = 0.55;
const ARROW_HEAD: f32 = 0.3;
const ARROW_THICKNESS: f32 = 0.12;
const TURN_RADIUS: f32 = 0.45;
const TURN_SWEEP: f32 = 1.5 * PI;
const TURN_STEPS: usize = 12;

pub const PICTURE_DOTS: TileShape = TileShape {
    power: 2.0,
    spacing: VISUAL_INPUT_SIZE / (RESOLUTION - 1) as f32,
};

pub fn tile_position(center: Vec3, column: usize, row: usize) -> Vec3 {
    tile_in(center, VISUAL_INPUT_SIZE, column, row)
}

fn gray(brightness: f64) -> Vec3 {
    Vec3::splat(DARK + LIT * brightness as f32)
}

pub fn shapes(world: &World, scene: &mut Scene) {
    scene.discs.extend(tiles(world));
    badges(world, scene);
}

fn tiles(world: &World) -> Vec<TileData> {
    let half_width = TILE_FILL * VISUAL_INPUT_SIZE / RESOLUTION as f32;
    let mut tiles = Vec::new();
    for (input, position) in world.query::<(&VisualInput, &Position)>().iter() {
        for (index, &brightness) in input.picture().iter().enumerate() {
            tiles.push(TileData {
                position: tile_at(position.position, VISUAL_INPUT_SIZE, index),
                half_width,
                color: gray(brightness).extend(1.0),
            });
        }
    }
    tiles
}

fn badges(world: &World, scene: &mut Scene) {
    for (input, position) in world.query::<(&VisualInput, &Position)>().iter() {
        let Some(motion) = input.motion() else {
            continue;
        };
        let base = direction_badge(position.position) + Vec3::Y * FRAME_LIFT;
        scene.cylinders.push(CylinderData {
            start: base,
            start_radius: DIRECTION_BADGE_RADIUS,
            end: base + Vec3::Y * BADGE_THICKNESS,
            end_radius: DIRECTION_BADGE_RADIUS,
            start_color: orange(),
            end_color: orange(),
            _padding: Default::default(),
        });
        let glyph = base + Vec3::Y * (BADGE_THICKNESS + GLYPH_LIFT);
        let points = arrow(glyph, motion);
        let [.., before, tip] = points[..] else {
            continue;
        };
        let forward = (tip - before).normalize_or_zero();
        let side = Vec3::Y.cross(forward);
        scene.stroke(&points, ARROW_THICKNESS, glyph_color());
        scene.stroke(
            &[
                tip + (side - forward) * ARROW_HEAD,
                tip,
                tip - (side + forward) * ARROW_HEAD,
            ],
            ARROW_THICKNESS,
            glyph_color(),
        );
    }
}

fn arrow(center: Vec3, motion: Motion) -> Vec<Vec3> {
    let turn = match motion {
        Motion::Along(direction) => {
            return vec![
                center - direction * ARROW_HALF_LENGTH,
                center + direction * ARROW_HALF_LENGTH,
            ]
        }
        Motion::Clockwise => -TURN_SWEEP,
        Motion::Counterclockwise => TURN_SWEEP,
    };
    (0..=TURN_STEPS)
        .map(|step| {
            let angle = FRAC_PI_2 + turn * step as f32 / TURN_STEPS as f32;
            center + (Vec3::Z * angle.cos() + Vec3::X * angle.sin()) * TURN_RADIUS
        })
        .collect()
}

pub fn lines(world: &World) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for position in world.query::<&Position>().with::<&VisualInput>().iter() {
        let frame = Frame::square(position.position, VISUAL_INPUT_SIZE);
        frame_lines(&frame, orange(), &mut lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_engine::units::Hertz;

    #[test]
    fn the_picture_is_a_mosaic_of_tiles_inside_the_frame() {
        let mut world = World::new();
        let center = Vec3::new(3.0, 0.0, -2.0);
        world.spawn((Position { position: center }, VisualInput::default()));
        let tiles = tiles(&world);
        assert_eq!(tiles.len(), RESOLUTION * RESOLUTION);
        let frame = Frame::square(center, VISUAL_INPUT_SIZE);
        for tile in &tiles {
            assert!(frame.contains(tile.position));
        }
        assert!(tiles[0].position.x > tiles[RESOLUTION * (RESOLUTION - 1)].position.x);
        assert!(tiles[0].position.z < tiles[RESOLUTION - 1].position.z);
    }

    #[test]
    fn the_badge_arrow_points_the_way_the_picture_moves_and_turns_with_it() {
        let mut world = World::new();
        let center = Vec3::new(1.0, 0.0, 2.0);
        let picture = world.spawn((Position { position: center }, VisualInput::default()));
        let badged = |world: &World| {
            let mut scene = Scene::default();
            badges(world, &mut scene);
            scene
        };
        let tip_offset = |world: &World| {
            let scene = badged(world);
            let badge = scene.cylinders[0].start.with_y(0.0);
            let tip = scene.cylinders[1].end.with_y(0.0);
            tip - badge
        };
        assert!(tip_offset(&world).z > 0.4);
        world.get::<&mut VisualInput>(picture).unwrap().reverse();
        assert!(tip_offset(&world).z < -0.4);
        world.get::<&mut VisualInput>(picture).unwrap().speed = Hertz(0.0);
        assert!(badged(&world).cylinders.is_empty());
    }
}
