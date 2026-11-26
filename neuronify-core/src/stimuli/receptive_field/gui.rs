use std::ops::RangeInclusive;

use egui::{Rect, Ui, Vec2};
use hecs::{Entity, World};
use neuronify_engine::units::Hertz;

use super::{ReceptiveField, ReceptiveFieldCell};
use super::{FIELD, FIELD_ANGLE, FIELD_SIDEWAYS, FIELD_UP_DOWN, OFF_CENTER, SENSITIVITY};
use crate::gui::icons::{linear, Glyph};
use crate::gui::inspector::{choice, hertz, logarithmic_parameter, parameter};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::rendering::colors::{blue, rate_color};
use crate::stimuli::gui::{degrees, HALF_TURN};

const OFFSETS: RangeInclusive<f64> = -0.5..=0.5;
const SENSITIVITIES: (RangeInclusive<Hertz>, Hertz) = (Hertz(0.0)..=Hertz(10_000.0), Hertz(10.0));

pub fn icon(glyph: &Glyph) {
    glyph.painter.rect_filled(
        Rect::from_center_size(glyph.at(16.0, 16.0), Vec2::splat(23.0 * glyph.scale)),
        5.0 * glyph.scale,
        linear(blue()),
    );
    glyph.painter.circle_filled(
        glyph.at(16.0, 16.0),
        7.0 * glyph.scale,
        linear(rate_color()),
    );
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut input) = world.get::<&mut ReceptiveFieldCell>(entity) else {
        return false;
    };
    let mut changed = false;
    heading(ui, chrome, "Receptive field", 0.0);
    let kinds = ReceptiveField::KINDS.map(|kind| kind.name());
    let mut kind = ReceptiveField::KINDS
        .iter()
        .position(|kind| kind.same_kind(input.field))
        .unwrap_or_default();
    let options: Vec<(usize, &str)> = kinds.iter().copied().enumerate().collect();
    if choice(ui, chrome, FIELD, &mut kind, &options) {
        input.field = ReceptiveField::KINDS[kind];
        changed = true;
    }
    let offset = |value: f64| format!("{:.1}", value * 10.0).replace('-', "\u{2212}");
    match &mut input.field {
        ReceptiveField::Orientation { angle, x, y } => {
            changed |= parameter(ui, chrome, FIELD_ANGLE, angle, HALF_TURN, degrees);
            changed |= parameter(ui, chrome, FIELD_SIDEWAYS, x, OFFSETS, offset);
            changed |= parameter(ui, chrome, FIELD_UP_DOWN, y, OFFSETS, offset);
        }
        ReceptiveField::CenterSurround { off_center, x, y } => {
            changed |= choice(
                ui,
                chrome,
                OFF_CENTER,
                off_center,
                &[(true, "On"), (false, "Off")],
            );
            changed |= parameter(ui, chrome, FIELD_SIDEWAYS, x, OFFSETS, offset);
            changed |= parameter(ui, chrome, FIELD_UP_DOWN, y, OFFSETS, offset);
        }
        ReceptiveField::Edge { angle } => {
            let quarters = [0.0, 0.5, 1.0, 1.5].map(|turn| turn * std::f64::consts::PI);
            let options = quarters.map(|quarter| (quarter, degrees(quarter)));
            let options: Vec<(f64, &str)> = options
                .iter()
                .map(|(quarter, name)| (*quarter, name.as_str()))
                .collect();
            changed |= choice(ui, chrome, FIELD_ANGLE, angle, &options);
        }
    }
    changed |= logarithmic_parameter(
        ui,
        chrome,
        SENSITIVITY,
        &mut input.sensitivity,
        SENSITIVITIES,
        hertz,
    );
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_stay_within_what_a_save_file_may_hold() {
        let offsets = super::super::OFFSETS;
        let sensitivities = super::super::SENSITIVITIES;
        assert!(offsets.contains(OFFSETS.start()) && offsets.contains(OFFSETS.end()));
        assert!(sensitivities.contains(SENSITIVITIES.0.start()));
        assert!(sensitivities.contains(SENSITIVITIES.0.end()));
    }
}
