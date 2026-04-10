//! Main menu screen shown before gameplay begins.

use crate::map::ScenarioMeta;
use crate::simulation::dev_scenarios::DevStage;

/// A playable scenario entry shown in the main menu.
pub struct ScenarioEntry {
    pub meta: ScenarioMeta,
    /// Embedded SVG content (compiled into the binary via `include_str!`).
    pub svg_content: &'static str,
    /// If `Some`, this is a dev-only stage scenario.
    pub dev_stage: Option<DevStage>,
}

/// Return value from `draw_main_menu`.
pub enum MenuAction {
    None,
    /// Player selected a scenario and pressed Play. Carries the embedded SVG content.
    StartScenario(&'static str),
    /// Player selected a dev scenario. Carries SVG content + stage.
    StartDevScenario(&'static str, DevStage),
    /// Player selected the Voronoi-based map scenario.
    StartVoronoiScenario,
    Exit,
}

/// Draw the full-screen main menu. Returns what the player chose this frame.
#[allow(deprecated)]
pub fn draw_main_menu(
    context: &egui::Context,
    scenarios: &[ScenarioEntry],
    selected: &mut Option<usize>,
    dev_mode: bool,
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

                // Split entries into regular and dev scenarios.
                let regular: Vec<(usize, &ScenarioEntry)> = scenarios
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.dev_stage.is_none())
                    .collect();
                let dev: Vec<(usize, &ScenarioEntry)> = scenarios
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.dev_stage.is_some())
                    .collect();

                // Scrollable area for scenario cards.
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 70.0) // reserve space for buttons
                    .show(ui, |ui| {
                        // Scenario cards — selection disabled so clicks register on the card.
                        ui.style_mut().interaction.selectable_labels = false;

                        for &(i, entry) in &regular {
                            if draw_scenario_card(ui, entry, *selected == Some(i), false) {
                                *selected = Some(i);
                            }
                            ui.add_space(8.0);
                        }

                        // Voronoi map scenario — special entry.
                        let voronoi_selected = *selected == Some(usize::MAX);
                        let voronoi_frame = egui::Frame::new()
                            .fill(if voronoi_selected {
                                egui::Color32::from_rgb(35, 50, 40)
                            } else {
                                egui::Color32::from_rgb(28, 28, 32)
                            })
                            .corner_radius(egui::CornerRadius::same(4))
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .stroke(if voronoi_selected {
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 200, 120))
                            } else {
                                egui::Stroke::new(0.5, egui::Color32::from_rgb(50, 50, 55))
                            });
                        let resp = voronoi_frame.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Neural Infiltration")
                                    .font(egui::FontId::monospace(13.0))
                                    .color(egui::Color32::from_rgb(140, 210, 175))
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("Voronoi terrain — convergent network vs inhibitory suppressor")
                                    .font(egui::FontId::monospace(10.0))
                                    .color(egui::Color32::from_rgb(120, 120, 130)),
                            );
                        });
                        if resp.response.interact(egui::Sense::click()).clicked() {
                            *selected = Some(usize::MAX);
                        }
                        ui.add_space(8.0);

                        // Dev scenario section — only when --dev was passed.
                        if dev_mode && !dev.is_empty() {
                            ui.add_space(16.0);
                            ui.separator();
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("DEV SCENARIOS")
                                    .font(egui::FontId::monospace(12.0))
                                    .color(egui::Color32::from_rgb(255, 190, 60))
                                    .strong(),
                            );
                            ui.add_space(8.0);

                            for &(i, entry) in &dev {
                                if draw_scenario_card(ui, entry, *selected == Some(i), true) {
                                    *selected = Some(i);
                                }
                                ui.add_space(8.0);
                            }
                        }
                    });

                ui.add_space(12.0);

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
                                if idx == usize::MAX {
                                    action = MenuAction::StartVoronoiScenario;
                                } else {
                                    let entry = &scenarios[idx];
                                    if let Some(stage) = entry.dev_stage {
                                        action = MenuAction::StartDevScenario(entry.svg_content, stage);
                                    } else {
                                        action = MenuAction::StartScenario(entry.svg_content);
                                    }
                                }
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

/// Render a single scenario card. `is_dev` adds an amber "DEV" badge.
/// Returns `true` if the card was clicked.
#[allow(deprecated)]
fn draw_scenario_card(
    ui: &mut egui::Ui,
    entry: &ScenarioEntry,
    is_selected: bool,
    is_dev: bool,
) -> bool {
    let card_color = if is_selected {
        egui::Color32::from_rgb(30, 50, 70)
    } else {
        egui::Color32::from_rgb(22, 22, 28)
    };
    let border_color = if is_selected {
        egui::Color32::from_rgb(80, 160, 240)
    } else if is_dev {
        egui::Color32::from_rgb(80, 60, 30)
    } else {
        egui::Color32::from_rgb(50, 50, 60)
    };

    let resp = egui::Frame::new()
        .fill(card_color)
        .stroke(egui::Stroke::new(1.5, border_color))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .corner_radius(egui::CornerRadius::same(6))
        .show(ui, |ui| {
            ui.set_min_width(480.0);
            ui.set_max_width(560.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&entry.meta.name)
                                .font(egui::FontId::monospace(15.0))
                                .color(egui::Color32::from_rgb(220, 220, 255))
                                .strong(),
                        );
                        if is_dev {
                            ui.label(
                                egui::RichText::new(" DEV")
                                    .font(egui::FontId::monospace(10.0))
                                    .color(egui::Color32::from_rgb(255, 190, 60))
                                    .strong(),
                            );
                        }
                    });
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

    resp.response.interact(egui::Sense::click()).clicked()
}
