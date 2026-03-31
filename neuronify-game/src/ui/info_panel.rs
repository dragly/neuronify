use hecs::Entity;
use neuronify_core::{Compartment, Inhibitory, LeakyDynamics, LeakyNeuron};

use crate::components::*;

pub fn draw_info_panel(context: &egui::Context, world: &hecs::World, entity: Entity) {
    egui::Window::new("Info")
        .anchor(egui::Align2::LEFT_BOTTOM, [8.0, -8.0])
        .resizable(false)
        .collapsible(false)
        .show(context, |ui| {
            if let Ok(glial) = world.get::<&GlialCell>(entity) {
                ui.label(
                    egui::RichText::new("Astrocyte")
                        .strong()
                        .color(egui::Color32::from_rgb(100, 180, 100)),
                );
                ui.label(format!(
                    "Glucose: {:.1} / {:.0}",
                    glial.glucose_stored, glial.max_glucose
                ));
                ui.label(format!(
                    "Blocks: {:.1} / {:.0}",
                    glial.blocks_stored, glial.max_blocks
                ));
                let connected = glial.glucose_stored > 0.01;
                if connected {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 160, 30),
                        "Receiving glucose from vessel",
                    );
                } else {
                    ui.colored_label(egui::Color32::GRAY, "Not connected to vessel");
                }
            } else if let Ok(vessel) = world.get::<&BloodVessel>(entity) {
                ui.label(
                    egui::RichText::new("Blood Vessel")
                        .strong()
                        .color(egui::Color32::from_rgb(180, 30, 30)),
                );
                ui.label(format!("Glucose supply: {:.1}/s", vessel.glucose_rate));
                ui.label(format!("Block supply: {:.1}/s", vessel.block_rate));
                ui.colored_label(
                    egui::Color32::GRAY,
                    "Connect glial processes to extract resources",
                );
            } else if world.get::<&LeakyNeuron>(entity).is_ok() {
                let is_inhibitory = world.get::<&Inhibitory>(entity).is_ok();
                let is_origin = world.get::<&OriginNeuron>(entity).is_ok();
                let type_name = if is_origin {
                    "Origin Neuron"
                } else if is_inhibitory {
                    "Inhibitory Neuron"
                } else {
                    "Excitatory Neuron"
                };
                let color = if is_inhibitory {
                    egui::Color32::from_rgb(200, 80, 80)
                } else {
                    egui::Color32::from_rgb(80, 120, 200)
                };
                ui.label(egui::RichText::new(type_name).strong().color(color));

                if let Ok(metab) = world.get::<&MetabolicState>(entity) {
                    let pct = (metab.energy / metab.max_energy * 100.0).clamp(0.0, 100.0);
                    ui.label(format!(
                        "Energy: {:.0} / {:.0} ({:.0}%)",
                        metab.energy, metab.max_energy, pct
                    ));
                }
                if let Ok(dynamics) = world.get::<&LeakyDynamics>(entity) {
                    if dynamics.enabled {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "Active");
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 100, 100),
                            "Dormant (no energy)",
                        );
                    }
                }
            } else if let Ok(comp) = world.get::<&Compartment>(entity) {
                let is_process = world.get::<&GlialProcess>(entity).is_ok();
                let is_dendrite = world.get::<&Dendrite>(entity).is_ok();
                let label = if is_process {
                    "Glial Process"
                } else if is_dendrite {
                    "Dendrite"
                } else {
                    "Axon Segment"
                };
                ui.label(
                    egui::RichText::new(label)
                        .strong()
                        .color(egui::Color32::GRAY),
                );
                ui.label(format!("Voltage: {:.1} mV", comp.voltage));
            }
        });
}
