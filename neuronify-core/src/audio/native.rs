use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};

use super::synthesize;
use crate::measurement::loudspeaker::Sound;

const MAXIMUM_VOICES: usize = 32;

struct Voice {
    samples: Arc<Vec<f32>>,
    position: isize,
    volume: f32,
}

pub struct Output {
    _stream: cpal::Stream,
    voices: Arc<Mutex<Vec<Voice>>>,
    sounds: HashMap<Sound, Arc<Vec<f32>>>,
    sample_rate: f32,
}

impl Output {
    pub fn open() -> Option<Output> {
        match Output::try_open() {
            Ok(output) => Some(output),
            Err(error) => {
                log::warn!("No sound: {error}");
                None
            }
        }
    }

    fn try_open() -> Result<Output, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("there is no output device")?;
        let config = device.default_output_config().map_err(|e| e.to_string())?;
        let voices = Arc::new(Mutex::new(Vec::new()));
        let sample_rate = config.sample_rate() as f32;
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => stream::<f32>(&device, &config.into(), voices.clone()),
            cpal::SampleFormat::I16 => stream::<i16>(&device, &config.into(), voices.clone()),
            cpal::SampleFormat::U16 => stream::<u16>(&device, &config.into(), voices.clone()),
            format => return Err(format!("unsupported sample format {format}")),
        }?;
        stream.play().map_err(|e| e.to_string())?;
        let sounds = Sound::ALL
            .into_iter()
            .map(|sound| (sound, Arc::new(synthesize(sound, sample_rate))))
            .collect();
        Ok(Output {
            _stream: stream,
            voices,
            sounds,
            sample_rate,
        })
    }

    pub fn unlock(&self) {}

    pub fn play(&mut self, sound: Sound, volume: f32, delay: f64) {
        let Ok(mut voices) = self.voices.lock() else {
            return;
        };
        if voices.len() >= MAXIMUM_VOICES {
            voices.remove(0);
        }
        voices.push(Voice {
            samples: self.sounds[&sound].clone(),
            position: -((delay * self.sample_rate as f64) as isize),
            volume,
        });
    }
}

fn stream<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    voices: Arc<Mutex<Vec<Voice>>>,
) -> Result<cpal::Stream, String> {
    let channels = config.channels as usize;
    device
        .build_output_stream(
            *config,
            move |data: &mut [T], _| {
                let Ok(mut voices) = voices.lock() else {
                    return;
                };
                for frame in data.chunks_mut(channels) {
                    let mut mixed = 0.0;
                    for voice in voices.iter_mut() {
                        if voice.position >= 0 {
                            mixed += voice.samples[voice.position as usize] * voice.volume;
                        }
                        voice.position += 1;
                    }
                    voices.retain(|voice| voice.position < voice.samples.len() as isize);
                    let value = T::from_sample(mixed.clamp(-1.0, 1.0));
                    for sample in frame {
                        *sample = value;
                    }
                }
            },
            |error| log::error!("Sound stopped: {error}"),
            None,
        )
        .map_err(|e| e.to_string())
}
