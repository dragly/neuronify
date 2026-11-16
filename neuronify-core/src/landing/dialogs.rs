use egui::{vec2, Align, Align2, Color32, Context, FontId, Id, Layout, Modal, RichText, Ui};

use super::{needs_newer_neuronify, Colors, CARD_RADIUS};
use crate::gui::text_field::{covered_by_keyboard, text_field, Lines};
use crate::library::Library;

const DIALOG_WIDTH: f32 = 300.0;
const DIALOG_PADDING: i8 = 20;
const BUTTON_HEIGHT: f32 = 36.0;
const DESCRIPTION_ROWS: usize = 3;

struct Renaming {
    id: String,
    name: String,
    description: String,
    focused: bool,
}

struct Deleting {
    id: String,
    name: String,
}

#[derive(Default)]
pub struct Dialogs {
    renaming: Option<Renaming>,
    deleting: Option<Deleting>,
    too_new: Option<String>,
}

impl Dialogs {
    pub fn is_open(&self) -> bool {
        self.renaming.is_some() || self.deleting.is_some() || self.too_new.is_some()
    }

    pub fn rename(&mut self, id: &str, name: &str, description: &str) {
        self.renaming = Some(Renaming {
            id: id.to_owned(),
            name: name.to_owned(),
            description: description.to_owned(),
            focused: false,
        });
    }

    pub fn too_new(&mut self, app: &str) {
        self.too_new = Some(app.to_owned());
    }

    pub fn delete(&mut self, id: &str, name: &str) {
        self.deleting = Some(Deleting {
            id: id.to_owned(),
            name: name.to_owned(),
        });
    }

    pub fn show(&mut self, context: &Context, colors: &Colors, library: &mut Library) {
        self.show_rename(context, colors, library);
        self.show_delete(context, colors, library);
        self.show_too_new(context, colors);
    }

    fn show_rename(&mut self, context: &Context, colors: &Colors, library: &mut Library) {
        let Some(renaming) = &mut self.renaming else {
            return;
        };
        let name_field = Id::new("rename name");
        let description_field = Id::new("rename description");
        let mut confirmed = false;
        let mut cancelled = false;
        let modal = dialog(context, "rename", colors, |ui| {
            title(ui, colors, "Rename");
            field_label(ui, colors, "Name");
            text_field(ui, name_field, &mut renaming.name, Lines::One, |edit| {
                edit.desired_width(f32::INFINITY)
                    .font(FontId::proportional(15.0))
                    .background_color(colors.background)
            });
            ui.add_space(12.0);
            field_label(ui, colors, "Description");
            text_field(
                ui,
                description_field,
                &mut renaming.description,
                Lines::Many,
                |edit| {
                    edit.desired_width(f32::INFINITY)
                        .desired_rows(DESCRIPTION_ROWS)
                        .font(FontId::proportional(15.0))
                        .background_color(colors.background)
                },
            );
            if !renaming.focused {
                ui.memory_mut(|memory| memory.request_focus(name_field));
                renaming.focused = true;
            }
            let describing = ui.memory(|memory| memory.has_focus(description_field));
            confirmed = !describing && ui.input(|input| input.key_pressed(egui::Key::Enter));
            ui.add_space(16.0);
            buttons(ui, |ui| {
                confirmed |= button(ui, "Save", colors.background, colors.accent);
                cancelled = button(ui, "Cancel", colors.text, colors.background);
            });
        });
        if confirmed {
            library.edit_details(&renaming.id, &renaming.name, &renaming.description);
        }
        if confirmed || cancelled || modal.should_close() {
            self.renaming = None;
        }
    }

    fn show_too_new(&mut self, context: &Context, colors: &Colors) {
        let Some(app) = &self.too_new else {
            return;
        };
        let mut closed = false;
        let modal = dialog(context, "too new", colors, |ui| {
            title(ui, colors, &needs_newer_neuronify(app));
            ui.label(
                RichText::new(too_new_message(app))
                    .size(14.0)
                    .color(colors.subtext),
            );
            ui.add_space(16.0);
            buttons(ui, |ui| {
                closed = button(ui, "OK", colors.background, colors.accent);
            });
        });
        if closed || modal.should_close() {
            self.too_new = None;
        }
    }

    fn show_delete(&mut self, context: &Context, colors: &Colors, library: &mut Library) {
        let Some(deleting) = &self.deleting else {
            return;
        };
        let mut confirmed = false;
        let mut cancelled = false;
        let modal = dialog(context, "delete", colors, |ui| {
            title(ui, colors, &format!("Delete “{}”?", deleting.name));
            ui.label(
                RichText::new("This cannot be undone.")
                    .size(14.0)
                    .color(colors.subtext),
            );
            ui.add_space(16.0);
            buttons(ui, |ui| {
                confirmed = button(ui, "Delete", colors.background, colors.danger);
                cancelled = button(ui, "Cancel", colors.text, colors.background);
            });
        });
        if confirmed {
            library.delete(&deleting.id);
        }
        if confirmed || cancelled || modal.should_close() {
            self.deleting = None;
        }
    }
}

fn dialog(
    context: &Context,
    id: &str,
    colors: &Colors,
    contents: impl FnOnce(&mut Ui),
) -> egui::ModalResponse<()> {
    let width = DIALOG_WIDTH.min(context.content_rect().width() - 2.0 * super::PAGE_MARGIN);
    let above_keyboard = vec2(0.0, -covered_by_keyboard() / 2.0);
    Modal::new(Id::new(id))
        .area(Modal::default_area(Id::new(id)).anchor(Align2::CENTER_CENTER, above_keyboard))
        .frame(
            egui::Frame::new()
                .fill(colors.surface)
                .corner_radius(CARD_RADIUS)
                .inner_margin(DIALOG_PADDING),
        )
        .show(context, |ui| {
            ui.set_width(width - 2.0 * DIALOG_PADDING as f32);
            contents(ui);
        })
}

fn title(ui: &mut Ui, colors: &Colors, text: &str) {
    ui.label(RichText::new(text).size(18.0).color(colors.text));
    ui.add_space(10.0);
}

fn field_label(ui: &mut Ui, colors: &Colors, text: &str) {
    ui.label(RichText::new(text).size(13.5).color(colors.subtext));
    ui.add_space(2.0);
}

fn too_new_message(app: &str) -> String {
    match app {
        "" => "This simulation was saved with a newer Neuronify than this one.".to_owned(),
        app => {
            format!("This simulation was saved with Neuronify {app}, which is newer than this one.")
        }
    }
}

fn buttons(ui: &mut Ui, add_buttons: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), BUTTON_HEIGHT),
        Layout::right_to_left(Align::Center),
        add_buttons,
    );
}

fn button(ui: &mut Ui, label: &str, text: Color32, fill: Color32) -> bool {
    ui.add(
        egui::Button::new(RichText::new(label).size(15.0).color(text))
            .fill(fill)
            .corner_radius(CARD_RADIUS)
            .min_size(vec2(88.0, BUTTON_HEIGHT)),
    )
    .clicked()
}
