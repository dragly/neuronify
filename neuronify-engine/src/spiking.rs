use std::f64::consts::PI;

pub const SPIKE_PEAK: f64 = 30.0;
pub const SPIKE_RISE: f64 = 0.25;
pub const SPIKE_DURATION: f64 = 1.0;
pub const CLAMP_CONDUCTANCE: f64 = 1000.0;
pub const KICK_VOLTAGE: f64 = SPIKE_PEAK;
pub const KICK_DURATION: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spiking {
    pub threshold: f64,
    pub reset: f64,
    pub refractory: f64,
    pub bounds: Option<(f64, f64)>,
}

impl Spiking {
    pub fn target(&self, since: f64) -> Option<f64> {
        if since < SPIKE_RISE {
            let x = since / SPIKE_RISE;
            Some(self.threshold + (SPIKE_PEAK - self.threshold) * (1.0 - (PI * x).cos()) / 2.0)
        } else if since < SPIKE_DURATION {
            let x = (since - SPIKE_RISE) / (SPIKE_DURATION - SPIKE_RISE);
            Some(self.reset + (SPIKE_PEAK - self.reset) * (1.0 + (PI * x).cos()) / 2.0)
        } else if since < self.refractory {
            Some(self.reset)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crossing {
    pub source: usize,
    pub fraction: f64,
}

pub(crate) struct Soma {
    pub source: usize,
    pub point: usize,
    pub spiking: Spiking,
    pub since: f64,
    pub forced: bool,
    pub clamped: bool,
    pub before: f64,
}

pub(crate) struct Kick {
    pub point: usize,
    pub remaining: f64,
}

pub(crate) struct Detector {
    pub source: usize,
    pub point: usize,
    pub threshold: f64,
    pub before: f64,
}

pub(crate) fn crossing(threshold: f64, before: f64, after: f64) -> Option<f64> {
    match (before < threshold, after >= threshold) {
        (true, true) => Some((threshold - before) / (after - before)),
        (false, true) => Some(0.0),
        _ => None,
    }
}
