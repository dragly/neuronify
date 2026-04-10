//! Integration between the Voronoi map editor and the main game.
//!
//! Converts a `MapModel` into the game's `ScenarioMap` format and builds
//! the terrain mesh using the Voronoi mesh builder.

use glam::{Quat, Vec3};
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use wgpu::util::DeviceExt;

use super::map_gen;
use super::mesh_builder;
use super::terrain_model::{EditorTerrain, MapModel};
use super::voronoi::Point2;

/// Try to load the Voronoi scenario from the maps directory.
/// Falls back to generating a fresh one if the file doesn't exist.
pub fn load_voronoi_model() -> MapModel {
    let path = find_maps_path("voronoi-scenario.json");
    if let Ok(json) = std::fs::read_to_string(&path) {
        if let Ok(model) = serde_json::from_str(&json) {
            return model;
        }
    }
    generate_default_voronoi_model()
}

/// Find the maps/ directory, checking both the working directory and relative to the executable.
fn find_maps_path(filename: &str) -> std::path::PathBuf {
    // Try relative to working directory first.
    let rel = std::path::PathBuf::from("maps").join(filename);
    if rel.exists() {
        return rel;
    }
    // Try relative to the neuronify-game crate root.
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_root.join("maps").join(filename)
}

/// Generate a fresh Voronoi scenario model (used if no saved JSON exists).
pub fn generate_default_voronoi_model() -> MapModel {
    map_gen::generate_scenario_map()
}

/// Create the Visula MeshPipeline for Voronoi terrain rendering.
pub fn create_voronoi_terrain_pipeline(
    rd: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rd,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::InputColor.lit(),
        },
    )?)
}

/// Build the terrain mesh from a Voronoi MapModel and upload to GPU.
pub fn build_voronoi_terrain_mesh(
    mesh: &mut MeshPipeline,
    model: &MapModel,
    device: &wgpu::Device,
) {
    let data = mesh_builder::build_map_mesh(
        model,
        map_gen::MAP_W,
        map_gen::MAP_H,
        map_gen::CELL_SPACING,
    );

    if data.vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("voronoi_terrain"),
        contents: bytemuck::cast_slice(&data.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("voronoi_terrain_idx"),
        contents: bytemuck::cast_slice(&data.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = data.indices.len();
}

/// Sample the Voronoi cell terrain at a given world position.
/// Returns the EditorTerrain of the nearest Voronoi cell.
pub fn sample_terrain_at(model: &MapModel, world_x: f32, world_z: f32) -> EditorTerrain {
    let (cen_x, cen_y) = voronoi_center(model);
    let vx = world_x + cen_x;
    let vy = -world_z + cen_y;
    let target = Point2::new(vx, vy);

    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, p) in model.cell_centers.iter().enumerate() {
        let d = p.dist_sq(target);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    model.cell_terrains[best_idx]
}

/// Average of all Voronoi cell centers (used for coordinate transforms).
pub fn voronoi_center(model: &MapModel) -> (f32, f32) {
    let n = model.cell_centers.len() as f32;
    let cx = model.cell_centers.iter().map(|p| p.x).sum::<f32>() / n;
    let cy = model.cell_centers.iter().map(|p| p.y).sum::<f32>() / n;
    (cx, cy)
}

/// Convert Voronoi coordinates to world coordinates.
pub fn voronoi_to_world(model: &MapModel, vx: f32, vy: f32) -> Vec3 {
    let (cx, cy) = voronoi_center(model);
    Vec3::new(vx - cx, 0.0, -(vy - cy))
}

/// Check that a world position is on Open terrain.
pub fn is_open(model: &MapModel, pos: Vec3) -> bool {
    sample_terrain_at(model, pos.x, pos.z) == EditorTerrain::Open
}

/// World-space bounding box of the Voronoi map: (min_x, min_z, max_x, max_z).
pub fn world_bounds(model: &MapModel) -> (f32, f32, f32, f32) {
    let (cx, cy) = voronoi_center(model);
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    for p in &model.cell_centers {
        let wx = p.x - cx;
        let wz = -(p.y - cy);
        min_x = min_x.min(wx);
        max_x = max_x.max(wx);
        min_z = min_z.min(wz);
        max_z = max_z.max(wz);
    }
    (min_x, min_z, max_x, max_z)
}

