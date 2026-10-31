use glam::Vec3;

use crate::components::*;
use crate::measurement::voltmeter::{Corner, Frame};
use crate::rendering::colors::Theme;
use crate::rendering::connections::collect_voltmeter_wires;
use crate::rendering::scene::collect_scene;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Bounds {
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    fn include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
}

pub fn circuit_bounds(world: &hecs::World) -> Option<Bounds> {
    bounds_around(&circuit_points(world))
}

pub fn canvas_bounds(world: &hecs::World) -> Option<Bounds> {
    let mut points = circuit_points(world);
    for (_, (position, size)) in world.query::<(&Position, &NoteSize)>().iter() {
        points.extend(note_corners(position.position, size));
    }
    bounds_around(&points)
}

fn circuit_points(world: &hecs::World) -> Vec<Vec3> {
    let mut points = Vec::new();
    let palette = Theme::default().palette();
    let scene = collect_scene(world, &palette, 0.0);
    for sphere in &scene.spheres {
        points.extend(footprint(sphere.position, sphere.radius));
    }
    for cylinder in &scene.cylinders {
        points.extend(footprint(cylinder.start, cylinder.start_radius));
        points.extend(footprint(cylinder.end, cylinder.end_radius));
    }
    for ring in &scene.rings {
        points.extend(footprint(
            ring.position,
            ring.major_radius + ring.minor_radius,
        ));
    }
    for connection in collect_voltmeter_wires(world, &palette) {
        points.push(connection.position_a);
        points.push(connection.position_b);
    }
    for (_, (position, size)) in world.query::<(&Position, &VoltmeterSize)>().iter() {
        let frame = Frame::new(position.position, size);
        points.push(frame.corner(Corner::BottomLeft));
        points.push(frame.corner(Corner::TopRight));
    }
    points
}

fn bounds_around(points: &[Vec3]) -> Option<Bounds> {
    let (&first, rest) = points.split_first()?;
    let mut bounds = Bounds {
        min: first,
        max: first,
    };
    for &point in rest {
        bounds.include(point);
    }
    Some(bounds)
}

fn footprint(center: Vec3, radius: f32) -> [Vec3; 2] {
    let reach = Vec3::new(radius, 0.0, radius);
    let center = center.with_y(0.0);
    [center - reach, center + reach]
}

pub fn note_corners(center: Vec3, size: &NoteSize) -> [Vec3; 2] {
    let half_extent = Vec3::new(size.height * 0.5, 0.0, size.width * 0.5);
    [center - half_extent, center + half_extent]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::shapes::PAD_RADIUS;

    fn world_with_clamp_and_voltmeter() -> hecs::World {
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::new(0.0, 0.0, 0.0),
            },
            CurrentClamp::default(),
        ));
        world.spawn((
            Position {
                position: Vec3::new(10.0, 0.0, 5.0),
            },
            VoltmeterSize {
                width: 8.0,
                height: 4.0,
            },
        ));
        world
    }

    #[test]
    fn an_empty_world_has_no_bounds() {
        let world = hecs::World::new();
        assert_eq!(circuit_bounds(&world), None);
        assert_eq!(canvas_bounds(&world), None);
    }

    #[test]
    fn circuit_bounds_cover_neurons_and_voltmeters() {
        let bounds = circuit_bounds(&world_with_clamp_and_voltmeter()).unwrap();
        assert_eq!(bounds.min, Vec3::new(-PAD_RADIUS, 0.0, -PAD_RADIUS));
        assert_eq!(bounds.max, Vec3::new(12.0, 0.0, 13.0));
    }

    #[test]
    fn canvas_bounds_also_cover_notes() {
        let mut world = world_with_clamp_and_voltmeter();
        world.spawn((
            Position {
                position: Vec3::new(-10.0, 0.0, 20.0),
            },
            Annotation {
                text: "A note".to_owned(),
            },
            NoteSize {
                width: 6.0,
                height: 4.0,
            },
        ));
        assert_eq!(
            circuit_bounds(&world),
            circuit_bounds(&world_with_clamp_and_voltmeter())
        );
        let bounds = canvas_bounds(&world).unwrap();
        assert_eq!(bounds.min, Vec3::new(-12.0, 0.0, -PAD_RADIUS));
        assert_eq!(bounds.max, Vec3::new(12.0, 0.0, 23.0));
    }
}
