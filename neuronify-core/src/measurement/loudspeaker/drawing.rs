use glam::Vec3;

use super::Loudspeaker;
use crate::components::Position;
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::{speaker_color, Palette};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{
    flash, glyph_color, pad, speaker_outline, speaker_waves, FLASH_BRIGHTNESS, GLYPH_HEIGHT,
    SCREEN_RIGHT, SCREEN_UP,
};

pub fn collect_loudspeakers(world: &hecs::World, palette: &Palette, shapes: &mut Scene) {
    for (speaker, position) in world.query::<(&Loudspeaker, &Position)>().iter() {
        let center = position.position;
        let color = speaker_color().lerp(
            palette.flash * FLASH_BRIGHTNESS,
            flash(speaker.time_since_heard),
        );
        pad(shapes, center, color);
        let point = |&(x, y): &(f32, f32)| {
            center
                + Vec3::Y * GLYPH_HEIGHT
                + (SCREEN_RIGHT * x + SCREEN_UP * y) * 0.85 * NODE_RADIUS
        };
        let mut cone: Vec<Vec3> = speaker_outline().iter().map(point).collect();
        cone.push(cone[0]);
        shapes.stroke(&cone, 0.07 * NODE_RADIUS, glyph_color());
        for wave in speaker_waves() {
            let wave: Vec<Vec3> = wave.iter().map(point).collect();
            shapes.stroke(&wave, 0.07 * NODE_RADIUS, glyph_color());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::loudspeaker::listen;
    use crate::measurement::Meter;
    use crate::rendering::shapes::FLASH_DURATION;
    use crate::rendering::Theme;
    use neuronify_engine::units::Milliseconds;

    fn loudspeakers(world: &hecs::World, palette: &Palette) -> Scene {
        let mut scene = Scene::default();
        collect_loudspeakers(world, palette, &mut scene);
        scene
    }

    #[test]
    fn a_loudspeaker_flashes_when_it_plays() {
        let mut world = hecs::World::new();
        let neuron = world.spawn((Position { position: Vec3::X },));
        let speaker = Meter::Loudspeaker.spawn_measuring(&mut world, neuron, Vec3::ZERO);
        let palette = Theme::Dark.palette();
        let pad_color =
            |world: &hecs::World| loudspeakers(world, &palette).cylinders[0].start_color;
        assert_eq!(pad_color(&world), speaker_color());
        world
            .get::<&mut Loudspeaker>(speaker)
            .unwrap()
            .time_since_heard = Milliseconds::ZERO;
        assert_ne!(pad_color(&world), speaker_color());
    }

    #[test]
    fn a_loudspeaker_flash_is_over_a_few_milliseconds_after_it_plays() {
        let mut world = hecs::World::new();
        let neuron = world.spawn((Position { position: Vec3::X },));
        Meter::Loudspeaker.spawn_measuring(&mut world, neuron, Vec3::ZERO);
        let palette = Theme::Dark.palette();
        let pad_color =
            |world: &hecs::World| loudspeakers(world, &palette).cylinders[0].start_color;
        let heard = Milliseconds(1000.0);
        let spike = crate::Spike {
            entity: neuron,
            time: heard,
        };

        listen(&mut world, &[spike], heard, Milliseconds::ZERO);
        assert_ne!(pad_color(&world), speaker_color());
        listen(
            &mut world,
            &[],
            heard + 2.0 * FLASH_DURATION,
            2.0 * FLASH_DURATION,
        );
        assert_eq!(pad_color(&world), speaker_color());
    }
}
