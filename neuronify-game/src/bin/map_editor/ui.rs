//! egui UI for the map editor.

use neuronify_game_lib::voronoi_map::terrain_model::EditorTerrain;

use super::app::{MapEditorApp, Mode, View};

#[allow(deprecated)]
pub fn draw_ui(
    app: &mut MapEditorApp,
    _application: &visula::Application,
    context: &egui::Context,
) {
    // Top panel: title and mode buttons.
    egui::Panel::top("top_panel").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("NEURONIFY RTS — TERRAIN MAP EDITOR")
                    .color(egui::Color32::from_rgb(232, 228, 219))
                    .size(16.0),
            );

            ui.separator();

            let mode_btn = |ui: &mut egui::Ui, label: &str, mode: Mode, current: Mode| -> bool {
                let selected = current == mode;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .color(if selected {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::from_rgb(200, 200, 200)
                        })
                        .size(12.0),
                )
                .fill(if selected {
                    egui::Color32::from_rgb(240, 160, 48)
                } else {
                    egui::Color32::from_rgb(42, 42, 46)
                });
                ui.add(btn).clicked()
            };

            if mode_btn(ui, "ORBIT", Mode::Orbit, app.mode) {
                app.mode = Mode::Orbit;
                app.dots_dirty = true;
            }
            if mode_btn(ui, "EDIT", Mode::EditVertex, app.mode) {
                app.mode = Mode::EditVertex;
                app.dots_dirty = true;
            }
            if mode_btn(ui, "PAINT", Mode::PaintTerrain, app.mode) {
                app.mode = Mode::PaintTerrain;
                app.dots_dirty = true;
            }

            ui.separator();

            let view_btn = |ui: &mut egui::Ui, label: &str, view: View, current: View| -> bool {
                let selected = current == view;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .color(if selected {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(150, 150, 150)
                        })
                        .size(11.0),
                )
                .fill(if selected {
                    egui::Color32::from_rgb(60, 60, 70)
                } else {
                    egui::Color32::from_rgb(30, 30, 34)
                });
                ui.add(btn).clicked()
            };

            if view_btn(ui, "3D", View::FullMap, app.view) {
                app.view = View::FullMap;
            }
            if view_btn(ui, "TOP", View::TopDown, app.view) {
                app.view = View::TopDown;
                app.phi = 0.01;
                app.theta = std::f32::consts::FRAC_PI_2;
                app.dist = 500.0;
            }
            if view_btn(ui, "TILES", View::TileCatalog, app.view) {
                app.view = View::TileCatalog;
                app.catalog_dirty = true;
                app.phi = 0.6;
                app.theta = 0.8;
                app.dist = 200.0;
                app.camera_center = glam::Vec3::ZERO;
            }
        });
    });

    // Left panel: terrain tools and info.
    egui::Panel::left("tools_panel")
        .default_size(160.0)
        .show(context, |ui| {
            ui.label(
                egui::RichText::new("Terrain")
                    .color(egui::Color32::from_rgb(170, 170, 170))
                    .size(13.0),
            );
            ui.add_space(4.0);

            for &terrain in &EditorTerrain::ALL {
                let [r, g, b] = terrain.color_u8();
                let selected = app.selected_terrain == terrain;
                let btn = egui::Button::new(
                    egui::RichText::new(terrain.name().to_uppercase())
                        .color(if selected {
                            if terrain == EditorTerrain::GlialScar {
                                egui::Color32::from_rgb(220, 220, 220)
                            } else {
                                egui::Color32::from_rgb(30, 30, 30)
                            }
                        } else {
                            egui::Color32::from_rgb(170, 170, 170)
                        })
                        .size(11.0),
                )
                .fill(if selected {
                    egui::Color32::from_rgb(r, g, b)
                } else {
                    egui::Color32::from_rgb(42, 42, 46)
                });

                if ui.add(btn).clicked() {
                    app.selected_terrain = terrain;
                }
            }

            ui.add_space(8.0);

            if ui.checkbox(&mut app.show_wireframe, "Show triangles").changed() {
                app.wireframe_dirty = true;
            }
            ui.checkbox(&mut app.height_only, "Height only (edit)");

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Info")
                    .color(egui::Color32::from_rgb(170, 170, 170))
                    .size(13.0),
            );
            ui.label(
                egui::RichText::new(format!("Cells: {}", app.model.cell_centers.len()))
                    .color(egui::Color32::from_rgb(100, 100, 100))
                    .size(10.0),
            );
            ui.label(
                egui::RichText::new(format!("Triangles: {}", app.model.triangles.len()))
                    .color(egui::Color32::from_rgb(100, 100, 100))
                    .size(10.0),
            );

            let mode_desc = match app.mode {
                Mode::Orbit => "Drag to orbit, scroll to zoom",
                Mode::EditVertex => "Drag vertices to adjust height",
                Mode::PaintTerrain => "Click cells to paint terrain",
            };
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(mode_desc)
                    .color(egui::Color32::from_rgb(100, 100, 100))
                    .size(9.0),
            );

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            // Transition bias sliders.
            ui.label(
                egui::RichText::new("Transition Bias")
                    .color(egui::Color32::from_rgb(170, 170, 170))
                    .size(13.0),
            );
            ui.label(
                egui::RichText::new("Shift terrain boundaries")
                    .color(egui::Color32::from_rgb(100, 100, 100))
                    .size(9.0),
            );
            ui.add_space(2.0);
            {
                use neuronify_game_lib::voronoi_map::terrain_model::{EdgeKey, EditorTerrain};
                let pairs: &[(EditorTerrain, EditorTerrain, &str)] = &[
                    (EditorTerrain::Open, EditorTerrain::Vessel, "open/vessel"),
                    (EditorTerrain::Open, EditorTerrain::GlialScar, "open/scar"),
                    (EditorTerrain::Open, EditorTerrain::Csf, "open/csf"),
                    (EditorTerrain::Vessel, EditorTerrain::GlialScar, "vessel/scar"),
                    (EditorTerrain::Vessel, EditorTerrain::Csf, "vessel/csf"),
                    (EditorTerrain::GlialScar, EditorTerrain::Csf, "scar/csf"),
                ];
                for &(a, b, label) in pairs {
                    let key = EdgeKey::new(a, b);
                    let mut val = *app.model.transition_biases.get(&key).unwrap_or(&0.5);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(label)
                                .color(egui::Color32::from_rgb(140, 140, 140))
                                .size(9.0),
                        );
                        if ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(false)).changed() {
                            app.model.transition_biases.insert(key, val);
                            app.mesh_dirty = true;
                        }
                    });
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            // Save/load buttons.
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("SAVE")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .size(11.0),
                ))
                .clicked()
            {
                match serde_json::to_string_pretty(&app.model) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write("maps/voronoi-scenario.json", &json) {
                            log::error!("Failed to save: {}", e);
                        } else {
                            log::info!("Saved to maps/voronoi-scenario.json");
                        }
                    }
                    Err(e) => log::error!("Failed to serialize: {}", e),
                }
            }

            if ui
                .add(egui::Button::new(
                    egui::RichText::new("LOAD")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .size(11.0),
                ))
                .clicked()
            {
                match std::fs::read_to_string("maps/voronoi-scenario.json") {
                    Ok(json) => match serde_json::from_str(&json) {
                        Ok(model) => {
                            app.model = model;
                            app.mesh_dirty = true;
                            log::info!("Loaded from maps/voronoi-scenario.json");
                        }
                        Err(e) => log::error!("Failed to parse: {}", e),
                    },
                    Err(e) => log::error!("Failed to read: {}", e),
                }
            }

            if ui
                .add(egui::Button::new(
                    egui::RichText::new("RESET HEIGHTS")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .size(11.0),
                ))
                .clicked()
            {
                app.model.edge_profiles.clear();
                app.model.interior_profiles.clear();
                app.model.transition_biases.clear();
                app.model.terrain_height_offsets.clear();
                app.model.ensure_profiles();
                app.mesh_dirty = true;
            }
        });
}
