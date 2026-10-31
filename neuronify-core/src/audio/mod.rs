#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::f32::consts::TAU;

use crate::measurement::loudspeaker::Sound;

#[cfg(not(target_arch = "wasm32"))]
use native::Output;
#[cfg(target_arch = "wasm32")]
use web::Output;

const MAXIMUM_PLAYS_PER_FRAME: usize = 16;

pub fn synthesize(sound: Sound, sample_rate: f32) -> Vec<f32> {
    let duration = match sound {
        Sound::Click => 0.004,
        Sound::Pop => 0.08,
        Sound::Tone => 0.25,
        Sound::Drop => 0.14,
    };
    let length = (duration * sample_rate).ceil() as usize;
    let mut noise = 0x2545_f491_u32;
    let mut phase = 0.0_f32;
    (0..length)
        .map(|index| {
            let time = index as f32 / sample_rate;
            let progress = index as f32 / length as f32;
            let sample = match sound {
                Sound::Click => {
                    noise ^= noise << 13;
                    noise ^= noise >> 17;
                    noise ^= noise << 5;
                    let white = noise as f32 / u32::MAX as f32 * 2.0 - 1.0;
                    0.8 * white * (-5.0 * progress).exp()
                }
                Sound::Pop => {
                    phase +=
                        TAU * 420.0 * (90.0_f32 / 420.0).powf(progress.min(0.75)) / sample_rate;
                    0.6 * phase.sin() * (-6.0 * progress).exp()
                }
                Sound::Tone => {
                    let attack = (time / 0.005).min(1.0);
                    0.4 * (TAU * 880.0 * time).sin() * attack * (-5.0 * progress).exp()
                }
                Sound::Drop => {
                    phase += TAU * 1400.0 * (500.0_f32 / 1400.0).powf(progress) / sample_rate;
                    0.5 * phase.sin() * (-5.0 * progress).exp()
                }
            };
            sample * (1.0 - progress).min(1.0).powf(0.25)
        })
        .collect()
}

pub fn schedule(
    heard: Vec<(Sound, f32, f64)>,
    start: f64,
    end: f64,
    seconds: f64,
) -> Vec<(Sound, f32, f64)> {
    heard
        .into_iter()
        .map(|(sound, volume, time)| {
            let progress = ((time - start) / (end - start).max(f64::EPSILON)).clamp(0.0, 1.0);
            (sound, volume, progress * seconds)
        })
        .collect()
}

#[derive(Default)]
pub struct Speakers {
    output: Option<Output>,
    opened: bool,
}

impl Speakers {
    fn open(&mut self) {
        if !self.opened {
            self.opened = true;
            self.output = Output::open();
        }
    }

    pub fn unlock(&mut self, needed: bool) {
        if needed || cfg!(target_arch = "wasm32") {
            self.open();
        }
        if let Some(output) = &self.output {
            output.unlock();
        }
    }

    pub fn play(&mut self, plays: &[(Sound, f32, f64)]) {
        if plays.is_empty() {
            return;
        }
        self.open();
        let Some(output) = &mut self.output else {
            return;
        };
        for &(sound, volume, delay) in plays.iter().take(MAXIMUM_PLAYS_PER_FRAME) {
            output.play(sound, volume, delay);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_short_quiet_enough_and_ends_in_silence() {
        for sound in Sound::ALL {
            let samples = synthesize(sound, 48_000.0);
            assert!(!samples.is_empty(), "{sound:?}");
            assert!(samples.len() <= 48_000 / 3, "{sound:?}");
            assert!(
                samples.iter().all(|sample| sample.abs() <= 1.0),
                "{sound:?}"
            );
            assert!(samples.last().unwrap().abs() < 0.05, "{sound:?}");
            let loudest = samples
                .iter()
                .fold(0.0_f32, |loudest, s| loudest.max(s.abs()));
            assert!(loudest > 0.1, "{sound:?} is silent");
        }
    }

    #[test]
    fn spikes_in_a_frame_play_spread_over_the_frame() {
        let heard = vec![(Sound::Click, 1.0, 1.0), (Sound::Pop, 0.5, 1.05)];
        let plays = schedule(heard, 1.0, 1.1, 0.02);
        assert_eq!(plays[0], (Sound::Click, 1.0, 0.0));
        assert_eq!(plays[1].0, Sound::Pop);
        assert!((plays[1].2 - 0.01).abs() < 1e-9);
    }

    #[test]
    fn sounds_are_the_same_every_time() {
        for sound in Sound::ALL {
            assert_eq!(synthesize(sound, 44_100.0), synthesize(sound, 44_100.0));
        }
    }
}
