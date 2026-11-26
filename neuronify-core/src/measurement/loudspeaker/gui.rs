use std::ops::RangeInclusive;

use egui::{Shape, Ui};
use hecs::{Entity, World};

use super::{Loudspeaker, Sound};
use super::{MUTE, SOUND, VOLUME};
use crate::gui::icons::{linear, Glyph, CREAM};
use crate::gui::inspector::{choice, parameter};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::rendering::colors::speaker_color;
use crate::rendering::shapes::{speaker_outline, speaker_waves};

const VOLUMES: RangeInclusive<f64> = 0.0..=1.0;

pub fn icon(glyph: &Glyph) {
    glyph.painter.circle_filled(
        glyph.at(16.0, 16.0),
        11.5 * glyph.scale,
        linear(speaker_color()),
    );
    let cone = speaker_outline()
        .iter()
        .map(|&(x, y)| glyph.at(16.0 + 9.0 * x, 16.0 - 9.0 * y))
        .collect();
    glyph
        .painter
        .add(Shape::closed_line(cone, glyph.stroke(1.6, CREAM)));
    for wave in speaker_waves() {
        let points = wave
            .iter()
            .map(|&(x, y)| glyph.at(16.0 + 9.0 * x, 16.0 - 9.0 * y))
            .collect();
        glyph
            .painter
            .add(Shape::line(points, glyph.stroke(1.6, CREAM)));
    }
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut speaker) = world.get::<&mut Loudspeaker>(entity) else {
        return false;
    };
    heading(ui, chrome, "Parameters", 0.0);
    let sounds = Sound::ALL.map(|sound| (sound, sound.name()));
    let mut changed = choice(ui, chrome, SOUND, &mut speaker.sound, &sounds);
    changed |= parameter(ui, chrome, VOLUME, &mut speaker.volume, VOLUMES, |volume| {
        format!("{:.0} %", volume * 100.0)
    });
    changed |= choice(
        ui,
        chrome,
        MUTE,
        &mut speaker.muted,
        &[(true, "On"), (false, "Off")],
    );
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_stays_within_what_a_save_file_may_hold() {
        let saved = super::super::VOLUMES;
        assert!(saved.contains(VOLUMES.start()) && saved.contains(VOLUMES.end()));
    }
}
