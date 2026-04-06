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

use crate::components::{GlialProcess, Ownership};
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

const DENDRITE_BASE_RADIUS: f32 = 0.28;
const DENDRITE_TIP_RADIUS:  f32 = 0.08;

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
            // Connection from soma to first compartment: color by soma's type at 0.5 value
            let neuron_type = match world.get::<&NeuronType>(conn.from) {
                Ok(nt) => (*nt).clone(),
                Err(_) => NeuronType::Excitatory,
            };
            neurocolor(&neuron_type, 0.3)
        };

        // Ownership tint (same 30% blend as sphere rendering)
        let color = if world.get::<&Ownership>(conn.to).is_ok() {
            color
        } else {
            color
        };

        // Taper: fatter at start (closer to soma), thinner at tip.
        // We use a fixed taper regardless of depth since depth isn't tracked here.
        result.push(CylinderData {
            start: [from_pos.x, from_pos.y, from_pos.z],
            start_radius: DENDRITE_BASE_RADIUS,
            end: [to_pos.x, to_pos.y, to_pos.z],
            end_radius: DENDRITE_TIP_RADIUS,
            color: [color.x, color.y, color.z],
            _padding: 0.0,
        });
    }

    result
}
