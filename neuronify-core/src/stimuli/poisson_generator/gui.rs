use egui::Ui;
use hecs::{Entity, World};

use super::PoissonGenerator;
use super::RATE;
use crate::gui::icons::Glyph;
use crate::gui::inspector::{hertz, logarithmic_parameter, type_switch};
use crate::gui::style::Chrome;
use crate::gui::widgets::heading;
use crate::stimuli::gui::{generator_icon, SPIKE_RATES};

pub fn icon(glyph: &Glyph) {
    generator_icon(glyph, true);
}

pub fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    heading(ui, chrome, "Parameters", 0.0);
    let mut changed = type_switch(ui, chrome, world, entity);
    if let Ok(mut generator) = world.get::<&mut PoissonGenerator>(entity) {
        changed |= logarithmic_parameter(ui, chrome, RATE, &mut generator.rate, SPIKE_RATES, hertz);
    }
    changed
}
