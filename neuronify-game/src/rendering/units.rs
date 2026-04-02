//! Low-poly mesh rendering for combat units.
//!
//! All shapes are flat polygons in the xz-plane (y = 0 ± small height),
//! using vertex colors for the cyberpunk Red Alert 2 aesthetic.
//! Pattern mirrors blood_vessels.rs: rebuild entire vertex/index buffer each frame.

use glam::{Quat, Vec3, Vec4};
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use wgpu::util::DeviceExt;

use crate::components::{MacrophageUnit, MicroglialCell, ReactiveAstrocyte, Health};
use neuronify_core::Position;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert linear sRGB u8 triplet to f32 in [0,1].
#[inline]
fn srgb(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/// Mix two [f32;3] colors by factor t (0=a, 1=b).
#[inline]
fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [u8; 4] {
    let r = (a[0] + (b[0] - a[0]) * t).clamp(0.0, 1.0);
    let g = (a[1] + (b[1] - a[1]) * t).clamp(0.0, 1.0);
    let bv = (a[2] + (b[2] - a[2]) * t).clamp(0.0, 1.0);
    [(r * 255.0) as u8, (g * 255.0) as u8, (bv * 255.0) as u8, 255]
}

fn color_u8(c: [f32; 3]) -> [u8; 4] {
    [(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255]
}

/// Generate a flat star polygon (alternating outer/inner radii) centered at `center`.
/// `spikes` = number of points; `outer_r` and `inner_r` are the tip and valley radii.
/// `height` = total y-extent of the flat disc (split ±height/2).
/// `outer_color` and `inner_color` drive vertex color interpolation.
fn generate_star(
    center: Vec3,
    spikes: usize,
    outer_r: f32,
    inner_r: f32,
    height: f32,
    outer_color: [f32; 3],
    inner_color: [f32; 3],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;
    let total_verts = spikes * 2; // alternating outer/inner
    let half_h = height / 2.0;
    let y_top = center.y + half_h;
    let y_bot = center.y - half_h;
    let normal_up = [0.0f32, 1.0, 0.0];
    let normal_dn = [0.0f32, -1.0, 0.0];

    // Push top-face vertices (y_top) and bottom-face vertices (y_bot).
    // Vertex layout: [0..total_verts] = top, [total_verts..2*total_verts] = bottom, last = top center.
    for face in 0..2u32 {
        let y = if face == 0 { y_top } else { y_bot };
        let normal = if face == 0 { normal_up } else { normal_dn };
        for i in 0..total_verts {
            let is_outer = i % 2 == 0;
            let r = if is_outer { outer_r } else { inner_r };
            let angle = std::f32::consts::TAU * i as f32 / total_verts as f32;
            let x = center.x + r * angle.cos();
            let z = center.z + r * angle.sin();
            let t = if is_outer { 0.0 } else { 1.0 };
            let color = mix_color(outer_color, inner_color, t);
            vertices.push(MeshVertexAttributes {
                position: [x, y, z],
                normal,
                uv: [i as f32 / total_verts as f32, face as f32],
                color,
            });
        }
    }

    // Center vertex for top fan.
    let center_color = color_u8(inner_color);
    vertices.push(MeshVertexAttributes {
        position: [center.x, y_top, center.z],
        normal: normal_up,
        uv: [0.5, 0.0],
        color: center_color,
    });
    let top_center = base + (total_verts * 2) as u32;

    // Center vertex for bottom fan.
    vertices.push(MeshVertexAttributes {
        position: [center.x, y_bot, center.z],
        normal: normal_dn,
        uv: [0.5, 1.0],
        color: center_color,
    });
    let bot_center = top_center + 1;

    let tv = total_verts as u32;

    // Top face: triangle fan from center.
    for i in 0..tv {
        let a = base + i;
        let b = base + (i + 1) % tv;
        indices.extend_from_slice(&[top_center, a, b]);
    }

    // Bottom face: triangle fan (reversed winding).
    for i in 0..tv {
        let a = base + tv + i;
        let b = base + tv + (i + 1) % tv;
        indices.extend_from_slice(&[bot_center, b, a]);
    }

    // Side walls: quad strips connecting top and bottom rings.
    for i in 0..tv {
        let t0 = base + i;
        let t1 = base + (i + 1) % tv;
        let b0 = base + tv + i;
        let b1 = base + tv + (i + 1) % tv;
        // Two triangles per quad.
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }
}

// ── MicroglialCell mesh ───────────────────────────────────────────────────────
//
// 8-pointed spiky star. Outer tips are animated (pulse with time + per-entity seed).
// Cyberpunk cyan palette: tips bright cyan, core dark teal.

const MICROGLIA_SPIKES: usize = 8;
const MICROGLIA_INNER_R: f32 = 0.75;
const MICROGLIA_OUTER_R_BASE: f32 = 2.0;
const MICROGLIA_OUTER_R_AMP: f32 = 0.30;
const MICROGLIA_HEIGHT: f32 = 0.30;

fn generate_microglia(
    center: Vec3,
    seed: f32,   // per-entity phase offset so they don't all pulse together
    time: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let outer_r = MICROGLIA_OUTER_R_BASE + (time * 4.0 + seed).sin() * MICROGLIA_OUTER_R_AMP;
    let outer_color = srgb(0, 210, 230);     // bright cyan tips
    let inner_color = srgb(0, 65, 85);      // dark teal core
    generate_star(
        center,
        MICROGLIA_SPIKES,
        outer_r,
        MICROGLIA_INNER_R,
        MICROGLIA_HEIGHT,
        outer_color,
        inner_color,
        vertices,
        indices,
    );
}

// ── ReactiveAstrocyte mesh ────────────────────────────────────────────────────
//
// 6-pointed star, wider and taller than microglia. Gold/amber palette.
// Health-based dimming: full health = bright gold, low health = dull gray.

const ASTROCYTE_SPIKES: usize = 6;
const ASTROCYTE_INNER_R: f32 = 1.1;
const ASTROCYTE_OUTER_R: f32 = 2.8;
const ASTROCYTE_HEIGHT: f32 = 0.40;

fn generate_astrocyte(
    center: Vec3,
    health_fraction: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let dim = health_fraction.max(0.25);
    let outer_color = [1.0 * dim, 0.69 * dim, 0.12 * dim];  // gold → dim
    let inner_color = [0.71 * dim, 0.31 * dim, 0.0 * dim];  // deep amber → dim
    generate_star(
        center,
        ASTROCYTE_SPIKES,
        ASTROCYTE_OUTER_R,
        ASTROCYTE_INNER_R,
        ASTROCYTE_HEIGHT,
        outer_color,
        inner_color,
        vertices,
        indices,
    );
}

// ── MacrophageUnit mesh ───────────────────────────────────────────────────────
//
// Organic amoeba silhouette: 12 boundary vertices at irregular radii driven by
// overlapping sine waves. The shape slowly morphs over time (slow-pulsing lobes).
// Palette: deep magenta outer boundary, near-black purple centre.
// A raised central boss (dome cap) gives it a 3-D feel distinct from the flat star shapes.

const AMOEBA_VERTS: usize = 12;
const AMOEBA_OUTER_BASE: f32 = 2.4;
const AMOEBA_HEIGHT: f32 = 3.2;  // tall enough so side walls are clearly visible from above
const AMOEBA_CAP_HEIGHT: f32 = 4.8; // apex well above the top rim for a prominent dome

/// Organic radius at angular index `i` of `n` — irregular lobes driven by the unit seed+time.
fn amoeba_radius(i: usize, n: usize, seed: f32, time: f32) -> f32 {
    let angle = std::f32::consts::TAU * i as f32 / n as f32;
    // Three overlapping sine waves at incommensurate frequencies give organic variation.
    let wave = (angle * 2.0 + seed + time * 0.7).sin() * 0.40
        + (angle * 3.0 - seed * 1.3 + time * 0.4).sin() * 0.20
        + (angle * 5.0 + seed * 0.7 + time * 0.25).sin() * 0.10;
    (AMOEBA_OUTER_BASE + wave).max(1.2) // clamp so lobes never collapse
}

/// Build the macrophage amoeba mesh into the provided vertex/index buffers.
///
/// Uses **separate vertex groups per face-type** so each face gets the correct
/// geometric normal for `visula_lit_vec4`.  The surface is split into:
///   • top flat disc   — normals [0, 1, 0]
///   • side skirt      — normals pointing radially outward   (cos θ, 0, sin θ)
///   • dome cap        — normals pointing outward + upward   (0.5·cos θ, 0.866, 0.5·sin θ)
///   • bottom disc     — normals [0, −1, 0]
///
/// All four groups experience *different* light intensities from the two sun
/// directions, so the lit() result has clear shading variation across the shape.
fn generate_macrophage(
    center: Vec3,
    seed: f32,
    time: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;
    let n = AMOEBA_VERTS;
    let nv = n as u32;
    let half_h = AMOEBA_HEIGHT / 2.0;
    let y_top = center.y + half_h;
    let y_bot = center.y - half_h;
    let y_cap = center.y + AMOEBA_CAP_HEIGHT;

    // ── Group 0: top disc ring — normals up ───────────────────────────────────
    // Vertex indices: [base .. base+n)
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time);
        vertices.push(MeshVertexAttributes {
            position: [center.x + r * angle.cos(), y_top, center.z + r * angle.sin()],
            normal: [0.0, 1.0, 0.0],
            uv: [i as f32 / n as f32, 0.0],
            color: [0, 0, 0, 255],
        });
    }
    // top disc centre
    vertices.push(MeshVertexAttributes {
        position: [center.x, y_top, center.z],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.0],
        color: [0, 0, 0, 255],
    });
    let top_ring  = base;
    let top_ctr   = base + nv;

    // ── Group 1: side skirt — normals outward (cos θ, 0, sin θ) ──────────────
    // Two rings: top edge [base+n+1 .. base+2n+1), bot edge [base+2n+1 .. base+3n+1)
    let side_top_start = top_ctr + 1;
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time);
        let (cos_a, sin_a) = (angle.cos(), angle.sin());
        vertices.push(MeshVertexAttributes {
            position: [center.x + r * cos_a, y_top, center.z + r * sin_a],
            normal: [cos_a, 0.0, sin_a],
            uv: [i as f32 / n as f32, 0.0],
            color: [0, 0, 0, 255],
        });
    }
    let side_bot_start = side_top_start + nv;
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time);
        let (cos_a, sin_a) = (angle.cos(), angle.sin());
        vertices.push(MeshVertexAttributes {
            position: [center.x + r * cos_a, y_bot, center.z + r * sin_a],
            normal: [cos_a, 0.0, sin_a],
            uv: [i as f32 / n as f32, 1.0],
            color: [0, 0, 0, 255],
        });
    }

    // ── Group 2: dome cap ring — normals outward+upward (0.5·cos, 0.866, 0.5·sin)
    // [base+3n+1 .. base+4n+1)  +  dome apex at base+4n+1
    let dome_ring_start = side_bot_start + nv;
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time);
        let (cos_a, sin_a) = (angle.cos(), angle.sin());
        // Normal: matches ~37° slope of the cone (base r≈2.4, height 3.2) → strong shading gradient
        vertices.push(MeshVertexAttributes {
            position: [center.x + r * cos_a, y_top, center.z + r * sin_a],
            normal: [0.60 * cos_a, 0.80, 0.60 * sin_a],
            uv: [i as f32 / n as f32, 0.0],
            color: [0, 0, 0, 255],
        });
    }
    let dome_apex = dome_ring_start + nv;
    vertices.push(MeshVertexAttributes {
        position: [center.x, y_cap, center.z],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.0],
        color: [0, 0, 0, 255],
    });

    // ── Group 3: bottom disc ring — normals down ──────────────────────────────
    let bot_ring = dome_apex + 1;
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time);
        vertices.push(MeshVertexAttributes {
            position: [center.x + r * angle.cos(), y_bot, center.z + r * angle.sin()],
            normal: [0.0, -1.0, 0.0],
            uv: [i as f32 / n as f32, 1.0],
            color: [0, 0, 0, 255],
        });
    }
    let bot_ctr = bot_ring + nv;
    vertices.push(MeshVertexAttributes {
        position: [center.x, y_bot, center.z],
        normal: [0.0, -1.0, 0.0],
        uv: [0.5, 1.0],
        color: [0, 0, 0, 255],
    });

    // ── Top disc fan ──────────────────────────────────────────────────────────
    for i in 0..nv {
        indices.extend_from_slice(&[top_ctr, top_ring + i, top_ring + (i + 1) % nv]);
    }

    // ── Side wall quads ───────────────────────────────────────────────────────
    for i in 0..nv {
        let t0 = side_top_start + i;
        let t1 = side_top_start + (i + 1) % nv;
        let b0 = side_bot_start + i;
        let b1 = side_bot_start + (i + 1) % nv;
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }

    // ── Dome cap fan ──────────────────────────────────────────────────────────
    for i in 0..nv {
        // winding: apex first so CCW from outside looking in from above
        indices.extend_from_slice(&[dome_apex, dome_ring_start + i, dome_ring_start + (i + 1) % nv]);
    }

    // ── Bottom disc fan (reversed winding) ───────────────────────────────────
    for i in 0..nv {
        indices.extend_from_slice(&[bot_ctr, bot_ring + (i + 1) % nv, bot_ring + i]);
    }
}

// ── Pipeline creation ─────────────────────────────────────────────────────────

pub fn create_microglia_pipeline(
    rendering_descriptor: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::from(Vec4::new(0.0, 0.82, 0.9, 1.0)).lit(),
        },
    )?)
}

pub fn create_astrocyte_pipeline(
    rendering_descriptor: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::from(Vec4::new(1.0, 0.69, 0.12, 1.0)).lit(),
        },
    )?)
}

pub fn create_macrophage_pipeline(
    rendering_descriptor: &RenderingDescriptor,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::from(Vec4::new(0.82, 0.0, 0.65, 1.0)).lit(),
        },
    )?)
}

// ── Per-frame buffer updates ──────────────────────────────────────────────────

pub fn update_microglia_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    device: &wgpu::Device,
    time: f32,
) {
    let mut vertices: Vec<MeshVertexAttributes> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (i, (_, (_, pos))) in world
        .query::<(&MicroglialCell, &Position)>()
        .iter()
        .enumerate()
    {
        let seed = i as f32 * 1.618; // golden ratio spacing of phases
        generate_microglia(pos.position, seed, time, &mut vertices, &mut indices);
    }

    if vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Microglia vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Microglia index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = indices.len();
}

pub fn update_astrocyte_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    device: &wgpu::Device,
) {
    let mut vertices: Vec<MeshVertexAttributes> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (entity, (_, pos)) in world
        .query::<(&ReactiveAstrocyte, &Position)>()
        .iter()
    {
        let health_frac = world
            .get::<&Health>(entity)
            .map(|h| h.fraction())
            .unwrap_or(1.0);
        generate_astrocyte(pos.position, health_frac, &mut vertices, &mut indices);
    }

    if vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ReactiveAstrocyte vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ReactiveAstrocyte index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = indices.len();
}

pub fn update_macrophage_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    device: &wgpu::Device,
    time: f32,
) {
    let mut vertices: Vec<MeshVertexAttributes> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (i, (_, (_, pos))) in world
        .query::<(&MacrophageUnit, &Position)>()
        .iter()
        .enumerate()
    {
        let seed = i as f32 * 2.399; // ~golden angle in radians for varied phases
        generate_macrophage(pos.position, seed, time, &mut vertices, &mut indices);
    }

    if vertices.is_empty() {
        mesh.vertex_count = 0;
        return;
    }

    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Macrophage vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Macrophage index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = indices.len();
}
