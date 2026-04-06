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

use glam::{Vec3, Vec4, Quat};
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use wgpu::util::DeviceExt;

use crate::components::{
    Dying, GlialCell, Health, MacrophageUnit, MicroglialCell, MobileUnit, Ownership, PlayerId,
};
use neuronify_core::{Inhibitory, LeakyNeuron, Position};

// ── Vertex helpers ────────────────────────────────────────────────────────────

const WHITE: [u8; 4] = [255, 255, 255, 255];

fn vert(pos: [f32; 3], normal: [f32; 3]) -> MeshVertexAttributes {
    MeshVertexAttributes { position: pos, normal, uv: [0.0, 0.0], color: WHITE }
}

fn vert_c(pos: [f32; 3], normal: [f32; 3], color: [u8; 4]) -> MeshVertexAttributes {
    MeshVertexAttributes { position: pos, normal, uv: [0.0, 0.0], color }
}

// Per-vertex color constants for each unit part.
const MICROGLIA_BODY_COLOR:    [u8; 4] = [0,   209, 230, 255]; // cyan
const MICROGLIA_NUCLEUS_COLOR: [u8; 4] = [220, 46,  13,  255]; // red-orange
const ASTROCYTE_BODY_COLOR:    [u8; 4] = [204, 133, 0,   255]; // amber
const ASTROCYTE_NUCLEUS_COLOR: [u8; 4] = [255, 230, 97,  255]; // gold
const ASTROCYTE_TENDRIL_COLOR: [u8; 4] = [235, 184, 26,  255]; // gold-yellow
const MACROPHAGE_BODY_COLOR:   [u8; 4] = [97,  5,   77,  255]; // dark purple
const MACROPHAGE_CAP_COLOR:    [u8; 4] = [242, 71,  209, 255]; // hot pink

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

// ── ReactiveAstrocyte ─────────────────────────────────────────────────────────
//
// Two layers: body + arms (amber) | nucleus/highlights (bright gold).
// Death: body disc sinks; each arm flies outward in its arm direction; nucleus shoots up.

const BODY_SIDES: usize = 10;
const BODY_HEIGHT: f32 = 0.5;
const BODY_R_BASE: f32 = 2.6;
const BODY_R_OFFSETS: [f32; 10] = [0.15, -0.10, 0.20, -0.05, 0.18, -0.12, 0.08, -0.18, 0.10, -0.08];

const ARM_ROOT_HW: f32 = 0.275;
const ARM_TIP_HW:  f32 = 0.090;
const ARM_HEIGHT: f32 = 0.70;
const ARM_BASE_LEN: f32 = 1.8;
const ARM_ANGLES_DEG: [f32; 5] = [0.0, 68.0, 130.0, 210.0, 290.0];

const NUCLEUS_SIDES: usize = 8;
const NUCLEUS_BASE_R: f32 = 1.0;
const NUCLEUS_HEIGHT: f32 = 1.6;

fn generate_astrocyte_body(
    center: Vec3,
    time: f32,
    seed: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));

    // ── Cell body disc — sinks in seed-derived direction ──────────────────────
    let body_scatter = Vec3::new(
        (seed * 1.3 + 0.5).sin() * disp * 0.6,
        -disp * 2.5,
        (seed * 1.9 + 1.1).cos() * disp * 0.6,
    );
    let bc = center + body_scatter;
    let bs = scale;
    let half_body = BODY_HEIGHT * bs / 2.0;
    let y_top = bc.y + half_body;
    let y_bot = bc.y - half_body;
    let n = BODY_SIDES as u32;
    let base = vertices.len() as u32;

    for i in 0..BODY_SIDES {
        let a = std::f32::consts::TAU * i as f32 / BODY_SIDES as f32;
        let r = (BODY_R_BASE + BODY_R_OFFSETS[i]) * bs;
        vertices.push(vert_c([bc.x + r * a.cos(), y_top, bc.z + r * a.sin()], [0.0, 1.0, 0.0], color));
    }
    let top_ctr = base + n;
    vertices.push(vert_c([bc.x, y_top, bc.z], [0.0, 1.0, 0.0], color));

    let bot_ring = top_ctr + 1;
    for i in 0..BODY_SIDES {
        let a = std::f32::consts::TAU * i as f32 / BODY_SIDES as f32;
        let r = (BODY_R_BASE + BODY_R_OFFSETS[i]) * bs;
        vertices.push(vert_c([bc.x + r * a.cos(), y_bot, bc.z + r * a.sin()], [0.0, -1.0, 0.0], color));
    }
    let bot_ctr = bot_ring + n;
    vertices.push(vert_c([bc.x, y_bot, bc.z], [0.0, -1.0, 0.0], color));

    let side_top = bot_ctr + 1;
    for i in 0..BODY_SIDES {
        let a = std::f32::consts::TAU * i as f32 / BODY_SIDES as f32;
        let r = (BODY_R_BASE + BODY_R_OFFSETS[i]) * bs;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([bc.x + r * ca, y_top, bc.z + r * sa], [ca, 0.0, sa], color));
    }
    let side_bot = side_top + n;
    for i in 0..BODY_SIDES {
        let a = std::f32::consts::TAU * i as f32 / BODY_SIDES as f32;
        let r = (BODY_R_BASE + BODY_R_OFFSETS[i]) * bs;
        let (ca, sa) = (a.cos(), a.sin());
        vertices.push(vert_c([bc.x + r * ca, y_bot, bc.z + r * sa], [ca, 0.0, sa], color));
    }

    for i in 0..n {
        indices.extend_from_slice(&[top_ctr, base + i, base + (i + 1) % n]);
    }
    for i in 0..n {
        indices.extend_from_slice(&[bot_ctr, bot_ring + (i + 1) % n, bot_ring + i]);
    }
    for i in 0..n {
        let t0 = side_top + i; let t1 = side_top + (i + 1) % n;
        let b0 = side_bot  + i; let b1 = side_bot  + (i + 1) % n;
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }

    // ── Process arms — each flies outward in its own arm direction ────────────
    let arm_y_half = ARM_HEIGHT * scale / 2.0;

    for (arm_idx, &angle_deg) in ARM_ANGLES_DEG.iter().enumerate() {
        let angle = angle_deg.to_radians();
        let arm_len = ARM_BASE_LEN + (time * 0.9 + arm_idx as f32 * 1.2).sin() * 0.18;
        let (ca, sa) = (angle.cos(), angle.sin());
        let (perp_x, perp_z) = (-sa, ca);

        // Stagger: outer arms fly slightly farther and faster
        let stagger = 1.0 + arm_idx as f32 * 0.07;
        let arm_scatter = Vec3::new(
            ca * disp * (BODY_R_BASE + arm_len * 0.6) * 0.7 * stagger,
            disp * 0.45 * stagger,
            sa * disp * (BODY_R_BASE + arm_len * 0.6) * 0.7 * stagger,
        );

        let root_x = center.x + BODY_R_BASE * ca + arm_scatter.x;
        let root_z = center.z + BODY_R_BASE * sa + arm_scatter.z;
        let tip_x  = center.x + (BODY_R_BASE + arm_len) * ca + arm_scatter.x;
        let tip_z  = center.z + (BODY_R_BASE + arm_len) * sa + arm_scatter.z;
        let arm_y_top = center.y + arm_y_half + arm_scatter.y;
        let arm_y_bot = center.y - arm_y_half + arm_scatter.y;
        let rhw = ARM_ROOT_HW * scale;
        let thw = ARM_TIP_HW  * scale;

        let arm_base = vertices.len() as u32;
        vertices.push(vert_c([root_x + perp_x * rhw, arm_y_top, root_z + perp_z * rhw], [0.0, 1.0, 0.0], color));
        vertices.push(vert_c([root_x - perp_x * rhw, arm_y_top, root_z - perp_z * rhw], [0.0, 1.0, 0.0], color));
        vertices.push(vert_c([tip_x  - perp_x * thw, arm_y_top, tip_z  - perp_z * thw], [0.0, 1.0, 0.0], color));
        vertices.push(vert_c([tip_x  + perp_x * thw, arm_y_top, tip_z  + perp_z * thw], [0.0, 1.0, 0.0], color));
        vertices.push(vert_c([root_x + perp_x * rhw, arm_y_bot, root_z + perp_z * rhw], [0.0, -1.0, 0.0], color));
        vertices.push(vert_c([root_x - perp_x * rhw, arm_y_bot, root_z - perp_z * rhw], [0.0, -1.0, 0.0], color));
        vertices.push(vert_c([tip_x  - perp_x * thw, arm_y_bot, tip_z  - perp_z * thw], [0.0, -1.0, 0.0], color));
        vertices.push(vert_c([tip_x  + perp_x * thw, arm_y_bot, tip_z  + perp_z * thw], [0.0, -1.0, 0.0], color));
        indices.extend_from_slice(&[arm_base, arm_base+1, arm_base+2, arm_base, arm_base+2, arm_base+3]);
        indices.extend_from_slice(&[arm_base+4, arm_base+6, arm_base+5, arm_base+4, arm_base+7, arm_base+6]);
    }
}

fn generate_astrocyte_nucleus(
    center: Vec3,
    health_frac: f32,
    time: f32,
    seed: f32,
    death_t: Option<f32>,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let (disp, scale) = death_t.map(death_phase).unwrap_or((0.0, 1.0));
    let half_body = BODY_HEIGHT / 2.0;
    let y_top = center.y + half_body;

    // ── End-feet at arm tips — fly with their arm ─────────────────────────────
    const EF_SIDES: usize = 6;
    const EF_R: f32 = ARM_TIP_HW * 2.5;
    const EF_H: f32 = 0.20;
    let ef_y = center.y + ARM_HEIGHT / 2.0 + EF_H;

    for (arm_idx, &angle_deg) in ARM_ANGLES_DEG.iter().enumerate() {
        let angle = angle_deg.to_radians();
        let arm_len = ARM_BASE_LEN + (time * 0.9 + arm_idx as f32 * 1.2).sin() * 0.18;
        let (ca, sa) = (angle.cos(), angle.sin());
        let stagger = 1.1 + arm_idx as f32 * 0.08;
        let arm_scatter = Vec3::new(
            ca * disp * (BODY_R_BASE + arm_len * 0.6) * 0.7 * stagger,
            disp * 0.6 * stagger,
            sa * disp * (BODY_R_BASE + arm_len * 0.6) * 0.7 * stagger,
        );
        let tip_x = center.x + (BODY_R_BASE + arm_len) * ca + arm_scatter.x;
        let tip_z = center.z + (BODY_R_BASE + arm_len) * sa + arm_scatter.z;
        let tip_y = ef_y + arm_scatter.y;

        let ef_r = EF_R * scale;
        let ef_ring = vertices.len() as u32;
        let efs = EF_SIDES as u32;
        for s in 0..EF_SIDES {
            let a = std::f32::consts::TAU * s as f32 / EF_SIDES as f32;
            vertices.push(vert_c([tip_x + a.cos() * ef_r, tip_y, tip_z + a.sin() * ef_r], [0.0, 1.0, 0.0], color));
        }
        let ef_ctr = ef_ring + efs;
        vertices.push(vert_c([tip_x, tip_y, tip_z], [0.0, 1.0, 0.0], color));
        for s in 0..efs {
            indices.extend_from_slice(&[ef_ctr, ef_ring + s, ef_ring + (s + 1) % efs]);
        }
    }

    // ── Mitochondria — scatter radially outward ───────────────────────────────
    const MITO_COUNT: usize = 5;
    const MITO_RADIUS: f32 = 1.8;
    const MITO_HALF_W: f32 = 0.25;
    const MITO_HALF_L: f32 = 0.50;
    let mito_y_top = y_top + 0.40;
    let mito_y_bot = y_top;

    for k in 0..MITO_COUNT {
        let angle = (ARM_ANGLES_DEG[k] + 34.0).to_radians();
        let (ca, sa) = (angle.cos(), angle.sin());
        let (perp_x, perp_z) = (-sa, ca);
        let scatter_r = MITO_RADIUS + disp * 3.0; // flies radially outward
        let mx = center.x + scatter_r * ca;
        let mz = center.z + scatter_r * sa;
        let my_t = mito_y_top + disp * 0.5;
        let my_b = mito_y_bot + disp * 0.5;
        let hw = MITO_HALF_W * scale;
        let hl = MITO_HALF_L * scale;
        let mb = vertices.len() as u32;
        let corners = [
            [mx + ca * hl + perp_x * hw, my_t, mz + sa * hl + perp_z * hw],
            [mx + ca * hl - perp_x * hw, my_t, mz + sa * hl - perp_z * hw],
            [mx - ca * hl - perp_x * hw, my_t, mz - sa * hl - perp_z * hw],
            [mx - ca * hl + perp_x * hw, my_t, mz - sa * hl + perp_z * hw],
        ];
        for c in &corners { vertices.push(vert_c(*c, [0.0, 1.0, 0.0], color)); }
        let m_top_ctr = mb + 4;
        vertices.push(vert_c([mx, my_t, mz], [0.0, 1.0, 0.0], color));
        indices.extend_from_slice(&[m_top_ctr, mb, mb+1, m_top_ctr, mb+1, mb+2, m_top_ctr, mb+2, mb+3, m_top_ctr, mb+3, mb]);
        let bb = mb + 5;
        for c in &corners { vertices.push(vert_c([c[0], my_b, c[2]], [0.0, -1.0, 0.0], color)); }
        for s in 0..4u32 {
            let t0 = mb + s; let t1 = mb + (s + 1) % 4;
            let b0 = bb + s; let b1 = bb + (s + 1) % 4;
            indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
        }
    }

    // ── Nucleus dome — shoots straight up ─────────────────────────────────────
    let nuc_scatter_y = disp * NUCLEUS_HEIGHT * 2.5;
    let nuc_scatter_x = (seed * 0.7 + 1.2).sin() * disp * 0.4;
    let nuc_scatter_z = (seed * 1.1 - 0.4).cos() * disp * 0.4;
    let nucleus_base_y = y_top + 0.02 + nuc_scatter_y;
    let nc = Vec3::new(center.x + nuc_scatter_x, 0.0, center.z + nuc_scatter_z);
    let nucleus_apex_y = nucleus_base_y
        + (NUCLEUS_HEIGHT * health_frac.max(0.25) + (time * 2.2).sin() * 0.12) * scale;
    let nk = NUCLEUS_SIDES as u32;

    let dome_ring = vertices.len() as u32;
    for i in 0..NUCLEUS_SIDES {
        let a = std::f32::consts::TAU * i as f32 / NUCLEUS_SIDES as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let nr = NUCLEUS_BASE_R * scale;
        vertices.push(vert_c(
            [nc.x + nr * ca, nucleus_base_y, nc.z + nr * sa],
            [0.60 * ca, 0.80, 0.60 * sa],
            color,
        ));
    }
    let dome_apex = dome_ring + nk;
    vertices.push(vert_c([nc.x, nucleus_apex_y, nc.z], [0.0, 1.0, 0.0], color));
    for i in 0..nk {
        indices.extend_from_slice(&[dome_apex, dome_ring + i, dome_ring + (i + 1) % nk]);
    }

    let disc_ring = dome_apex + 1;
    for i in 0..NUCLEUS_SIDES {
        let a = std::f32::consts::TAU * i as f32 / NUCLEUS_SIDES as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let nr = NUCLEUS_BASE_R * scale;
        vertices.push(vert_c([nc.x + nr * ca, nucleus_base_y, nc.z + nr * sa], [0.0, 1.0, 0.0], color));
    }
    let disc_ctr = disc_ring + nk;
    vertices.push(vert_c([nc.x, nucleus_base_y, nc.z], [0.0, 1.0, 0.0], color));
    for i in 0..nk {
        indices.extend_from_slice(&[disc_ctr, disc_ring + i, disc_ring + (i + 1) % nk]);
    }

    // ── Nuclear pores — drift with the nucleus ────────────────────────────────
    const PORE_COUNT: usize = 8;
    const PORE_R: f32 = 0.12;
    const PORE_SIDES: usize = 5;
    let pore_y = nucleus_base_y + 0.05;
    for k in 0..PORE_COUNT {
        let pa = std::f32::consts::TAU * k as f32 / PORE_COUNT as f32;
        let pr = NUCLEUS_BASE_R * scale;
        let px = nc.x + pr * pa.cos();
        let pz = nc.z + pr * pa.sin();
        let pb = vertices.len() as u32;
        let ps = PORE_SIDES as u32;
        for s in 0..PORE_SIDES {
            let a = std::f32::consts::TAU * s as f32 / PORE_SIDES as f32;
            let por = PORE_R * scale;
            vertices.push(vert_c([px + por * a.cos(), pore_y, pz + por * a.sin()], [0.0, 1.0, 0.0], color));
        }
        let pc = pb + ps;
        vertices.push(vert_c([px, pore_y, pz], [0.0, 1.0, 0.0], color));
        for s in 0..ps {
            indices.extend_from_slice(&[pc, pb + s, pb + (s + 1) % ps]);
        }
    }
}

// ── Astrocyte absorption tendrils ─────────────────────────────────────────────

fn generate_tendril(
    from: Vec3,
    to: Vec3,
    dist: f32,
    absorb_radius: f32,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    if dist < 0.5 { return; }
    const SEGS: usize = 4;
    const Y_LIFT: f32 = 0.6;

    let depth_frac = (1.0 - (dist / absorb_radius).clamp(0.0, 1.0)).max(0.0);
    let wide_hw   = (0.275 + depth_frac * 0.20).min(0.60);
    let narrow_hw = 0.075 + depth_frac * 0.10;

    let dir  = Vec3::new(to.x - from.x, 0.0, to.z - from.z).normalize_or_zero();
    let perp = Vec3::new(-dir.z, 0.0, dir.x);
    let y = from.y + Y_LIFT;

    let base = vertices.len() as u32;
    for col in 0..=(SEGS as u32) {
        let t = col as f32 / SEGS as f32;
        let px = from.x + dir.x * dist * t;
        let pz = from.z + dir.z * dist * t;
        let hw = wide_hw + (narrow_hw - wide_hw) * t;
        vertices.push(vert_c([px + perp.x * hw, y, pz + perp.z * hw], [0.0, 1.0, 0.0], color));
        vertices.push(vert_c([px - perp.x * hw, y, pz - perp.z * hw], [0.0, 1.0, 0.0], color));
    }
    for seg in 0..SEGS as u32 {
        let c0 = base + seg * 2;
        let c1 = base + (seg + 1) * 2;
        indices.extend_from_slice(&[c0, c0 + 1, c1 + 1, c0, c1 + 1, c1]);
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

// ── Pipeline creation ─────────────────────────────────────────────────────────

fn make_pipeline(
    rd: &RenderingDescriptor,
    color: Vec4,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rd,
        &MeshGeometry {
            position: Vec3::ZERO.into(),
            rotation: Quat::IDENTITY.into(),
            scale: Vec3::ONE.into(),
        },
        &MeshMaterial {
            color: Expression::from(color).lit(),
        },
    )?)
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
        generate_macrophage_body(pos.position, seed, time, death_t, MACROPHAGE_BODY_COLOR, &mut verts, &mut idx);
        generate_macrophage_cap(pos.position, seed, time, health_frac, death_t, MACROPHAGE_CAP_COLOR, &mut verts, &mut idx);
    }
    flush(mesh, verts, idx, device, "macrophage");
}

// ── Toon neuron / glial geometry ──────────────────────────────────────────────
//
// Soma sphere + dendrite arms built from tapered cylinder segments with wander.
// Matches the aesthetic of the toon_neurons.rs example but uses MeshPipeline
// since the game's visula version has no Cylinders primitive.

/// Deterministic pseudo-random float in [-1, 1] given a position in (seed, arm, seg, component).
fn pseudo_rng(seed: f32, arm: usize, seg: usize, component: usize) -> f32 {
    let s = seed * 7.3 + arm as f32 * 13.7 + seg as f32 * 17.3 + component as f32 * 23.1;
    (s.sin() * 43758.5453_f32).fract() * 2.0 - 1.0
}

/// Low-poly lat-long sphere emitted as triangles.
fn generate_sphere_low(
    center: Vec3,
    radius: f32,
    rings: usize,
    sectors: usize,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    if radius <= 0.001 || rings == 0 || sectors < 3 { return; }
    let base = vertices.len() as u32;
    let s = sectors as u32;

    // Top pole
    vertices.push(vert_c([center.x, center.y + radius, center.z], [0.0, 1.0, 0.0], color));

    // Ring vertices (rings bands, not counting poles)
    for r in 1..=(rings) {
        let phi = std::f32::consts::PI * r as f32 / (rings + 1) as f32;
        let y = center.y + radius * phi.cos();
        let ring_r = radius * phi.sin();
        for sc in 0..sectors {
            let theta = std::f32::consts::TAU * sc as f32 / sectors as f32;
            let (ct, st) = (theta.cos(), theta.sin());
            let ny = phi.cos();
            let nxz = phi.sin();
            vertices.push(vert_c(
                [center.x + ring_r * ct, y, center.z + ring_r * st],
                [nxz * ct, ny, nxz * st],
                color,
            ));
        }
    }

    // Bottom pole
    vertices.push(vert_c([center.x, center.y - radius, center.z], [0.0, -1.0, 0.0], color));

    let top_pole = base;
    let bot_pole = base + 1 + (rings * sectors) as u32;

    // Top cap
    for sc in 0..s {
        let a = base + 1 + sc;
        let b = base + 1 + (sc + 1) % s;
        indices.extend_from_slice(&[top_pole, a, b]);
    }

    // Middle quads
    for r in 0..(rings.saturating_sub(1)) {
        let row_a = base + 1 + (r * sectors) as u32;
        let row_b = base + 1 + ((r + 1) * sectors) as u32;
        for sc in 0..s {
            let a0 = row_a + sc;
            let a1 = row_a + (sc + 1) % s;
            let b0 = row_b + sc;
            let b1 = row_b + (sc + 1) % s;
            indices.extend_from_slice(&[a0, a1, b1, a0, b1, b0]);
        }
    }

    // Bottom cap
    let last_row = base + 1 + ((rings - 1) * sectors) as u32;
    for sc in 0..s {
        let a = last_row + sc;
        let b = last_row + (sc + 1) % s;
        indices.extend_from_slice(&[bot_pole, b, a]);
    }
}

/// Low-poly tapered cylinder from `start` (radius `r_start`) to `end` (radius `r_end`).
/// Orientation is computed from the axis, so arms can point in any direction.
fn generate_tapered_cylinder(
    start: Vec3,
    end: Vec3,
    r_start: f32,
    r_end: f32,
    sides: usize,
    color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let axis = end - start;
    let len = axis.length();
    if len < 0.001 || sides < 3 { return; }
    let axis_n = axis / len;

    // Build two perpendicular vectors via Gram-Schmidt
    let perp = if axis_n.x.abs() < 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let u = axis_n.cross(perp).normalize_or_zero();
    let v = axis_n.cross(u);

    let base = vertices.len() as u32;
    let s = sides as u32;

    // Start ring
    for i in 0..sides {
        let a = std::f32::consts::TAU * i as f32 / sides as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let p = start + u * (r_start * ca) + v * (r_start * sa);
        let n = (u * ca + v * sa).normalize_or_zero();
        vertices.push(vert_c([p.x, p.y, p.z], [n.x, n.y, n.z], color));
    }

    // End ring
    for i in 0..sides {
        let a = std::f32::consts::TAU * i as f32 / sides as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let p = end + u * (r_end * ca) + v * (r_end * sa);
        let n = (u * ca + v * sa).normalize_or_zero();
        vertices.push(vert_c([p.x, p.y, p.z], [n.x, n.y, n.z], color));
    }

    // Side quads
    let top_ring = base;
    let bot_ring = base + s;
    for i in 0..s {
        let t0 = top_ring + i;
        let t1 = top_ring + (i + 1) % s;
        let b0 = bot_ring + i;
        let b1 = bot_ring + (i + 1) % s;
        indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }
}

// ── Constants for toon neurons and glia ───────────────────────────────────────

const NEURON_SOMA_RADIUS: f32 = 1.3;
const NEURON_BASE_RADIUS: f32 = 0.28;
const NEURON_TIP_RADIUS:  f32 = 0.07;
const NEURON_ARM_LENGTH:  f32 = 4.5;
const NEURON_SEGMENTS:    usize = 4;
const NEURON_CYL_SIDES:   usize = 6;
const NEURON_WANDER:      f32 = 0.06;

const EXCITATORY_SOMA_COLOR: [u8; 4] = [80, 140, 255, 255];
const EXCITATORY_ARM_COLOR:  [u8; 4] = [55, 105, 220, 255];
const INHIBITORY_SOMA_COLOR: [u8; 4] = [220, 65, 65, 255];
const INHIBITORY_ARM_COLOR:  [u8; 4] = [175, 40, 40, 255];

const GLIAL_SOMA_RADIUS: f32 = 1.8;
const GLIAL_BASE_RADIUS: f32 = 0.32;
const GLIAL_TIP_RADIUS:  f32 = 0.08;
const GLIAL_ARM_LENGTH:  f32 = 5.5;
const GLIAL_SEGMENTS:    usize = 4;
const GLIAL_SOMA_COLOR:  [u8; 4] = [210, 140,  10, 255]; // amber
const GLIAL_ARM_COLOR:   [u8; 4] = [235, 185,  30, 255]; // gold-yellow

/// Generate toon-style soma + dendrite arms for a single entity.
///
/// Arms are evenly spaced in the XZ plane with slight random elevation.
/// Each arm wanders with `wander` radians of random perturbation per segment.
fn generate_toon_unit(
    center: Vec3,
    seed: f32,
    arm_count: usize,
    arm_length: f32,
    segments: usize,
    soma_radius: f32,
    base_radius: f32,
    tip_radius: f32,
    wander: f32,
    cyl_sides: usize,
    soma_color: [u8; 4],
    arm_color: [u8; 4],
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    generate_sphere_low(center, soma_radius, 4, 8, soma_color, vertices, indices);

    let seg_len = arm_length / segments as f32;

    for arm in 0..arm_count {
        let base_angle = std::f32::consts::TAU * arm as f32 / arm_count as f32 + seed * 0.5;
        let y_tilt = pseudo_rng(seed, arm, 0, 2) * 0.3;
        let horiz = Vec3::new(base_angle.cos(), 0.0, base_angle.sin());
        let mut dir = (horiz + Vec3::Y * y_tilt).normalize_or_zero();
        if dir.length_squared() < 0.01 { dir = horiz; }

        let mut pos = center + dir * soma_radius;

        for seg in 0..segments {
            let t_start = seg as f32 / segments as f32;
            let t_end   = (seg + 1) as f32 / segments as f32;
            let r_start = base_radius + (tip_radius - base_radius) * t_start;
            let r_end   = base_radius + (tip_radius - base_radius) * t_end;

            let next_pos = pos + dir * seg_len;
            generate_tapered_cylinder(pos, next_pos, r_start, r_end, cyl_sides, arm_color, vertices, indices);

            // Joint sphere between segments (not at the very tip)
            if seg + 1 < segments {
                generate_sphere_low(next_pos, r_end * 1.3, 3, 6, arm_color, vertices, indices);
            }

            pos = next_pos;

            // Wander: perturb direction by small random amounts
            let wx = pseudo_rng(seed, arm, seg, 0) * wander;
            let wy = pseudo_rng(seed, arm, seg, 1) * wander * 0.4;
            let wz = pseudo_rng(seed, arm, seg, 2) * wander;
            dir = (dir + Vec3::new(wx, wy, wz)).normalize_or_zero();
            if dir.length_squared() < 0.01 { dir = horiz; }
        }
    }
}

// ── Pipeline creation ─────────────────────────────────────────────────────────

pub fn create_neuron_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

pub fn create_glial_pipeline(rd: &RenderingDescriptor) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    make_vertex_color_pipeline(rd)
}

// ── Per-frame buffer updates ──────────────────────────────────────────────────

pub fn update_neuron_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device) {
    let mut verts = Vec::new();
    let mut idx   = Vec::new();
    for (entity, (_, pos)) in world.query::<(&LeakyNeuron, &Position)>().iter() {
        let seed = entity.id() as f32 * 1.618_034;
        let arm_count = 4 + (entity.id() % 4) as usize; // 4–7
        let is_inhibitory = world.get::<&Inhibitory>(entity).is_ok();
        let (soma_color, arm_color) = if is_inhibitory {
            (INHIBITORY_SOMA_COLOR, INHIBITORY_ARM_COLOR)
        } else {
            (EXCITATORY_SOMA_COLOR, EXCITATORY_ARM_COLOR)
        };
        generate_toon_unit(
            pos.position, seed, arm_count,
            NEURON_ARM_LENGTH, NEURON_SEGMENTS,
            NEURON_SOMA_RADIUS, NEURON_BASE_RADIUS, NEURON_TIP_RADIUS,
            NEURON_WANDER, NEURON_CYL_SIDES,
            soma_color, arm_color,
            &mut verts, &mut idx,
        );
    }
    flush(mesh, verts, idx, device, "neuron");
}

pub fn update_glial_mesh(mesh: &mut MeshPipeline, world: &hecs::World, device: &wgpu::Device) {
    let mut verts = Vec::new();
    let mut idx   = Vec::new();
    for (entity, (_, pos)) in world.query::<(&GlialCell, &Position)>().iter() {
        let seed = entity.id() as f32 * 2.399_963;
        let arm_count = 5 + (entity.id() % 3) as usize; // 5–7
        generate_toon_unit(
            pos.position, seed, arm_count,
            GLIAL_ARM_LENGTH, GLIAL_SEGMENTS,
            GLIAL_SOMA_RADIUS, GLIAL_BASE_RADIUS, GLIAL_TIP_RADIUS,
            NEURON_WANDER, NEURON_CYL_SIDES,
            GLIAL_SOMA_COLOR, GLIAL_ARM_COLOR,
            &mut verts, &mut idx,
        );
    }
    flush(mesh, verts, idx, device, "glial");
}
