use std::ops::RangeInclusive;

use egui::{Shape, Ui};
use hecs::{Entity, World};
use neuronify_engine::units::Nanoamperes;

use super::CURRENT;
use crate::gui::icons::{linear, Glyph, CREAM};
use crate::gui::inspector::{logarithmic_parameter, nanoamperes};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::rendering::colors::yellow;
use crate::stimuli::CurrentClamp;

const CURRENTS: (RangeInclusive<Nanoamperes>, Nanoamperes) =
    (Nanoamperes(0.0)..=Nanoamperes(30.0), Nanoamperes(0.01));

pub fn icon(glyph: &Glyph) {
    glyph
        .painter
        .circle_filled(glyph.at(16.0, 16.0), 11.5 * glyph.scale, linear(yellow()));
    let arrow = glyph.stroke(2.4, CREAM);
    glyph
        .painter
        .line_segment([glyph.at(16.0, 22.5), glyph.at(16.0, 9.8)], arrow);
    glyph.painter.add(Shape::line(
        vec![
            glyph.at(11.8, 14.0),
            glyph.at(16.0, 9.8),
            glyph.at(20.2, 14.0),
        ],
        arrow,
    ));
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut clamp) = world.get::<&mut CurrentClamp>(entity) else {
        return false;
    };
    heading(ui, chrome, "Parameters", 0.0);
    logarithmic_parameter(
        ui,
        chrome,
        CURRENT,
        &mut clamp.current_output,
        CURRENTS,
        nanoamperes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_stays_within_what_a_save_file_may_hold() {
        let saved = crate::stimuli::CURRENTS;
        assert!(saved.contains(CURRENTS.0.start()) && saved.contains(CURRENTS.0.end()));
    }
}
