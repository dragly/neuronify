use chrono::{DateTime, Utc};

const FPS_LOW_PASS_FACTOR: f64 = 0.05;
#[cfg(not(target_arch = "wasm32"))]
const TARGET_FRAME_MS: i64 = 16;
const MAX_STEPPED_SECONDS_PER_FRAME: f64 = 0.1;

pub struct FrameClock {
    last_frame: DateTime<Utc>,
    fps: f64,
}

impl Default for FrameClock {
    fn default() -> FrameClock {
        FrameClock {
            last_frame: Utc::now(),
            fps: 60.0,
        }
    }
}

impl FrameClock {
    pub fn fps(&self) -> f64 {
        self.fps
    }

    pub fn tick(&mut self) -> f64 {
        #[cfg(not(target_arch = "wasm32"))]
        self.wait_for_next_frame();
        let now = Utc::now();
        let seconds = (now - self.last_frame).num_nanoseconds().unwrap() as f64 * 1e-9;
        let fps = 1.0 / seconds.max(1e-7);
        self.fps = (1.0 - FPS_LOW_PASS_FACTOR) * self.fps + FPS_LOW_PASS_FACTOR * fps;
        self.last_frame = now;
        seconds
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn wait_for_next_frame(&self) {
        let remaining =
            chrono::Duration::milliseconds(TARGET_FRAME_MS) - (Utc::now() - self.last_frame);
        if remaining > chrono::Duration::zero() {
            std::thread::sleep(std::time::Duration::from_millis(
                remaining.num_milliseconds() as u64,
            ));
        }
    }
}

#[derive(Default)]
pub struct StepClock {
    pending_steps: f64,
}

impl StepClock {
    pub fn steps(&mut self, seconds: f64, steps_per_second: f64) -> u32 {
        self.pending_steps += seconds.min(MAX_STEPPED_SECONDS_PER_FRAME) * steps_per_second;
        let steps = self.pending_steps.floor();
        self.pending_steps -= steps;
        steps as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps_in_one_second(fps: u32, steps_per_second: f64) -> u32 {
        let mut clock = StepClock::default();
        (0..fps)
            .map(|_| clock.steps(1.0 / fps as f64, steps_per_second))
            .sum()
    }

    #[test]
    fn steps_do_not_depend_on_the_frame_rate() {
        for fps in [30, 60, 120, 144] {
            let steps = steps_in_one_second(fps, 62.5);
            assert!((61..=62).contains(&steps), "{steps} steps at {fps} fps");
        }
    }

    #[test]
    fn fewer_steps_than_frames_skip_frames() {
        let mut clock = StepClock::default();
        let steps: Vec<u32> = (0..4).map(|_| clock.steps(1.0 / 64.0, 32.0)).collect();
        assert_eq!(steps, [0, 1, 0, 1]);
    }

    #[test]
    fn a_long_pause_does_not_catch_up() {
        let mut clock = StepClock::default();
        assert_eq!(clock.steps(10.0, 100.0), 10);
    }
}
