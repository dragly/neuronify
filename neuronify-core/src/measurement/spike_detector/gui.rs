use std::ops::RangeInclusive;

use egui::Ui;
use hecs::{Entity, World};
use neuronify_engine::units::Milliseconds;

use super::SpikeDetector;
use crate::gui::icons::Glyph;
use crate::gui::inspector::{choice, milliseconds, parameter};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::measurement::frame::RowOrder;
use crate::settings::Setting;

const TIME_RANGES: RangeInclusive<Milliseconds> = Milliseconds(1.0)..=Milliseconds(1000.0);

pub fn icon(glyph: &Glyph) {
    glyph.frame();
    for (row, ticks) in [
        (11.5, [8.5, 13.5, 21.5]),
        (16.0, [10.5, 17.5, 23.5]),
        (20.5, [7.5, 15.0, 19.5]),
    ] {
        for tick in ticks {
            glyph.painter.line_segment(
                [glyph.at(tick, row - 1.8), glyph.at(tick, row + 1.8)],
                glyph.stroke(1.6, glyph.chrome.accent),
            );
        }
    }
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let mut changed = false;
    if let Ok(mut detector) = world.get::<&mut SpikeDetector>(entity) {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::TimeRange,
            &mut detector.time_range,
            TIME_RANGES,
            milliseconds,
        );
    }
    if let Ok(mut order) = world.get::<&mut RowOrder>(entity) {
        changed |= choice(
            ui,
            chrome,
            Setting::RowOrder,
            &mut *order,
            &[
                (RowOrder::Height, "Height"),
                (RowOrder::Connection, "Connection"),
            ],
        );
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_stays_within_what_a_save_file_may_hold() {
        let saved = super::super::TIME_RANGES;
        assert!(saved.contains(TIME_RANGES.start()) && saved.contains(TIME_RANGES.end()));
    }
}
