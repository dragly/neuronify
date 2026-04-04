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

/// Per-particle instance data. pos_x/pos_z are absolute world-space offsets
/// within the vessel disk (x-z plane); pos_y_center is the vessel's world y; seed is a
/// phase offset in [0, 1] that staggers each particle along the cylinder.
#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct BloodParticle {
    pub pos_x: f32,
    pub pos_z: f32,
    pub pos_y_center: f32,
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

/// Pseudo-perlin radial displacement: multiply three sine waves at different
/// frequencies and phases so they interfere into a naturalistic ripple.
///   - `angle`  : angular position around the cylinder (radians)
///   - `t_vert` : normalised vertical position [0 = bottom, 1 = top]
///   - `time`   : wall-clock time (seconds)
#[inline]
fn organic_displacement(angle: f32, t_vert: f32, time: f32) -> f32 {
    // Three waves; the product of sines creates beating / pseudo-noise.
    let w1 = (angle * 2.0 + time * 1.1).sin();   // 2 bumps, slow drift
    let w2 = (angle * 5.0 - time * 1.7).sin();   // 5 bumps, faster counter-drift
    let w3 = (t_vert * std::f32::consts::PI + time * 0.9).cos(); // top vs bottom oscillation
    w1 * w2 * w3 * 0.12 // ≈ ±12 % of radius
}

/// Generate an open cylinder aligned along the y-axis (up), centered at `center`,
/// with a time-driven organic ripple on each vertex's radial position.
/// Runs from center.y − height/2 to center.y + height/2.  No end caps.
fn generate_cylinder(
    center: Vec3,
    radius: f32,
    height: f32,
    time: f32,
    vertices: &mut Vec<MeshVertexAttributes>,
    indices: &mut Vec<u32>,
) {
    let base_index = vertices.len() as u32;
    let segments = CYLINDER_SEGMENTS;
    let half_h = height / 2.0;

    // Two rings: ring 0 at y = center.y − half_h, ring 1 at y = center.y + half_h.
    // Radial direction is in the x-z plane; normals point outward.
    for ring in 0..=1u32 {
        let t_vert = ring as f32; // 0.0 = bottom, 1.0 = top
        let y = center.y + if ring == 0 { -half_h } else { half_h };
        for i in 0..=segments {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            let nx = angle.cos();
            let nz = angle.sin();
            let r = radius * (1.0 + organic_displacement(angle, t_vert, time));
            vertices.push(MeshVertexAttributes {
                position: [center.x + r * nx, y, center.z + r * nz],
                normal: [nx, 0.0, nz],
                uv: [i as f32 / segments as f32, t_vert],
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
/// Particle z-position and size are computed entirely via visula Expressions:
///   t      = fract(seed + time * PARTICLE_SPEED / height)
///   z      = pos_z_center − height/2 + t * height
///   radius = sin(π * t) * MAX_RADIUS   (grow in from bottom, shrink to zero at top)
///
/// Note: instance buffer fields may only be used in the vertex stage (SphereGeometry).
/// Using them in SphereMaterial (fragment stage) causes a naga FunctionArgument index
/// out-of-bounds because visula stores shader_location as function_argument, but naga
/// expects the array index of the function argument.
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

    // World-space y position of the particle (vessel is y-axis aligned).
    let y = particle.pos_y_center.clone()
        + Expression::from(-half_height)
        + t.clone() * Expression::from(BLOOD_VESSEL_HEIGHT);

    let position = Expression::Vector3 {
        x: particle.pos_x.into(),
        y: y.into(),
        z: particle.pos_z.into(),
    };

    // Size fades in from the bottom end and out at the top end (same visual effect as alpha fade).
    // All instance-field usage stays in the vertex stage (SphereGeometry).
    let radius = (Expression::from(std::f32::consts::PI) * t).sin()
        * Expression::from(0.35f32);

    Ok(Spheres::new(
        rendering_descriptor,
        &SphereGeometry {
            position,
            radius,
            color: Vec3::new(1.0, 0.78, 0.2).into(),
        },
        &SphereMaterial {
            color: Expression::InputColor.lit(),
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
            // Uniform disk sample in the x-z plane (vessel is y-axis aligned).
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let r = rng.gen::<f32>().sqrt() * BLOOD_VESSEL_VISUAL_RADIUS * 0.65;
            particles.push(BloodParticle {
                pos_x: center.x + r * angle.cos(),
                pos_z: center.z + r * angle.sin(),
                pos_y_center: center.y,
                seed: i as f32 / PARTICLE_COUNT as f32,
            });
        }
    }

    particles
}

/// Rebuild the vessel mesh buffers from the current world state.
/// `time` drives the organic wall animation; call this every frame.
pub fn update_vessel_mesh(
    mesh: &mut MeshPipeline,
    world: &hecs::World,
    device: &wgpu::Device,
    time: f32,
) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (_, (_, position)) in world.query::<(&BloodVessel, &Position)>().iter() {
        generate_cylinder(
            position.position,
            BLOOD_VESSEL_VISUAL_RADIUS,
            BLOOD_VESSEL_HEIGHT,
            time,
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
