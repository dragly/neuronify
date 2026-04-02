use hecs::Entity;
use neuronify_core::{Compartment, Inhibitory, LeakyDynamics, LeakyNeuron};

use crate::components::*;

fn health_bar(ui: &mut egui::Ui, health: &Health) {
    let pct = health.fraction().clamp(0.0, 1.0);
    let bar_color = if pct > 0.6 {
        egui::Color32::from_rgb(50, 190, 50)
    } else if pct > 0.3 {
        egui::Color32::from_rgb(210, 170, 0)
    } else {
        egui::Color32::from_rgb(210, 50, 50)
    };
    ui.label(format!(
        "Health: {:.0} / {:.0}",
        health.current, health.max
    ));
    ui.add(
        egui::ProgressBar::new(pct)
            .fill(bar_color)
            .desired_width(160.0),
    );
}

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

                if let Ok(health) = world.get::<&Health>(entity) {
                    health_bar(ui, &health);
                }
                if let Ok(metab) = world.get::<&MetabolicState>(entity) {
                    let pct = (metab.energy / metab.max_energy * 100.0).clamp(0.0, 100.0);
                    let bar_color = if pct > 50.0 {
                        egui::Color32::from_rgb(50, 120, 220)
                    } else if pct > 20.0 {
                        egui::Color32::from_rgb(100, 80, 200)
                    } else {
                        egui::Color32::from_rgb(180, 50, 180)
                    };
                    ui.label(format!(
                        "Energy: {:.0} / {:.0} ({:.0}%)",
                        metab.energy, metab.max_energy, pct
                    ));
                    ui.add(
                        egui::ProgressBar::new((pct / 100.0) as f32)
                            .fill(bar_color)
                            .desired_width(160.0),
                    );
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
            } else if world.get::<&MacrophageUnit>(entity).is_ok() {
                ui.label(
                    egui::RichText::new("Macrophage")
                        .strong()
                        .color(egui::Color32::from_rgb(200, 50, 200)),
                );
                if let Ok(mobile) = world.get::<&MobileUnit>(entity) {
                    ui.label(format!("Faction: {:?}", mobile.faction));
                }
                ui.label("Role: Neuron Destroyer");
                ui.label("Drains neuron energy on contact.");
                if let Ok(health) = world.get::<&Health>(entity) {
                    health_bar(ui, &health);
                }
            } else if world.get::<&MicroglialCell>(entity).is_ok() {
                ui.label(
                    egui::RichText::new("Microglial Cell")
                        .strong()
                        .color(egui::Color32::from_rgb(0, 190, 210)),
                );
                if let Ok(mobile) = world.get::<&MobileUnit>(entity) {
                    ui.label(format!("Faction: {:?}", mobile.faction));
                }
                ui.label("Role: Connection Cutter");
                ui.label("Severs axon compartments.");
                if let Ok(health) = world.get::<&Health>(entity) {
                    health_bar(ui, &health);
                }
            } else if world.get::<&ReactiveAstrocyte>(entity).is_ok() {
                ui.label(
                    egui::RichText::new("Reactive Astrocyte")
                        .strong()
                        .color(egui::Color32::from_rgb(220, 160, 20)),
                );
                let faction = if world.get::<&Ownership>(entity).is_ok() {
                    "Biological (yours)"
                } else {
                    "Enemy"
                };
                ui.label(format!("Faction: {}", faction));
                ui.label("Role: Area Control");
                ui.label("Drains health of nearby enemy units.");
                if let Ok(health) = world.get::<&Health>(entity) {
                    health_bar(ui, &health);
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
                if let Ok(axon_health) = world.get::<&AxonHealth>(entity) {
                    let pct = (axon_health.current / axon_health.max).clamp(0.0, 1.0);
                    let bar_color = if pct > 0.6 {
                        egui::Color32::from_rgb(50, 190, 50)
                    } else if pct > 0.3 {
                        egui::Color32::from_rgb(210, 170, 0)
                    } else {
                        egui::Color32::from_rgb(210, 50, 50)
                    };
                    ui.label(format!("Integrity: {:.0} / {:.0}", axon_health.current, axon_health.max));
                    ui.add(
                        egui::ProgressBar::new(pct)
                            .fill(bar_color)
                            .desired_width(160.0),
                    );
                }
            }
        });
}
