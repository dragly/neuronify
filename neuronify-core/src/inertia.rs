use glam::DVec2;
use visula::winit::dpi::PhysicalPosition;

use crate::components::Seconds;

const GLIDE_TIME_CONSTANT: Seconds = Seconds(0.196);
const VELOCITY_WINDOW: Seconds = Seconds(0.05);
const LONGEST_GAP: Seconds = Seconds(0.1);
const SAME_MOMENT: Seconds = Seconds(0.002);
const SLOWEST_POINTS_PER_SECOND: f64 = 50.0;
const FASTEST_POINTS_PER_SECOND: f64 = 8000.0;
const REST_PIXELS: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
struct Glide {
    at: DVec2,
    velocity: DVec2,
    since: Seconds,
}

#[derive(Debug, Default)]
pub struct Inertia {
    samples: Vec<(Seconds, DVec2)>,
    panned: bool,
    glide: Option<Glide>,
}

fn decay_per_second() -> f64 {
    1.0 / GLIDE_TIME_CONSTANT.0
}

fn vector(position: PhysicalPosition<f64>) -> DVec2 {
    DVec2::new(position.x, position.y)
}

fn position(vector: DVec2) -> PhysicalPosition<f64> {
    PhysicalPosition::new(vector.x, vector.y)
}

impl Inertia {
    pub fn press(&mut self, scale_factor: f64) -> bool {
        self.samples.clear();
        self.panned = false;
        self.glide
            .take()
            .is_some_and(|glide| glide.velocity.length() > SLOWEST_POINTS_PER_SECOND * scale_factor)
    }

    pub fn track(&mut self, now: Seconds, at: PhysicalPosition<f64>) {
        self.panned = true;
        self.add(now, vector(at));
    }

    pub fn release(&mut self, now: Seconds, scale_factor: f64) {
        let Some(&(_, last)) = self.samples.last().filter(|_| self.panned) else {
            return;
        };
        self.panned = false;
        self.add(now, last);
        let velocity = self.velocity();
        self.samples.clear();
        let speed = velocity.length();
        if speed < SLOWEST_POINTS_PER_SECOND * scale_factor {
            return;
        }
        let fastest = FASTEST_POINTS_PER_SECOND * scale_factor;
        self.glide = Some(Glide {
            at: last,
            velocity: velocity * (fastest / speed).min(1.0),
            since: now,
        });
    }

    pub fn stop(&mut self) {
        self.glide = None;
    }

    pub fn step(&mut self, now: Seconds) -> Option<(PhysicalPosition<f64>, PhysicalPosition<f64>)> {
        let glide = self.glide.as_mut()?;
        let elapsed = (now - glide.since).max(Seconds::ZERO);
        glide.since = now;
        let decay = decay_per_second();
        let kept = (-decay * elapsed.0).exp();
        let travel = glide.velocity * (1.0 - kept) / decay;
        glide.velocity *= kept;
        let from = glide.at;
        if glide.velocity.length() / decay < REST_PIXELS {
            self.glide = None;
        }
        Some((position(from), position(from + travel)))
    }

    fn add(&mut self, now: Seconds, at: DVec2) {
        match self.samples.last_mut() {
            Some(last) if now - last.0 < SAME_MOMENT => *last = (now, at),
            _ => self.samples.push((now, at)),
        }
        self.samples.retain(|&(time, _)| now - time <= LONGEST_GAP);
    }

    fn velocity(&self) -> DVec2 {
        let [.., previous, (latest, last)] = self.samples[..] else {
            return DVec2::ZERO;
        };
        let (start, first) = self
            .samples
            .iter()
            .copied()
            .find(|&(time, _)| time < latest && latest - time <= VELOCITY_WINDOW)
            .unwrap_or(previous);
        (last - first) / (latest - start).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Seconds = Seconds(1.0 / 60.0);

    fn at(x: f64) -> PhysicalPosition<f64> {
        PhysicalPosition::new(x, 0.0)
    }

    fn pan(inertia: &mut Inertia, speed: f64, frames: usize) -> Seconds {
        inertia.press(1.0);
        let mut now = Seconds::ZERO;
        for frame in 0..=frames {
            now = FRAME * frame as f64;
            inertia.track(now, at(speed * now.0));
        }
        now
    }

    fn glide_to_rest(inertia: &mut Inertia, mut now: Seconds) -> f64 {
        let mut travel = 0.0;
        while let Some((from, to)) = inertia.step(now) {
            travel += to.x - from.x;
            now += FRAME;
        }
        travel
    }

    #[test]
    fn a_released_pan_glides_as_far_as_its_speed_covers_in_0_196_s() {
        let mut inertia = Inertia::default();
        let now = pan(&mut inertia, 1000.0, 10);
        inertia.release(now, 1.0);
        let travel = glide_to_rest(&mut inertia, now + FRAME);
        assert!((travel - 196.0).abs() < 1.0, "{travel}");
    }

    #[test]
    fn the_glide_keeps_a_tenth_of_its_speed_after_0_45_s() {
        let mut inertia = Inertia::default();
        let now = pan(&mut inertia, 1000.0, 10);
        inertia.release(now, 1.0);
        let elapsed = GLIDE_TIME_CONSTANT * 10f64.ln();
        inertia.step(now + elapsed);
        let (from, to) = inertia.step(now + elapsed + Seconds(0.001)).unwrap();
        let speed = (to.x - from.x) / 0.001;
        assert!((speed - 100.0).abs() < 1.0, "{speed}");
    }

    #[test]
    fn a_pause_before_letting_go_leaves_the_view_still() {
        let mut inertia = Inertia::default();
        let now = pan(&mut inertia, 1000.0, 10);
        inertia.release(now + Seconds(0.06), 1.0);
        assert_eq!(inertia.step(now + Seconds(0.1)), None);
    }

    #[test]
    fn letting_go_without_panning_does_not_glide() {
        let mut inertia = Inertia::default();
        inertia.press(1.0);
        inertia.release(FRAME, 1.0);
        assert_eq!(inertia.step(FRAME * 2.0), None);
    }

    #[test]
    fn a_press_stops_the_glide_and_catches_it_only_while_it_moves_visibly() {
        let mut inertia = Inertia::default();
        let now = pan(&mut inertia, 1000.0, 10);
        inertia.release(now, 1.0);
        inertia.step(now + Seconds(0.1));
        assert!(inertia.press(1.0));
        assert_eq!(inertia.step(now + Seconds(0.2)), None);

        let now = pan(&mut inertia, 1000.0, 10);
        inertia.release(now, 1.0);
        inertia.step(now + Seconds(0.8));
        assert!(!inertia.press(1.0));
    }
}
