use glam::{Quat, Vec3};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};

use crate::constants::*;
use crate::rendering::colors::floor_color;

/// A plane under the network that catches the shadows.
pub fn create_floor(
    rendering_descriptor: &RenderingDescriptor,
    device: &wgpu::Device,
) -> Result<MeshPipeline, visula_core::ShaderError> {
    let mut floor = MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::new(0.0, FLOOR_HEIGHT, 0.0).into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::splat(FLOOR_SIZE).into(),
        },
        // Plain lighting, since `shaded` loses the shadows on meshes in visula aa83a2b
        &MeshMaterial {
            color: Expression::from(floor_color().extend(1.0)).directional_lit(),
        },
    )?;
    let corner = |x: f32, z: f32| MeshVertexAttributes {
        position: [x, 0.0, z],
        normal: [0.0, 1.0, 0.0],
        uv: [x + 0.5, z + 0.5],
        color: [255, 255, 255, 255],
    };
    let vertices = [
        corner(-0.5, -0.5),
        corner(0.5, -0.5),
        corner(0.5, 0.5),
        corner(-0.5, 0.5),
    ];
    floor.set_mesh_data(device, &vertices, &[0, 1, 2, 0, 2, 3]);
    Ok(floor)
}
