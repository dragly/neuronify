use std::f32::consts::PI;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_engine::units::{Milliseconds, Picocoulombs};

use crate::components::Position;
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::current_color;
use crate::rendering::current_flow::FASTEST_SHOWN_CURRENT;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{
    glyph_color, pad, GLYPH_HEIGHT, SCREEN_RIGHT, SCREEN_UP, WAVE_STEPS,
};
use crate::stimuli::alternating_current::AlternatingCurrent;
use crate::stimuli::CurrentClamp;

pub fn shapes(world: &World, shapes: &mut Scene) {
    for (position, clamp) in world
        .query::<(&Position, &CurrentClamp)>()
        .with::<&AlternatingCurrent>()
        .iter()
    {
        let center = position.position;
        pad(shapes, center, current_color(clamp.current_output));
        let wave: Vec<Vec3> = (0..=WAVE_STEPS)
            .map(|step| {
                let u = 2.0 * step as f32 / WAVE_STEPS as f32 - 1.0;
                let v = 0.45 * (u * PI).sin();
                center
                    + Vec3::Y * GLYPH_HEIGHT
                    + (SCREEN_RIGHT * u * 0.62 + SCREEN_UP * v) * 0.85 * NODE_RADIUS
            })
            .collect();
        shapes.stroke(&wave, 0.11 * NODE_RADIUS, glyph_color());
    }
}

pub fn charge(world: &World, source: Entity, time: Milliseconds) -> Picocoulombs {
    let Ok(alternating) = world.get::<&AlternatingCurrent>(source) else {
        return Picocoulombs::ZERO;
    };
    let slowdown = (FASTEST_SHOWN_CURRENT / alternating.amplitude.abs()).min(1.0);
    alternating.charge_until(time) * slowdown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stimuli::Stimulus;
    use neuronify_engine::units::Nanoamperes;

    fn drawn(world: &World) -> Scene {
        let mut scene = Scene::default();
        shapes(world, &mut scene);
        scene
    }

    #[test]
    fn alternating_current_sources_carry_a_wave_instead_of_an_arrow() {
        let mut world = World::new();
        Stimulus::AlternatingCurrentSource.spawn(&mut world, Vec3::ZERO);
        let shapes = drawn(&world);
        let (pad, wave) = shapes.cylinders.split_first().unwrap();
        assert_eq!(wave.len(), WAVE_STEPS);
        let heights: Vec<f32> = wave.iter().map(|segment| segment.start.x).collect();
        assert!(heights.iter().any(|&x| x > 0.2) && heights.iter().any(|&x| x < -0.2));
        assert!(wave.iter().all(|segment| segment.start.y > pad.end.y));
    }

    #[test]
    fn an_alternating_current_source_glows_with_the_size_of_its_current() {
        let pad_brightness = |current: f64| {
            let mut world = World::new();
            let source = Stimulus::AlternatingCurrentSource.spawn(&mut world, Vec3::ZERO);
            world
                .get::<&mut CurrentClamp>(source)
                .unwrap()
                .current_output = Nanoamperes(current);
            let pad = drawn(&world).cylinders[0].start_color;
            pad.x + pad.y + pad.z
        };
        assert!(pad_brightness(0.0) < pad_brightness(0.3));
        assert_eq!(pad_brightness(0.3), pad_brightness(-0.3));
    }
}
