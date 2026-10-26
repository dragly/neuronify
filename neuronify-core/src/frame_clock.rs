use chrono::{DateTime, Utc};

const FPS_LOW_PASS_FACTOR: f64 = 0.05;
#[cfg(not(target_arch = "wasm32"))]
const TARGET_FRAME_MS: i64 = 16;

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

    pub fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.wait_for_next_frame();
        let seconds = (Utc::now() - self.last_frame).num_nanoseconds().unwrap() as f64 * 1e-9;
        let fps = 1.0 / seconds.max(1e-7);
        self.fps = (1.0 - FPS_LOW_PASS_FACTOR) * self.fps + FPS_LOW_PASS_FACTOR * fps;
        self.last_frame = Utc::now();
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
