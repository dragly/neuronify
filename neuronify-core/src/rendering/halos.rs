use std::f32::consts::TAU;

use glam::{Vec2, Vec3, Vec4};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{
    Expression, InstanceBuffer, MeshBlending, MeshGeometry, MeshMaterial, MeshPipeline,
    RenderingDescriptor,
};

use crate::components::*;
use crate::constants::*;
use crate::rendering::colors::{firing_rate_color, Palette};
use crate::rendering::gpu_types::HaloData;

const FIRING_RATE_SCALE: f64 = 60.0;
const HALO_RADIUS: f32 = 3.15 * NODE_RADIUS;
const HALO_LIFT: f32 = 0.1;
const HALO_RINGS: usize = 8;
const HALO_SEGMENTS: usize = 32;

fn disk() -> (Vec<MeshVertexAttributes>, Vec<u32>) {
    let vertex = |offset: Vec2, fraction: f32| MeshVertexAttributes {
        position: [offset.x, 0.0, offset.y],
        normal: [0.0, 1.0, 0.0],
        uv: [(1.0 - fraction * fraction).powi(2), 0.0],
        color: [255; 4],
    };
    let mut vertices = vec![vertex(Vec2::ZERO, 0.0)];
    for ring in 1..=HALO_RINGS {
        let fraction = ring as f32 / HALO_RINGS as f32;
        for segment in 0..HALO_SEGMENTS {
            let angle = TAU * segment as f32 / HALO_SEGMENTS as f32;
            vertices.push(vertex(Vec2::from_angle(angle) * fraction, fraction));
        }
    }
    let at = |ring: usize, segment: usize| {
        (1 + (ring - 1) * HALO_SEGMENTS + segment % HALO_SEGMENTS) as u32
    };
    let mut indices = Vec::new();
    for segment in 0..HALO_SEGMENTS {
        indices.extend([0, at(1, segment), at(1, segment + 1)]);
        for ring in 1..HALO_RINGS {
            let [a, b] = [at(ring, segment), at(ring, segment + 1)];
            let [c, d] = [at(ring + 1, segment), at(ring + 1, segment + 1)];
            indices.extend([a, c, d, a, d, b]);
        }
    }
    (vertices, indices)
}

pub fn create_halos(
    rendering_descriptor: &RenderingDescriptor,
    device: &wgpu::Device,
    halo_buffer: &InstanceBuffer<HaloData>,
    palette: &Palette,
) -> Result<MeshPipeline, visula_core::ShaderError> {
    let halo = halo_buffer.instance();
    let lit = Expression::InputColor.directional_lit().shared();
    let mut halos = MeshPipeline::with_blending(
        rendering_descriptor,
        &MeshGeometry {
            position: halo.position,
            scale: Vec3::splat(HALO_RADIUS).into(),
            color: halo.color,
            ..Default::default()
        },
        &MeshMaterial {
            color: Expression::Vector4 {
                x: lit.dot(Expression::from(Vec4::X)).into(),
                y: lit.dot(Expression::from(Vec4::Y)).into(),
                z: lit.dot(Expression::from(Vec4::Z)).into(),
                w: Expression::UV
                    .dot(Expression::from(Vec2::new(palette.halo, 0.0)))
                    .into(),
            },
        },
        MeshBlending::Transparent,
    )?;
    let (vertices, indices) = disk();
    halos.set_mesh_data(device, &vertices, &indices);
    Ok(halos)
}

fn firing_rate_level(firing_rate: &FiringRate) -> f32 {
    (1.0 - (-firing_rate.rate / FIRING_RATE_SCALE).exp()) as f32
}

pub fn collect_halos(world: &hecs::World) -> Vec<HaloData> {
    if !NeuronDisplay::of(world).firing_rate {
        return Vec::new();
    }
    let mut halos: Vec<(f32, Vec3)> = world
        .query::<(&FiringRate, &Position)>()
        .iter()
        .map(|(_, (firing_rate, position))| (firing_rate_level(firing_rate), position.position))
        .collect();
    halos.sort_by(|a, b| a.0.total_cmp(&b.0));
    halos
        .into_iter()
        .map(|(level, position)| HaloData {
            position: Vec3::new(position.x, FLOOR_HEIGHT + HALO_LIFT, position.z),
            _padding: 0.0,
            color: firing_rate_color(level).extend(1.0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_rates(rates: &[f64], shown: bool) -> hecs::World {
        let mut world = hecs::World::new();
        for (index, &rate) in rates.iter().enumerate() {
            world.spawn((
                Position {
                    position: Vec3::new(index as f32, 0.0, 0.0),
                },
                FiringRate { rate },
            ));
        }
        NeuronDisplay {
            voltage: true,
            firing_rate: shown,
        }
        .set_in(&mut world);
        world
    }

    #[test]
    fn halos_show_only_when_turned_on() {
        assert!(collect_halos(&world_with_rates(&[50.0], false)).is_empty());
        let halos = collect_halos(&world_with_rates(&[50.0], true));
        assert_eq!(halos.len(), 1);
        assert!(halos[0].position.y > FLOOR_HEIGHT);
    }

    #[test]
    fn silent_neurons_get_a_purple_halo() {
        let halos = collect_halos(&world_with_rates(&[0.0], true));
        let color = halos[0].color;
        assert!(color.z > color.x.max(color.y));
    }

    #[test]
    fn busier_halos_are_drawn_on_top() {
        let halos = collect_halos(&world_with_rates(&[150.0, 0.0, 50.0], true));
        let order: Vec<f32> = halos.iter().map(|halo| halo.position.x).collect();
        assert_eq!(order, [1.0, 2.0, 0.0]);
    }

    #[test]
    fn halos_fade_out_towards_the_rim() {
        let (vertices, indices) = disk();
        assert_eq!(vertices[0].uv[0], 1.0);
        assert_eq!(vertices.last().unwrap().uv[0], 0.0);
        assert!(indices
            .iter()
            .all(|&index| (index as usize) < vertices.len()));
    }

    #[test]
    fn firing_rate_colors_brighten_towards_yellow_and_not_white() {
        let luminance = |color: Vec3| color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let colors: Vec<Vec3> = (0..=20)
            .map(|step| firing_rate_color(step as f32 / 20.0))
            .collect();
        assert!(colors
            .windows(2)
            .all(|pair| luminance(pair[1]) > luminance(pair[0])));
        let top = firing_rate_color(1.0);
        assert!(top.z < 0.1 * top.x.min(top.y));
    }
}
