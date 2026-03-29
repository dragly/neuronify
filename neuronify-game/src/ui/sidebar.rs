use crate::tools::GameTool;

pub fn draw_sidebar(
    context: &egui::Context,
    tool: &mut GameTool,
    p1_neurons: u32,
    p1_energy: f64,
    p1_blocks: f64,
    p2_neurons: u32,
    p2_energy: f64,
    p2_blocks: f64,
    pending_new_game: &mut bool,
    pending_exit_game: &mut bool,
) {
    egui::SidePanel::right("rts_sidebar")
        .min_width(180.0)
        .max_width(200.0)
        .show(context, |ui| {
            // Player stats
            ui.colored_label(
                egui::Color32::from_rgb(64, 160, 43),
                egui::RichText::new(format!(
                    "YOU: {} neurons | {:.0}E | {:.0}B",
                    p1_neurons, p1_energy, p1_blocks
                ))
                .strong(),
            );
            ui.colored_label(
                egui::Color32::from_rgb(136, 57, 239),
                format!(
                    "AI:  {} neurons | {:.0}E | {:.0}B",
                    p2_neurons, p2_energy, p2_blocks
                ),
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
                    tool_button(ui, tool, GameTool::GlialCell, "Glial", "8B");
                    tool_button(ui, tool, GameTool::ExcitatoryNeuron, "Excitatory", "25B");
                    ui.end_row();
                    tool_button(ui, tool, GameTool::InhibitoryNeuron, "Inhibitory", "25B");
                    tool_button(ui, tool, GameTool::Axon, "Axon", "3B/seg");
                    ui.end_row();
                    tool_button(ui, tool, GameTool::MembraneSegment, "Membrane", "15B");
                    tool_button(ui, tool, GameTool::MotorCilia, "Motor", "20B");
                    ui.end_row();
                    tool_button(ui, tool, GameTool::SpikeGenerator, "Spike", "15B");
                    tool_button(ui, tool, GameTool::PoissonGenerator, "Poisson", "15B");
                    ui.end_row();
                    tool_button(ui, tool, GameTool::ActivitySensor, "Activity", "20B");
                    tool_button(ui, tool, GameTool::TouchSensor, "Touch", "20B");
                    ui.end_row();
                    tool_button(ui, tool, GameTool::ChemicalSensor, "Chemical", "20B");
                    ui.label("");
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
                    tool_button(ui, tool, GameTool::Select, "Select", "");
                    tool_button(ui, tool, GameTool::Erase, "Erase", "");
                    ui.end_row();
                });

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
) {
    let selected = *current_tool == tool;
    let text = if cost.is_empty() {
        egui::RichText::new(label).small()
    } else {
        egui::RichText::new(format!("{}\n{}", label, cost)).small()
    };
    let btn = egui::Button::new(text)
        .min_size(egui::vec2(82.0, 40.0))
        .selected(selected);
    if ui.add(btn).clicked() {
        *current_tool = tool;
    }
}
