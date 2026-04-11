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
const DENDRITE_ROOT_RADIUS: f32 = 0.60;
/// Radius shrinks by this factor per depth hop.
const DENDRITE_TAPER: f32 = 0.78;
/// Floor so dendrites/axons never disappear entirely.
const DENDRITE_MIN_RADIUS: f32 = 0.16;

/// Consistent radius for axon compartments (no DendriteDepth).
const AXON_RADIUS: f32 = 0.28;

pub fn depth_radius(depth: u32) -> f32 {
    (DENDRITE_ROOT_RADIUS * DENDRITE_TAPER.powi(depth as i32)).max(DENDRITE_MIN_RADIUS)
}

/// Get the cylinder radius for a given entity (for matching sphere size at joints).
pub fn entity_radius(world: &hecs::World, entity: hecs::Entity) -> f32 {
    use neuronify_core::LeakyNeuron;

    if world.get::<&LeakyNeuron>(entity).is_ok() {
        return DENDRITE_ROOT_RADIUS;
    }
    if let Ok(pos) = world.get::<&Position>(entity) {
        let pos = pos.position;
        let min_dist = world
            .query::<(&LeakyNeuron, &Position)>()
            .iter()
            .map(|(_, (_, np))| {
                let dx = pos.x - np.position.x;
                let dz = pos.z - np.position.z;
                (dx * dx + dz * dz).sqrt()
            })
            .fold(f32::MAX, f32::min);
        radius_from_neuron_distance(min_dist)
    } else {
        DENDRITE_MIN_RADIUS
    }
}

/// Compute radius from distance to nearest neuron soma.
fn radius_from_neuron_distance(dist: f32) -> f32 {
    // Thick near soma, tapering with distance, but never below minimum.
    (DENDRITE_ROOT_RADIUS * DENDRITE_TAPER.powf(dist / 3.0)).max(DENDRITE_MIN_RADIUS)
}

pub fn collect_dendrite_cylinders(world: &hecs::World) -> Vec<CylinderData> {
    use neuronify_core::LeakyNeuron;

    // Collect all neuron soma positions for distance lookups.
    let neuron_positions: Vec<Vec3> = world
        .query::<(&LeakyNeuron, &Position)>()
        .iter()
        .map(|(_, (_, p))| p.position)
        .collect();

    let nearest_neuron_dist = |pos: Vec3| -> f32 {
        neuron_positions
            .iter()
            .map(|np| {
                let dx = pos.x - np.x;
                let dz = pos.z - np.z;
                (dx * dx + dz * dz).sqrt()
            })
            .fold(f32::MAX, f32::min)
    };

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

        // Radius based on distance to nearest neuron soma.
        let from_is_soma = world.get::<&LeakyNeuron>(conn.from).is_ok();
        let to_is_soma = world.get::<&LeakyNeuron>(conn.to).is_ok();
        let start_radius = if from_is_soma {
            DENDRITE_ROOT_RADIUS
        } else {
            radius_from_neuron_distance(nearest_neuron_dist(from_pos))
        };
        let end_radius = if to_is_soma {
            DENDRITE_ROOT_RADIUS
        } else {
            radius_from_neuron_distance(nearest_neuron_dist(to_pos))
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
