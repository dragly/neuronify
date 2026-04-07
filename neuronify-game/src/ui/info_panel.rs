use hecs::Entity;
use neuronify_core::{Compartment, GeneratorDynamics, Inhibitory, LeakyDynamics, LeakyNeuron, RegularSpikeGenerator};

use crate::components::*;

/// Actions that can be triggered from the info panel.
#[derive(Debug, PartialEq)]
pub enum InfoPanelAction {
    None,
    /// User clicked "Attack..." — app should enter attack-cursor mode.
    EnterAttackMode,
    /// User clicked "Clear target" — app should clear manual targets.
    ClearManualTargets,
    /// User clicked "Go to" on a selected neuroblast — enter move-destination mode.
    GoToDestination,
    /// User clicked "Plant here" — mature selected neuroblast at its current position.
    PlantNeuroblast,
}

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

/// Draw the attack-targeting section for player-owned mobile units.
/// Returns whether the user pressed the Attack or Clear buttons.
fn draw_attack_section(
    ui: &mut egui::Ui,
    world: &hecs::World,
    entities: &[Entity],
    attack_mode: bool,
) -> InfoPanelAction {
    // Collect manual targets from all selected player-owned mobile units.
    let manual_targets: Vec<Option<Entity>> = entities
        .iter()
        .filter_map(|&e| {
            world.get::<&Ownership>(e).ok().and_then(|o| {
                if o.player == PlayerId::Player1 {
                    world.get::<&MobileUnit>(e).ok().map(|m| m.manual_target)
                } else {
                    None
                }
            })
        })
        .collect();

    if manual_targets.is_empty() {
        return InfoPanelAction::None;
    }

    ui.separator();

    // Describe current target state.
    let target_label = if manual_targets.is_empty() {
        "None".to_string()
    } else if manual_targets.iter().all(|t| t.is_none()) {
        "None (AI)".to_string()
    } else if manual_targets.iter().all(|t| *t == manual_targets[0]) {
        if let Some(Some(t)) = manual_targets.first() {
            // Try to name the target.
            let name = if world.get::<&MacrophageUnit>(*t).is_ok() {
                "Macrophage"
            } else if world.get::<&MicroglialCell>(*t).is_ok() {
                "Microglia"
            } else if world.get::<&LeakyNeuron>(*t).is_ok() {
                "Neuron"
            } else {
                "Unknown"
            };
            format!("{name}")
        } else {
            "None (AI)".to_string()
        }
    } else {
        "(mixed)".to_string()
    };

    if attack_mode {
        ui.colored_label(
            egui::Color32::from_rgb(255, 220, 50),
            "Click a target...",
        );
    } else {
        ui.label(format!("Attack target: {target_label}"));
    }

    let mut action = InfoPanelAction::None;
    ui.horizontal(|ui| {
        if ui.button("Attack...").clicked() {
            action = InfoPanelAction::EnterAttackMode;
        }
        if ui.button("Clear target").clicked() {
            action = InfoPanelAction::ClearManualTargets;
        }
    });
    action
}

pub fn draw_info_panel(
    context: &egui::Context,
    world: &hecs::World,
    entities: &[Entity],
    attack_mode: bool,
) -> InfoPanelAction {
    if entities.is_empty() {
        return InfoPanelAction::None;
    }

    let mut panel_action = InfoPanelAction::None;

    egui::Window::new("Info")
        .anchor(egui::Align2::LEFT_BOTTOM, [8.0, -8.0])
        .resizable(false)
        .collapsible(false)
        .show(context, |ui| {
            if entities.len() == 1 {
                panel_action = draw_single_entity(ui, world, entities[0], attack_mode);
            } else {
                panel_action = draw_multi_entity(ui, world, entities, attack_mode);
            }
        });

    panel_action
}

fn draw_single_entity(
    ui: &mut egui::Ui,
    world: &hecs::World,
    entity: Entity,
    attack_mode: bool,
) -> InfoPanelAction {
    if let Ok(glial) = world.get::<&GlialCell>(entity) {
        ui.label(
            egui::RichText::new("Astrocyte")
                .strong()
                .color(egui::Color32::from_rgb(100, 180, 100)),
        );
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        ui.label(format!(
            "Glucose: {:.1} / {:.0}",
            glial.glucose_stored, glial.max_glucose
        ));
        ui.label(format!(
            "Blocks: {:.1} / {:.0}",
            glial.blocks_stored, glial.max_blocks
        ));
        let harvesting = glial.glucose_stored > 0.01;
        if harvesting {
            ui.colored_label(
                egui::Color32::from_rgb(220, 160, 30),
                "Harvesting glucose from vessel terrain",
            );
        } else {
            ui.colored_label(egui::Color32::GRAY, "Not adjacent to vessel terrain");
        }
        ui.separator();
        ui.colored_label(
            egui::Color32::from_rgb(100, 180, 100),
            "Absorbs nearby enemies",
        );
    } else if world.get::<&Neuroblast>(entity).is_ok() {
        let cell_name = if let Ok(nb) = world.get::<&Neuroblast>(entity) {
            match nb.cell_type {
                ProducibleCell::ExcitatoryNeuroblast => "Excitatory Neuroblast",
                ProducibleCell::InhibitoryNeuroblast => "Inhibitory Neuroblast",
            }
        } else {
            "Neuroblast"
        };
        ui.label(
            egui::RichText::new(cell_name)
                .strong()
                .color(egui::Color32::from_rgb(180, 180, 80)),
        );
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        let is_idle = world.get::<&MovePath>(entity).is_err();
        if is_idle {
            ui.colored_label(egui::Color32::GRAY, "Idle — right-click to move");
            if ui.button("Plant here").clicked() {
                return InfoPanelAction::PlantNeuroblast;
            }
        } else {
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "Migrating...");
        }
        if ui.button("Go to").clicked() {
            return InfoPanelAction::GoToDestination;
        }
    } else if world.get::<&MaturingNeuron>(entity).is_ok() {
        if let Ok(maturing) = world.get::<&MaturingNeuron>(entity) {
            let name = match maturing.cell_type {
                ProducibleCell::ExcitatoryNeuroblast => "Excitatory Neuron",
                ProducibleCell::InhibitoryNeuroblast => "Inhibitory Neuron",
            };
            ui.label(
                egui::RichText::new(name)
                    .strong()
                    .color(egui::Color32::from_rgb(120, 120, 200)),
            );
            ui.colored_label(egui::Color32::from_rgb(120, 200, 120), "Maturing...");
            let pct = (maturing.timer / maturing.duration).clamp(0.0, 1.0);
            ui.add(
                egui::ProgressBar::new(pct)
                    .fill(egui::Color32::from_rgb(80, 160, 80))
                    .desired_width(160.0),
            );
        }
    } else if world.get::<&LeakyNeuron>(entity).is_ok() {
        let is_inhibitory = world.get::<&Inhibitory>(entity).is_ok();
        let is_origin = world.get::<&OriginNeuron>(entity).is_ok();
        let has_ownership = world.get::<&Ownership>(entity).is_ok();
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
        if !is_origin {
            let faction = if has_ownership { "Player" } else { "Enemy" };
            ui.label(format!("Faction: {}", faction));
        }

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

        // Fire charge indicator — auto-firing neurons show a buildup bar.
        if let (Ok(gen), Ok(spike)) = (
            world.get::<&GeneratorDynamics>(entity),
            world.get::<&RegularSpikeGenerator>(entity),
        ) {
            if spike.frequency > 0.0 {
                let period = 1.0 / spike.frequency;
                let charge = (gen.time_since_fire / period).clamp(0.0, 1.0) as f32;
                ui.separator();
                let charge_color = egui::Color32::from_rgb(
                    (255.0 * charge) as u8,
                    (200.0 * (1.0 - charge * 0.5)) as u8,
                    (50.0 + 180.0 * (1.0 - charge)) as u8,
                );
                ui.label(format!("Fire rate: {:.0} Hz", spike.frequency));
                ui.add(
                    egui::ProgressBar::new(charge)
                        .text("Charge")
                        .fill(charge_color)
                        .desired_width(160.0),
                );
            }
        }

        // Production status — only for the origin (radial glial) neuron.
        if is_origin {
            if let Ok(queue) = world.get::<&ProductionQueue>(entity) {
                if let Some(front) = queue.items.front() {
                    ui.separator();
                    ui.colored_label(
                        egui::Color32::from_rgb(80, 200, 255),
                        format!("Building: {}…", front.item.label()),
                    );
                    let pct = (front.timer / front.duration).clamp(0.0, 1.0);
                    ui.add(
                        egui::ProgressBar::new(pct)
                            .fill(egui::Color32::from_rgb(80, 160, 220))
                            .desired_width(160.0),
                    );
                    if queue.items.len() > 1 {
                        ui.colored_label(
                            egui::Color32::GRAY,
                            format!("{} more queued", queue.items.len() - 1),
                        );
                    }
                }
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
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        return draw_attack_section(ui, world, &[entity], attack_mode);
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
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        return draw_attack_section(ui, world, &[entity], attack_mode);
    } else if world.get::<&TCellUnit>(entity).is_ok() {
        ui.label(
            egui::RichText::new("T-Cell")
                .strong()
                .color(egui::Color32::from_rgb(120, 220, 50)),
        );
        if let Ok(mobile) = world.get::<&MobileUnit>(entity) {
            ui.label(format!("Faction: {:?}", mobile.faction));
        }
        ui.label("Role: Fast Raider");
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        return draw_attack_section(ui, world, &[entity], attack_mode);
    } else if let Ok(mast) = world.get::<&MastCell>(entity) {
        ui.label(
            egui::RichText::new("Mast Cell")
                .strong()
                .color(egui::Color32::from_rgb(220, 170, 50)),
        );
        ui.label("Role: Immune Relay");
        if let Ok(health) = world.get::<&Health>(entity) {
            health_bar(ui, &health);
        }
        // Show driver neuron status.
        let driver_alive = world.get::<&neuronify_core::Position>(mast.driver).is_ok();
        if driver_alive {
            ui.colored_label(
                egui::Color32::from_rgb(100, 200, 100),
                "Driver neuron: alive",
            );
        } else {
            ui.colored_label(
                egui::Color32::from_rgb(200, 100, 100),
                "Driver neuron: dead",
            );
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
    InfoPanelAction::None
}

fn draw_multi_entity(
    ui: &mut egui::Ui,
    world: &hecs::World,
    entities: &[Entity],
    attack_mode: bool,
) -> InfoPanelAction {
    // Count unit types.
    let n_macro = entities.iter().filter(|&&e| world.get::<&MacrophageUnit>(e).is_ok()).count();
    let n_micro = entities.iter().filter(|&&e| world.get::<&MicroglialCell>(e).is_ok()).count();
    let n_neuron = entities.iter().filter(|&&e| world.get::<&LeakyNeuron>(e).is_ok()).count();
    let n_other = entities.len() - n_macro - n_micro - n_neuron;

    ui.label(
        egui::RichText::new(format!("{} units selected", entities.len()))
            .strong()
            .color(egui::Color32::from_rgb(180, 180, 180)),
    );

    // Type breakdown.
    let mut parts = Vec::new();
    if n_macro > 0 { parts.push(format!("{n_macro}× Macrophage")); }
    if n_micro > 0 { parts.push(format!("{n_micro}× Microglia")); }
    if n_neuron > 0 { parts.push(format!("{n_neuron}× Neuron")); }
    if n_other > 0 { parts.push(format!("{n_other}× Other")); }
    ui.label(parts.join(", "));

    // Aggregate health bar.
    let healths: Vec<f32> = entities
        .iter()
        .filter_map(|&e| world.get::<&Health>(e).ok().map(|h| h.fraction()))
        .collect();
    if !healths.is_empty() {
        let avg = healths.iter().sum::<f32>() / healths.len() as f32;
        let bar_color = if avg > 0.6 {
            egui::Color32::from_rgb(50, 190, 50)
        } else if avg > 0.3 {
            egui::Color32::from_rgb(210, 170, 0)
        } else {
            egui::Color32::from_rgb(210, 50, 50)
        };
        ui.label(format!("Avg. health: {:.0}%", avg * 100.0));
        ui.add(egui::ProgressBar::new(avg).fill(bar_color).desired_width(160.0));
    }

    draw_attack_section(ui, world, entities, attack_mode)
}
