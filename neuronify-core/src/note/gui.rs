use egui::{Id, Margin, Rect, StrokeKind, Ui};
use hecs::{Entity, World};

use super::{Annotation, Link, NAME};
use crate::gui::icons::Glyph;
use crate::gui::style::Chrome;
use crate::gui::text_field::{text_field, Lines};
use crate::gui::widgets::heading;

const TEXT: &str = "inspector_note";

pub fn icon(glyph: &Glyph) {
    glyph.painter.rect(
        Rect::from_min_max(glyph.at(6.0, 6.0), glyph.at(26.0, 26.0)),
        3.0 * glyph.scale,
        glyph.chrome.panel,
        glyph.stroke(1.5, glyph.chrome.border),
        StrokeKind::Inside,
    );
    for (y, end) in [(12.0, 22.0), (16.0, 22.0), (20.0, 17.0)] {
        glyph.painter.line_segment(
            [glyph.at(10.0, y), glyph.at(end, y)],
            glyph.stroke(1.6, glyph.chrome.subtext),
        );
    }
}

pub fn title(world: &World, entity: Entity) -> Option<&'static str> {
    if world.satisfies::<&Link>(entity) {
        Some("Link")
    } else if world.satisfies::<&Annotation>(entity) {
        Some(NAME)
    } else {
        None
    }
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut annotation) = world.get::<&mut Annotation>(entity) else {
        return false;
    };
    heading(ui, chrome, "Text", 0.0);
    let changed = text_field(
        ui,
        Id::new(TEXT),
        &mut annotation.text,
        Lines::Many,
        |edit| {
            edit.desired_rows(4)
                .desired_width(f32::INFINITY)
                .margin(Margin::same(8))
        },
    );
    ui.add_space(8.0);
    changed
}
