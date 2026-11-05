use glam::Vec3;
use hecs::Entity;
use visula::{
    Expression, InstanceBuffer, QuadPipeline, QuadPipelineDescriptor, RenderingDescriptor,
};
use visula_derive::Delegate;

use crate::components::*;
use crate::measurement::visual_input::{
    grid, receptive_field_frame, visual_input_frame, watched_picture, ReceptiveFieldCell,
    VisualInput, RECEPTIVE_FIELD_SIZE, RESOLUTION, VISUAL_INPUT_SIZE,
};
use crate::measurement::voltmeter::{Corner, Frame};
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{ConnectionData, TileData};
use crate::rendering::lighting::SHADING;
use crate::rendering::shapes::generator_color;

const TILE_HEIGHT: f32 = 0.05;
const TILE_FILL: f32 = 0.42;
const FIELD_TILE_FILL: f32 = 0.5;
const DARK: f32 = 0.04;
const LIT: f32 = 0.8;
const LIGHT_ON_FIELD: f32 = 0.9;
const BAR_GAP: f32 = 0.6;
const BAR_STRENGTH: f32 = 1.2;
const FRAME_STRENGTH: f32 = 0.3;

pub struct TileShape {
    power: f32,
    spacing: f32,
}

pub const PICTURE_DOTS: TileShape = TileShape {
    power: 2.0,
    spacing: VISUAL_INPUT_SIZE / (RESOLUTION - 1) as f32,
};
pub const FIELD_SQUIRCLES: TileShape = TileShape {
    power: 4.0,
    spacing: RECEPTIVE_FIELD_SIZE / (RESOLUTION - 1) as f32,
};

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
    tile_in(center, VISUAL_INPUT_SIZE, column, row)
}

fn tile_in(center: Vec3, size: f32, column: usize, row: usize) -> Vec3 {
    center + Vec3::Z * grid(column) as f32 * size - Vec3::X * grid(row) as f32 * size
        + Vec3::Y * TILE_HEIGHT
}

fn tile_at(center: Vec3, size: f32, index: usize) -> Vec3 {
    tile_in(center, size, index % RESOLUTION, index / RESOLUTION)
}

fn gray(brightness: f64) -> Vec3 {
    Vec3::splat(DARK + LIT * brightness as f32)
}

pub fn collect_visual_input_tiles(world: &hecs::World) -> Vec<TileData> {
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

pub fn collect_field_tiles(world: &hecs::World, palette: &Palette) -> Vec<TileData> {
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

fn superellipse_area(power: f32) -> f32 {
    let steps = 10_000;
    let width = 1.0 / steps as f32;
    let quarter: f32 = (0..steps)
        .map(|step| {
            let x = (step as f32 + 0.5) * width;
            (1.0 - x.powf(power)).powf(1.0 / power) * width
        })
        .sum();
    4.0 * quarter
}

#[derive(Delegate)]
struct TileDelegate {
    position: Expression,
    half_width: Expression,
    half_spacing: Expression,
    power: Expression,
    area: Expression,
    shading: Expression,
    color: Expression,
}

pub fn create_tiles(
    rendering_descriptor: &RenderingDescriptor,
    tile_buffer: &InstanceBuffer<TileData>,
    shape: TileShape,
) -> Result<QuadPipeline, visula_core::ShaderError> {
    let tile = tile_buffer.instance();
    let corners: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
    QuadPipeline::new(
        rendering_descriptor,
        &QuadPipelineDescriptor {
            label: "tiles",
            shader_source: include_str!("tile.wgsl"),
            shader_variable_name: "tile",
            fragment_shader_variable_name: None,
            shadow_shader_source: None,
            vertex_data: bytemuck::cast_slice(&corners),
            vertex_stride: size_of::<[f32; 2]>(),
            vertex_format: wgpu::VertexFormat::Float32x2,
            index_data: bytemuck::cast_slice(&indices),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group_layouts: &[],
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        },
        &TileDelegate {
            position: tile.position,
            half_width: tile.half_width,
            half_spacing: (shape.spacing / 2.0).into(),
            power: shape.power.into(),
            area: superellipse_area(shape.power).into(),
            shading: SHADING.into(),
            color: tile.color,
        },
        None,
    )
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

fn frame_lines(frame: &Frame, color: Vec3, lines: &mut Vec<ConnectionData>) {
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
            FRAME_STRENGTH,
            color,
        ));
    }
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

pub fn collect_visual_input_lines(world: &hecs::World, palette: &Palette) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for position in world.query::<&Position>().with::<&VisualInput>().iter() {
        frame_lines(&visual_input_frame(position.position), orange(), &mut lines);
    }
    for (field, position, dynamics, neuron_type) in world
        .query::<(
            &ReceptiveFieldCell,
            &Position,
            &GeneratorDynamics,
            Option<&NeuronType>,
        )>()
        .iter()
    {
        let frame = receptive_field_frame(position.position);
        let color = generator_color(dynamics, neuron_type.copied().unwrap_or_default(), palette);
        frame_lines(&frame, color, &mut lines);
        match_bar(&frame, field.response, &mut lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::visual_input::{spawn_picture_wire, Pattern, ReceptiveField};
    use crate::rendering::Theme;
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
            assert!(frame.contains(tile.position));
        }
        assert!(tiles[0].position.x > tiles[RESOLUTION * (RESOLUTION - 1)].position.x);
        assert!(tiles[0].position.z < tiles[RESOLUTION - 1].position.z);
    }

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
            collect_field_tiles(world, &palette)
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
        let tiles = collect_field_tiles(&world, &Theme::Dark.palette());
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
    fn a_superellipse_of_power_two_is_a_circle() {
        assert!((superellipse_area(2.0) - std::f32::consts::PI).abs() < 1e-3);
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
            let bar = *collect_visual_input_lines(&world, &palette).last().unwrap();
            assert_eq!(bar.position_b.z > bar.position_a.z, rightwards);
            assert!(
                ((bar.position_b.z - bar.position_a.z).abs() - 0.2 * RECEPTIVE_FIELD_SIZE).abs()
                    < 1e-4
            );
        }
    }
}
