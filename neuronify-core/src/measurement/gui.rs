use egui::Ui;
use hecs::{Entity, World};

use crate::gui::icons::Glyph;
use crate::gui::style::Chrome;
use crate::measurement::{firing_rate, loudspeaker, spike_detector, voltmeter, Meter};

impl Meter {
    pub fn icon(self, glyph: &Glyph) {
        match self {
            Meter::Voltmeter => voltmeter::gui::icon(glyph),
            Meter::SpikeDetector => spike_detector::gui::icon(glyph),
            Meter::RatePlot => firing_rate::gui::icon(glyph),
            Meter::Loudspeaker => loudspeaker::gui::icon(glyph),
        }
    }

    pub fn parameters(self, ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
        match self {
            Meter::Voltmeter => voltmeter::gui::parameters(ui, chrome, world, entity),
            Meter::SpikeDetector => spike_detector::gui::parameters(ui, chrome, world, entity),
            Meter::RatePlot => firing_rate::gui::parameters(ui, chrome, world, entity),
            Meter::Loudspeaker => loudspeaker::gui::parameters(ui, chrome, world, entity),
        }
    }

    pub fn readout(self, world: &World, entity: Entity) -> Option<(&'static str, String)> {
        match self {
            Meter::Voltmeter => voltmeter::gui::readout(world, entity),
            Meter::RatePlot => firing_rate::gui::readout(world, entity),
            Meter::SpikeDetector | Meter::Loudspeaker => None,
        }
    }
}
