use glam::Vec3;

use crate::components::NeuronType;

pub fn shown_fraction(voltage: f64) -> f32 {
    ((voltage - LOWEST_SHOWN_VOLTAGE) / (HIGHEST_SHOWN_VOLTAGE - LOWEST_SHOWN_VOLTAGE))
        .clamp(0.0, 1.0) as f32
}

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

pub fn paler(color: Vec3) -> Vec3 {
    color.lerp(Vec3::splat(color.dot(LUMINANCE)), DENDRITE_DESATURATION)
}

pub fn terminal_color() -> Vec3 {
    Vec3::new(136.0 / 255.0, 57.0 / 255.0, 239.0 / 255.0)
}

pub fn adaptation_gauge_color() -> Vec3 {
    srgb(32, 170, 150)
}

pub fn speaker_color() -> Vec3 {
    srgb(114, 135, 253)
}

pub fn rate_color() -> Vec3 {
    srgb(232, 176, 75)
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

const GLARE_START: f64 = -20.0;
const DENDRITE_DESATURATION: f32 = 0.2;
const SHEATH_TINT: f32 = 0.5;
const LUMINANCE: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);
const GLARE_FULL: f64 = 20.0;
const GLARE_PEAK: f32 = 2.0;
const LOWEST_SHOWN_VOLTAGE: f64 = -86.0;
const HIGHEST_SHOWN_VOLTAGE: f64 = 20.0;

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
    pub meter_background: Vec3,
    pub excitatory: Vec3,
    pub excitatory_active: Vec3,
    pub inhibitory: Vec3,
    pub inhibitory_active: Vec3,
    pub excitatory_below: Vec3,
    pub inhibitory_below: Vec3,
    /// A soma right after it fired
    pub flash: Vec3,
    /// Connections from anything that is not a neuron or a source
    pub neutral: Vec3,
    pub myelin: Vec3,
    pub selection: Vec3,
    pub halo: f32,
    pub transmitter: f32,
}

impl Theme {
    pub fn palette(self) -> Palette {
        match self {
            Theme::Dark => Palette {
                floor: srgb(56, 58, 80),
                meter_background: srgb(34, 36, 52),
                excitatory: blue(),
                excitatory_active: blue().lerp(base(), VOLTAGE_TINT),
                inhibitory: red(),
                inhibitory_active: red().lerp(mantle(), VOLTAGE_TINT),
                excitatory_below: srgb(18, 138, 138),
                inhibitory_below: srgb(212, 98, 76),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: crust(),
                myelin: srgb(222, 214, 190),
                selection: srgb(80, 240, 110),
                halo: 0.6,
                transmitter: 0.6,
            },
            Theme::Light => Palette {
                floor: srgb(236, 239, 245),
                meter_background: srgb(252, 253, 255),
                excitatory: srgb(66, 105, 199),
                excitatory_active: srgb(66, 105, 199).lerp(base(), VOLTAGE_TINT),
                inhibitory: srgb(184, 61, 85),
                inhibitory_active: srgb(184, 61, 85).lerp(mantle(), VOLTAGE_TINT),
                excitatory_below: srgb(26, 126, 126),
                inhibitory_below: srgb(196, 94, 74),
                flash: Vec3::new(1.0, 1.0, 0.55),
                neutral: srgb(140, 143, 161),
                myelin: srgb(244, 233, 205),
                selection: srgb(15, 85, 45),
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

    pub fn dendrite(&self, neuron_type: &NeuronType, voltage: f64) -> Vec3 {
        paler(self.compartment(neuron_type, voltage))
    }

    pub fn sheath(&self, axon: Vec3) -> Vec3 {
        let cream = self.myelin * axon.dot(LUMINANCE) / self.myelin.dot(LUMINANCE);
        axon.lerp(cream, SHEATH_TINT)
    }

    pub fn compartment(&self, neuron_type: &NeuronType, voltage: f64) -> Vec3 {
        glared(
            self.neurocolor(neuron_type, shown_fraction(voltage)),
            voltage,
        )
    }
}

fn glared(color: Vec3, voltage: f64) -> Vec3 {
    let glare = ((voltage - GLARE_START) / (GLARE_FULL - GLARE_START)).clamp(0.0, 1.0) as f32;
    color.lerp(color / color.max_element() * GLARE_PEAK, glare)
}

pub fn lit(color: Vec3, voltage: f64) -> Vec3 {
    let value = shown_fraction(voltage);
    let tint = 1.0 / (1.0 + (-5.0 * (value - 0.5)).exp());
    glared(color.lerp(color.lerp(base(), VOLTAGE_TINT), tint), voltage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::lighting::BLOOM_THRESHOLD;
    use glam::Mat3;

    fn oklab(color: Vec3) -> Vec3 {
        let [red, green, blue] = color.clamp(Vec3::ZERO, Vec3::ONE).to_array();
        let [long, medium, short] = [
            0.412_221_47 * red + 0.536_332_54 * green + 0.051_445_99 * blue,
            0.211_903_5 * red + 0.680_699_6 * green + 0.107_396_96 * blue,
            0.088_302_46 * red + 0.281_718_84 * green + 0.629_978_7 * blue,
        ]
        .map(f32::cbrt);
        Vec3::new(
            0.210_454_26 * long + 0.793_617_8 * medium - 0.004_072_05 * short,
            1.977_998_5 * long - 2.428_592_2 * medium + 0.450_593_7 * short,
            0.025_904_04 * long + 0.782_771_77 * medium - 0.808_675_77 * short,
        )
    }

    fn seen_without(rows: [f32; 9]) -> Mat3 {
        Mat3::from_cols_array(&rows).transpose()
    }

    #[test]
    fn the_selection_stands_apart_from_both_neuron_types_also_for_color_blind_eyes() {
        let eyes = [
            Mat3::IDENTITY,
            seen_without([
                0.152_286, 1.052_583, -0.204_868, 0.114_503, 0.786_281, 0.099_216, -0.003_882,
                -0.048_116, 1.051_998,
            ]),
            seen_without([
                0.367_322, 0.860_646, -0.227_968, 0.280_085, 0.672_501, 0.047_413, -0.011_82,
                0.042_94, 0.968_881,
            ]),
            seen_without([
                1.255_528, -0.076_749, -0.178_779, -0.078_411, 0.930_809, 0.147_602, 0.004_733,
                0.691_367, 0.303_9,
            ]),
        ];
        for theme in [Theme::Dark, Theme::Light] {
            let palette = theme.palette();
            for neuron in [
                palette.excitatory,
                palette.excitatory_active,
                palette.inhibitory,
                palette.inhibitory_active,
            ] {
                for eye in eyes {
                    let apart = oklab(eye * palette.selection).distance(oklab(eye * neuron));
                    assert!(apart > 0.06, "{theme:?} {neuron} {apart}");
                }
            }
        }
    }

    #[test]
    fn dendrites_are_paler_but_not_brighter() {
        for theme in [Theme::Dark, Theme::Light] {
            for neuron_type in [NeuronType::Excitatory, NeuronType::Inhibitory] {
                for voltage in [-65.0, 0.0, GLARE_FULL] {
                    let palette = theme.palette();
                    let axon = palette.compartment(&neuron_type, voltage);
                    let dendrite = palette.dendrite(&neuron_type, voltage);
                    let spread = |color: Vec3| color.max_element() - color.min_element();
                    assert!((axon.dot(LUMINANCE) - dendrite.dot(LUMINANCE)).abs() < 1e-5);
                    assert!(spread(dendrite) < spread(axon));
                }
            }
        }
    }

    #[test]
    fn sheaths_turn_towards_myelin_but_not_brighter() {
        for theme in [Theme::Dark, Theme::Light] {
            for voltage in [-65.0, 0.0, GLARE_FULL] {
                let palette = theme.palette();
                let axon = palette.compartment(&NeuronType::Excitatory, voltage);
                let sheath = palette.sheath(axon);
                let hue = |color: Vec3| color / color.dot(LUMINANCE);
                let myelin = hue(palette.myelin);
                assert!((axon.dot(LUMINANCE) - sheath.dot(LUMINANCE)).abs() < 1e-5);
                assert!(hue(sheath).distance(myelin) < hue(axon).distance(myelin));
            }
        }
    }

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
                assert!(brightest(crate::constants::RESTING_POTENTIAL) < BLOOM_THRESHOLD);
                assert!(brightest(GLARE_START) < BLOOM_THRESHOLD);
                assert!(brightest(GLARE_FULL) > BLOOM_THRESHOLD);
            }
        }
    }
}
