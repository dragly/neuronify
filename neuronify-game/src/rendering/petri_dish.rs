use neuronify_core::rendering::colors::frame_color;
use neuronify_core::rendering::ConnectionData;

use crate::constants::*;
use crate::simulation::setup::PetriDish;
use glam::Vec3;

pub fn collect_petri_dish(dish: &PetriDish) -> Vec<ConnectionData> {
    let segments = PETRI_DISH_SEGMENTS;
    let mut lines = Vec::with_capacity(segments);
    let color = frame_color();
    for i in 0..segments {
        let angle_a = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let angle_b = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
        let a = dish.center
            + Vec3::new(
                dish.radius * angle_a.cos(),
                0.0,
                dish.radius * angle_a.sin(),
            );
        let b = dish.center
            + Vec3::new(
                dish.radius * angle_b.cos(),
                0.0,
                dish.radius * angle_b.sin(),
            );
        lines.push(ConnectionData {
            position_a: a,
            position_b: b,
            strength: 0.5,
            directional: 0.0,
            start_color: color,
            end_color: color,
            _padding: Default::default(),
        });
    }
    lines
}

