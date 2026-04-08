//! Integration between the Voronoi map editor and the main game.
//!
//! Converts a `MapModel` into the game's `ScenarioMap` format and builds
//! the terrain mesh using the Voronoi mesh builder.

use std::collections::HashMap;

use glam::{Quat, Vec3};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
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

/// Convert EditorTerrain to the game's TerrainType string representation.
fn editor_terrain_to_game(t: EditorTerrain) -> (&'static str, bool, f32, bool) {
    // Returns (terrain_name, passable, speed_mult, resource_glucose)
    match t {
        EditorTerrain::Open => ("open", true, 1.0, false),
        EditorTerrain::Vessel => ("vessel", false, 1.0, true),
        EditorTerrain::GlialScar => ("glial_scar", false, 1.0, false),
        EditorTerrain::Csf => ("csf", false, 1.0, false),
    }
}

/// Sample the Voronoi cell terrain at a given world position.
/// Returns the EditorTerrain of the nearest Voronoi cell.
pub fn sample_terrain_at(model: &MapModel, world_x: f32, world_z: f32) -> EditorTerrain {
    let cen_x: f32 = model.cell_centers.iter().map(|p| p.x).sum::<f32>()
        / model.cell_centers.len() as f32;
    let cen_y: f32 = model.cell_centers.iter().map(|p| p.y).sum::<f32>()
        / model.cell_centers.len() as f32;

    // Reverse to_world: world_x = voronoi_x - cen_x, world_z = -(voronoi_y - cen_y)
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
