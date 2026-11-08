use std::cell::Cell;
use std::collections::HashMap;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, AudioBuffer, AudioContext, AudioContextState};

use super::synthesize;
use crate::components::Seconds;
use crate::measurement::loudspeaker::Sound;

thread_local! {
    static RESUMING_ON_RELEASE: Cell<bool> = const { Cell::new(false) };
}

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
        if self.running() {
            return;
        }
        let _ = self.context.resume();
        self.resume_on_release();
    }

    fn running(&self) -> bool {
        self.context.state() == AudioContextState::Running
    }

    fn resume_on_release(&self) {
        if RESUMING_ON_RELEASE.replace(true) {
            return;
        }
        let context = self.context.clone();
        let resume = Closure::once_into_js(move || {
            RESUMING_ON_RELEASE.set(false);
            let _ = context.resume();
        });
        let once = AddEventListenerOptions::new();
        once.set_once(true);
        let listening = web_sys::window().map(|window| {
            window.add_event_listener_with_callback_and_add_event_listener_options(
                "pointerup",
                resume.unchecked_ref(),
                &once,
            )
        });
        if !matches!(listening, Some(Ok(()))) {
            RESUMING_ON_RELEASE.set(false);
        }
    }

    pub fn play(&mut self, sound: Sound, volume: f32, delay: Seconds) {
        if let Err(error) = self.try_play(sound, volume, delay) {
            log::warn!("Could not play a sound: {error:?}");
        }
    }

    fn try_play(
        &self,
        sound: Sound,
        volume: f32,
        delay: Seconds,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let Some(buffer) = self.sounds.get(&sound).filter(|_| self.running()) else {
            return Ok(());
        };
        let source = self.context.create_buffer_source()?;
        source.set_buffer(Some(buffer));
        let gain = self.context.create_gain()?;
        gain.gain().set_value(volume);
        source.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.context.destination())?;
        source.start_with_when(self.context.current_time() + delay.0)
    }
}
