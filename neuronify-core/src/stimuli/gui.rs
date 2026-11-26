use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use egui::{Shape, Ui};
use hecs::{Entity, World};
use neuronify_engine::units::Hertz;

use crate::gui::icons::{arc, linear, sphere, Glyph};
use crate::gui::style::Chrome;
use crate::rendering::colors::orange;
use crate::stimuli::{
    alternating_current, current_source, poisson_generator, receptive_field,
    regular_spike_generator, touch_sensor, visual_input, Stimulus,
};

pub const SPIKE_RATES: (RangeInclusive<Hertz>, Hertz) = (Hertz(0.0)..=Hertz(1000.0), Hertz(1.0));
pub const HALF_TURN: RangeInclusive<f64> = 0.0..=std::f64::consts::PI;

impl Stimulus {
    pub fn icon(self, glyph: &Glyph) {
        match self {
            Stimulus::CurrentSource => current_source::gui::icon(glyph),
            Stimulus::AlternatingCurrentSource => alternating_current::gui::icon(glyph),
            Stimulus::TouchSensor => touch_sensor::gui::icon(glyph),
            Stimulus::RegularSpikeGenerator => regular_spike_generator::gui::icon(glyph),
            Stimulus::PoissonGenerator => poisson_generator::gui::icon(glyph),
            Stimulus::VisualInput => visual_input::gui::icon(glyph),
            Stimulus::ReceptiveField => receptive_field::gui::icon(glyph),
        }
    }

    pub fn parameters(self, ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
        match self {
            Stimulus::CurrentSource => current_source::gui::parameters(ui, chrome, world, entity),
            Stimulus::AlternatingCurrentSource => {
                alternating_current::gui::parameters(ui, chrome, world, entity)
            }
            Stimulus::RegularSpikeGenerator => {
                regular_spike_generator::gui::parameters(ui, chrome, world, entity)
            }
            Stimulus::PoissonGenerator => {
                poisson_generator::gui::parameters(ui, chrome, world, entity)
            }
            Stimulus::VisualInput => visual_input::gui::parameters(ui, chrome, world, entity),
            Stimulus::ReceptiveField => receptive_field::gui::parameters(ui, chrome, world, entity),
            Stimulus::TouchSensor => false,
        }
    }
}

pub fn generator_icon(glyph: &Glyph, dashed: bool) {
    let generator = linear(orange());
    sphere(
        glyph.painter,
        glyph.at(16.0, 16.0),
        7.0 * glyph.scale,
        generator,
    );
    let ring = arc(glyph.at(16.0, 16.0), 12.0 * glyph.scale, 0.0, TAU);
    if dashed {
        glyph.painter.extend(Shape::dashed_line(
            &ring,
            glyph.stroke(2.6, generator),
            3.4 * glyph.scale,
            2.6 * glyph.scale,
        ));
    } else {
        let closed = ring[..ring.len() - 1].to_vec();
        glyph
            .painter
            .add(Shape::closed_line(closed, glyph.stroke(2.6, generator)));
    }
}

pub fn degrees(radians: f64) -> String {
    format!("{:.0}\u{b0}", radians.to_degrees())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::bounds::ANGLES;

    #[test]
    fn the_shared_sliders_stay_within_what_a_save_file_may_hold() {
        let inside = |slider: &RangeInclusive<Hertz>, bound: &RangeInclusive<Hertz>| {
            bound.contains(slider.start()) && bound.contains(slider.end())
        };
        assert!(inside(&SPIKE_RATES.0, &crate::stimuli::SPIKE_RATES));
        assert!(ANGLES.contains(HALF_TURN.start()) && ANGLES.contains(HALF_TURN.end()));
    }
}
