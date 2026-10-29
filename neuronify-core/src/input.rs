use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{Touch, TouchPhase};

const TAP_SLOP: f64 = 8.0;
const DOUBLE_TAP_SLOP: f64 = 40.0;
const DOUBLE_TAP_DELAY: f64 = 0.3;
const SECOND_FINGER_DELAY: f64 = 0.12;
const HOLD: f64 = 0.33;

pub fn now() -> f64 {
    chrono::Utc::now().timestamp_micros() as f64 * 1e-6
}

fn distance(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mouse {
    pub left_down: bool,
    pub position: Option<PhysicalPosition<f64>>,
    tap_start: Option<(PhysicalPosition<f64>, f64)>,
}

impl Mouse {
    pub fn press(&mut self, pressed: bool, now: f64) -> bool {
        let tapped = !pressed && self.tap_start.take().is_some();
        self.left_down = pressed;
        if pressed {
            self.tap_start = self.position.map(|position| (position, now));
        }
        tapped
    }

    pub fn take_hold(&mut self, now: f64) -> Option<PhysicalPosition<f64>> {
        match self.tap_start {
            Some((_, since)) if self.left_down && now - since >= HOLD => {
                self.tap_start = None;
                self.position
            }
            _ => None,
        }
    }

    pub fn move_to(&mut self, position: PhysicalPosition<f64>, scale_factor: f64) {
        self.position = Some(position);
        let moved_away = self
            .tap_start
            .is_some_and(|(start, _)| distance(start, position) > TAP_SLOP * scale_factor);
        if moved_away {
            self.tap_start = None;
        }
    }

    pub fn cancel(&mut self) {
        self.left_down = false;
        self.tap_start = None;
    }
}

#[derive(Default)]
pub struct Keyboard {
    pub shift_down: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchGesture {
    Press(PhysicalPosition<f64>),
    Drag(PhysicalPosition<f64>),
    Release,
    Tap(PhysicalPosition<f64>),
    Cancel,
    Pinch {
        previous: [PhysicalPosition<f64>; 2],
        current: [PhysicalPosition<f64>; 2],
    },
    Zoom {
        anchor: PhysicalPosition<f64>,
        previous: PhysicalPosition<f64>,
        current: PhysicalPosition<f64>,
    },
    None,
}

#[derive(Clone, Copy, Debug)]
enum Stroke {
    Still {
        start: PhysicalPosition<f64>,
        since: f64,
        second_tap: bool,
    },
    Zoom {
        anchor: PhysicalPosition<f64>,
        last: PhysicalPosition<f64>,
    },
}

#[derive(Debug, Default)]
pub struct Touches {
    fingers: Vec<(u64, PhysicalPosition<f64>)>,
    pinched: bool,
    waiting: Option<(PhysicalPosition<f64>, f64)>,
    stroke: Option<Stroke>,
    last_tap: Option<(PhysicalPosition<f64>, f64)>,
}

impl Touches {
    pub fn update(
        &mut self,
        touch: &Touch,
        now: f64,
        wait: bool,
        scale_factor: f64,
    ) -> TouchGesture {
        let index = self.fingers.iter().position(|(id, _)| *id == touch.id);
        match (touch.phase, index) {
            (TouchPhase::Started, None) => {
                self.fingers.push((touch.id, touch.location));
                match self.fingers.len() {
                    1 => {
                        let second_tap = self.last_tap.take().is_some_and(|(tap, at)| {
                            now - at <= DOUBLE_TAP_DELAY
                                && distance(tap, touch.location) <= DOUBLE_TAP_SLOP * scale_factor
                        });
                        self.stroke = Some(Stroke::Still {
                            start: touch.location,
                            since: now,
                            second_tap,
                        });
                        if wait {
                            self.waiting = Some((touch.location, now));
                            TouchGesture::None
                        } else {
                            TouchGesture::Press(touch.location)
                        }
                    }
                    2 => {
                        self.pinched = true;
                        self.stroke = None;
                        match self.waiting.take() {
                            Some(_) => TouchGesture::None,
                            None => TouchGesture::Cancel,
                        }
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
                    _ if self.pinched => TouchGesture::None,
                    _ => self.move_stroke(touch.location, now, scale_factor),
                }
            }
            (TouchPhase::Ended | TouchPhase::Cancelled, Some(index)) => {
                self.fingers.remove(index);
                let released = self.fingers.is_empty() && !self.pinched;
                if self.fingers.is_empty() {
                    self.pinched = false;
                }
                match (self.stroke.take(), touch.phase) {
                    (Some(Stroke::Zoom { .. }), _) => return TouchGesture::None,
                    (Some(Stroke::Still { start, since, .. }), TouchPhase::Ended)
                        if now - since < HOLD =>
                    {
                        self.last_tap = Some((start, now));
                    }
                    _ => {}
                }
                match (released, touch.phase, self.waiting.take()) {
                    (false, _, _) | (true, TouchPhase::Cancelled, Some(_)) => TouchGesture::None,
                    (true, TouchPhase::Cancelled, None) => TouchGesture::Cancel,
                    (true, _, Some((start, _))) => TouchGesture::Tap(start),
                    (true, _, None) => TouchGesture::Release,
                }
            }
            (_, None) => TouchGesture::None,
        }
    }

    pub fn poll(&mut self, now: f64) -> TouchGesture {
        let delay = match self.stroke {
            Some(Stroke::Still {
                second_tap: true, ..
            }) => HOLD,
            _ => SECOND_FINGER_DELAY,
        };
        match self.waiting {
            Some((start, since)) if now - since >= delay => {
                self.waiting = None;
                TouchGesture::Press(start)
            }
            _ => TouchGesture::None,
        }
    }

    pub fn may_double_tap(&self, now: f64) -> bool {
        match self.stroke {
            Some(Stroke::Zoom { .. }) => true,
            Some(Stroke::Still {
                second_tap: true,
                since,
                ..
            }) => now - since < HOLD,
            _ => self
                .last_tap
                .is_some_and(|(_, at)| now - at <= DOUBLE_TAP_DELAY),
        }
    }

    pub fn forget_tap(&mut self) {
        self.last_tap = None;
    }

    pub fn forget_fingers(&mut self) {
        *self = Touches {
            last_tap: self.last_tap,
            ..Touches::default()
        };
    }

    fn move_stroke(
        &mut self,
        location: PhysicalPosition<f64>,
        now: f64,
        scale_factor: f64,
    ) -> TouchGesture {
        let near = |start| distance(start, location) <= TAP_SLOP * scale_factor;
        match self.stroke {
            Some(Stroke::Zoom { anchor, last }) => {
                self.stroke = Some(Stroke::Zoom {
                    anchor,
                    last: location,
                });
                TouchGesture::Zoom {
                    anchor,
                    previous: last,
                    current: location,
                }
            }
            Some(Stroke::Still { start, .. }) if near(start) => self.drag(location),
            Some(Stroke::Still {
                since,
                second_tap: true,
                ..
            }) if now - since < HOLD => {
                self.waiting = None;
                self.stroke = Some(Stroke::Zoom {
                    anchor: location,
                    last: location,
                });
                TouchGesture::Cancel
            }
            _ => {
                self.stroke = None;
                self.drag(location)
            }
        }
    }

    fn drag(&self, location: PhysicalPosition<f64>) -> TouchGesture {
        match self.waiting {
            Some(_) => TouchGesture::None,
            None => TouchGesture::Drag(location),
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
            touches.update(&touch(1, TouchPhase::Started, 10.0, 20.0), 0.0, false, 1.0),
            TouchGesture::Press(at(10.0, 20.0))
        );
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Moved, 15.0, 25.0), 0.0, false, 1.0),
            TouchGesture::Drag(at(15.0, 25.0))
        );
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Ended, 15.0, 25.0), 0.0, false, 1.0),
            TouchGesture::Release
        );
    }

    #[test]
    fn second_finger_turns_a_drag_into_a_pinch() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, false, 1.0);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0), 0.0, false, 1.0),
            TouchGesture::Cancel
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 50.0, 10.0), 0.0, false, 1.0),
            TouchGesture::Pinch {
                previous: [at(10.0, 10.0), at(30.0, 10.0)],
                current: [at(10.0, 10.0), at(50.0, 10.0)],
            }
        );
    }

    #[test]
    fn finger_left_after_a_pinch_does_nothing() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, false, 1.0);
        touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0), 0.0, false, 1.0);
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Ended, 10.0, 10.0), 0.0, false, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 60.0, 10.0), 0.0, false, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Ended, 60.0, 10.0), 0.0, false, 1.0),
            TouchGesture::None
        );
    }

    #[test]
    fn touch_after_a_pinch_presses_again() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, false, 1.0);
        touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0), 0.0, false, 1.0);
        touches.update(&touch(1, TouchPhase::Ended, 10.0, 10.0), 0.0, false, 1.0);
        touches.update(&touch(2, TouchPhase::Ended, 30.0, 10.0), 0.0, false, 1.0);
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Started, 5.0, 5.0), 0.0, false, 1.0),
            TouchGesture::Press(at(5.0, 5.0))
        );
    }

    #[test]
    fn third_finger_is_ignored() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 0.0, 0.0), 0.0, false, 1.0);
        touches.update(&touch(2, TouchPhase::Started, 10.0, 0.0), 0.0, false, 1.0);
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Started, 20.0, 0.0), 0.0, false, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(3, TouchPhase::Moved, 25.0, 0.0), 0.0, false, 1.0),
            TouchGesture::None
        );
    }

    #[test]
    fn unknown_fingers_are_ignored() {
        let mut touches = Touches::default();
        assert_eq!(
            touches.update(&touch(7, TouchPhase::Moved, 1.0, 1.0), 0.0, false, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(7, TouchPhase::Ended, 1.0, 1.0), 0.0, false, 1.0),
            TouchGesture::None
        );
    }

    #[test]
    fn cancelled_touch_cancels() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 0.0, 0.0), 0.0, false, 1.0);
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Cancelled, 0.0, 0.0), 0.0, false, 1.0),
            TouchGesture::Cancel
        );
    }

    #[test]
    fn while_editing_a_press_waits_for_a_second_finger() {
        let mut touches = Touches::default();
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, true, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Moved, 12.0, 10.0), 0.02, true, 1.0),
            TouchGesture::None
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 30.0, 10.0), 0.05, true, 1.0),
            TouchGesture::None
        );
        assert_eq!(touches.poll(1.0), TouchGesture::None);
        assert!(matches!(
            touches.update(&touch(2, TouchPhase::Moved, 50.0, 10.0), 1.0, true, 1.0),
            TouchGesture::Pinch { .. }
        ));
    }

    #[test]
    fn a_waiting_press_goes_through_after_the_delay() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, true, 1.0);
        assert_eq!(touches.poll(0.05), TouchGesture::None);
        assert_eq!(touches.poll(0.2), TouchGesture::Press(at(10.0, 10.0)));
        assert_eq!(touches.poll(0.3), TouchGesture::None);
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Moved, 20.0, 10.0), 0.3, true, 1.0),
            TouchGesture::Drag(at(20.0, 10.0))
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 40.0, 10.0), 0.4, true, 1.0),
            TouchGesture::Cancel
        );
    }

    #[test]
    fn a_quick_touch_while_waiting_is_a_tap() {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, true, 1.0);
        assert_eq!(
            touches.update(&touch(1, TouchPhase::Ended, 11.0, 10.0), 0.05, true, 1.0),
            TouchGesture::Tap(at(10.0, 10.0))
        );
        assert_eq!(touches.poll(1.0), TouchGesture::None);
    }

    fn tapped_at(x: f64, y: f64, wait: bool) -> Touches {
        let mut touches = Touches::default();
        touches.update(&touch(1, TouchPhase::Started, x, y), 0.0, wait, 1.0);
        touches.update(&touch(1, TouchPhase::Ended, x, y), 0.1, wait, 1.0);
        touches
    }

    #[test]
    fn dragging_the_second_tap_of_a_double_tap_zooms() {
        let mut touches = tapped_at(10.0, 10.0, false);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 12.0, 10.0), 0.3, false, 1.0),
            TouchGesture::Press(at(12.0, 10.0))
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 12.0, 15.0), 0.32, false, 1.0),
            TouchGesture::Drag(at(12.0, 15.0))
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 12.0, 30.0), 0.35, false, 1.0),
            TouchGesture::Cancel
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 12.0, 50.0), 0.4, false, 1.0),
            TouchGesture::Zoom {
                anchor: at(12.0, 30.0),
                previous: at(12.0, 30.0),
                current: at(12.0, 50.0),
            }
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 12.0, 40.0), 0.45, false, 1.0),
            TouchGesture::Zoom {
                anchor: at(12.0, 30.0),
                previous: at(12.0, 50.0),
                current: at(12.0, 40.0),
            }
        );
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Ended, 12.0, 40.0), 0.5, false, 1.0),
            TouchGesture::None
        );
    }

    #[test]
    fn while_editing_the_second_tap_waits_until_it_zooms() {
        let mut touches = tapped_at(10.0, 10.0, true);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, true, 1.0),
            TouchGesture::None
        );
        assert_eq!(touches.poll(0.5), TouchGesture::None);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), 0.5, true, 1.0),
            TouchGesture::Cancel
        );
        assert!(matches!(
            touches.update(&touch(2, TouchPhase::Moved, 10.0, 50.0), 0.55, true, 1.0),
            TouchGesture::Zoom { .. }
        ));
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Ended, 10.0, 50.0), 0.6, true, 1.0),
            TouchGesture::None
        );
        assert_eq!(touches.poll(1.0), TouchGesture::None);
    }

    #[test]
    fn a_double_tap_without_dragging_is_two_taps() {
        let mut touches = tapped_at(10.0, 10.0, true);
        touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, true, 1.0);
        assert_eq!(touches.poll(0.5), TouchGesture::None);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Ended, 10.0, 10.0), 0.5, true, 1.0),
            TouchGesture::Tap(at(10.0, 10.0))
        );
    }

    #[test]
    fn holding_the_second_tap_still_ends_the_double_tap() {
        let mut touches = tapped_at(10.0, 10.0, true);
        touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, true, 1.0);
        assert_eq!(touches.poll(0.7), TouchGesture::Press(at(10.0, 10.0)));
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), 0.75, true, 1.0),
            TouchGesture::Drag(at(10.0, 30.0))
        );
    }

    #[test]
    fn a_late_or_distant_second_tap_drags() {
        for (x, now) in [(10.0, 0.5), (60.0, 0.3)] {
            let mut touches = tapped_at(10.0, 10.0, false);
            touches.update(&touch(2, TouchPhase::Started, x, 10.0), now, false, 1.0);
            assert_eq!(
                touches.update(&touch(2, TouchPhase::Moved, x, 30.0), now, false, 1.0),
                TouchGesture::Drag(at(x, 30.0))
            );
        }
    }

    #[test]
    fn the_double_tap_slop_follows_the_scale_factor() {
        let mut touches = tapped_at(10.0, 10.0, false);
        touches.update(&touch(2, TouchPhase::Started, 60.0, 10.0), 0.3, false, 2.0);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 60.0, 50.0), 0.35, false, 2.0),
            TouchGesture::Cancel
        );
    }

    #[test]
    fn a_drag_or_a_hold_is_no_first_tap() {
        let mut dragged = Touches::default();
        dragged.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, false, 1.0);
        dragged.update(&touch(1, TouchPhase::Moved, 10.0, 30.0), 0.05, false, 1.0);
        dragged.update(&touch(1, TouchPhase::Ended, 10.0, 10.0), 0.1, false, 1.0);
        let mut held = Touches::default();
        held.update(&touch(1, TouchPhase::Started, 10.0, 10.0), 0.0, false, 1.0);
        held.update(&touch(1, TouchPhase::Ended, 10.0, 10.0), 0.5, false, 1.0);
        for (mut touches, now) in [(dragged, 0.3), (held, 0.7)] {
            touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), now, false, 1.0);
            assert_eq!(
                touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), now, false, 1.0),
                TouchGesture::Drag(at(10.0, 30.0))
            );
        }
    }

    #[test]
    fn a_double_tap_stays_possible_until_the_second_touch_settles() {
        let mut touches = tapped_at(10.0, 10.0, false);
        assert!(touches.may_double_tap(0.3));
        assert!(!touches.may_double_tap(0.5));
        touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, false, 1.0);
        assert!(touches.may_double_tap(0.5));
        assert!(!touches.may_double_tap(0.7));
        touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), 0.4, false, 1.0);
        assert!(touches.may_double_tap(1.0));
        touches.update(&touch(2, TouchPhase::Ended, 10.0, 30.0), 1.0, false, 1.0);
        assert!(!touches.may_double_tap(1.0));
    }

    #[test]
    fn a_forgotten_tap_starts_no_double_tap() {
        let mut touches = tapped_at(10.0, 10.0, false);
        touches.forget_tap();
        touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, false, 1.0);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), 0.35, false, 1.0),
            TouchGesture::Drag(at(10.0, 30.0))
        );
    }

    #[test]
    fn forgetting_the_fingers_keeps_the_last_tap() {
        let mut touches = tapped_at(10.0, 10.0, false);
        touches.forget_fingers();
        touches.update(&touch(2, TouchPhase::Started, 10.0, 10.0), 0.3, false, 1.0);
        assert_eq!(
            touches.update(&touch(2, TouchPhase::Moved, 10.0, 30.0), 0.35, false, 1.0),
            TouchGesture::Cancel
        );
    }

    fn pressed_at(x: f64, y: f64) -> Mouse {
        let mut mouse = Mouse::default();
        mouse.move_to(at(x, y), 1.0);
        mouse.press(true, 0.0);
        mouse
    }

    #[test]
    fn a_press_released_where_it_started_is_a_tap() {
        let mut mouse = pressed_at(10.0, 10.0);
        mouse.move_to(at(14.0, 15.0), 1.0);
        assert!(mouse.press(false, 0.0));
        assert!(!mouse.press(false, 0.0));
    }

    #[test]
    fn moving_away_turns_a_tap_into_a_drag() {
        let mut mouse = pressed_at(10.0, 10.0);
        mouse.move_to(at(30.0, 10.0), 1.0);
        mouse.move_to(at(10.0, 10.0), 1.0);
        assert!(!mouse.press(false, 0.0));
    }

    #[test]
    fn the_tap_slop_follows_the_scale_factor() {
        let mut mouse = pressed_at(10.0, 10.0);
        mouse.move_to(at(22.0, 10.0), 2.0);
        assert!(mouse.press(false, 0.0));
    }

    #[test]
    fn keeping_still_is_a_hold_and_no_tap() {
        let mut mouse = pressed_at(10.0, 10.0);
        assert_eq!(mouse.take_hold(0.1), None);
        mouse.move_to(at(12.0, 10.0), 1.0);
        assert_eq!(mouse.take_hold(0.4), Some(at(12.0, 10.0)));
        assert_eq!(mouse.take_hold(0.5), None);
        assert!(!mouse.press(false, 0.6));
    }

    #[test]
    fn moving_away_is_no_hold() {
        let mut mouse = pressed_at(10.0, 10.0);
        mouse.move_to(at(30.0, 10.0), 1.0);
        assert_eq!(mouse.take_hold(1.0), None);
    }

    #[test]
    fn a_cancelled_press_is_no_tap() {
        let mut mouse = pressed_at(10.0, 10.0);
        mouse.cancel();
        assert!(!mouse.left_down);
        assert!(!mouse.press(false, 0.0));
    }
}
