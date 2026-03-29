use glam::Vec3;
use neuronify_core::rendering::colors::srgb;

pub fn player1_color() -> Vec3 {
    srgb(64, 160, 43)
}

pub fn player2_color() -> Vec3 {
    srgb(136, 57, 239)
}

pub fn blood_vessel_color() -> Vec3 {
    srgb(180, 30, 30)
}

pub fn glial_color() -> Vec3 {
    srgb(100, 180, 100)
}

pub fn activity_sensor_color() -> Vec3 {
    srgb(80, 200, 220)
}

pub fn chemical_sensor_color() -> Vec3 {
    srgb(220, 200, 60)
}

pub fn touch_sensor_color() -> Vec3 {
    srgb(220, 140, 50)
}

pub fn membrane_color() -> Vec3 {
    srgb(60, 60, 70)
}
