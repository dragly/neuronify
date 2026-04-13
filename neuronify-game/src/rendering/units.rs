//! Low-poly mesh rendering for combat units.
//!
//! Each unit type uses a single MeshPipeline with `Expression::InputColor`,
//! reading per-vertex RGBA colors embedded in `MeshVertexAttributes::color`.
//! Body and nucleus/cap geometry are generated into the same vertex buffer
//! with different color constants, eliminating the need for multiple pipelines.
//!
//! # Death animation
//!
//! When a unit has the `Dying` component, mesh update functions pass
//! `death_t = Some(timer / duration)` (range 0 → 1) into the generate
//! functions.  Each "piece" of the unit then receives a scatter offset and
//! size scale from `death_phase(t)`:
//!
//!   Phase 1 (0 → 0.15): sharp implosion — cubic-ease-in inward crush.
//!   Phase 2 (0.15 → 0.55): explosive scatter — cubic-ease-out outburst.
//!   Phase 3 (0.55 → 1.0): slow drift + linear shrink to zero.
//!
//! Per-piece scatter directions are derived from each piece's natural
//! position relative to the unit origin, so the unit visually "breaks apart"
//! rather than just shrinking uniformly.

use glam::{Vec3, Quat};
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use wgpu::util::DeviceExt;

use crate::components::{
    Dying, Health, MacrophageActivation, MacrophageUnit, MicroglialCell, TCellUnit,
};
use neuronify_core::Position;

// ── Vertex helpers ────────────────────────────────────────────────────────────

fn vert_c(pos: [f32; 3], normal: [f32; 3], color: [u8; 4]) -> MeshVertexAttributes {
    MeshVertexAttributes { position: pos, normal, uv: [0.0, 0.0], color }
}

// Per-vertex color constants for each unit part.
const MICROGLIA_BODY_COLOR:    [u8; 4] = [0,   209, 230, 255]; // cyan
const MICROGLIA_NUCLEUS_COLOR: [u8; 4] = [220, 46,  13,  255]; // red-orange
const MACROPHAGE_BODY_COLOR:   [u8; 4] = [97,  5,   77,  255]; // dark purple (dormant)
const MACROPHAGE_CAP_COLOR:    [u8; 4] = [242, 71,  209, 255]; // hot pink (dormant)
const MACROPHAGE_BODY_ACTIVE:  [u8; 4] = [200, 20,  160, 255]; // vivid magenta (active)
const MACROPHAGE_CAP_ACTIVE:   [u8; 4] = [255, 160,  80, 255]; // bright orange (active)
const TCELL_BODY_COLOR:        [u8; 4] = [80,  200, 60,  255]; // lime green
const TCELL_NUCLEUS_COLOR:     [u8; 4] = [40,  120, 30,  255]; // dark green
const TCELL_TENDRIL_COLOR:     [u8; 4] = [60,  170, 45,  255]; // medium green

// ── Death animation curve ─────────────────────────────────────────────────────

/// Returns `(outward_displacement_factor, size_scale)` for death progress `t ∈ [0,1]`.
///
/// Designed to feel like a physical explosion:
///   0.00 → 0.15: implosion — cubic-ease-in inward collapse (slow start, fast finish)
///   0.15 → 0.55: explosion — cubic-ease-out outward blast (fast start, decelerating)
///   0.55 → 1.00: drift — slow continued outward motion + linear shrink to nothing
fn death_phase(t: f32) -> (f32, f32) {
    if t < 0.15 {
        let p = t / 0.15;
        let p3 = p * p * p;          // cubic ease-in: slow start → fast finish
        (-(p3 * 0.8), 1.0 + p3 * 0.08) // implode to -0.8, slight bulge
    } else if t < 0.55 {
        let p = (t - 0.15) / 0.40;
        let out = 1.0 - (1.0 - p).powi(3); // cubic ease-out: fast start → slow finish
        let disp = -0.8 + out * 6.3;        // -0.8 → +5.5
        let scale = 1.08 - out * 0.65;      // 1.08 → 0.43
        (disp, scale.max(0.0))
    } else {
        let p = (t - 0.55) / 0.45;
        let disp = 5.5 + p * 2.5;           // continue drifting outward
        let scale = 0.43 * (1.0 - p);       // linear shrink to 0
        (disp, scale.max(0.0))
    }
}

// ── Star polygon helper ───────────────────────────────────────────────────────

fn generate_star(
    center: Vec3,
    spikes: usize,
    outer_r: f32,
    inner_r: f32,
    height: f32,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    if outer_r <= 0.001 || height <= 0.0 { return; }
    let base = vertices.len() as u32;
    let total_verts = spikes * 2;
    let tv = total_verts as u32;
    let half_h = height / 2.0;
    let y_top = center.y + half_h;
    let y_bot = center.y - half_h;

    for i in 0..total_verts {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let a = std::f32::consts::TAU * i as f32 / total_verts as f32;
        vertices.push(vert_c([center.x + r * a.cos(), y_top, center.z + r * a.sin()], [0.0, 1.0, 0.0], color));
    }
    let top_ctr = base + tv;
    vertices.push(vert_c([center.x, y_top, center.z], [0.0, 1.0, 0.0], color));

    let bot_base = top_ctr + 1;
    for i in 0..total_verts {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let a = std::f32::consts::TAU * i as f32 / total_verts as f32;
        vertices.push(vert_c([center.x + r * a.cos(), y_bot, center.z + r * a.sin()], [0.0, -1.0, 0.0], color));
    }
    let bot_ctr = bot_base + tv;
    vertices.push(vert_c([center.x, y_bot, center.z], [0.0, -1.0, 0.0], color));

    let side_top = bot_ctr + 1;
    for i in 0..total_verts {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let a = std::f32::consts::TAU * i as f32 / total_verts as f32;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([center.x + r * ca, y_top, center.z + r * sa], [ca, 0.0, sa], color));
    }
    let side_bot = side_top + tv;
    for i in 0..total_verts {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let a = std::f32::consts::TAU * i as f32 / total_verts as f32;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([center.x + r * ca, y_bot, center.z + r * sa], [ca, 0.0, sa], color));
    }

    for i in 0..tv {
        indices.extend_from_slice(&[top_ctr, base + i, base + (i + 1) % tv]);
    }
    for i in 0..tv {
        indices.extend_from_slice(&[bot_ctr, bot_base + (i + 1) % tv, bot_base + i]);
    }
    for i in 0..tv {
        let t0 = side_top + i; let t1 = side_top + (i + 1) % tv;
        let b0 = side_bot  + i; let b1 = side_bot  + (i + 1) % tv;
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }
}

// ── MicroglialCell ────────────────────────────────────────────────────────────
//
// Two layers: star body (cyan) + nucleus cap (red-orange).
// Death: body scatters in seed-derived XZ direction; nucleus shoots upward.

const MICROGLIA_SPIKES: usize = 8;
const MICROGLIA_INNER_R: f32 = 0.75;
const MICROGLIA_OUTER_R_BASE: f32 = 2.0;
const MICROGLIA_OUTER_R_AMP: f32 = 0.30;
const MICROGLIA_HEIGHT: f32 = 0.30;

fn generate_microglia_body(
    center: Vec3,
    seed: f32,
    time: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));
    // Scatter in a seed-derived horizontal direction, slight downward
    let scatter_dir = Vec3::new(
        (seed * 1.7 + 0.3).sin(),
        -0.22,
        (seed * 2.3 + 0.7).cos(),
    ).normalize_or_zero();
    let c = center + scatter_dir * disp * (MICROGLIA_OUTER_R_BASE + 1.0);
    let outer_r = (MICROGLIA_OUTER_R_BASE + (time * 4.0 + seed).sin() * MICROGLIA_OUTER_R_AMP) * scale;
    generate_star(c, MICROGLIA_SPIKES, outer_r.max(0.01), (MICROGLIA_INNER_R * scale).max(0.001), (MICROGLIA_HEIGHT * scale).max(0.001), color, vertices, indices);
}

fn generate_microglia_nucleus(
    center: Vec3,
    seed: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));
    // Shoots mostly upward with a seed-derived horizontal component
    let hx = (seed * 0.9 + 1.5).sin() * 0.38;
    let hz = (seed * 1.4 - 0.8).cos() * 0.38;
    let scatter_dir = Vec3::new(hx, 1.0, hz).normalize_or_zero();
    let c = center + scatter_dir * disp * 4.0;

    const NUC_SIDES: usize = 8;
    let nuc_r = (0.45 * scale).max(0.001);
    let nuc_h = (0.35 * scale).max(0.001);
    let nuc_base_y = c.y + MICROGLIA_HEIGHT / 2.0 + 0.01;
    let nuc_apex_y = nuc_base_y + nuc_h;
    let nb = vertices.len() as u32;
    let ns = NUC_SIDES as u32;
    for s in 0..NUC_SIDES {
        let a = std::f32::consts::TAU * s as f32 / NUC_SIDES as f32;
        let (cs, ss) = (a.cos(), a.sin());
        vertices.push(vert_c([c.x + nuc_r * cs, nuc_base_y, c.z + nuc_r * ss], [0.7 * cs, 0.7, 0.7 * ss], color));
    }
    let apex = nb + ns;
    vertices.push(vert_c([c.x, nuc_apex_y, c.z], [0.0, 1.0, 0.0], color));
    for s in 0..ns {
        indices.extend_from_slice(&[apex, nb + s, nb + (s + 1) % ns]);
    }
}


// ── MacrophageUnit ────────────────────────────────────────────────────────────
//
// Two layers: amoeba body (dark purple) + dome cap + organelle ring (hot pink).
// Death: body scatters in seed direction; cap/dome launches straight up.

const AMOEBA_VERTS: usize = 12;
const AMOEBA_OUTER_BASE: f32 = 2.4;
const AMOEBA_HEIGHT: f32 = 3.2;
const AMOEBA_CAP_HEIGHT: f32 = 4.8;

fn amoeba_radius(i: usize, n: usize, seed: f32, time: f32) -> f32 {
    let angle = std::f32::consts::TAU * i as f32 / n as f32;
    let wave = (angle * 2.0 + seed + time * 0.7).sin() * 0.40
        + (angle * 3.0 - seed * 1.3 + time * 0.4).sin() * 0.20
        + (angle * 5.0 + seed * 0.7 + time * 0.25).sin() * 0.10;
    (AMOEBA_OUTER_BASE + wave).max(1.2)
}

fn generate_macrophage_body(
    center: Vec3,
    seed: f32,
    time: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));
    // Body shell scatters in seed-derived direction, slightly downward
    let scatter_dir = Vec3::new(
        (seed * 1.5 + 0.8).sin(),
        -0.30,
        (seed * 2.1 + 0.3).cos(),
    ).normalize_or_zero();
    let c = center + scatter_dir * disp * AMOEBA_OUTER_BASE;

    let n  = AMOEBA_VERTS;
    let nv = n as u32;
    let half_h = AMOEBA_HEIGHT * scale / 2.0;
    let y_top = c.y + half_h;
    let y_bot = c.y - half_h;

    let base = vertices.len() as u32;
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time) * scale;
        vertices.push(vert_c([c.x + r * a.cos(), y_top, c.z + r * a.sin()], [0.0, 1.0, 0.0], color));
    }
    let top_ctr = base + nv;
    vertices.push(vert_c([c.x, y_top, c.z], [0.0, 1.0, 0.0], color));

    let side_top = top_ctr + 1;
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time) * scale;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([c.x + r * ca, y_top, c.z + r * sa], [ca, 0.0, sa], color));
    }
    let side_bot = side_top + nv;
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time) * scale;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([c.x + r * ca, y_bot, c.z + r * sa], [ca, 0.0, sa], color));
    }

    let bot_ring = side_bot + nv;
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time) * scale;
        vertices.push(vert_c([c.x + r * a.cos(), y_bot, c.z + r * a.sin()], [0.0, -1.0, 0.0], color));
    }
    let bot_ctr = bot_ring + nv;
    vertices.push(vert_c([c.x, y_bot, c.z], [0.0, -1.0, 0.0], color));

    for i in 0..nv {
        indices.extend_from_slice(&[top_ctr, base + i, base + (i + 1) % nv]);
    }
    for i in 0..nv {
        let t0 = side_top + i; let t1 = side_top + (i + 1) % nv;
        let b0 = side_bot  + i; let b1 = side_bot  + (i + 1) % nv;
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }
    for i in 0..nv {
        indices.extend_from_slice(&[bot_ctr, bot_ring + (i + 1) % nv, bot_ring + i]);
    }
}

fn generate_macrophage_cap(
    center: Vec3,
    seed: f32,
    time: f32,
    health_frac: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));
    let n  = AMOEBA_VERTS;
    let nv = n as u32;
    let half_h = AMOEBA_HEIGHT * scale / 2.0;
    // Cap dome launches upward with slight horizontal drift
    let cap_up   = disp * AMOEBA_CAP_HEIGHT * 1.5;
    let cap_hx   = (seed * 0.6 + 2.1).sin() * disp * 0.5;
    let cap_hz   = (seed * 1.2 - 1.3).cos() * disp * 0.5;
    let y_top    = center.y + AMOEBA_HEIGHT / 2.0 * scale + 0.02 + cap_up;
    let y_cap    = center.y + AMOEBA_CAP_HEIGHT * health_frac.max(0.25) * scale + cap_up;
    let cc       = Vec3::new(center.x + cap_hx, 0.0, center.z + cap_hz);

    let dome_ring = vertices.len() as u32;
    for i in 0..n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        let r = amoeba_radius(i, n, seed, time) * scale;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([cc.x + r * ca, y_top, cc.z + r * sa], [0.60 * ca, 0.80, 0.60 * sa], color));
    }
    let dome_apex = dome_ring + nv;
    vertices.push(vert_c([cc.x, y_cap, cc.z], [0.0, 1.0, 0.0], color));
    for i in 0..nv {
        indices.extend_from_slice(&[dome_apex, dome_ring + i, dome_ring + (i + 1) % nv]);
    }

    // ── Organelle ring — scatter radially outward ─────────────────────────────
    const ORG_COUNT: usize = 5;
    const ORG_BASE_R: f32 = 1.5;
    const ORG_CONE_R: f32 = 0.30;
    const ORG_CONE_H: f32 = 0.55;
    const ORG_SIDES: usize = 6;
    let org_apex_y = y_top + ORG_CONE_H * scale;

    for k in 0..ORG_COUNT {
        let a = std::f32::consts::TAU * k as f32 / ORG_COUNT as f32 + seed * 0.5;
        let (ca, sa) = (a.cos(), a.sin());
        // Scatter radially outward + slight upward, with per-organelle stagger
        let stagger = 1.0 + k as f32 * 0.1;
        let org_r = ORG_BASE_R * scale + disp * 3.5 * stagger;
        let cx = cc.x + org_r * ca;
        let cz = cc.z + org_r * sa;
        let ob = vertices.len() as u32;
        let os = ORG_SIDES as u32;
        let cone_r = ORG_CONE_R * scale;
        for s in 0..ORG_SIDES {
            let sa2 = std::f32::consts::TAU * s as f32 / ORG_SIDES as f32;
            let (cs, ss) = (sa2.cos(), sa2.sin());
            vertices.push(vert_c([cx + cone_r * cs, y_top, cz + cone_r * ss], [0.7 * cs, 0.7, 0.7 * ss], color));
        }
        let apex_v = ob + os;
        vertices.push(vert_c([cx, org_apex_y, cz], [0.0, 1.0, 0.0], color));
        for s in 0..os {
            indices.extend_from_slice(&[apex_v, ob + s, ob + (s + 1) % os]);
        }
    }
    let _ = half_h; // suppress warning; half_h is used implicitly via y_top
}

// ── Vertex-color pipelines (one per unit type) ──────────────────────────────
// Colors are embedded per-vertex; Expression::InputColor reads them in the shader.

fn make_vertex_color_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
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

pub fn create_microglia_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

pub fn create_astrocyte_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

pub fn create_macrophage_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

// ── Buffer flush helper ───────────────────────────────────────────────────────

fn flush(
    mesh: &mut MeshPipeline,
    verts: Vec<MeshVertexAttributes>,
    idx: Vec<u32>,
    device: &wgpu::Device,
    label: &str,
) {
    if verts.is_empty() {
        mesh.vertex_count = 0;
        return;
    }
    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&idx),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = idx.len();
}

// ── Per-frame buffer updates ──────────────────────────────────────────────────
//
// Each unit type has a single update function that generates both body and
// nucleus/cap geometry into one vertex buffer, using per-vertex colors.

pub fn update_microglia_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device, time: f32) {
    let mut verts = Vec::new();
    let mut idx   = Vec::new();
    for (entity, (_, pos)) in world.query::<(&MicroglialCell, &Position)>().iter() {
        let seed = (entity.id() as f32) * 1.618;
        let death_t = world.get::<&Dying>(entity).ok().map(|d| d.timer / d.duration);
        generate_microglia_body(pos.position, seed, time, death_t, MICROGLIA_BODY_COLOR, &mut verts, &mut idx);
        generate_microglia_nucleus(pos.position, seed, death_t, MICROGLIA_NUCLEUS_COLOR, &mut verts, &mut idx);
    }
    flush(mesh, verts, idx, device, "microglia");
}

pub fn update_astrocyte_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    device: &wgpu::Device,
    time: f32,
) {
    // ReactiveAstrocyte has been removed; clear any leftover geometry.
    flush(mesh, Vec::new(), Vec::new(), device, "astrocyte");
    let _ = (world, time);
}

pub fn update_macrophage_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device, time: f32) {
    let mut verts = Vec::new();
    let mut idx   = Vec::new();
    for (entity, (_, pos)) in world.query::<(&MacrophageUnit, &Position)>().iter() {
        let seed = (entity.id() as f32) * 2.399;
        let health_frac = world.get::<&Health>(entity).map(|h| h.fraction()).unwrap_or(1.0);
        let death_t = world.get::<&Dying>(entity).ok().map(|d| d.timer / d.duration);
        // Active macrophages use brighter colors; dormant ones use dark muted palette.
        let active = world.get::<&MacrophageActivation>(entity)
            .map(|a| a.active)
            .unwrap_or(true); // no component = always active
        let body_color = if active { MACROPHAGE_BODY_ACTIVE } else { MACROPHAGE_BODY_COLOR };
        let cap_color  = if active { MACROPHAGE_CAP_ACTIVE  } else { MACROPHAGE_CAP_COLOR  };
        generate_macrophage_body(pos.position, seed, time, death_t, body_color, &mut verts, &mut idx);
        generate_macrophage_cap(pos.position, seed, time, health_frac, death_t, cap_color, &mut verts, &mut idx);
    }
    flush(mesh, verts, idx, device, "macrophage");
}

// ── T-cell mesh (organic lymphocyte with pseudopods) ─────────────────────────

/// Generate T-cell body: a flattened dome with 4 pseudopod tendrils.
fn generate_tcell_body(
    center: Vec3,
    seed: f32,
    time: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    tendril_color: [u8; 4],
    verts: &mut Vec<MeshVertexAttributes>,
    idx: &mut Vec<u32>,
) {
    let (death_disp, death_scale) = death_t.map(|t| death_phase(t)).unwrap_or((0.0, 1.0));
    let scale = 1.2 * death_scale;

    // Central dome (low-poly sphere, flattened in Y).
    let segments = 8;
    let rings = 4;
    let base = verts.len() as u32;

    // Top vertex.
    let top = center + Vec3::new(0.0, scale * 0.6, 0.0);
    verts.push(vert_c([top.x, top.y, top.z], [0.0, 1.0, 0.0], color));

    for ring in 1..rings {
        let phi = std::f32::consts::FRAC_PI_2 * ring as f32 / rings as f32;
        let r = scale * phi.sin();
        let y = center.y + scale * 0.6 * phi.cos();
        for seg in 0..segments {
            let theta = std::f32::consts::TAU * seg as f32 / segments as f32;
            let x = center.x + r * theta.cos();
            let z = center.z + r * theta.sin();
            let n = Vec3::new(theta.cos() * phi.sin(), phi.cos(), theta.sin() * phi.sin()).normalize();
            verts.push(vert_c([x, y, z], [n.x, n.y, n.z], color));
        }
    }
    // Bottom ring (y = center.y).
    for seg in 0..segments {
        let theta = std::f32::consts::TAU * seg as f32 / segments as f32;
        let x = center.x + scale * theta.cos();
        let z = center.z + scale * theta.sin();
        verts.push(vert_c([x, center.y, z], [0.0, -1.0, 0.0], color));
    }

    // Top fan.
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        idx.extend_from_slice(&[base, base + 1 + seg as u32, base + 1 + next as u32]);
    }
    // Ring strips.
    for ring in 0..(rings - 1) {
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            let cur_ring_start = base + 1 + (ring * segments) as u32;
            let next_ring_start = base + 1 + ((ring + 1) * segments) as u32;
            let a = cur_ring_start + seg as u32;
            let b = cur_ring_start + next as u32;
            let c = next_ring_start + seg as u32;
            let d = next_ring_start + next as u32;
            idx.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    // 4 pseudopod tendrils radiating outward with wave animation.
    for arm in 0..4 {
        let base_angle = seed + std::f32::consts::FRAC_PI_2 * arm as f32;
        let wave = (time * 2.0 + seed + arm as f32 * 1.7).sin() * 0.3;
        let dir = Vec3::new(
            (base_angle + wave).cos(),
            0.0,
            (base_angle + wave).sin(),
        );
        let perp = Vec3::new(-dir.z, 0.0, dir.x);

        let tendril_len = scale * 1.8;
        let tendril_w = scale * 0.25;
        let tip = center + dir * tendril_len;
        let tip_wave = (time * 3.0 + arm as f32 * 2.3).sin() * scale * 0.2;
        let tip = tip + Vec3::new(0.0, tip_wave, 0.0);

        let b = verts.len() as u32;
        let root_l = center + perp * tendril_w + Vec3::new(0.0, scale * 0.1, 0.0);
        let root_r = center - perp * tendril_w + Vec3::new(0.0, scale * 0.1, 0.0);
        let n_up = Vec3::new(0.0, 1.0, 0.0);
        verts.push(vert_c([root_l.x, root_l.y, root_l.z], [n_up.x, n_up.y, n_up.z], tendril_color));
        verts.push(vert_c([root_r.x, root_r.y, root_r.z], [n_up.x, n_up.y, n_up.z], tendril_color));
        verts.push(vert_c([tip.x, tip.y, tip.z], [dir.x, 0.5, dir.z], tendril_color));
        idx.extend_from_slice(&[b, b + 1, b + 2]);
        // Bottom face.
        let root_l_b = root_l - Vec3::new(0.0, scale * 0.15, 0.0);
        let root_r_b = root_r - Vec3::new(0.0, scale * 0.15, 0.0);
        let tip_b = tip - Vec3::new(0.0, scale * 0.1, 0.0);
        let b2 = verts.len() as u32;
        verts.push(vert_c([root_l_b.x, root_l_b.y, root_l_b.z], [0.0, -1.0, 0.0], tendril_color));
        verts.push(vert_c([root_r_b.x, root_r_b.y, root_r_b.z], [0.0, -1.0, 0.0], tendril_color));
        verts.push(vert_c([tip_b.x, tip_b.y, tip_b.z], [0.0, -1.0, 0.0], tendril_color));
        idx.extend_from_slice(&[b2, b2 + 2, b2 + 1]);
    }
}

/// Generate T-cell nucleus: small dark sphere at center.
fn generate_tcell_nucleus(
    center: Vec3,
    death_t: Option<f32>,
    color: [u8; 4],
    verts: &mut Vec<MeshVertexAttributes>,
    idx: &mut Vec<u32>,
) {
    let (_, death_scale) = death_t.map(|t| death_phase(t)).unwrap_or((0.0, 1.0));
    let r = 0.5 * death_scale;
    let y_off = 0.3 * death_scale;
    let nc = center + Vec3::new(0.0, y_off, 0.0);

    // Simple octahedron.
    let base = verts.len() as u32;
    let positions = [
        [nc.x, nc.y + r, nc.z],        // top
        [nc.x + r, nc.y, nc.z],        // right
        [nc.x, nc.y, nc.z + r],        // front
        [nc.x - r, nc.y, nc.z],        // left
        [nc.x, nc.y, nc.z - r],        // back
        [nc.x, nc.y - r, nc.z],        // bottom
    ];
    for &p in &positions {
        let n = Vec3::new(p[0] - nc.x, p[1] - nc.y, p[2] - nc.z).normalize();
        verts.push(vert_c(p, [n.x, n.y, n.z], color));
    }
    // Top 4 faces.
    idx.extend_from_slice(&[base, base+1, base+2]);
    idx.extend_from_slice(&[base, base+2, base+3]);
    idx.extend_from_slice(&[base, base+3, base+4]);
    idx.extend_from_slice(&[base, base+4, base+1]);
    // Bottom 4 faces.
    idx.extend_from_slice(&[base+5, base+2, base+1]);
    idx.extend_from_slice(&[base+5, base+3, base+2]);
    idx.extend_from_slice(&[base+5, base+4, base+3]);
    idx.extend_from_slice(&[base+5, base+1, base+4]);
}

pub fn create_tcell_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

pub fn update_tcell_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device, time: f32) {
    let mut verts = Vec::new();
    let mut idx = Vec::new();
    for (entity, (_, pos)) in world.query::<(&TCellUnit, &Position)>().iter() {
        let seed = (entity.id() as f32) * 3.14;
        let death_t = world.get::<&Dying>(entity).ok().map(|d| d.timer / d.duration);
        generate_tcell_body(pos.position, seed, time, death_t, TCELL_BODY_COLOR, TCELL_TENDRIL_COLOR, &mut verts, &mut idx);
        generate_tcell_nucleus(pos.position, death_t, TCELL_NUCLEUS_COLOR, &mut verts, &mut idx);
    }
    flush(mesh, verts, idx, device, "tcell");
}

