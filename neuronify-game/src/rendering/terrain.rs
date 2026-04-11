//! Terrain mesh rendering (SVG hex-based — retained for reference).
//!
//! Generates a dense triangulated grid over the scenario map extent.  Each
//! vertex is assigned a terrain type via a **noise-perturbed Voronoi** lookup:
//!
//! 1. Find the nearest hex centre (and its terrain) among the vertex's own hex
//!    plus its 6 neighbours.
//! 2. Find the nearest hex centre that has a *different* terrain type.
//! 3. The boundary between the two is a mathematical perpendicular bisector.
//!    Displace it by multi-octave noise scaled by a per-pair band width.
//! 4. Assign the winning terrain's color to the vertex — **no interpolation,
//!    just a hard flip** — so edges look ragged like a shoreline, not like a
//!    smooth gradient or a visible hex grid.
//!
//! The mesh is built once when a scenario loads; it never changes per-frame.

use std::cmp::Ordering;
use std::collections::HashMap;

use glam::{Quat, Vec3};
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use wgpu::util::DeviceExt;

use crate::map::{ScenarioMap, TerrainType};
use crate::map::hex::{hex_center, hex_neighbors, world_to_hex, SVG_CX, SVG_CY, MAP_SCALE};

// ── Grid parameters ───────────────────────────────────────────────────────────

/// World-unit spacing between grid vertices.
const GRID_STEP: f32 = 1.0;
/// Grid extents — slightly larger than the SVG map extent.
const GRID_X_MIN: f32 = -163.0;
const GRID_X_MAX: f32 =  163.0;
const GRID_Z_MIN: f32 = -118.0;
const GRID_Z_MAX: f32 =  118.0;

// ── Hex geometry ──────────────────────────────────────────────────────────────

/// Hex cell radius in world units (= 24 px × MAP_SCALE).
const HEX_R: f32 = 24.0 * MAP_SCALE;  // ≈ 6.72

// ── Noise ─────────────────────────────────────────────────────────────────────

/// Multi-octave sum-of-sines noise.  Range ≈ [−1, 1].
fn noise2d(x: f32, z: f32) -> f32 {
    let n = (x * 0.23 + z * 0.17).sin()
          + (x * 0.41 - z * 0.29 + 1.73).sin() * 0.5
          + (x * 0.73 + z * 0.61 + 3.14).sin() * 0.25
          + (x * 1.37 - z * 1.11 + 0.77).sin() * 0.125;
    n / 1.875
}

fn tint(base: [u8; 4], offset: f32) -> [u8; 4] {
    let o = offset as i32;
    [
        (base[0] as i32 + o).clamp(0, 255) as u8,
        (base[1] as i32 + o).clamp(0, 255) as u8,
        (base[2] as i32 + o).clamp(0, 255) as u8,
        base[3],
    ]
}

// ── Terrain palette ───────────────────────────────────────────────────────────

fn base_color(t: &TerrainType) -> [u8; 4] {
    match t {
        TerrainType::Open      => [210, 198, 178, 255], // warm pale beige
        TerrainType::EcmSparse => [182, 168, 148, 255], // slightly darker
        TerrainType::EcmDense  => [142, 128, 108, 255], // dark brown-grey
        TerrainType::Csf       => [158, 183, 210, 255], // blue-grey fluid
        TerrainType::GlialScar => [78,  68,  52,  255], // dark olive
        TerrainType::Vessel    => [162, 42,  52,  255], // deep red (fallback)
    }
}

/// Amplitude (±channels) for within-terrain noise tinting.
fn noise_amplitude(t: &TerrainType) -> f32 {
    match t {
        TerrainType::Open      => 18.0,
        TerrainType::EcmSparse => 15.0,
        TerrainType::EcmDense  => 12.0,
        TerrainType::Csf       => 22.0,
        TerrainType::GlialScar => 10.0,
        TerrainType::Vessel    => 9.0,
    }
}

/// Noise half-band width (world units) for the boundary between `t1` and `t2`.
/// A larger value = wider noisy transition zone = more ragged edge.
fn transition_band(t1: &TerrainType, t2: &TerrainType) -> f32 {
    use TerrainType::*;
    match (t1, t2) {
        (Vessel, _)    | (_, Vessel)    => 0.8,  // hard vessel walls
        (Csf, _)       | (_, Csf)       => 3.2,  // shoreline-scale
        (GlialScar, _) | (_, GlialScar) => 5.8,  // very ragged scar edge
        _                               => 4.5,  // soft ECM transitions
    }
}

/// Spatial frequency of the boundary noise — lower = larger bumps.
fn boundary_freq(t1: &TerrainType, t2: &TerrainType) -> f32 {
    use TerrainType::*;
    match (t1, t2) {
        (Vessel, _)    | (_, Vessel)    => 1.6,
        (GlialScar, _) | (_, GlialScar) => 0.26,  // large irregular lobes
        (Csf, _)       | (_, Csf)       => 0.38,  // medium-scale shoreline
        _                               => 0.42,
    }
}

// ── Hex world-centre cache ────────────────────────────────────────────────────

fn build_centers(map: &ScenarioMap) -> HashMap<(i32, i32), (f32, f32)> {
    map.terrain.keys().map(|&(col, row)| {
        let (sx, sy) = hex_center(col, row);
        // Mirror the svg_to_world transform from scenarios.rs.
        let wx = -(sx - SVG_CX) * MAP_SCALE;
        let wz = -(sy - SVG_CY) * MAP_SCALE;
        ((col, row), (wx, wz))
    }).collect()
}

// ── Per-vertex sampling ───────────────────────────────────────────────────────

fn sample_vertex(
    wx: f32,
    wz: f32,
    map: &ScenarioMap,
    centers: &HashMap<(i32, i32), (f32, f32)>,
) -> Option<MeshVertexAttributes> {
    let h = world_to_hex(wx, wz);

    // Collect candidate hexes: own + 6 neighbours, restricted to those in the map.
    let mut cands: [(i32, i32, f32); 7] = [(-1, -1, f32::MAX); 7];
    let mut n_cands = 0usize;

    for coord in std::iter::once((h.col, h.row))
        .chain(hex_neighbors(h.col, h.row).iter().map(|nb| (nb.col, nb.row)))
    {
        if let Some(&(cx, cz)) = centers.get(&coord) {
            let d2 = (wx - cx).powi(2) + (wz - cz).powi(2);
            cands[n_cands] = (coord.0, coord.1, d2);
            n_cands += 1;
        }
    }

    if n_cands == 0 { return None; }

    // Sort by ascending distance² (insertion sort over ≤7 elements).
    let cands = &mut cands[..n_cands];
    cands.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));

    let (best_col, best_row, best_d2) = cands[0];
    let best_dist = best_d2.sqrt();

    // Cull vertices more than one hex radius from any terrain hex centre — they
    // are genuinely outside the map boundary.
    if best_dist > HEX_R * 1.15 { return None; }

    let best_terrain = &map.terrain[&(best_col, best_row)].terrain;

    // Find the nearest candidate with a DIFFERENT terrain type.
    let alt = cands[1..].iter().find(|(col, row, _)| {
        map.terrain.get(&(*col, *row))
            .map(|h| &h.terrain != best_terrain)
            .unwrap_or(false)
    });

    let (win_col, win_row) = if let Some(&(ac, ar, ad2)) = alt {
        let alt_terrain = &map.terrain[&(ac, ar)].terrain;
        let alt_dist = ad2.sqrt();
        // Signed distance to the Voronoi boundary (0 = on the boundary).
        // Positive = inside own territory; negative = inside alt territory.
        let signed = (alt_dist - best_dist) * 0.5;
        let freq = boundary_freq(best_terrain, alt_terrain);
        let band = transition_band(best_terrain, alt_terrain);
        let noise_val = noise2d(wx * freq, wz * freq) * band;
        if signed < noise_val { (ac, ar) } else { (best_col, best_row) }
    } else {
        (best_col, best_row)
    };

    let winning_terrain = &map.terrain[&(win_col, win_row)].terrain;
    let &(cx, cz) = centers.get(&(win_col, win_row))?;
    let dist_from_center = ((wx - cx).powi(2) + (wz - cz).powi(2)).sqrt();

    let (color, y) = vertex_appearance(winning_terrain, dist_from_center, wx, wz);

    Some(MeshVertexAttributes {
        position: [wx, y, wz],
        normal:   [0.0, 1.0, 0.0],
        uv:       [0.0, 0.0],
        color,
    })
}

fn vertex_appearance(
    terrain: &TerrainType,
    dist_from_center: f32,
    wx: f32,
    wz: f32,
) -> ([u8; 4], f32) {
    match terrain {
        TerrainType::Vessel => vessel_vertex(dist_from_center, wx, wz),
        TerrainType::GlialScar => {
            let base = base_color(terrain);
            let n = noise2d(wx * 0.22, wz * 0.22);
            // Rough raised surface
            let y = 0.18 + n.abs() * 0.14;
            (tint(base, n * noise_amplitude(terrain)), y)
        }
        other => {
            let base = base_color(other);
            let n = noise2d(wx * 0.13, wz * 0.13);
            (tint(base, n * noise_amplitude(other)), 0.0)
        }
    }
}

/// Blood vessel: dome cross-section + lumen (lighter inner circle).
fn vessel_vertex(dist: f32, wx: f32, wz: f32) -> ([u8; 4], f32) {
    let t = (dist / HEX_R).clamp(0.0, 1.0);
    // Dome: highest at centre (0.55 wu), blends down to 0.15 at the edge.
    let y = 0.55 * (1.0 - t * t * t) + 0.15 * (t * t * t);

    // Lumen boundary is noisy to look like a real vessel wall, not a circle.
    let lumen_radius = 0.42 + noise2d(wx * 1.5, wz * 1.5) * 0.07;
    let color = if t < lumen_radius {
        // Bright pink-red lumen
        let n = noise2d(wx * 0.35, wz * 0.35);
        tint([218, 105, 112, 255], n * 12.0)
    } else {
        // Dark vessel wall
        let n = noise2d(wx * 0.40, wz * 0.40);
        tint([152, 38, 48, 255], n * 10.0)
    };
    (color, y)
}

// ── Pipeline creation ─────────────────────────────────────────────────────────

pub fn create_terrain_pipeline(
    rd: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rd,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale:    Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::InputColor.lit(),
        },
    )?)
}

// ── Mesh build ────────────────────────────────────────────────────────────────

/// Rebuild the terrain mesh from `map`.  Call once after a scenario loads.
pub fn build_terrain_mesh(
    mesh: &mut MeshPipeline,
    map: &ScenarioMap,
    device: &wgpu::Device,
) {
    let centers = build_centers(map);

    // Build axis point lists.
    let nx = ((GRID_X_MAX - GRID_X_MIN) / GRID_STEP).ceil() as usize + 1;
    let nz = ((GRID_Z_MAX - GRID_Z_MIN) / GRID_STEP).ceil() as usize + 1;

    // Sample all vertices.
    let mut samples: Vec<Option<MeshVertexAttributes>> = Vec::with_capacity(nx * nz);
    for iz in 0..nz {
        let z = GRID_Z_MIN + iz as f32 * GRID_STEP;
        for ix in 0..nx {
            let x = GRID_X_MIN + ix as f32 * GRID_STEP;
            samples.push(sample_vertex(x, z, map, &centers));
        }
    }

    // Generate quads (2 triangles), skipping any quad where a vertex is absent.
    let mut verts: Vec<MeshVertexAttributes> = Vec::new();
    let mut idx:   Vec<u32>                  = Vec::new();

    for iz in 0..(nz - 1) {
        for ix in 0..(nx - 1) {
            let i00 = iz * nx + ix;
            let i01 = iz * nx + ix + 1;
            let i10 = (iz + 1) * nx + ix;
            let i11 = (iz + 1) * nx + ix + 1;

            let (v00, v01, v10, v11) = match (
                samples[i00],
                samples[i01],
                samples[i10],
                samples[i11],
            ) {
                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                _ => continue,
            };

            let base = verts.len() as u32;
            verts.extend_from_slice(&[v00, v01, v10, v11]);
            // Two triangles: (v00,v01,v11) and (v00,v11,v10) — CCW from above.
            idx.extend_from_slice(&[base, base+1, base+3, base, base+3, base+2]);
        }
    }

    if verts.is_empty() {
        mesh.vertex_count = 0;
        return;
    }
    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("terrain"),
        contents: bytemuck::cast_slice(&verts),
        usage:    wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("terrain"),
        contents: bytemuck::cast_slice(&idx),
        usage:    wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = idx.len();
}
