//! Main menu screen shown before gameplay begins.

use crate::map::ScenarioMeta;

/// A playable scenario entry shown in the main menu.
pub struct ScenarioEntry {
    pub meta: ScenarioMeta,
    pub svg_path: std::path::PathBuf,
}

/// Return value from `draw_main_menu`.
pub enum MenuAction {
    None,
    /// Player selected a scenario and pressed Play. Carries the SVG file path.
    StartScenario(std::path::PathBuf),
    Exit,
}

/// Draw the full-screen main menu. Returns what the player chose this frame.
#[allow(deprecated)]
pub fn draw_main_menu(
    context: &egui::Context,
    scenarios: &[ScenarioEntry],
    selected: &mut Option<usize>,
) -> MenuAction {
    let mut action = MenuAction::None;

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(egui::Color32::from_rgb(18, 18, 22)))
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(48.0);

                ui.label(
                    egui::RichText::new("NEURAL STRATEGY")
                        .font(egui::FontId::proportional(48.0))
                        .color(egui::Color32::from_rgb(100, 200, 255))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("real-time neural strategy")
                        .font(egui::FontId::monospace(13.0))
                        .color(egui::Color32::from_rgb(100, 100, 120)),
                );

                ui.add_space(40.0);
                ui.separator();
                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("SELECT SCENARIO")
                        .font(egui::FontId::monospace(14.0))
                        .color(egui::Color32::from_rgb(160, 160, 180))
                        .strong(),
                );

                ui.add_space(16.0);

                // Scenario cards — selection disabled so clicks register on the card, not the text.
                ui.style_mut().interaction.selectable_labels = false;

                for (i, entry) in scenarios.iter().enumerate() {
                    let is_selected = *selected == Some(i);

                    let card_color = if is_selected {
                        egui::Color32::from_rgb(30, 50, 70)
                    } else {
                        egui::Color32::from_rgb(22, 22, 28)
                    };

                    let border_color = if is_selected {
                        egui::Color32::from_rgb(80, 160, 240)
                    } else {
                        egui::Color32::from_rgb(50, 50, 60)
                    };

                    let frame = egui::Frame::new()
                        .fill(card_color)
                        .stroke(egui::Stroke::new(1.5, border_color))
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .corner_radius(egui::CornerRadius::same(6));

                    let resp = frame.show(ui, |ui| {
                        ui.set_min_width(480.0);
                        ui.set_max_width(560.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&entry.meta.name)
                                        .font(egui::FontId::monospace(15.0))
                                        .color(egui::Color32::from_rgb(220, 220, 255))
                                        .strong(),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(&entry.meta.objective)
                                        .font(egui::FontId::monospace(11.0))
                                        .color(egui::Color32::from_rgb(140, 200, 140)),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(&entry.meta.briefing)
                                        .font(egui::FontId::monospace(10.0))
                                        .color(egui::Color32::from_rgb(140, 140, 160)),
                                );
                            });
                        });
                    });

                    // Clicking anywhere on the card selects it
                    if resp.response.interact(egui::Sense::click()).clicked() {
                        *selected = Some(i);
                    }

                    ui.add_space(8.0);
                }

                ui.add_space(24.0);

                // Play / Exit buttons
                ui.horizontal(|ui| {
                    let play_enabled = selected.is_some();

                    ui.add_enabled_ui(play_enabled, |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("▶  PLAY")
                                        .font(egui::FontId::monospace(14.0))
                                        .color(egui::Color32::from_rgb(80, 220, 120)),
                                )
                                .min_size(egui::vec2(140.0, 40.0))
                                .fill(egui::Color32::from_rgb(25, 50, 30)),
                            )
                            .clicked()
                        {
                            if let Some(idx) = *selected {
                                action = MenuAction::StartScenario(
                                    scenarios[idx].svg_path.clone(),
                                );
                            }
                        }
                    });

                    ui.add_space(12.0);

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("EXIT")
                                    .font(egui::FontId::monospace(13.0))
                                    .color(egui::Color32::from_rgb(200, 100, 100)),
                            )
                            .min_size(egui::vec2(80.0, 40.0))
                            .fill(egui::Color32::from_rgb(40, 20, 20)),
                        )
                        .clicked()
                    {
                        action = MenuAction::Exit;
                    }
                });

                ui.add_space(16.0);
            });
        });

    action
}
