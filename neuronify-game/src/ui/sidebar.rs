use crate::simulation::scenarios::ScenarioId;
use crate::tools::GameTool;

// ── Programmatic cell-type icons ──────────────────────────────────────────────

enum IconShape {
    Circle { radius: f32 },
    HBar,   // three small circles in a row (axon / process)
    Ring,   // hollow circle (astrocyte)
    Cross,  // diagonal X (erase)
}

fn make_icon(
    ctx: &egui::Context,
    name: &str,
    color: egui::Color32,
    shape: IconShape,
) -> egui::TextureHandle {
    const S: usize = 32;
    let mut pixels = vec![egui::Color32::TRANSPARENT; S * S];
    match shape {
        IconShape::Circle { radius } => {
            for y in 0..S {
                for x in 0..S {
                    let dx = x as f32 - (S as f32 / 2.0 - 0.5);
                    let dy = y as f32 - (S as f32 / 2.0 - 0.5);
                    if dx * dx + dy * dy <= radius * radius {
                        pixels[y * S + x] = color;
                    }
                }
            }
        }
        IconShape::HBar => {
            for &cx in &[6u32, 15, 25] {
                for y in 0..S {
                    for x in 0..S {
                        let dx = x as f32 - cx as f32;
                        let dy = y as f32 - 15.5;
                        if dx * dx + dy * dy <= 5.0 * 5.0 {
                            pixels[y * S + x] = color;
                        }
                    }
                }
            }
        }
        IconShape::Ring => {
            for y in 0..S {
                for x in 0..S {
                    let dx = x as f32 - 15.5;
                    let dy = y as f32 - 15.5;
                    let r2 = dx * dx + dy * dy;
                    if r2 <= 14.0 * 14.0 && r2 >= 7.0 * 7.0 {
                        pixels[y * S + x] = color;
                    }
                }
            }
        }
        IconShape::Cross => {
            for i in 0..S {
                let j = S - 1 - i;
                for d in 0..3i32 {
                    let o = d - 1;
                    let ix = (i as i32 + o).clamp(0, S as i32 - 1) as usize;
                    let jx = (j as i32 + o).clamp(0, S as i32 - 1) as usize;
                    pixels[i * S + ix] = color;
                    pixels[i * S + jx] = color;
                }
            }
        }
    }
    ctx.load_texture(
        name,
        egui::ColorImage { size: [S, S], pixels, source_size: egui::Vec2::splat(S as f32) },
        egui::TextureOptions::default(),
    )
}

pub struct SidebarIcons {
    pub glial:      egui::TextureHandle,
    pub excitatory: egui::TextureHandle,
    pub inhibitory: egui::TextureHandle,
    pub axon:       egui::TextureHandle,
    pub process:    egui::TextureHandle,
    pub astrocyte:  egui::TextureHandle,
    pub select:     egui::TextureHandle,
    pub erase:      egui::TextureHandle,
}

impl SidebarIcons {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            glial:      make_icon(ctx, "icon_glial",      egui::Color32::from_rgb(0,   180,  60), IconShape::Circle { radius: 10.0 }),
            excitatory: make_icon(ctx, "icon_excitatory", egui::Color32::from_rgb(80,  120, 200), IconShape::Circle { radius: 13.0 }),
            inhibitory: make_icon(ctx, "icon_inhibitory", egui::Color32::from_rgb(200,  80,  80), IconShape::Circle { radius: 13.0 }),
            axon:       make_icon(ctx, "icon_axon",       egui::Color32::from_rgb(150, 150, 150), IconShape::HBar),
            process:    make_icon(ctx, "icon_process",    egui::Color32::from_rgb(0,   180,  60), IconShape::HBar),
            astrocyte:  make_icon(ctx, "icon_astrocyte",  egui::Color32::from_rgb(220, 140,  30), IconShape::Ring),
            select:     make_icon(ctx, "icon_select",     egui::Color32::from_rgb(180, 180, 180), IconShape::Circle { radius: 10.0 }),
            erase:      make_icon(ctx, "icon_erase",      egui::Color32::from_rgb(220,  80,  80), IconShape::Cross),
        }
    }
}

// ── Sidebar drawing ───────────────────────────────────────────────────────────

pub fn draw_sidebar(
    context: &egui::Context,
    tool: &mut GameTool,
    neurons: u32,
    energy: f64,
    blocks: f64,
    funds_flash: bool,
    pending_new_game: &mut bool,
    pending_exit_game: &mut bool,
    current_scenario: &mut ScenarioId,
    icons: &SidebarIcons,
) {
    egui::SidePanel::right("rts_sidebar")
        .min_width(180.0)
        .max_width(200.0)
        .show(context, |ui| {
            let stats_color = if funds_flash {
                egui::Color32::from_rgb(220, 60, 60)
            } else {
                egui::Color32::from_rgb(64, 160, 43)
            };
            ui.colored_label(
                stats_color,
                egui::RichText::new(format!(
                    "{} neurons | {:.0}E | {:.0}B",
                    neurons, energy, blocks
                ))
                .strong(),
            );
            ui.separator();

            // BUILD section
            ui.label(
                egui::RichText::new("BUILD")
                    .strong()
                    .color(egui::Color32::GRAY),
            );
            egui::Grid::new("build_grid")
                .num_columns(2)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    tool_button(ui, tool, GameTool::GlialCell,        "Glial",      "8B",     Some(&icons.glial));
                    tool_button(ui, tool, GameTool::ExcitatoryNeuron, "Excitatory", "25B",    Some(&icons.excitatory));
                    ui.end_row();
                    tool_button(ui, tool, GameTool::InhibitoryNeuron, "Inhibitory", "25B",    Some(&icons.inhibitory));
                    tool_button(ui, tool, GameTool::Axon,             "Axon",       "3B/seg", Some(&icons.axon));
                    ui.end_row();
                    tool_button(ui, tool, GameTool::GlialProcess,     "Process",    "3B/seg", Some(&icons.process));
                    tool_button(ui, tool, GameTool::ReactiveAstrocyte,"Astrocyte",  "20B",    Some(&icons.astrocyte));
                    ui.end_row();
                });

            ui.separator();

            // ACTIONS section
            ui.label(
                egui::RichText::new("ACTIONS")
                    .strong()
                    .color(egui::Color32::GRAY),
            );
            egui::Grid::new("actions_grid")
                .num_columns(2)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    tool_button(ui, tool, GameTool::Select, "Select", "", Some(&icons.select));
                    tool_button(ui, tool, GameTool::Erase,  "Erase",  "", Some(&icons.erase));
                    ui.end_row();
                });

            ui.separator();

            // SCENARIO section
            ui.label(
                egui::RichText::new("SCENARIO")
                    .strong()
                    .color(egui::Color32::GRAY),
            );
            let mut changed = false;
            for &scenario in ScenarioId::all() {
                let selected = *current_scenario == scenario;
                if ui.radio(selected, scenario.label()).clicked() && !selected {
                    *current_scenario = scenario;
                    changed = true;
                }
            }
            if changed {
                *pending_new_game = true;
            }

            ui.separator();

            // GAME section
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("New Game").min_size(egui::vec2(80.0, 30.0)))
                    .clicked()
                {
                    *pending_new_game = true;
                }
                if ui
                    .add(egui::Button::new("Exit").min_size(egui::vec2(60.0, 30.0)))
                    .clicked()
                {
                    *pending_exit_game = true;
                }
            });
        });
}

fn tool_button(
    ui: &mut egui::Ui,
    current_tool: &mut GameTool,
    tool: GameTool,
    label: &str,
    cost: &str,
    icon: Option<&egui::TextureHandle>,
) {
    let selected = *current_tool == tool;
    let text = if cost.is_empty() {
        egui::RichText::new(label).small()
    } else {
        egui::RichText::new(format!("{}\n{}", label, cost)).small()
    };
    let btn = if let Some(handle) = icon {
        let img = egui::Image::new(handle).fit_to_exact_size(egui::vec2(20.0, 20.0));
        egui::Button::image_and_text(img, text)
            .min_size(egui::vec2(82.0, 40.0))
            .selected(selected)
    } else {
        egui::Button::new(text)
            .min_size(egui::vec2(82.0, 40.0))
            .selected(selected)
    };
    if ui.add(btn).clicked() {
        *current_tool = tool;
    }
}
