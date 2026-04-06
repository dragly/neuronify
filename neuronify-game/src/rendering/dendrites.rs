//! Cylinder rendering for dendrite/process connections.
//!
//! Each `Connection + CompartmentCurrent` entity is rendered as a tapered cylinder
//! from the `from` entity's position to the `to` entity's position.
//! Color is driven by the `to` compartment's voltage (same palette as sphere rendering).

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use visula::{CylinderGeometry, CylinderMaterial, Cylinders, Expression, InstanceBuffer, RenderingDescriptor};
use visula_derive::Instance;

use neuronify_core::rendering::colors::neurocolor;
use neuronify_core::{Compartment, CompartmentCurrent, Connection, NeuronType, Position};

use crate::components::{DendriteDepth, GlialProcess, Ownership};
use crate::rendering::colors::glial_color;

// ── Data struct ───────────────────────────────────────────────────────────────

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct CylinderData {
    pub start: [f32; 3],
    pub start_radius: f32,
    pub end: [f32; 3],
    pub end_radius: f32,
    pub color: [f32; 3],
    pub _padding: f32,
}

// ── Pipeline creation ─────────────────────────────────────────────────────────

pub fn create_dendrite_pipeline(
    rd: &RenderingDescriptor,
    cylinder_buffer: &InstanceBuffer<CylinderData>,
) -> Result<Cylinders, Box<dyn std::error::Error>> {
    let inst = cylinder_buffer.instance();
    Ok(Cylinders::new(
        rd,
        &CylinderGeometry {
            start: inst.start,
            end: inst.end,
            start_radius: inst.start_radius,
            end_radius: inst.end_radius,
            color: inst.color,
        },
        &CylinderMaterial {
            color: Expression::InputColor.lit(),
        },
    )?)
}

// ── Per-frame data collection ─────────────────────────────────────────────────

/// Radius at depth 0 (soma surface). Each depth step multiplies by TAPER.
const DENDRITE_ROOT_RADIUS: f32 = 0.30;
/// Radius shrinks by this factor per depth hop.
const DENDRITE_TAPER: f32 = 0.72;
/// Floor so cylinders never disappear entirely.
const DENDRITE_MIN_RADIUS: f32 = 0.04;

fn depth_radius(depth: u32) -> f32 {
    (DENDRITE_ROOT_RADIUS * DENDRITE_TAPER.powi(depth as i32)).max(DENDRITE_MIN_RADIUS)
}

pub fn collect_dendrite_cylinders(world: &hecs::World) -> Vec<CylinderData> {
    let mut result = Vec::new();

    for (_, (conn, _)) in world.query::<(&Connection, &CompartmentCurrent)>().iter() {
        let from_pos = match world.get::<&Position>(conn.from) {
            Ok(p) => p.position,
            Err(_) => continue,
        };
        let to_pos = match world.get::<&Position>(conn.to) {
            Ok(p) => p.position,
            Err(_) => continue,
        };

        // Depth-based taper: from_depth drives start_radius, to_depth drives end_radius.
        let from_depth = world.get::<&DendriteDepth>(conn.from).map(|d| d.0).unwrap_or(0);
        let to_depth   = world.get::<&DendriteDepth>(conn.to).map(|d| d.0).unwrap_or(1);
        let start_radius = depth_radius(from_depth);
        let end_radius   = depth_radius(to_depth);

        // Determine color from the destination compartment's voltage and type.
        let color: Vec3 = if world.get::<&GlialProcess>(conn.to).is_ok() {
            glial_color()
        } else if let Ok(comp) = world.get::<&Compartment>(conn.to) {
            let neuron_type = match world.get::<&NeuronType>(conn.to) {
                Ok(nt) => (*nt).clone(),
                Err(_) => NeuronType::Excitatory,
            };
            let value = ((comp.voltage + 50.0) / 200.0).clamp(0.0, 1.0) as f32;
            neurocolor(&neuron_type, value)
        } else {
            let neuron_type = match world.get::<&NeuronType>(conn.from) {
                Ok(nt) => (*nt).clone(),
                Err(_) => NeuronType::Excitatory,
            };
            neurocolor(&neuron_type, 0.3)
        };

        let _ = world.get::<&Ownership>(conn.to); // ownership tinting handled by color pipeline

        result.push(CylinderData {
            start: [from_pos.x, from_pos.y, from_pos.z],
            start_radius,
            end: [to_pos.x, to_pos.y, to_pos.z],
            end_radius,
            color: [color.x, color.y, color.z],
            _padding: 0.0,
        });
    }

    result
}
