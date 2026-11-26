use std::ops::RangeInclusive;

use egui::{Shape, Ui};
use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Nanoamperes};

use super::AlternatingCurrent;
use super::{AMPLITUDE, FREQUENCY};
use crate::gui::icons::{linear, sine_wave, Glyph, CREAM};
use crate::gui::inspector::{hertz, logarithmic_parameter, picoamperes};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::rendering::colors::yellow;

const AMPLITUDES: (RangeInclusive<Nanoamperes>, Nanoamperes) =
    (Nanoamperes(0.0)..=Nanoamperes(1.0), Nanoamperes(0.01));
const ALTERNATING_FREQUENCIES: (RangeInclusive<Hertz>, Hertz) =
    (Hertz(0.0)..=Hertz(200.0), Hertz(1.0));

pub fn icon(glyph: &Glyph) {
    glyph
        .painter
        .circle_filled(glyph.at(16.0, 16.0), 11.5 * glyph.scale, linear(yellow()));
    let wave = sine_wave()
        .map(|(x, y)| glyph.at(16.0 + 7.5 * x, 16.0 - 5.0 * y))
        .collect();
    glyph
        .painter
        .add(Shape::line(wave, glyph.stroke(2.4, CREAM)));
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut alternating) = world.get::<&mut AlternatingCurrent>(entity) else {
        return false;
    };
    heading(ui, chrome, "Parameters", 0.0);
    let amplitude = logarithmic_parameter(
        ui,
        chrome,
        AMPLITUDE,
        &mut alternating.amplitude,
        AMPLITUDES,
        picoamperes,
    );
    let frequency = logarithmic_parameter(
        ui,
        chrome,
        FREQUENCY,
        &mut alternating.frequency,
        ALTERNATING_FREQUENCIES,
        hertz,
    );
    amplitude || frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_stay_within_what_a_save_file_may_hold() {
        let amplitudes = super::super::AMPLITUDES;
        let frequencies = super::super::ALTERNATING_FREQUENCIES;
        assert!(amplitudes.contains(AMPLITUDES.0.start()));
        assert!(amplitudes.contains(AMPLITUDES.0.end()));
        assert!(frequencies.contains(ALTERNATING_FREQUENCIES.0.start()));
        assert!(frequencies.contains(ALTERNATING_FREQUENCIES.0.end()));
    }
}
