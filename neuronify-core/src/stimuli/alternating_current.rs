pub mod drawing;
pub mod gui;

use std::ops::RangeInclusive;

use hecs::World;
use neuronify_engine::units::{Hertz, Kilohertz, Milliseconds, Nanoamperes, Picocoulombs};
use serde::{Deserialize, Serialize};

use crate::serialization::bounds::within;
use crate::settings::Setting;
use crate::stimuli::About;

pub const AMPLITUDES: RangeInclusive<Nanoamperes> = Nanoamperes(0.0)..=Nanoamperes(10_000.0);
pub const ALTERNATING_FREQUENCIES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(10_000.0);

pub const ABOUT: About = About {
    name: "Alternating current",
    description: "Injects a current that swings up and down like a wave.",
    size: None,
};

pub const AMPLITUDE: Setting = Setting {
    label: "Amplitude",
    description: "How far the current swings up and down.",
};

pub const FREQUENCY: Setting = Setting {
    label: "Frequency",
    description: "How many times a second the current swings.",
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlternatingCurrent {
    pub amplitude: Nanoamperes,
    pub frequency: Hertz,
}

impl Default for AlternatingCurrent {
    fn default() -> Self {
        Self {
            amplitude: Nanoamperes(0.3),
            frequency: Hertz(50.0),
        }
    }
}

impl AlternatingCurrent {
    pub fn at(&self, time: Milliseconds) -> Nanoamperes {
        self.amplitude * (std::f64::consts::TAU * self.frequency.cycles_in(time)).sin()
    }

    pub fn charge_until(&self, time: Milliseconds) -> Picocoulombs {
        let angular_frequency = std::f64::consts::TAU * self.frequency.to_kilohertz();
        if angular_frequency == Kilohertz::ZERO {
            return Picocoulombs::ZERO;
        }
        self.amplitude * (1.0 - (angular_frequency * time).cos()) * (1.0 / angular_frequency)
    }
}

pub fn bound(world: &mut World) {
    for alternating in world.query_mut::<&mut AlternatingCurrent>() {
        let default = AlternatingCurrent::default();
        within(&mut alternating.amplitude, AMPLITUDES, default.amplitude);
        within(
            &mut alternating.frequency,
            ALTERNATING_FREQUENCIES,
            default.frequency,
        );
    }
}
