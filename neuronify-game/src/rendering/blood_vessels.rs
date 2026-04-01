use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3, Vec4};
use rand::Rng;
use std::{cell::RefCell, rc::Rc};
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{
    Expression, InstanceBuffer, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor,
    SphereGeometry, SphereMaterial, Spheres, UniformBuffer,
};
use visula_core::{
    UniformBufferInner, UniformDescriptor, UniformField, UniformFieldDescriptor, UniformHandle,
};
use visula_derive::Instance;
use wgpu::util::DeviceExt;

use crate::components::BloodVessel;
use crate::constants::{BLOOD_VESSEL_HEIGHT, BLOOD_VESSEL_VISUAL_RADIUS};
use neuronify_core::Position;

const CYLINDER_SEGMENTS: usize = 24;
const PARTICLE_COUNT: usize = 60;
/// Speed at which particles travel along the z-axis (world units per second).
const PARTICLE_SPEED: f32 = 8.0;

// ── Particle types ────────────────────────────────────────────────────────────

/// Per-particle instance data. pos_x/pos_y are absolute world-space offsets
/// within the vessel disk; pos_z_center is the vessel's world z; seed is a
/// phase offset in [0, 1] that staggers each particle along the cylinder.
#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct BloodParticle {
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z_center: f32,
    pub seed: f32,
}

/// Uniform buffer updated every frame with the current simulation time.
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct VesselTime {
    pub time: f32,
    pub _padding: [f32; 3],
}

/// Expression-typed view of `VesselTime` for use in visula pipelines.
pub(crate) struct VesselTimeUniform {
    pub time: Expression,
}

impl UniformHandle for VesselTimeUniform {}

impl visula_core::Uniform for VesselTime {
    type Type = VesselTimeUniform;

    fn uniform(inner: Rc<RefCell<UniformBufferInner>>) -> VesselTimeUniform {
        let descriptor = Rc::new(UniformDescriptor {
            struct_name: "VesselTime".into(),
            variable_name: "vessel_time_uniform_variable".into(),
            struct_span: std::mem::size_of::<VesselTime>() as u32,
            fields: vec![UniformFieldDescriptor {
                name: "time".into(),
                size: 4,
                naga_type: visula_core::naga::Type {
                    name: None,
                    inner: visula_core::naga::TypeInner::Scalar(visula_core::naga::Scalar {
                        kind: visula_core::naga::ScalarKind::Float,
                        width: 4,
                    }),
                },
            }],
        });
        let bind_group_layout = inner.borrow().bind_group_layout.clone();
        let handle = inner.borrow().handle;
        VesselTimeUniform {
            time: Expression::UniformField(UniformField {
                bind_group_layout,
                buffer_handle: handle,
                field_index: 0,
                inner,
                descriptor,
            }),
        }
    }
}

// ── Cylinder mesh (z-axis, no caps, centered at entity position) ──────────────

/// Generate an open cylinder aligned along the z-axis, centered at `center`.
/// Runs from center.z − height/2 to center.z + height/2.  No end caps.
fn generate_cylinder(
    center: Vec3,
    radius: f32,
    height: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let base_index = vertices.len() as u32;
    let segments = CYLINDER_SEGMENTS;
    let half_h = height / 2.0;

    // Two rings: ring 0 at z = center.z − half_h, ring 1 at z = center.z + half_h.
    // Radial direction is in the x-y plane; normals point outward.
    for ring in 0..=1u32 {
        let z = center.z + if ring == 0 { -half_h } else { half_h };
        for i in 0..=segments {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            let nx = angle.cos();
            let ny = angle.sin();
            vertices.push(MeshVertexAttributes {
                position: [center.x + radius * nx, center.y + radius * ny, z],
                normal: [nx, ny, 0.0],
                uv: [i as f32 / segments as f32, ring as f32],
                color: [255, 255, 255, 255],
            });
        }
    }

    // Side faces — two triangles per segment strip.
    let ring_verts = (segments + 1) as u32;
    for i in 0..segments as u32 {
        let bl = base_index + i;
        let br = base_index + i + 1;
        let tl = base_index + ring_verts + i;
        let tr = base_index + ring_verts + i + 1;
        indices.extend_from_slice(&[bl, br, tr, bl, tr, tl]);
    }
}

// ── Pipeline creation ─────────────────────────────────────────────────────────

/// Create the blood vessel MeshPipeline (z-axis tube, no caps, lit).
pub fn create_vessel_pipeline(
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
            color: Expression::from(Vec4::new(0.7, 0.12, 0.12, 1.0)).lit(),
        },
    )?)
}

/// Create the particle `Spheres` pipeline.
/// Particle z-position and alpha are computed entirely via visula Expressions:
///   t   = fract(seed + time * PARTICLE_SPEED / height)
///   z   = pos_z_center − height/2 + t * height
///   α   = sin(π * t)   (fade in from bottom, fade out at top)
pub fn create_particle_pipeline(
    rendering_descriptor: &RenderingDescriptor,
    particle_buffer: &InstanceBuffer<BloodParticle>,
    time_buffer: &UniformBuffer<VesselTime>,
) -> Result<Spheres, Box<dyn std::error::Error>> {
    let particle = particle_buffer.instance();
    let vessel_time = time_buffer.uniform();

    let half_height = BLOOD_VESSEL_HEIGHT / 2.0;

    // Normalised position along the tube [0, 1).
    let t_raw = particle.seed.clone()
        + vessel_time.time.clone() * (PARTICLE_SPEED / BLOOD_VESSEL_HEIGHT);
    let t = t_raw.clone() - t_raw.floor();

    // World-space z position of the particle.
    let z = particle.pos_z_center.clone()
        + Expression::from(-half_height)
        + t.clone() * Expression::from(BLOOD_VESSEL_HEIGHT);

    let position = Expression::Vector3 {
        x: particle.pos_x.into(),
        y: particle.pos_y.into(),
        z: z.into(),
    };

    // Alpha fades in from the bottom end and out at the top end.
    let alpha = (Expression::from(std::f32::consts::PI) * t).sin();

    Ok(Spheres::new(
        rendering_descriptor,
        &SphereGeometry {
            position,
            radius: Expression::from(0.28f32),
            color: Vec3::new(1.0, 0.78, 0.2).into(),
        },
        &SphereMaterial {
            color: Expression::Vector4 {
                x: 1.0f32.into(),
                y: 0.78f32.into(),
                z: 0.2f32.into(),
                w: alpha.into(),
            },
        },
    )?)
}

// ── World-state helpers ───────────────────────────────────────────────────────

/// Generate per-particle data for all blood vessels in the world.
/// Particles are uniformly distributed inside the vessel disk (x-y plane)
/// with evenly spaced phase seeds so they appear continuously distributed.
pub fn generate_vessel_particles(world: &hecs::World) -> Vec<BloodParticle> {
    let mut rng = rand::thread_rng();
    let mut particles = Vec::new();

    for (_, (_, position)) in world.query::<(&BloodVessel, &Position)>().iter() {
        let center = position.position;
        for i in 0..PARTICLE_COUNT {
            // Uniform disk sample so particles fill the cylinder interior.
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let r = rng.gen::<f32>().sqrt() * BLOOD_VESSEL_VISUAL_RADIUS * 0.65;
            particles.push(BloodParticle {
                pos_x: center.x + r * angle.cos(),
                pos_y: center.y + r * angle.sin(),
                pos_z_center: center.z,
                seed: i as f32 / PARTICLE_COUNT as f32,
            });
        }
    }

    particles
}

/// Rebuild the vessel mesh buffers from the current world state.
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
