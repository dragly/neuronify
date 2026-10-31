use glam::{Quat, Vec3};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};

use crate::constants::*;

/// A plane under the network that catches the shadows.
pub fn create_floor(
    rendering_descriptor: &RenderingDescriptor,
    device: &wgpu::Device,
    color: Vec3,
) -> Result<MeshPipeline, visula_core::ShaderError> {
    let mut floor = MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::new(0.0, FLOOR_HEIGHT, 0.0).into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::splat(FLOOR_SIZE).into(),
            ..Default::default()
        },
        // Plain lighting, since `shaded` loses the shadows on meshes in visula aa83a2b
        &MeshMaterial {
            color: Expression::from(color.extend(1.0)).directional_lit(),
        },
    )?;
    let corner = |x: f32, z: f32| MeshVertexAttributes {
        position: [x, 0.0, z],
        normal: [0.0, 1.0, 0.0],
        uv: [x + 0.5, z + 0.5],
        color: [255, 255, 255, 255],
    };
    let halves: Vec<f32> = (0..FLOOR_RINGS)
        .map(|ring| 0.5 * 0.5f32.powi(ring))
        .collect();
    let coordinates: Vec<f32> = halves
        .iter()
        .map(|&half| -half)
        .chain([0.0])
        .chain(halves.iter().rev().copied())
        .collect();
    let vertices: Vec<MeshVertexAttributes> = coordinates
        .iter()
        .flat_map(|&z| coordinates.iter().map(move |&x| corner(x, z)))
        .collect();
    let columns = coordinates.len() as u32;
    let indices: Vec<u32> = (0..columns - 1)
        .flat_map(|row| (0..columns - 1).map(move |column| row * columns + column))
        .flat_map(|a| {
            let [b, c] = [a + 1, a + columns];
            [a, b, c + 1, a, c + 1, c]
        })
        .collect();
    floor.set_mesh_data(device, &vertices, &indices);
    Ok(floor)
}
