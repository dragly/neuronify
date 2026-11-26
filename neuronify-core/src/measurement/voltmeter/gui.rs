use std::ops::RangeInclusive;

use egui::{Shape, Ui};
use hecs::{Entity, World};
use neuronify_engine::units::Millivolts;

use super::{VoltageSeries, VoltmeterRange};
use super::{HIGHEST_VOLTAGE, LOWEST_VOLTAGE};
use crate::gui::icons::Glyph;
use crate::gui::inspector::{millivolts, parameter};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::measurement::frame::measured;

const PLOTTED_VOLTAGES: RangeInclusive<Millivolts> = Millivolts(-250.0)..=Millivolts(250.0);
const PLOTTED_VOLTAGE_GAP: Millivolts = Millivolts(1.0);

pub fn icon(glyph: &Glyph) {
    glyph.frame();
    glyph.painter.add(Shape::line(
        [
            (7.0, 19.0),
            (11.0, 19.0),
            (13.0, 11.0),
            (15.0, 22.5),
            (17.0, 16.0),
            (25.0, 16.0),
        ]
        .map(|(x, y)| glyph.at(x, y))
        .to_vec(),
        glyph.stroke(1.8, glyph.chrome.accent),
    ));
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut range) = world.get::<&mut VoltmeterRange>(entity) else {
        return false;
    };
    heading(ui, chrome, "Parameters", 0.0);
    let shown = |voltage: Millivolts| format!("{:.0} mV", voltage.0).replace('-', "\u{2212}");
    let highest = parameter(
        ui,
        chrome,
        HIGHEST_VOLTAGE,
        &mut range.maximum,
        PLOTTED_VOLTAGES,
        shown,
    );
    if highest {
        range.maximum = range.maximum.max(range.minimum + PLOTTED_VOLTAGE_GAP);
    }
    let lowest = parameter(
        ui,
        chrome,
        LOWEST_VOLTAGE,
        &mut range.minimum,
        PLOTTED_VOLTAGES,
        shown,
    );
    if lowest {
        range.minimum = range.minimum.min(range.maximum - PLOTTED_VOLTAGE_GAP);
    }
    highest || lowest
}

pub fn readout(world: &World, entity: Entity) -> Option<(&'static str, String)> {
    let (wire, _) = *measured(world, entity).first()?;
    let series = world.get::<&VoltageSeries>(wire).ok()?;
    Some(("Voltage", millivolts(series.measurements.last()?.voltage)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_stay_within_what_a_save_file_may_hold() {
        let saved = super::super::PLOTTED_VOLTAGES;
        assert!(saved.contains(PLOTTED_VOLTAGES.start()) && saved.contains(PLOTTED_VOLTAGES.end()));
    }
}
