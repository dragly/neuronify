use std::f32::consts::{FRAC_PI_6, PI};

use glam::{Vec2, Vec3};
use hecs::World;

use crate::components::{NeuronType, Position};
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::Palette;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{
    generator_color, glyph_color, pad, GLYPH_HEIGHT, SCREEN_RIGHT, SCREEN_UP,
};
use crate::stimuli::touch_sensor::TouchSensor;
use crate::stimuli::GeneratorDynamics;

struct Digit {
    width: f32,
    side: f32,
    valley: f32,
}

/// The index finger and the knuckles of the hand on a touch pad, after Neuronify 1's icon.
const HAND: [Digit; 4] = [
    Digit {
        width: 1.0,
        side: 2.55,
        valley: 0.45,
    },
    Digit {
        width: 0.55,
        side: 1.36,
        valley: 0.5,
    },
    Digit {
        width: 0.5,
        side: 1.32,
        valley: 0.56,
    },
    Digit {
        width: 0.45,
        side: 1.18,
        valley: 0.6,
    },
];
const HAND_SCALE: f32 = 0.45 * NODE_RADIUS;
const HAND_TILT: f32 = FRAC_PI_6;
const FINGERTIP_STEPS: usize = 8;

/// The outline of the hand, centred on the pad, with x to the right and y up on the screen.
pub fn hand_outline() -> Vec<Vec2> {
    let mut points = vec![Vec2::ZERO];
    let mut left = 0.0;
    for digit in HAND {
        let radius = digit.width / 2.0;
        points.extend((0..=FINGERTIP_STEPS).map(|step| {
            let angle = PI * step as f32 / FINGERTIP_STEPS as f32;
            Vec2::new(
                left + radius - radius * angle.cos(),
                digit.side + radius * angle.sin(),
            )
        }));
        left += digit.width;
        points.push(Vec2::new(left, digit.valley));
    }
    let rotation = Vec2::from_angle(HAND_TILT);
    let points: Vec<Vec2> = points
        .iter()
        .map(|&point| rotation.rotate(point) * HAND_SCALE)
        .collect();
    let min = points.iter().copied().reduce(Vec2::min).unwrap_or_default();
    let max = points.iter().copied().reduce(Vec2::max).unwrap_or_default();
    let center = (min + max) / 2.0;
    points.iter().map(|&point| point - center).collect()
}

pub fn shapes(world: &World, palette: &Palette, shapes: &mut Scene) {
    let hand = hand_outline();
    for (dynamics, position) in world
        .query::<(&GeneratorDynamics, &Position)>()
        .with::<&TouchSensor>()
        .iter()
    {
        let center = position.position;
        pad(
            shapes,
            center,
            generator_color(dynamics, NeuronType::Excitatory, palette),
        );
        let outline: Vec<Vec3> = hand
            .iter()
            .map(|point| {
                center + Vec3::Y * GLYPH_HEIGHT + SCREEN_RIGHT * point.x + SCREEN_UP * point.y
            })
            .collect();
        shapes.stroke(&outline, 0.06 * NODE_RADIUS, glyph_color());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DT;
    use crate::rendering::colors::Theme;
    use crate::rendering::shapes::{FLASH_DURATION, PAD_RADIUS};
    use crate::simulation::{fire_nearby, step_simulation, Engine};
    use crate::stimuli::Stimulus;
    use neuronify_engine::units::Milliseconds;

    fn drawn(world: &World, palette: &Palette) -> Scene {
        let mut scene = Scene::default();
        shapes(world, palette, &mut scene);
        scene
    }

    #[test]
    fn touch_pads_carry_a_hand_inside_the_pad() {
        let mut world = World::new();
        Stimulus::TouchSensor.spawn(&mut world, Vec3::X);
        let shapes = drawn(&world, &Theme::Dark.palette());
        let (pad, hand) = shapes.cylinders.split_first().unwrap();
        assert!(!hand.is_empty());
        for stroke in hand {
            for end in [stroke.start, stroke.end] {
                assert!(end.y > pad.end.y);
                let reach = (end - Vec3::X).with_y(0.0).length() + stroke.start_radius;
                assert!(reach < PAD_RADIUS, "{reach}");
            }
        }
    }

    #[test]
    fn touch_sensors_stop_flashing_after_a_touch() {
        let mut world = World::new();
        Stimulus::TouchSensor.spawn(&mut world, Vec3::ZERO);
        let palette = Theme::Dark.palette();
        let pad_color = |world: &World| drawn(world, &palette).cylinders[0].start_color;
        let resting = pad_color(&world);
        let (mut engine, mut time) = (Engine::default(), Milliseconds::ZERO);
        fire_nearby(&mut world, &mut engine, Vec3::ZERO);
        step_simulation(&mut world, &mut engine, &mut time, 1, None);
        assert_ne!(pad_color(&world), resting);
        let steps = (FLASH_DURATION / DT).ceil() as u32;
        step_simulation(&mut world, &mut engine, &mut time, steps, None);
        assert_eq!(pad_color(&world), resting);
    }
}
