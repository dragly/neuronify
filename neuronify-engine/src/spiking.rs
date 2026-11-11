use std::f64::consts::PI;

use crate::units::{Microsiemens, Milliseconds, Millivolts};

pub const SPIKE_PEAK: Millivolts = Millivolts(30.0);
pub const SPIKE_RISE: Milliseconds = Milliseconds(0.25);
pub const SPIKE_DURATION: Milliseconds = Milliseconds(1.0);
pub const CLAMP_CONDUCTANCE: Microsiemens = Microsiemens(1000.0);
pub const KICK_VOLTAGE: Millivolts = SPIKE_PEAK;
pub const KICK_DURATION: Milliseconds = Milliseconds(0.5);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spiking {
    pub threshold: Millivolts,
    pub reset: Millivolts,
    pub refractory: Milliseconds,
    pub bounds: Option<(Millivolts, Millivolts)>,
}

impl Spiking {
    pub fn target(&self, since: Milliseconds) -> Option<Millivolts> {
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
    pub since: Milliseconds,
    pub forced: bool,
    pub clamped: bool,
    pub before: Millivolts,
}

pub(crate) struct Kick {
    pub point: usize,
    pub remaining: Milliseconds,
}

pub(crate) struct Detector {
    pub source: usize,
    pub point: usize,
    pub threshold: Millivolts,
    pub before: Millivolts,
    pub since: Milliseconds,
}

pub(crate) fn crossing(
    threshold: Millivolts,
    before: Millivolts,
    after: Millivolts,
) -> Option<f64> {
    match (before < threshold, after >= threshold) {
        (true, true) => Some((threshold - before) / (after - before)),
        (false, true) => Some(0.0),
        _ => None,
    }
}
