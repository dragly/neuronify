use std::ops::RangeInclusive;

use egui::{Pos2, Rect, Shape, Ui, Vec2};
use hecs::{Entity, World};
use neuronify_engine::units::Hertz;

use super::{Pattern, VisualInput};
use super::{PHASE, PICTURE, PICTURE_ANGLE, REVERSE, SPEED};
use crate::gui::icons::{linear, Glyph, Icon, CREAM, SINE_STEPS};
use crate::gui::inspector::{
    self, choice, description, logarithmic_parameter, parameter, row_button,
};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::rendering::colors::orange;
use crate::stimuli::gui::{degrees, HALF_TURN};

const PICTURE_SPEEDS: (RangeInclusive<Hertz>, Hertz) = (Hertz(0.0)..=Hertz(50.0), Hertz(0.1));
const PICTURE_PHASES: RangeInclusive<f64> = 0.0..=1.0;

pub fn icon(glyph: &Glyph) {
    glyph.painter.rect_filled(
        Rect::from_center_size(glyph.at(16.0, 16.0), Vec2::splat(23.0 * glyph.scale)),
        5.0 * glyph.scale,
        linear(orange()),
    );
    let lid = |sign: f32| {
        (0..=SINE_STEPS)
            .map(|step| {
                let x = 2.0 * step as f32 / SINE_STEPS as f32 - 1.0;
                glyph.at(16.0 + 8.5 * x, 16.0 + sign * 5.0 * (1.0 - x * x))
            })
            .collect::<Vec<Pos2>>()
    };
    glyph
        .painter
        .add(Shape::line(lid(1.0), glyph.stroke(1.6, CREAM)));
    glyph
        .painter
        .add(Shape::line(lid(-1.0), glyph.stroke(1.6, CREAM)));
    glyph
        .painter
        .circle_filled(glyph.at(16.0, 16.0), 2.8 * glyph.scale, CREAM);
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut input) = world.get::<&mut VisualInput>(entity) else {
        return false;
    };
    let mut changed = false;
    heading(ui, chrome, "Picture", 0.0);
    let patterns = Pattern::ALL.map(|pattern| (pattern, pattern.name()));
    changed |= choice(ui, chrome, PICTURE, &mut input.pattern, &patterns);
    if row_button(
        ui,
        chrome,
        inspector::REVERSE,
        Icon::Reverse,
        REVERSE.label,
        chrome.text,
    )
    .clicked()
    {
        input.reverse();
        changed = true;
    }
    description(ui, chrome, REVERSE.description);
    ui.add_space(8.0);
    changed |= parameter(
        ui,
        chrome,
        PICTURE_ANGLE,
        &mut input.pattern_angle,
        HALF_TURN,
        degrees,
    );
    let mut speed = input.speed.abs();
    if logarithmic_parameter(ui, chrome, SPEED, &mut speed, PICTURE_SPEEDS, |speed| {
        format!("{:.1} Hz", speed.0)
    }) {
        input.speed = Hertz(speed.0.copysign(input.speed.0));
        changed = true;
    }
    changed |= parameter(
        ui,
        chrome,
        PHASE,
        &mut input.phase,
        PICTURE_PHASES,
        |phase| format!("{:.0}\u{b0}", phase * 360.0),
    );
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_stay_within_what_a_save_file_may_hold() {
        let speeds = super::super::PICTURE_SPEEDS;
        let phases = super::super::PICTURE_PHASES;
        assert!(
            speeds.contains(PICTURE_SPEEDS.0.start()) && speeds.contains(PICTURE_SPEEDS.0.end())
        );
        assert!(phases.contains(PICTURE_PHASES.start()) && phases.contains(PICTURE_PHASES.end()));
    }
}
