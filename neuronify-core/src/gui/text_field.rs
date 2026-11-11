use egui::{Id, TextEdit, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lines {
    One,
    Many,
}

pub fn text_field(
    ui: &mut Ui,
    id: Id,
    text: &mut String,
    lines: Lines,
    style: impl FnOnce(TextEdit<'_>) -> TextEdit<'_>,
) -> bool {
    #[cfg(target_arch = "wasm32")]
    let typed = crate::web::take_typed(ui.ctx(), id, text, lines == Lines::Many);
    #[cfg(not(target_arch = "wasm32"))]
    let typed = false;
    let edit = match lines {
        Lines::One => TextEdit::singleline(text),
        Lines::Many => TextEdit::multiline(text),
    };
    let output = style(edit.id(id)).show(ui);
    #[cfg(target_arch = "wasm32")]
    crate::web::follow_text_field(ui, &output, text, lines == Lines::Many);
    typed || output.response.changed()
}

#[cfg(target_arch = "wasm32")]
pub use crate::web::covered_by_keyboard;

#[cfg(not(target_arch = "wasm32"))]
pub fn covered_by_keyboard() -> f32 {
    0.0
}
