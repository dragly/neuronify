use std::collections::HashMap;

use web_sys::{AudioBuffer, AudioContext};

use super::synthesize;
use crate::measurement::loudspeaker::Sound;

pub struct Output {
    context: AudioContext,
    sounds: HashMap<Sound, AudioBuffer>,
}

impl Output {
    pub fn open() -> Option<Output> {
        let context = AudioContext::new()
            .map_err(|error| log::warn!("No sound: {error:?}"))
            .ok()?;
        let sample_rate = context.sample_rate();
        let sounds = Sound::ALL
            .into_iter()
            .filter_map(|sound| {
                let samples = synthesize(sound, sample_rate);
                let buffer = context
                    .create_buffer(1, samples.len() as u32, sample_rate)
                    .ok()?;
                buffer.copy_to_channel(&samples, 0).ok()?;
                Some((sound, buffer))
            })
            .collect();
        Some(Output { context, sounds })
    }

    pub fn unlock(&self) {
        let _ = self.context.resume();
    }

    pub fn play(&mut self, sound: Sound, volume: f32, delay: f64) {
        if let Err(error) = self.try_play(sound, volume, delay) {
            log::warn!("Could not play a sound: {error:?}");
        }
    }

    fn try_play(&self, sound: Sound, volume: f32, delay: f64) -> Result<(), wasm_bindgen::JsValue> {
        let Some(buffer) = self.sounds.get(&sound) else {
            return Ok(());
        };
        let source = self.context.create_buffer_source()?;
        source.set_buffer(Some(buffer));
        let gain = self.context.create_gain()?;
        gain.gain().set_value(volume);
        source.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.context.destination())?;
        source.start_with_when(self.context.current_time() + delay)
    }
}
