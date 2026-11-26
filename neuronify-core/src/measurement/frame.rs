use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::components::{Connection, MeterSize, Position};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum RowOrder {
    #[default]
    Height,
    Connection,
}

pub fn measured(world: &hecs::World, meter: Entity) -> Vec<(Entity, Entity)> {
    let by_height = world
        .get::<&RowOrder>(meter)
        .map_or(true, |order| *order == RowOrder::Height);
    let height = |entity: Entity| {
        world
            .get::<&Position>(entity)
            .map_or(0.0, |position| position.position.x)
    };
    let mut wires: Vec<(Entity, Entity)> = world
        .query::<(Entity, &Connection)>()
        .iter()
        .filter(|(_, connection)| connection.to == meter)
        .map(|(wire, connection)| (wire, connection.from))
        .collect();
    if by_height {
        wires.sort_by(|(a_wire, a), (b_wire, b)| {
            height(*b)
                .total_cmp(&height(*a))
                .then(a_wire.id().cmp(&b_wire.id()))
        });
    } else {
        wires.sort_by_key(|(wire, _)| wire.id());
    }
    wires
}

const MIN_FRAME_WIDTH: f32 = 2.0;
const MIN_FRAME_HEIGHT: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    fn is_top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }

    fn is_right(self) -> bool {
        matches!(self, Corner::TopRight | Corner::BottomRight)
    }

    fn opposite(self) -> Corner {
        match self {
            Corner::TopLeft => Corner::BottomRight,
            Corner::TopRight => Corner::BottomLeft,
            Corner::BottomLeft => Corner::TopRight,
            Corner::BottomRight => Corner::TopLeft,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub bottom_left: Vec3,
    pub width: f32,
    pub height: f32,
}

impl Frame {
    pub fn new(position: Vec3, size: &MeterSize) -> Frame {
        Frame {
            bottom_left: position - Vec3::new(size.height * 0.5, 0.0, 0.0),
            width: size.width,
            height: size.height,
        }
    }

    pub fn square(center: Vec3, size: f32) -> Frame {
        Frame {
            bottom_left: center - Vec3::new(size, 0.0, size) * 0.5,
            width: size,
            height: size,
        }
    }

    pub fn of(world: &hecs::World, voltmeter: Entity) -> Option<Frame> {
        let position = world.get::<&Position>(voltmeter).ok()?;
        let size = world.get::<&MeterSize>(voltmeter).ok()?;
        Some(Frame::new(position.position, &size))
    }

    pub fn position(&self) -> Vec3 {
        self.bottom_left + Vec3::new(self.height * 0.5, 0.0, 0.0)
    }

    pub fn size(&self) -> MeterSize {
        MeterSize {
            width: self.width,
            height: self.height,
        }
    }

    pub fn row(&self, index: usize, count: usize) -> Frame {
        let height = self.height / count.max(1) as f32;
        Frame {
            bottom_left: self.bottom_left + Vec3::X * (self.height - (index as f32 + 1.0) * height),
            width: self.width,
            height,
        }
    }

    pub fn point(&self, right: f32, up: f32) -> Vec3 {
        self.bottom_left + Vec3::new(up * self.height, 0.0, right * self.width)
    }

    pub fn corner(&self, corner: Corner) -> Vec3 {
        let fraction = |far: bool| if far { 1.0 } else { 0.0 };
        self.point(fraction(corner.is_right()), fraction(corner.is_top()))
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let top_right = self.corner(Corner::TopRight);
        (self.bottom_left.x..=top_right.x).contains(&point.x)
            && (self.bottom_left.z..=top_right.z).contains(&point.z)
    }

    pub fn corner_near(&self, point: Vec3, reach: f32) -> Option<Corner> {
        Corner::ALL
            .into_iter()
            .find(|&corner| (point - self.corner(corner)).with_y(0.0).length() < reach)
    }

    pub fn resized(&self, corner: Corner, point: Vec3) -> Frame {
        let anchor = self.corner(corner.opposite());
        let width = if corner.is_right() {
            point.z - anchor.z
        } else {
            anchor.z - point.z
        };
        let height = if corner.is_top() {
            point.x - anchor.x
        } else {
            anchor.x - point.x
        };
        let width = width.max(MIN_FRAME_WIDTH);
        let height = height.max(MIN_FRAME_HEIGHT);
        let bottom = if corner.is_top() {
            anchor.x
        } else {
            anchor.x - height
        };
        let left = if corner.is_right() {
            anchor.z
        } else {
            anchor.z - width
        };
        Frame {
            bottom_left: Vec3::new(bottom, 0.0, left),
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::Meter;

    fn frame() -> Frame {
        Frame::new(
            Vec3::new(10.0, 0.0, 5.0),
            &MeterSize {
                width: 8.0,
                height: 4.0,
            },
        )
    }

    #[test]
    fn rows_follow_the_measured_neurons_from_the_highest_down() {
        let mut world = hecs::World::new();
        let neuron_at = |world: &mut hecs::World, up: f32| {
            crate::elements::Element::ExcitatoryNeuron.spawn(world, Vec3::new(up, 0.0, 0.0))
        };
        let low = neuron_at(&mut world, -5.0);
        let high = neuron_at(&mut world, 5.0);
        let middle = neuron_at(&mut world, 0.0);
        let voltmeter =
            Meter::Voltmeter.spawn_measuring(&mut world, low, Vec3::new(0.0, 0.0, 10.0));
        Meter::Voltmeter.wire(&mut world, high, voltmeter);
        Meter::Voltmeter.wire(&mut world, middle, voltmeter);
        let rows = |world: &hecs::World| -> Vec<Entity> {
            measured(world, voltmeter)
                .into_iter()
                .map(|(_, neuron)| neuron)
                .collect()
        };
        assert_eq!(rows(&world), [high, middle, low]);

        world.get::<&mut Position>(low).unwrap().position.x = 10.0;
        assert_eq!(rows(&world), [low, high, middle]);
    }

    #[test]
    fn spike_detector_rows_follow_the_order_of_connection_when_asked() {
        let mut world = hecs::World::new();
        let neurons = [-5.0, 5.0, 0.0].map(|up| {
            crate::elements::Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(up, 0.0, 0.0))
        });
        let detector =
            Meter::SpikeDetector.spawn_measuring(&mut world, neurons[0], Vec3::new(0.0, 0.0, 10.0));
        for &neuron in &neurons[1..] {
            Meter::SpikeDetector.wire(&mut world, neuron, detector);
        }
        let rows = |world: &hecs::World| -> Vec<Entity> {
            measured(world, detector)
                .into_iter()
                .map(|(_, neuron)| neuron)
                .collect()
        };
        assert_eq!(rows(&world), [neurons[1], neurons[2], neurons[0]]);

        *world.get::<&mut RowOrder>(detector).unwrap() = RowOrder::Connection;
        assert_eq!(rows(&world), neurons);
    }

    #[test]
    fn the_position_is_the_middle_of_the_left_edge() {
        let frame = frame();
        assert_eq!(frame.corner(Corner::BottomLeft), Vec3::new(8.0, 0.0, 5.0));
        assert_eq!(frame.corner(Corner::TopRight), Vec3::new(12.0, 0.0, 13.0));
        assert_eq!(frame.position(), Vec3::new(10.0, 0.0, 5.0));
    }

    #[test]
    fn dragging_a_corner_keeps_the_opposite_corner_in_place() {
        let frame = frame();
        for corner in Corner::ALL {
            let dragged = frame.corner(corner) + Vec3::new(0.5, 0.0, -0.5);
            let resized = frame.resized(corner, dragged);
            let opposite = corner.opposite();
            assert_eq!(resized.corner(opposite), frame.corner(opposite));
            assert_eq!(resized.corner(corner), dragged);
        }
    }

    #[test]
    fn frames_keep_a_minimum_size() {
        let frame = frame();
        let resized = frame.resized(Corner::TopRight, frame.bottom_left);
        assert_eq!(resized.size().width, MIN_FRAME_WIDTH);
        assert_eq!(resized.size().height, MIN_FRAME_HEIGHT);
        assert_eq!(resized.bottom_left, frame.bottom_left);
    }
}
