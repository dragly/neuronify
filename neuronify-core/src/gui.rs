use std::path::PathBuf;

use egui::Context;
use hecs::{Entity, World};

use crate::components::*;
use crate::editor::Editor;
use crate::examples::Example;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use crate::rendering::Theme;
use crate::tools::{ToolCategory, TOOL_CATEGORIES};

pub enum MenuChoice {
    Save(PathBuf),
    Open(PathBuf),
    Example(&'static Example),
}

pub fn load_error(context: &Context, error: &str) {
    egui::Window::new("Could not load file").show(context, |ui| {
        ui.label(error);
    });
}

pub fn corner_buttons(context: &Context, edit_enabled: &mut bool) -> bool {
    let mut examples_clicked = false;
    egui::Area::new("edit_button_area".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .show(context, |ui| {
            ui.horizontal(|ui| {
                examples_clicked = ui.button("Examples").clicked();
                ui.toggle_value(edit_enabled, "Edit");
            });
        });
    examples_clicked
}

#[cfg(not(target_arch = "wasm32"))]
pub fn menu_bar(context: &Context) -> Option<MenuChoice> {
    let mut choice = None;
    #[allow(deprecated)]
    egui::Panel::top("top_panel").show(context, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {
                    choice = rfd::FileDialog::new().save_file().map(MenuChoice::Save);
                }
                if ui.button("Open").clicked() {
                    choice = rfd::FileDialog::new().pick_file().map(MenuChoice::Open);
                }
            });
            ui.menu_button("Examples", |ui| {
                for category in crate::examples::EXAMPLE_CATEGORIES {
                    ui.menu_button(category.name, |ui| {
                        for example in category.examples {
                            if ui.button(example.name).clicked() {
                                choice = Some(MenuChoice::Example(example));
                                ui.close();
                            }
                        }
                    });
                }
            });
        });
    });
    choice
}

pub fn elements_window(context: &Context, editor: &mut Editor) {
    egui::Window::new("Elements").show(context, |ui| {
        for category in &TOOL_CATEGORIES {
            ui.collapsing(category.label(), |ui| {
                for (tool, label) in category.tools() {
                    ui.selectable_value(&mut editor.tool, tool, label);
                }
                if *category == ToolCategory::Neurons {
                    ui.checkbox(&mut editor.dendrites, "With dendrites");
                }
            });
        }
    });
}

pub fn settings_window(context: &Context, fps: f64, iterations: &mut u32, theme: &mut Theme) {
    egui::Window::new("Settings").show(context, |ui| {
        ui.label(format!("FPS: {fps:.0}"));
        ui.label("Simulation speed");
        ui.add(egui::Slider::new(iterations, 1..=20));
        ui.horizontal(|ui| {
            ui.label("Theme");
            ui.selectable_value(theme, Theme::Dark, "Dark");
            ui.selectable_value(theme, Theme::Light, "Light");
        });
    });
}

pub fn selection_window(context: &Context, world: &World, entity: Entity) {
    egui::Window::new("Selection").show(context, |ui| {
        if let Ok(compartment) = world.get::<&Compartment>(entity) {
            ui.collapsing("Compartment", |ui| {
                egui::Grid::new("compartment_state").show(ui, |ui| {
                    ui.label("Voltage:");
                    ui.label(format!("{:.2} mV", compartment.voltage));
                    ui.end_row();
                    ui.label("m:");
                    ui.label(format!("{:.2}", compartment.m));
                    ui.end_row();
                    ui.label("n:");
                    ui.label(format!("{:.2}", compartment.n));
                    ui.end_row();
                    ui.label("h:");
                    ui.label(format!("{:.2}", compartment.h));
                    ui.end_row();
                });
            });
        }
        if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
            let is_inhibitory = neuron_type(world, entity) == NeuronType::Inhibitory;
            let label = if is_inhibitory {
                "LIF Neuron (Inhibitory)"
            } else {
                "LIF Neuron (Excitatory)"
            };
            ui.collapsing(label, |ui| {
                egui::Grid::new("neuron_settings").show(ui, |ui| {
                    ui.label("Threshold:");
                    ui.add(egui::Slider::new(&mut neuron.threshold, -0.08..=-0.03).suffix(" V"));
                    ui.end_row();
                    ui.label("Resting potential:");
                    ui.add(
                        egui::Slider::new(&mut neuron.resting_potential, -0.09..=-0.05)
                            .suffix(" V"),
                    );
                    ui.end_row();
                    ui.label("Capacitance:");
                    ui.add(egui::Slider::new(&mut neuron.capacitance, 1e-11..=1e-9).suffix(" F"));
                    ui.end_row();
                });
            });
        }
        if let Ok(dynamics) = world.get::<&LeakyDynamics>(entity) {
            ui.collapsing("Dynamics", |ui| {
                egui::Grid::new("neuron_dynamics").show(ui, |ui| {
                    ui.label("Voltage:");
                    ui.label(format!("{:.4} V", dynamics.voltage));
                    ui.end_row();
                    ui.label("Fired:");
                    ui.label(format!("{}", dynamics.fired));
                    ui.end_row();
                });
            });
        }
        if let Ok(mut clamp) = world.get::<&mut CurrentClamp>(entity) {
            ui.collapsing("Current Source", |ui| {
                egui::Grid::new("clamp_settings").show(ui, |ui| {
                    ui.label("Current:");
                    ui.add(egui::Slider::new(&mut clamp.current_output, 0.0..=1e-8).suffix(" A"));
                    ui.end_row();
                });
            });
        }
        if let Ok(mut gen) = world.get::<&mut RegularSpikeGenerator>(entity) {
            ui.collapsing("Spike Generator", |ui| {
                egui::Grid::new("spike_gen_settings").show(ui, |ui| {
                    ui.label("Frequency:");
                    ui.add(egui::Slider::new(&mut gen.frequency, 1.0..=200.0).suffix(" Hz"));
                    ui.end_row();
                });
            });
        }
        if let Ok(mut gen) = world.get::<&mut PoissonGenerator>(entity) {
            ui.collapsing("Poisson Generator", |ui| {
                egui::Grid::new("poisson_gen_settings").show(ui, |ui| {
                    ui.label("Rate:");
                    ui.add(egui::Slider::new(&mut gen.rate, 1.0..=200.0).suffix(" Hz"));
                    ui.end_row();
                });
            });
        }
        if world.get::<&Voltmeter>(entity).is_ok() {
            ui.collapsing("Voltmeter", |ui| {
                if let Ok(mut size) = world.get::<&mut VoltmeterSize>(entity) {
                    egui::Grid::new("voltmeter_size_settings").show(ui, |ui| {
                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut size.width, 2.0..=20.0));
                        ui.end_row();
                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut size.height, 1.0..=10.0));
                        ui.end_row();
                    });
                }
                if let Ok(series) = world.get::<&VoltageSeries>(entity) {
                    if let Some(last) = series.measurements.last() {
                        ui.label(format!("Voltage: {:.2} mV", last.voltage));
                    }
                }
            });
        }
        if world.get::<&Annotation>(entity).is_ok() {
            ui.collapsing("Note", |ui| {
                if let Ok(mut annotation) = world.get::<&mut Annotation>(entity) {
                    ui.label("Text:");
                    ui.add(
                        egui::TextEdit::multiline(&mut annotation.text)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                }
                if let Ok(mut size) = world.get::<&mut NoteSize>(entity) {
                    egui::Grid::new("note_size_settings").show(ui, |ui| {
                        ui.label("Width:");
                        ui.add(egui::Slider::new(&mut size.width, 2.0..=30.0));
                        ui.end_row();
                        ui.label("Height:");
                        ui.add(egui::Slider::new(&mut size.height, 1.0..=20.0));
                        ui.end_row();
                    });
                }
            });
        }
    });
}
