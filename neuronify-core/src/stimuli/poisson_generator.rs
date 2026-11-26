pub mod gui;

use hecs::World;
use neuronify_engine::units::Hertz;
use serde::{Deserialize, Serialize};

use crate::serialization::bounds::within;
use crate::settings::Setting;
use crate::stimuli::{About, SPIKE_RATES};

pub const ABOUT: About = About {
    name: "Random spikes",
    description: "Sends spikes at random moments, at a set average rate.",
    size: None,
};

pub const RATE: Setting = Setting {
    label: "Rate",
    description: "How many spikes it sends each second on average.",
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoissonGenerator {
    pub rate: Hertz,
}

impl Default for PoissonGenerator {
    fn default() -> Self {
        Self { rate: Hertz(20.0) }
    }
}

pub fn bound(world: &mut World) {
    for generator in world.query_mut::<&mut PoissonGenerator>() {
        let default = PoissonGenerator::default();
        within(&mut generator.rate, SPIKE_RATES, default.rate);
    }
}
