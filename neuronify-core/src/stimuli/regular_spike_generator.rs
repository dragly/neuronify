pub mod gui;

use hecs::World;
use neuronify_engine::units::Hertz;
use serde::{Deserialize, Serialize};

use crate::serialization::bounds::within;
use crate::settings::Setting;
use crate::stimuli::{About, SPIKE_RATES};

pub const ABOUT: About = About {
    name: "Regular spikes",
    description: "Sends spikes at a steady beat.",
    size: None,
};

pub const FREQUENCY: Setting = Setting {
    label: "Frequency",
    description: "How many spikes it sends each second.",
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegularSpikeGenerator {
    pub frequency: Hertz,
}

impl Default for RegularSpikeGenerator {
    fn default() -> Self {
        Self {
            frequency: Hertz(20.0),
        }
    }
}

pub fn bound(world: &mut World) {
    for generator in world.query_mut::<&mut RegularSpikeGenerator>() {
        let default = RegularSpikeGenerator::default();
        within(&mut generator.frequency, SPIKE_RATES, default.frequency);
    }
}
