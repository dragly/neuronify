use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{Touch, TouchPhase};

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub left_down: bool,
    pub position: Option<PhysicalPosition<f64>>,
    pub delta_position: Option<PhysicalPosition<f64>>,
}

pub struct Keyboard {
    pub shift_down: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchGesture {
    Press(PhysicalPosition<f64>),
    Drag(PhysicalPosition<f64>),
    Release,
    Pinch {
        previous: [PhysicalPosition<f64>; 2],
        current: [PhysicalPosition<f64>; 2],
    },
    None,
}

#[derive(Debug, Default)]
pub struct Touches {
    fingers: Vec<(u64, PhysicalPosition<f64>)>,
    pinched: bool,
}

impl Touches {
    pub fn update(&mut self, touch: &Touch) -> TouchGesture {
        let index = self.fingers.iter().position(|(id, _)| *id == touch.id);
        match (touch.phase, index) {
            (TouchPhase::Started, None) => {
                self.fingers.push((touch.id, touch.location));
                match self.fingers.len() {
                    1 => TouchGesture::Press(touch.location),
                    2 => {
                        self.pinched = true;
                        TouchGesture::Release
                    }
                    _ => TouchGesture::None,
                }
            }
            (TouchPhase::Started, Some(_)) => TouchGesture::None,
            (TouchPhase::Moved, Some(index)) => {
                let previous = self.pinch_points();
                self.fingers[index].1 = touch.location;
                match (previous, self.pinch_points()) {
                    (Some(previous), Some(current)) if index < 2 => {
                        TouchGesture::Pinch { previous, current }
                    }
                    _ if !self.pinched => TouchGesture::Drag(touch.location),
                    _ => TouchGesture::None,
                }
            }
            (TouchPhase::Ended | TouchPhase::Cancelled, Some(index)) => {
                self.fingers.remove(index);
                let released = self.fingers.is_empty() && !self.pinched;
                if self.fingers.is_empty() {
                    self.pinched = false;
                }
                if released {
                    TouchGesture::Release
                } else {
                    TouchGesture::None
                }
            }
            (_, None) => TouchGesture::None,
        }
    }

    fn pinch_points(&self) -> Option<[PhysicalPosition<f64>; 2]> {
        match self.fingers.as_slice() {
            [(_, a), (_, b), ..] => Some([*a, *b]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use visula::winit::event::DeviceId;

    fn touch(id: u64, phase: TouchPhase, x: f64, y: f64) -> Touch {
        Touch {
            device_id: DeviceId::dummy(),
            phase,
            location: PhysicalPosition::new(x, y),
            force: None,
            id,
        }
    }

    fn at(x: f64, y: f64) -> PhysicalPosition<f64> {
        PhysicalPosition::new(x, y)
    }

    #[test]
    fn one_finger_acts_like_the_left_mouse_button() {
        let mut touches = Touches::default();
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Started, 10.0, 20.0)),
            TouchGesture::Press(at(10.0, 20.0))
        );
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Moved, 15.0, 25.0)),
            TouchGesture::Drag(at(15.0, 25.0))
        );
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Ended, 15.0, 25.0)),
            TouchGesture::Release
        );
    }

    #[test]
    fn second_finger_turns_a_drag_into_a_pinch() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0));
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0)),
            TouchGesture::Release
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 50.0, 10.0)),
            TouchGesture::Pinch {
                previous: [at(10.0, 10.0), at(30.0, 10.0)],
                current: [at(10.0, 10.0), at(50.0, 10.0)],
            }
        );
    }

    #[test]
    fn finger_left_after_a_pinch_does_nothing() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0));
        touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0));
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Ended, 10.0, 10.0)),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 60.0, 10.0)),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Ended, 60.0, 10.0)),
            TouchGesture::None
        );
    }

    #[test]
    fn touch_after_a_pinch_presses_again() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0));
        touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0));
        touches.update(&touch(1, TouchPhase::Ended, 10.0, 10.0));
        touches.update(&touch(2, TouchPhase::Ended, 30.0, 10.0));
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Started, 5.0, 5.0)),
            TouchGesture::Press(at(5.0, 5.0))
        );
    }

    #[test]
    fn third_finger_is_ignored() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 0.0, 0.0));
        touches.update(&touch(2, TouchPhase::Started, 10.0, 0.0));
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Started, 20.0, 0.0)),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Moved, 25.0, 0.0)),
            TouchGesture::None
        );
    }

    #[test]
    fn unknown_fingers_are_ignored() {
        let mut touches = Touches::default();
        assert_eq!(
            touches.update(&touch(7, TouchPhase::Moved, 1.0, 1.0)),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(7, TouchPhase::Ended, 1.0, 1.0)),
            TouchGesture::None
        );
    }

    #[test]
    fn cancelled_touch_releases() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 0.0, 0.0));
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Cancelled, 0.0, 0.0)),
            TouchGesture::Release
        );
    }
}
