use glam::Vec3;
use hecs::{Entity, World};

use crate::components::{NeuronType, Position};
use crate::measurement::frame::Frame;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{ConnectionData, TileData};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::generator_color;
use crate::rendering::tiles::TileShape;
use crate::stimuli::drawing::{frame_lines, line, tile_at, DARK};
use crate::stimuli::receptive_field::{watched_picture, ReceptiveFieldCell, RECEPTIVE_FIELD_SIZE};
use crate::stimuli::visual_input::{VisualInput, RESOLUTION};
use crate::stimuli::GeneratorDynamics;

const FIELD_TILE_FILL: f32 = 0.5;
const LIGHT_ON_FIELD: f32 = 0.9;
const BAR_GAP: f32 = 0.6;
const BAR_STRENGTH: f32 = 1.2;

pub const FIELD_SQUIRCLES: TileShape = TileShape {
    power: 4.0,
    spacing: RECEPTIVE_FIELD_SIZE / (RESOLUTION - 1) as f32,
};

pub fn shapes(world: &World, palette: &Palette, scene: &mut Scene) {
    scene.squircles.extend(tiles(world, palette));
}

fn tiles(world: &World, palette: &Palette) -> Vec<TileData> {
    let half_width = FIELD_TILE_FILL * RECEPTIVE_FIELD_SIZE / RESOLUTION as f32;
    let mut tiles = Vec::new();
    for (cell, field, position) in world
        .query::<(Entity, &ReceptiveFieldCell, &Position)>()
        .iter()
    {
        let picture = watched_picture(world, cell)
            .and_then(|picture| world.get::<&VisualInput>(picture).ok())
            .map(|input| input.picture());
        for (index, color) in field_colors(field, picture.as_deref(), palette)
            .into_iter()
            .enumerate()
        {
            tiles.push(TileData {
                position: tile_at(position.position, RECEPTIVE_FIELD_SIZE, index),
                half_width,
                color: color.extend(1.0),
            });
        }
    }
    tiles
}

pub fn field_colors(
    field: &ReceptiveFieldCell,
    picture: Option<&[f64]>,
    palette: &Palette,
) -> Vec<Vec3> {
    let weights = field.weights();
    let peak = weights
        .iter()
        .fold(0.0, |peak: f64, weight| peak.max(weight.abs()));
    weights
        .iter()
        .enumerate()
        .map(|(index, &weight)| {
            let seen = picture.map_or(0.0, |picture| picture[index]) as f32;
            let tint = if weight > 0.0 {
                palette.excitatory
            } else {
                palette.inhibitory
            };
            let shape = if peak > 0.0 {
                (weight.abs() / peak).sqrt() as f32
            } else {
                0.0
            };
            (Vec3::splat(DARK) + tint * shape + yellow() * LIGHT_ON_FIELD * seen).min(Vec3::ONE)
        })
        .collect()
}

fn match_bar(frame: &Frame, response: f64, lines: &mut Vec<ConnectionData>) {
    let below = frame.bottom_left - Vec3::X * BAR_GAP;
    let middle = below + Vec3::Z * frame.width * 0.5;
    lines.push(line(
        below,
        below + Vec3::Z * frame.width,
        BAR_STRENGTH,
        frame_color(),
    ));
    let reach = (response.abs() as f32).min(0.49) * frame.width;
    if reach > 0.0 {
        let (end, color) = if response > 0.0 {
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

pub fn lines(world: &World, palette: &Palette) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for (field, position, dynamics, neuron_type) in world
        .query::<(
            &ReceptiveFieldCell,
            &Position,
            &GeneratorDynamics,
            Option<&NeuronType>,
        )>()
        .iter()
    {
        let frame = Frame::square(position.position, RECEPTIVE_FIELD_SIZE);
        let color = generator_color(dynamics, neuron_type.copied().unwrap_or_default(), palette);
        frame_lines(&frame, color, &mut lines);
        match_bar(&frame, field.response, &mut lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::tiles::superellipse_area;
    use crate::rendering::Theme;
    use crate::stimuli::drawing::FRAME_STRENGTH;
    use crate::stimuli::receptive_field::ReceptiveField;
    use crate::stimuli::visual_input::{spawn_picture_wire, Pattern};
    use neuronify_engine::units::Hertz;

    fn center_surround() -> ReceptiveFieldCell {
        ReceptiveFieldCell {
            field: ReceptiveField::CenterSurround {
                off_center: false,
                x: 0.0,
                y: 0.0,
            },
            ..Default::default()
        }
    }

    const CENTER: usize = RESOLUTION / 2 * RESOLUTION + RESOLUTION / 2;

    #[test]
    fn a_field_shows_its_center_in_the_full_excitatory_color_and_its_surround_inhibitory() {
        for theme in [Theme::Dark, Theme::Light] {
            let palette = theme.palette();
            let colors = field_colors(&center_surround(), None, &palette);
            let excitatory = Vec3::splat(DARK) + palette.excitatory;
            assert!(colors[CENTER].abs_diff_eq(excitatory, 1e-5), "{theme:?}");
            let ring = colors[CENTER + 4] - Vec3::splat(DARK);
            assert!(
                ring.normalize().dot(palette.inhibitory.normalize()) > 0.999,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn light_on_a_field_adds_its_own_color_where_it_falls() {
        let palette = Theme::Dark.palette();
        let field = center_surround();
        let dark = vec![0.0; RESOLUTION * RESOLUTION];
        let mut lit = dark.clone();
        lit[CENTER] = 1.0;
        let in_the_dark = field_colors(&field, Some(&dark), &palette);
        let in_the_light = field_colors(&field, Some(&lit), &palette);
        let added = in_the_light[CENTER] - in_the_dark[CENTER];
        assert!(
            added.abs_diff_eq(yellow() * LIGHT_ON_FIELD, 1e-5),
            "{added}"
        );
        assert_eq!(in_the_light[0], in_the_dark[0]);
    }

    #[test]
    fn a_wired_field_shows_the_picture_it_watches() {
        let mut world = World::new();
        let picture = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            VisualInput {
                pattern: Pattern::Dot,
                speed: Hertz::ZERO,
                ..Default::default()
            },
        ));
        let cell = world.spawn((
            Position {
                position: Vec3::new(0.0, 0.0, 10.0),
            },
            center_surround(),
        ));
        let palette = Theme::Dark.palette();
        let colors = |world: &World| {
            tiles(world, &palette)
                .iter()
                .map(|tile| tile.color)
                .collect::<Vec<_>>()
        };
        let unwired = colors(&world);
        spawn_picture_wire(&mut world, picture, cell);
        assert_ne!(unwired, colors(&world));
    }

    #[test]
    fn a_field_is_tiled_almost_without_gaps() {
        let mut world = World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            center_surround(),
        ));
        let tiles = tiles(&world, &Theme::Dark.palette());
        let shape = superellipse_area(FIELD_SQUIRCLES.power);
        let step = FIELD_SQUIRCLES.spacing;
        let covered: f32 = tiles
            .iter()
            .map(|tile| shape * tile.half_width * tile.half_width)
            .sum();
        assert!(covered > 0.8 * (step * RESOLUTION as f32).powi(2));
        assert!(tiles.iter().all(|tile| 2.0 * tile.half_width < step));
    }

    #[test]
    fn the_tiles_never_cover_the_frame() {
        let mut world = World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            VisualInput::default(),
        ));
        world.spawn((
            Position {
                position: Vec3::new(0.0, 0.0, 10.0),
            },
            ReceptiveFieldCell::default(),
            GeneratorDynamics::default(),
        ));
        let palette = Theme::Dark.palette();
        let mut scene = Scene::default();
        crate::stimuli::drawing::shapes(&world, &palette, &mut scene);
        let top = scene
            .discs
            .iter()
            .chain(&scene.squircles)
            .map(|tile| tile.position.y)
            .fold(f32::MIN, f32::max);
        let frames: Vec<_> = crate::stimuli::drawing::lines(&world, &palette)
            .into_iter()
            .filter(|line| line.strength == FRAME_STRENGTH)
            .collect();
        assert_eq!(frames.len(), 8);
        for line in frames {
            assert!(line.position_a.y > top && line.position_b.y > top);
        }
    }

    #[test]
    fn every_receptive_field_is_drawn_as_clearly_as_the_edge() {
        let palette = Theme::Dark.palette();
        let tints = |field: ReceptiveField| {
            let colors = field_colors(
                &ReceptiveFieldCell {
                    field,
                    ..Default::default()
                },
                None,
                &palette,
            );
            let dark = Vec3::splat(DARK);
            let strongest = |tint: Vec3, excitatory: bool| {
                colors
                    .iter()
                    .filter(|color| (color.z > color.x) == excitatory)
                    .map(|&color| (color - dark).length() / tint.length())
                    .fold(0.0, f32::max)
            };
            (
                strongest(palette.excitatory, true),
                strongest(palette.inhibitory, false),
            )
        };
        let (edge, _) = tints(ReceptiveField::KINDS[2]);
        for field in ReceptiveField::KINDS {
            let (center, lobes) = tints(field);
            assert!((center - edge).abs() < 1e-4, "{field:?}");
            assert!(lobes > edge / 4.0, "{field:?}");
        }
    }

    #[test]
    fn the_match_bar_grows_right_for_a_match_and_left_for_the_opposite() {
        let mut world = World::new();
        let cell = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            ReceptiveFieldCell::default(),
            GeneratorDynamics::default(),
        ));
        let palette = Theme::Dark.palette();
        for (response, rightwards) in [(0.2, true), (-0.2, false)] {
            world.get::<&mut ReceptiveFieldCell>(cell).unwrap().response = response;
            let bar = *crate::stimuli::drawing::lines(&world, &palette)
                .last()
                .unwrap();
            assert_eq!(bar.position_b.z > bar.position_a.z, rightwards);
            assert!(
                ((bar.position_b.z - bar.position_a.z).abs() - 0.2 * RECEPTIVE_FIELD_SIZE).abs()
                    < 1e-4
            );
        }
    }
}
