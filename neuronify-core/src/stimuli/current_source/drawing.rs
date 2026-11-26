use std::collections::HashMap;

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::{Connection, Position};
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::current_color;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{glyph_color, pad, GLYPH_HEIGHT, SCREEN_UP};
use crate::stimuli::alternating_current::AlternatingCurrent;
use crate::stimuli::CurrentClamp;

/// Where the arrow of a current source without a target points.
const UNCONNECTED_ARROW: Vec3 = SCREEN_UP;

pub fn shapes(world: &World, shapes: &mut Scene) {
    let targets: HashMap<Entity, Vec3> = world
        .query::<&Connection>()
        .iter()
        .filter(|connection| world.get::<&CurrentClamp>(connection.from).is_ok())
        .filter_map(|connection| {
            let target = world.get::<&Position>(connection.to).ok()?;
            Some((connection.from, target.position))
        })
        .collect();
    for (entity, (position, clamp)) in world
        .query::<(Entity, (&Position, &CurrentClamp))>()
        .without::<&AlternatingCurrent>()
        .iter()
    {
        let center = position.position;
        pad(shapes, center, current_color(clamp.current_output));
        let forward = targets
            .get(&entity)
            .map(|target| (*target - center).with_y(0.0).normalize_or_zero())
            .filter(|direction| *direction != Vec3::ZERO)
            .unwrap_or(UNCONNECTED_ARROW);
        let side = Vec3::Y.cross(forward);
        let point = |u: f32, v: f32| {
            center + Vec3::Y * GLYPH_HEIGHT + (side * u + forward * v) * 0.85 * NODE_RADIUS
        };
        let color = glyph_color();
        let radius = 0.11 * NODE_RADIUS;
        shapes.stroke(&[point(0.0, -0.62), point(0.0, 0.62)], radius, color);
        shapes.stroke(
            &[point(-0.4, 0.2), point(0.0, 0.62), point(0.4, 0.2)],
            radius,
            color,
        );
    }
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
    fn current_source_arrow_points_at_its_target() {
        let mut world = World::new();
        let source = Stimulus::CurrentSource.spawn(&mut world, Vec3::ZERO);
        let target = world.spawn((Position {
            position: Vec3::new(0.0, 0.0, 5.0),
        },));
        world.spawn((Connection {
            from: source,
            to: target,
            strength: 1.0,
            directional: true,
        },));
        let shapes = drawn(&world);
        let shaft = &shapes.cylinders[1];
        assert!((shaft.end - shaft.start).normalize().dot(Vec3::Z) > 0.99);
    }

    #[test]
    fn a_stronger_current_source_has_a_brighter_pad() {
        let pad_brightness = |current: f64| {
            let mut world = World::new();
            let source = Stimulus::CurrentSource.spawn(&mut world, Vec3::ZERO);
            world
                .get::<&mut CurrentClamp>(source)
                .unwrap()
                .current_output = Nanoamperes(current);
            let pad = drawn(&world).cylinders[0].start_color;
            pad.x + pad.y + pad.z
        };
        assert!(pad_brightness(0.0) < pad_brightness(0.1));
        assert!(pad_brightness(0.1) < pad_brightness(0.35));
        assert!(pad_brightness(0.35) < pad_brightness(0.7));
        assert!(pad_brightness(0.7) < pad_brightness(2.0));
    }
}
