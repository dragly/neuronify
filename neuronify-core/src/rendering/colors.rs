use glam::Vec3;

use crate::components::NeuronType;

pub fn srgb_component(value: u8) -> f32 {
    (value as f32 / 255.0 + 0.055_f32).powf(2.44) / 1.055
}

pub fn srgb(red: u8, green: u8, blue: u8) -> Vec3 {
    Vec3::new(
        srgb_component(red),
        srgb_component(green),
        srgb_component(blue),
    )
}

pub fn red() -> Vec3 {
    srgb(210, 15, 57)
}

pub fn blue() -> Vec3 {
    srgb(30, 102, 245)
}

pub fn base() -> Vec3 {
    srgb(239, 241, 245)
}

pub fn mantle() -> Vec3 {
    srgb(230, 233, 239)
}

pub fn crust() -> Vec3 {
    srgb(220, 224, 232)
}

pub fn yellow() -> Vec3 {
    srgb(223, 142, 29)
}

pub fn orange() -> Vec3 {
    srgb(254, 100, 11)
}

pub fn green() -> Vec3 {
    srgb(64, 160, 43)
}

pub fn frame_color() -> Vec3 {
    srgb(80, 80, 100)
}

pub fn terminal_color() -> Vec3 {
    Vec3::new(136.0 / 255.0, 57.0 / 255.0, 239.0 / 255.0)
}

pub fn adaptation_gauge_color() -> Vec3 {
    srgb(32, 170, 150)
}

pub fn burst_gauge_color() -> Vec3 {
    srgb(228, 92, 152)
}

/// How far a neuron at threshold has gone from the colour of its type towards white.
pub const VOLTAGE_TINT: f32 = 0.55;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/// The colours of one theme.
///
/// The voltage of a neuron runs from the colour of its type at rest towards a pale tint of
/// it, so it lights up as it nears threshold and still looks different from a flash. The
/// light theme starts from softer colours.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub floor: Vec3,
    pub excitatory: Vec3,
    pub excitatory_active: Vec3,
    pub inhibitory: Vec3,
    pub inhibitory_active: Vec3,
    /// A spike on its way along a synapse
    pub spike: Vec3,
    /// A soma right after it fired
    pub flash: Vec3,
    /// Connections from anything that is not a neuron or a source
    pub neutral: Vec3,
}

impl Theme {
    pub fn palette(self) -> Palette {
        match self {
            Theme::Dark => Palette {
                floor: srgb(56, 58, 80),
                excitatory: blue(),
                excitatory_active: blue().lerp(base(), VOLTAGE_TINT),
                inhibitory: red(),
                inhibitory_active: red().lerp(mantle(), VOLTAGE_TINT),
                spike: crust(),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: crust(),
            },
            Theme::Light => Palette {
                floor: srgb(236, 239, 245),
                excitatory: srgb(66, 105, 199),
                excitatory_active: srgb(66, 105, 199).lerp(base(), VOLTAGE_TINT),
                inhibitory: srgb(184, 61, 85),
                inhibitory_active: srgb(184, 61, 85).lerp(mantle(), VOLTAGE_TINT),
                spike: crust(),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: srgb(140, 143, 161),
            },
        }
    }
}

impl Palette {
    /// A soma, from rest at 0 to threshold at 1.
    pub fn soma(&self, neuron_type: &NeuronType, value: f32) -> Vec3 {
        match *neuron_type {
            NeuronType::Excitatory => self.excitatory.lerp(self.excitatory_active, value),
            NeuronType::Inhibitory => self.inhibitory.lerp(self.inhibitory_active, value),
        }
    }

    /// A compartment, which turns over mostly around the middle of the range.
    pub fn neurocolor(&self, neuron_type: &NeuronType, value: f32) -> Vec3 {
        let v = 1.0 / (1.0 + (-5.0 * (value - 0.5)).exp());
        self.soma(neuron_type, v)
    }
}
