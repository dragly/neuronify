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

const CURRENT_BRIGHTNESS_SCALE: f64 = 0.5e-9;
const CURRENT_RAMP: [(f32, (u8, u8, u8)); 3] = [
    (0.0, (96, 40, 8)),
    (0.7, (240, 116, 20)),
    (1.0, (255, 178, 104)),
];

pub fn current_color(amperes: f64) -> Vec3 {
    let level = (1.0 - (-amperes.abs() / CURRENT_BRIGHTNESS_SCALE).exp()) as f32;
    let upper = CURRENT_RAMP
        .iter()
        .position(|&(at, _)| level <= at)
        .unwrap_or(CURRENT_RAMP.len() - 1)
        .max(1);
    let (low_at, low) = CURRENT_RAMP[upper - 1];
    let (high_at, high) = CURRENT_RAMP[upper];
    let color = |(red, green, blue): (u8, u8, u8)| srgb(red, green, blue);
    color(low).lerp(color(high), (level - low_at) / (high_at - low_at))
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

const PLASMA: [(u8, u8, u8); 11] = [
    (13, 8, 135),
    (65, 4, 157),
    (106, 0, 168),
    (143, 13, 164),
    (177, 42, 144),
    (204, 71, 120),
    (225, 100, 98),
    (242, 132, 75),
    (252, 166, 54),
    (252, 206, 37),
    (240, 249, 33),
];

pub fn firing_rate_color(level: f32) -> Vec3 {
    let position = level.clamp(0.0, 1.0) * (PLASMA.len() - 1) as f32;
    let index = (position as usize).min(PLASMA.len() - 2);
    let [low, high] = [PLASMA[index], PLASMA[index + 1]].map(|(r, g, b)| srgb(r, g, b));
    low.lerp(high, position - index as f32)
}

/// How far a neuron at threshold has gone from the colour of its type towards white.
pub const VOLTAGE_TINT: f32 = 0.55;

const GLARE_START: f64 = 90.0;
const GLARE_FULL: f64 = 130.0;
const GLARE_PEAK: f32 = 2.0;

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
    pub excitatory_below: Vec3,
    pub inhibitory_below: Vec3,
    /// A spike on its way along a synapse
    pub spike: Vec3,
    /// A soma right after it fired
    pub flash: Vec3,
    /// Connections from anything that is not a neuron or a source
    pub neutral: Vec3,
    pub selection: Vec3,
    pub halo: f32,
    pub transmitter: f32,
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
                excitatory_below: srgb(18, 138, 138),
                inhibitory_below: srgb(212, 98, 76),
                spike: crust(),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: crust(),
                selection: srgb(137, 180, 250),
                halo: 0.6,
                transmitter: 0.6,
            },
            Theme::Light => Palette {
                floor: srgb(236, 239, 245),
                excitatory: srgb(66, 105, 199),
                excitatory_active: srgb(66, 105, 199).lerp(base(), VOLTAGE_TINT),
                inhibitory: srgb(184, 61, 85),
                inhibitory_active: srgb(184, 61, 85).lerp(mantle(), VOLTAGE_TINT),
                excitatory_below: srgb(26, 126, 126),
                inhibitory_below: srgb(196, 94, 74),
                spike: crust(),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: srgb(140, 143, 161),
                selection: srgb(30, 102, 245),
                halo: 1.0,
                transmitter: 0.8,
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

    pub fn soma_below_rest(&self, neuron_type: &NeuronType, depth: f32) -> Vec3 {
        match *neuron_type {
            NeuronType::Excitatory => self.excitatory.lerp(self.excitatory_below, depth),
            NeuronType::Inhibitory => self.inhibitory.lerp(self.inhibitory_below, depth),
        }
    }

    pub fn generator(&self, neuron_type: NeuronType) -> Vec3 {
        match neuron_type {
            NeuronType::Excitatory => orange(),
            NeuronType::Inhibitory => self.inhibitory,
        }
    }

    /// A compartment, which turns over mostly around the middle of the range.
    pub fn neurocolor(&self, neuron_type: &NeuronType, value: f32) -> Vec3 {
        let v = 1.0 / (1.0 + (-5.0 * (value - 0.5)).exp());
        self.soma(neuron_type, v)
    }

    pub fn compartment(&self, neuron_type: &NeuronType, voltage: f64) -> Vec3 {
        let value = ((voltage + 50.0) / 200.0).clamp(0.0, 1.0) as f32;
        let color = self.neurocolor(neuron_type, value);
        let glare = ((voltage - GLARE_START) / (GLARE_FULL - GLARE_START)).clamp(0.0, 1.0) as f32;
        color.lerp(color / color.max_element() * GLARE_PEAK, glare)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::lighting::BLOOM_THRESHOLD;

    #[test]
    fn only_the_crest_of_a_spike_glares() {
        for theme in [Theme::Dark, Theme::Light] {
            for neuron_type in [NeuronType::Excitatory, NeuronType::Inhibitory] {
                let brightest = |voltage| {
                    theme
                        .palette()
                        .compartment(&neuron_type, voltage)
                        .max_element()
                };
                assert!(brightest(-10.0) < BLOOM_THRESHOLD);
                assert!(brightest(GLARE_START) < BLOOM_THRESHOLD);
                assert!(brightest(GLARE_FULL) > BLOOM_THRESHOLD);
            }
        }
    }
}
