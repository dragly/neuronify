use neuronify_core::rendering::colors::frame_color;
use neuronify_core::rendering::ConnectionData;

use crate::constants::*;
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

pub fn collect_vessel_supply_rings(world: &hecs::World) -> Vec<ConnectionData> {
    use crate::components::{BloodVessel, Ownership, PlayerId};
    use crate::rendering::colors::{blood_vessel_color, player1_color, player2_color};
    use neuronify_core::Position;

    let mut lines = Vec::new();
    let segments = 32;

    for (entity, (vessel, position)) in world.query::<(&BloodVessel, &Position)>().iter() {
        let center = position.position;
        let radius = vessel.supply_radius;
        let base_color = blood_vessel_color() * 0.3;
        let color = if let Ok(ownership) = world.get::<&Ownership>(entity) {
            let pc = match ownership.player {
                PlayerId::Player1 => player1_color(),
                PlayerId::Player2 => player2_color(),
            };
            base_color * 0.5 + pc * 0.5
        } else {
            base_color
        };
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

/// Draw supply lines between glial cells and the blood vessels they're gathering from.
pub fn collect_glial_vessel_links(world: &hecs::World) -> Vec<ConnectionData> {
    use crate::components::{BloodVessel, GlialCell, Ownership, PlayerId};
    use crate::rendering::colors::{glial_color, player1_color, player2_color};
    use neuronify_core::Position;

    let mut lines = Vec::new();

    let vessels: Vec<(Vec3, f32, PlayerId)> = world
        .query::<(&BloodVessel, &Position, &Ownership)>()
        .iter()
        .map(|(_, (v, p, o))| (p.position, v.supply_radius, o.player))
        .collect();

    for (entity, (glial, pos)) in world.query::<(&GlialCell, &Position)>().iter() {
        let glial_player = match world.get::<&Ownership>(entity).map(|o| o.player).ok() {
            Some(p) => p,
            None => continue,
        };
        let fullness = (glial.blocks_stored / glial.max_blocks).clamp(0.0, 1.0) as f32;
        let base = glial_color() * 0.5;
        let pc = match glial_player {
            PlayerId::Player1 => player1_color(),
            PlayerId::Player2 => player2_color(),
        };
        let color = base * 0.5 + pc * 0.5;

        for &(vpos, _supply_radius, vessel_player) in &vessels {
            if vessel_player != glial_player {
                continue;
            }
            let dist =
                Vec3::new(pos.position.x - vpos.x, 0.0, pos.position.z - vpos.z).length();
            if dist < glial.gather_radius {
                lines.push(ConnectionData {
                    position_a: pos.position,
                    position_b: vpos,
                    strength: 0.15 + fullness * 0.25,
                    directional: 1.0,
                    start_color: color,
                    end_color: color,
                    _padding: Default::default(),
                });
            }
        }
    }
    lines
}

pub fn collect_substrate_zone_rings(world: &hecs::World) -> Vec<ConnectionData> {
    use crate::components::{SubstrateZone, SubstrateZoneType};
    use neuronify_core::rendering::colors as core_colors;
    use neuronify_core::Position;

    let mut lines = Vec::new();
    let segments = 32;

    for (_, (zone, position)) in world.query::<(&SubstrateZone, &Position)>().iter() {
        let center = position.position;
        let radius = zone.radius;
        let color = match zone.zone_type {
            SubstrateZoneType::HighPotassium => core_colors::orange() * 0.5,
            SubstrateZoneType::HighMagnesium => core_colors::blue() * 0.4,
            SubstrateZoneType::Noise => core_colors::yellow() * 0.3,
            SubstrateZoneType::Damage => core_colors::red() * 0.5,
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
