use std::collections::VecDeque;

use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::components::{Position, VoltmeterSize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollingWindow<T> {
    data: VecDeque<T>,
    max_capacity: usize,
}

impl<T> RollingWindow<T> {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        while !self.data.is_empty() && self.data.len() >= self.max_capacity {
            self.data.pop_front();
        }
        if self.data.len() < self.max_capacity {
            self.data.push_back(value);
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator + '_ {
        self.data.iter()
    }
    pub fn last(&self) -> Option<&T> {
        self.data.back()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageMeasurement {
    pub voltage: f64,
    pub time: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageSeries {
    pub measurements: RollingWindow<VoltageMeasurement>,
    #[serde(default)]
    pub spike_times: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Voltmeter {}

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
    pub fn new(position: Vec3, size: &VoltmeterSize) -> Frame {
        Frame {
            bottom_left: position - Vec3::new(size.height * 0.5, 0.0, 0.0),
            width: size.width,
            height: size.height,
        }
    }

    pub fn of(world: &hecs::World, voltmeter: Entity) -> Option<Frame> {
        let position = world.get::<&Position>(voltmeter).ok()?;
        let size = world.get::<&VoltmeterSize>(voltmeter).ok()?;
        Some(Frame::new(position.position, &size))
    }

    pub fn position(&self) -> Vec3 {
        self.bottom_left + Vec3::new(self.height * 0.5, 0.0, 0.0)
    }

    pub fn size(&self) -> VoltmeterSize {
        VoltmeterSize {
            width: self.width,
            height: self.height,
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

    fn frame() -> Frame {
        Frame::new(
            Vec3::new(10.0, 0.0, 5.0),
            &VoltmeterSize {
                width: 8.0,
                height: 4.0,
            },
        )
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

    #[derive(Debug, Deserialize, Serialize)]
    struct RollingWindowBackedByVec {
        data: Vec<u32>,
        max_capacity: usize,
    }

    #[test]
    fn rolling_window_keeps_vec_behaviour_and_format() {
        let mut window = RollingWindow::new(5);
        let mut reference = RollingWindowBackedByVec {
            data: Vec::new(),
            max_capacity: 5,
        };
        for value in 0..12 {
            window.push(value);
            if reference.data.len() == reference.max_capacity {
                reference.data.remove(0);
            }
            reference.data.push(value);

            assert_eq!(window.iter().copied().collect::<Vec<_>>(), reference.data);
            assert_eq!(window.last(), reference.data.last());

            let bytes = postcard::to_stdvec(&window).unwrap();
            assert_eq!(bytes, postcard::to_stdvec(&reference).unwrap());
            let loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(loaded.iter().copied().collect::<Vec<_>>(), reference.data);
        }
    }

    #[test]
    fn rolling_window_stays_within_capacity() {
        let mut empty = RollingWindow::new(0);
        empty.push(1);
        empty.push(2);
        assert!(empty.is_empty());

        let corrupt = RollingWindowBackedByVec {
            data: vec![0, 1],
            max_capacity: 0,
        };
        let bytes = postcard::to_stdvec(&corrupt).unwrap();
        let mut loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
        loaded.push(2);
        assert!(loaded.is_empty());

        let oversized = RollingWindowBackedByVec {
            data: (0..8).collect(),
            max_capacity: 3,
        };
        let bytes = postcard::to_stdvec(&oversized).unwrap();
        let mut loaded: RollingWindow<u32> = postcard::from_bytes(&bytes).unwrap();
        loaded.push(8);
        assert_eq!(loaded.iter().copied().collect::<Vec<_>>(), [6, 7, 8]);
    }
}
