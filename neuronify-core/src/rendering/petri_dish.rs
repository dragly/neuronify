use crate::constants::*;
use crate::rendering::colors::frame_color;
use crate::rendering::gpu_types::ConnectionData;
use crate::simulation::game::PetriDish;
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

pub fn collect_resource_node_rings(world: &hecs::World) -> Vec<ConnectionData> {
    use crate::components::{Position, ResourceNode};
    use crate::rendering::colors::resource_color;

    let mut lines = Vec::new();
    let color = resource_color() * 0.4;
    let segments = 32;

    for (_, (resource, position)) in world.query::<(&ResourceNode, &Position)>().iter() {
        let center = position.position;
        let radius = resource.radius;
        for i in 0..segments {
            let angle_a = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let angle_b = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
            let a = center + Vec3::new(radius * angle_a.cos(), 0.0, radius * angle_a.sin());
            let b = center + Vec3::new(radius * angle_b.cos(), 0.0, radius * angle_b.sin());
            lines.push(ConnectionData {
                position_a: a,
                position_b: b,
                strength: 0.2,
                directional: 0.0,
                start_color: color,
                end_color: color,
                _padding: Default::default(),
            });
        }
    }
    lines
}

pub fn collect_substrate_zone_rings(world: &hecs::World) -> Vec<ConnectionData> {
    use crate::components::{Position, SubstrateZone, SubstrateZoneType};
    use crate::rendering::colors;

    let mut lines = Vec::new();
    let segments = 32;

    for (_, (zone, position)) in world.query::<(&SubstrateZone, &Position)>().iter() {
        let center = position.position;
        let radius = zone.radius;
        let color = match zone.zone_type {
            SubstrateZoneType::HighPotassium => colors::orange() * 0.5,
            SubstrateZoneType::HighMagnesium => colors::blue() * 0.4,
            SubstrateZoneType::Noise => colors::yellow() * 0.3,
            SubstrateZoneType::Damage => colors::red() * 0.5,
        };

        for i in 0..segments {
            let angle_a = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let angle_b = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
            let a = center + Vec3::new(radius * angle_a.cos(), 0.0, radius * angle_a.sin());
            let b = center + Vec3::new(radius * angle_b.cos(), 0.0, radius * angle_b.sin());
            lines.push(ConnectionData {
                position_a: a,
                position_b: b,
                strength: 0.3,
                directional: 0.0,
                start_color: color,
                end_color: color,
                _padding: Default::default(),
            });
        }
    }
    lines
}
