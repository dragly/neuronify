use glam::{Quat, Vec3, Vec4};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{Expression, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor};
use wgpu::util::DeviceExt;

use crate::components::{Health, MacrophageUnit, MetabolicState};
use neuronify_core::{LeakyNeuron, Position};

/// Total half-width of a bar in world units (Z direction = screen horizontal).
const BAR_HALF_WIDTH: f32 = 3.5;
/// Depth of a bar in the X direction (appears as bar height on screen).
const BAR_DEPTH: f32 = 1.0;
/// Y offset above a macrophage (tall dome mesh).
const MACROPHAGE_BAR_Y: f32 = 8.0;
/// Y offset above standard-height combat units.
const UNIT_BAR_Y: f32 = 3.2;
/// Y offset for the health bar above a neuron soma (top slot).
const NEURON_HEALTH_BAR_Y: f32 = 3.5;
/// Y offset for the energy bar above a neuron soma (bottom slot, below health bar).
const NEURON_ENERGY_BAR_Y: f32 = 2.0;
/// Fill bar sits this much above the background to avoid z-fighting.
const FILL_Y_LIFT: f32 = 0.08;

fn add_rect(
    cx: f32,
    cy: f32,
    cz: f32,
    z_start: f32,
    z_end: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    if z_start >= z_end {
        return;
    }
    let base = vertices.len() as u32;
    let half_d = BAR_DEPTH / 2.0;
    let normal = [0.0_f32, 1.0, 0.0];

    vertices.push(MeshVertexAttributes {
        position: [cx - half_d, cy, cz + z_start],
        normal,
        uv: [0.0, 0.0],
        color: [255, 255, 255, 255],
    });
    vertices.push(MeshVertexAttributes {
        position: [cx + half_d, cy, cz + z_start],
        normal,
        uv: [1.0, 0.0],
        color: [255, 255, 255, 255],
    });
    vertices.push(MeshVertexAttributes {
        position: [cx + half_d, cy, cz + z_end],
        normal,
        uv: [1.0, 1.0],
        color: [255, 255, 255, 255],
    });
    vertices.push(MeshVertexAttributes {
        position: [cx - half_d, cy, cz + z_end],
        normal,
        uv: [0.0, 1.0],
        color: [255, 255, 255, 255],
    });

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn make_mesh(
    rendering_descriptor: &RenderingDescriptor,
    color: Vec4,
) -> Result<MeshPipeline, Box<dyn std::error::Error>> {
    Ok(MeshPipeline::new(
        rendering_descriptor,
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

fn flush_mesh(
    mesh: &mut MeshPipeline,
    verts: Vec<MeshVertexAttributes>,
    idx: Vec<u32>,
    device: &wgpu::Device,
) {
    if verts.is_empty() {
        mesh.vertex_count = 0;
        return;
    }
    mesh.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hbar vertex buffer"),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    });
    mesh.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hbar index buffer"),
        contents: bytemuck::cast_slice(&idx),
        usage: wgpu::BufferUsages::INDEX,
    });
    mesh.vertex_count = idx.len();
}

// ── Health bar pipelines (combat units: green/yellow/red) ─────────────────────

/// Create [bg, green, yellow, red] pipelines for combat-unit health bars.
pub fn create_health_bar_pipelines(
    rd: &RenderingDescriptor,
) -> Result<[MeshPipeline; 4], Box<dyn std::error::Error>> {
    let bg     = make_mesh(rd, Vec4::new(0.12, 0.12, 0.14, 1.0))?;
    let green  = make_mesh(rd, Vec4::new(0.15, 0.85, 0.15, 1.0))?;
    let yellow = make_mesh(rd, Vec4::new(0.90, 0.70, 0.05, 1.0))?;
    let red    = make_mesh(rd, Vec4::new(0.90, 0.08, 0.08, 1.0))?;
    Ok([bg, green, yellow, red])
}

/// Rebuild health-bar geometry for combat units (`Health` component).
/// `pipelines` order: [bg, green, yellow, red].
pub fn update_health_bar_meshes(
    pipelines: &mut [MeshPipeline; 4],
    world: &hecs::World,
    device: &wgpu::Device,
) {
    let mut bg_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut bg_i: Vec<u32> = Vec::new();
    let mut green_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut green_i: Vec<u32> = Vec::new();
    let mut yellow_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut yellow_i: Vec<u32> = Vec::new();
    let mut red_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut red_i: Vec<u32> = Vec::new();

    for (entity, (health, pos)) in world.query::<(&Health, &Position)>().iter() {
        let frac = health.fraction().clamp(0.0, 1.0);
        // Show health bar whenever the unit is below 80% health.
        if frac >= 0.8 {
            continue;
        }

        let is_macrophage = world.get::<&MacrophageUnit>(entity).is_ok();
        let is_neuron = world.get::<&LeakyNeuron>(entity).is_ok();
        let y_offset = if is_macrophage {
            MACROPHAGE_BAR_Y
        } else if is_neuron {
            NEURON_HEALTH_BAR_Y
        } else {
            UNIT_BAR_Y
        };
        let cy = pos.position.y + y_offset;
        let cx = pos.position.x;
        let cz = pos.position.z;

        add_rect(cx, cy, cz, -BAR_HALF_WIDTH, BAR_HALF_WIDTH, &mut bg_v, &mut bg_i);

        let fill_end = -BAR_HALF_WIDTH + frac * 2.0 * BAR_HALF_WIDTH;
        let fy = cy + FILL_Y_LIFT;
        if frac > 0.6 {
            add_rect(cx, fy, cz, -BAR_HALF_WIDTH, fill_end, &mut green_v, &mut green_i);
        } else if frac > 0.3 {
            add_rect(cx, fy, cz, -BAR_HALF_WIDTH, fill_end, &mut yellow_v, &mut yellow_i);
        } else {
            add_rect(cx, fy, cz, -BAR_HALF_WIDTH, fill_end, &mut red_v, &mut red_i);
        }
    }

    flush_mesh(&mut pipelines[0], bg_v, bg_i, device);
    flush_mesh(&mut pipelines[1], green_v, green_i, device);
    flush_mesh(&mut pipelines[2], yellow_v, yellow_i, device);
    flush_mesh(&mut pipelines[3], red_v, red_i, device);
}

// ── Energy bar pipelines (neurons: blue fill) ─────────────────────────────────

/// Create [bg, blue_fill] pipelines for neuron energy bars.
pub fn create_energy_bar_pipelines(
    rd: &RenderingDescriptor,
) -> Result<[MeshPipeline; 2], Box<dyn std::error::Error>> {
    let bg   = make_mesh(rd, Vec4::new(0.10, 0.10, 0.16, 1.0))?;
    let fill = make_mesh(rd, Vec4::new(0.25, 0.55, 1.00, 1.0))?;
    Ok([bg, fill])
}

/// Rebuild energy-bar geometry for the selected neuron only.
/// Pass `None` to hide all energy bars.
/// `pipelines` order: [bg, blue_fill].
pub fn update_energy_bar_meshes(
    pipelines: &mut [MeshPipeline; 2],
    world: &hecs::World,
    device: &wgpu::Device,
    selected: Option<hecs::Entity>,
) {
    let mut bg_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut bg_i: Vec<u32> = Vec::new();
    let mut fill_v: Vec<MeshVertexAttributes> = Vec::new();
    let mut fill_i: Vec<u32> = Vec::new();

    if let Some(entity) = selected {
        if let (Ok(metab), Ok(pos)) = (
            world.get::<&MetabolicState>(entity),
            world.get::<&Position>(entity),
        ) {
            if world.get::<&LeakyNeuron>(entity).is_ok() {
                let frac = (metab.energy / metab.max_energy).clamp(0.0, 1.0) as f32;
                let cy = pos.position.y + NEURON_ENERGY_BAR_Y;
                let cx = pos.position.x;
                let cz = pos.position.z;

                add_rect(cx, cy, cz, -BAR_HALF_WIDTH, BAR_HALF_WIDTH, &mut bg_v, &mut bg_i);

                let fill_end = -BAR_HALF_WIDTH + frac * 2.0 * BAR_HALF_WIDTH;
                add_rect(cx, cy + FILL_Y_LIFT, cz, -BAR_HALF_WIDTH, fill_end, &mut fill_v, &mut fill_i);
            }
        }
    }

    flush_mesh(&mut pipelines[0], bg_v, bg_i, device);
    flush_mesh(&mut pipelines[1], fill_v, fill_i, device);
}
