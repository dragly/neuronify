use glam::{Quat, Vec3, Vec4};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use wgpu::util::DeviceExt;

use crate::components::{BloodVessel, GlialCell, Ownership, PlayerId};
use crate::constants::PETRI_DISH_RADIUS;
use crate::simulation::game::{is_within_build_range, is_within_glial_range};
use neuronify_core::Position;

/// Generate a 2D mesh representing the valid buildable area with contour coloring.
/// Valid areas are colored, invalid areas are grayscale, with a clear border.
pub fn generate_build_area_mesh(
    world: &hecs::World,
    player: PlayerId,
    resolution: usize,
) -> (Vec<MeshVertexAttributes>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Build a 2D grid representation of valid/invalid areas
    let step = (PETRI_DISH_RADIUS * 2.0) as f32 / resolution as f32;
    let grid_size = resolution + 1;
    let mut grid: Vec<Vec<bool>> = vec![vec![false; grid_size]; grid_size];
    
    for (i, x) in (-resolution / 2..=resolution / 2).enumerate() {
        for (j, z) in (-resolution / 2..=resolution / 2).enumerate() {
            let pos = Vec3::new(x as f32 * step, 0.0, z as f32 * step);
            if is_valid_build_position(world, pos, player) {
                grid[i][j] = true;
            }
        }
    }

    // Generate mesh with contour coloring
    for i in 0..resolution {
        for j in 0..resolution {
            let corners = [
                grid[i][j],
                grid[i + 1][j],
                grid[i][j + 1],
                grid[i + 1][j + 1],
            ];

            let count = corners.iter().filter(|&&v| v).count();
            
            // Skip if all same (no contour through this cell)
            if count == 0 || count == 4 {
                continue;
            }

            // Calculate vertex positions
            let x0 = (i as i32 - resolution / 2) as f32 * step;
            let x1 = ((i + 1) as i32 - resolution / 2) as f32 * step;
            let z0 = (j as i32 - resolution / 2) as f32 * step;
            let z1 = ((j + 1) as i32 - resolution / 2) as f32 * step;

            let v00 = Vec3::new(x0, 0.0, z0);
            let v10 = Vec3::new(x1, 0.0, z0);
            let v01 = Vec3::new(x0, 0.0, z1);
            let v11 = Vec3::new(x1, 0.0, z1);

            // Determine color based on whether this cell is valid or invalid
            let is_valid_cell = count == 4;
            let is_boundary = count > 0 && count < 4;
            
            let base_color = if is_valid_cell {
                Vec3::new(0.2, 0.8, 0.2) // Green for valid
            } else if is_boundary {
                Vec3::new(1.0, 1.0, 1.0) // White for boundary
            } else {
                Vec3::new(0.3, 0.3, 0.3) // Gray for invalid
            };

            // Add vertices with interpolated colors
            let idx_base = (i * grid_size + j) as u32 * 4;
            
            // Vertex 00
            vertices.push(MeshVertexAttributes {
                position: [v00.x, v00.y, v00.z],
                normal: [0.0, 1.0, 0.0],
                uv: [i as f32 / resolution as f32, j as f32 / resolution as f32],
                color: color_to_u8(base_color),
            });
            // Vertex 10
            vertices.push(MeshVertexAttributes {
                position: [v10.x, v10.y, v10.z],
                normal: [0.0, 1.0, 0.0],
                uv: [(i + 1) as f32 / resolution as f32, j as f32 / resolution as f32],
                color: color_to_u8(base_color),
            });
            // Vertex 01
            vertices.push(MeshVertexAttributes {
                position: [v01.x, v01.y, v01.z],
                normal: [0.0, 1.0, 0.0],
                uv: [i as f32 / resolution as f32, (j + 1) as f32 / resolution as f32],
                color: color_to_u8(base_color),
            });
            // Vertex 11
            vertices.push(MeshVertexAttributes {
                position: [v11.x, v11.y, v11.z],
                normal: [0.0, 1.0, 0.0],
                uv: [(i + 1) as f32 / resolution as f32, (j + 1) as f32 / resolution as f32],
                color: color_to_u8(base_color),
            });

            // Create triangles for this cell
            if count > 0 {
                // Triangle 1: bottom-left
                indices.extend_from_slice(&[
                    idx_base, idx_base + 1, idx_base + 2,
                ]);
                // Triangle 2: top-right
                indices.extend_from_slice(&[
                    idx_base + 1, idx_base + 3, idx_base + 2,
                ]);
            }
        }
    }

    (vertices, indices)
}

fn color_to_u8(color: Vec3) -> [u8; 4] {
    [
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8,
        255,
    ]
}

fn is_valid_build_position(world: &hecs::World, position: Vec3, player: PlayerId) -> bool {
    is_within_build_range(world, position, player)
}

/// Create the build area MeshPipeline.
pub fn create_build_area_pipeline(
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
            color: Vec4::new(1.0, 1.0, 1.0, 1.0).into(), // Use vertex colors
        },
    )
}

/// Rebuild the build area mesh buffers from current world state.
pub fn update_build_area_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    player: PlayerId,
    device: &wgpu::Device,
) {
    let (vertices, indices) = generate_build_area_mesh(world, player, 64);

    if vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Build area vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Build area index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = indices.len();
}
