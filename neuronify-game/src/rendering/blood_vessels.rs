use glam::{Quat, Vec3, Vec4};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use wgpu::util::DeviceExt;

use crate::components::BloodVessel;
use crate::constants::{BLOOD_VESSEL_HEIGHT, BLOOD_VESSEL_VISUAL_RADIUS};
use neuronify_core::Position;

const CYLINDER_SEGMENTS: usize = 12;

/// Generate cylinder vertices and indices at a given world position.
fn generate_cylinder(
    center: Vec3,
    radius: f32,
    height: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let base_index = vertices.len() as u32;
    let segments = CYLINDER_SEGMENTS;

    // Bottom ring and top ring vertices
    for ring in 0..=1 {
        let y = if ring == 0 { 0.0 } else { height };
        for i in 0..=segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let nx = angle.cos();
            let nz = angle.sin();
            let x = center.x + radius * nx;
            let z = center.z + radius * nz;
            vertices.push(MeshVertexAttributes {
                position: [x, center.y + y, z],
                normal: [nx, 0.0, nz],
                uv: [i as f32 / segments as f32, ring as f32],
                color: [255, 255, 255, 255],
            });
        }
    }

    // Side faces
    let ring_verts = (segments + 1) as u32;
    for i in 0..segments as u32 {
        let bl = base_index + i;
        let br = base_index + i + 1;
        let tl = base_index + ring_verts + i;
        let tr = base_index + ring_verts + i + 1;
        indices.extend_from_slice(&[bl, br, tr, bl, tr, tl]);
    }

    // Bottom cap
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(MeshVertexAttributes {
        position: [center.x, center.y, center.z],
        normal: [0.0, -1.0, 0.0],
        uv: [0.5, 0.5],
        color: [255, 255, 255, 255],
    });
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let nx = angle.cos();
        let nz = angle.sin();
        vertices.push(MeshVertexAttributes {
            position: [center.x + radius * nx, center.y, center.z + radius * nz],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5 + 0.5 * nx, 0.5 + 0.5 * nz],
            color: [255, 255, 255, 255],
        });
    }
    for i in 0..segments as u32 {
        let next = (i + 1) % segments as u32;
        indices.extend_from_slice(&[
            bottom_center_idx,
            bottom_center_idx + 1 + next,
            bottom_center_idx + 1 + i,
        ]);
    }

    // Top cap
    let top_center_idx = vertices.len() as u32;
    vertices.push(MeshVertexAttributes {
        position: [center.x, center.y + height, center.z],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.5],
        color: [255, 255, 255, 255],
    });
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let nx = angle.cos();
        let nz = angle.sin();
        vertices.push(MeshVertexAttributes {
            position: [
                center.x + radius * nx,
                center.y + height,
                center.z + radius * nz,
            ],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5 + 0.5 * nx, 0.5 + 0.5 * nz],
            color: [255, 255, 255, 255],
        });
    }
    for i in 0..segments as u32 {
        let next = (i + 1) % segments as u32;
        indices.extend_from_slice(&[
            top_center_idx,
            top_center_idx + 1 + i,
            top_center_idx + 1 + next,
        ]);
    }
}

/// Create the blood vessel MeshPipeline.
pub fn create_vessel_pipeline(
    rendering_descriptor: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Vec4::new(0.7, 0.12, 0.12, 1.0).into(),
        },
    )
}

/// Rebuild the vessel mesh buffers from current world state.
pub fn update_vessel_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (_, (_, position)) in world.query::<(&BloodVessel, &Position)>().iter() {
        generate_cylinder(
            position.position,
            BLOOD_VESSEL_VISUAL_RADIUS,
            BLOOD_VESSEL_HEIGHT,
            &mut vertices,
            &mut indices,
        );
    }

    if vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Blood vessel vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Blood vessel index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = indices.len();
}
