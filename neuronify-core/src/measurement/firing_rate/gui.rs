use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use egui::{Pos2, Shape, Ui};
use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Milliseconds};

use super::{RatePlot, RateSeries};
use crate::gui::icons::{Glyph, SINE_STEPS};
use crate::gui::inspector::{hertz, milliseconds, parameter};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::settings::Setting;

const RATE_WINDOWS: RangeInclusive<Milliseconds> = Milliseconds(10.0)..=Milliseconds(1000.0);
const PLOTTED_RATES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(2500.0);
const PLOTTED_RATE_GAP: Hertz = Hertz(1.0);

pub fn icon(glyph: &Glyph) {
    glyph.frame();
    let curve: Vec<Pos2> = (0..=SINE_STEPS)
        .map(|step| {
            let x = step as f32 / SINE_STEPS as f32;
            let rise = 1.0 / (1.0 + (-12.0 * (x - 0.45)).exp());
            glyph.at(
                7.0 + 18.0 * x,
                21.5 - 10.0 * rise + 2.0 * (x * TAU).sin() * rise,
            )
        })
        .collect();
    glyph
        .painter
        .add(Shape::line(curve, glyph.stroke(1.8, glyph.chrome.accent)));
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Ok(mut plot) = world.get::<&mut RatePlot>(entity) else {
        return false;
    };
    heading(ui, chrome, "Parameters", 0.0);
    let mut changed = parameter(
        ui,
        chrome,
        Setting::Window,
        &mut plot.window,
        RATE_WINDOWS,
        milliseconds,
    );
    changed |= parameter(
        ui,
        chrome,
        Setting::KernelWidth,
        &mut plot.kernel_width,
        RATE_WINDOWS,
        milliseconds,
    );
    let highest = parameter(
        ui,
        chrome,
        Setting::HighestRate,
        &mut plot.maximum,
        PLOTTED_RATES,
        hertz,
    );
    if highest {
        plot.maximum = plot.maximum.max(plot.minimum + PLOTTED_RATE_GAP);
    }
    let lowest = parameter(
        ui,
        chrome,
        Setting::LowestRate,
        &mut plot.minimum,
        PLOTTED_RATES,
        hertz,
    );
    if lowest {
        plot.minimum = plot.minimum.min(plot.maximum - PLOTTED_RATE_GAP);
    }
    changed || highest || lowest
}

pub fn readout(world: &World, entity: Entity) -> Option<(&'static str, String)> {
    let series = world.get::<&RateSeries>(entity).ok()?;
    let &(_, rate) = series.samples.back()?;
    Some(("Firing rate", format!("{:.1} Hz", rate.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_stay_within_what_a_save_file_may_hold() {
        let windows = super::super::RATE_WINDOWS;
        let rates = super::super::PLOTTED_RATES;
        assert!(windows.contains(RATE_WINDOWS.start()) && windows.contains(RATE_WINDOWS.end()));
        assert!(rates.contains(PLOTTED_RATES.start()) && rates.contains(PLOTTED_RATES.end()));
    }
}
