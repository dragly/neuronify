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

pub fn neurocolor(neuron_type: &NeuronType, value: f32) -> Vec3 {
    let v = 1.0 / (1.0 + (-5.0 * (value - 0.5)).exp());
    match *neuron_type {
        NeuronType::Excitatory => v * base() + (1.0 - v) * blue(),
        NeuronType::Inhibitory => v * mantle() + (1.0 - v) * red(),
    }
}
