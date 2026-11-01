use egui::{vec2, Align, Color32, Context, Id, Layout, Modal, Response, RichText, Ui};

use super::{Colors, CARD_RADIUS};
use crate::library::Library;

const DIALOG_WIDTH: f32 = 300.0;
const DIALOG_PADDING: i8 = 20;
const BUTTON_HEIGHT: f32 = 36.0;

struct Renaming {
    id: String,
    name: String,
}

struct Deleting {
    id: String,
    name: String,
}

#[derive(Default)]
pub struct Dialogs {
    renaming: Option<Renaming>,
    deleting: Option<Deleting>,
}

impl Dialogs {
    pub fn is_open(&self) -> bool {
        self.renaming.is_some() || self.deleting.is_some()
    }

    pub fn rename(&mut self, id: &str, name: &str) {
        self.renaming = Some(Renaming {
            id: id.to_owned(),
            name: name.to_owned(),
        });
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
    }

    #[cfg(target_arch = "wasm32")]
    fn show_rename(&mut self, _context: &Context, _colors: &Colors, library: &mut Library) {
        let Some(renaming) = self.renaming.take() else {
            return;
        };
        if let Some(name) = crate::web::prompt("Name", &renaming.name) {
            library.rename(&renaming.id, &name);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn show_rename(&mut self, context: &Context, colors: &Colors, library: &mut Library) {
        let Some(renaming) = &mut self.renaming else {
            return;
        };
        let mut confirmed = false;
        let mut cancelled = false;
        let modal = dialog(context, "rename", colors, |ui| {
            title(ui, colors, "Rename");
            let field = ui.add(
                egui::TextEdit::singleline(&mut renaming.name)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(15.0)),
            );
            field.request_focus();
            confirmed = ui.input(|input| input.key_pressed(egui::Key::Enter));
            ui.add_space(16.0);
            buttons(ui, |ui| {
                confirmed |= button(ui, "Rename", colors.background, colors.accent).clicked();
                cancelled = button(ui, "Cancel", colors.text, colors.background).clicked();
            });
        });
        if confirmed {
            library.rename(&renaming.id, &renaming.name);
        }
        if confirmed || cancelled || modal.should_close() {
            self.renaming = None;
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
                confirmed = button(ui, "Delete", colors.background, colors.danger).clicked();
                cancelled = button(ui, "Cancel", colors.text, colors.background).clicked();
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
    Modal::new(Id::new(id))
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

fn buttons(ui: &mut Ui, add_buttons: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), BUTTON_HEIGHT),
        Layout::right_to_left(Align::Center),
        add_buttons,
    );
}

fn button(ui: &mut Ui, label: &str, text: Color32, fill: Color32) -> Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(15.0).color(text))
            .fill(fill)
            .corner_radius(CARD_RADIUS)
            .min_size(vec2(88.0, BUTTON_HEIGHT)),
    )
}
