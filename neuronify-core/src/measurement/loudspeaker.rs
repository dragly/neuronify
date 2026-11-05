use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::simulation::Spike;

const MAXIMUM_HEARD: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Sound {
    #[default]
    Click,
    Pop,
    Tone,
    Drop,
}

impl Sound {
    pub const ALL: [Sound; 4] = [Sound::Click, Sound::Pop, Sound::Tone, Sound::Drop];

    pub fn name(self) -> &'static str {
        match self {
            Sound::Click => "Click",
            Sound::Pop => "Pop",
            Sound::Tone => "Tone",
            Sound::Drop => "Drop",
        }
    }

    pub fn from_legacy(file: &str) -> Sound {
        match file {
            "drip.wav" => Sound::Drop,
            "thump.wav" => Sound::Pop,
            "glass.wav" | "sonar.wav" => Sound::Tone,
            _ => Sound::Click,
        }
    }
}

fn never() -> f64 {
    f64::INFINITY
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Loudspeaker {
    pub sound: Sound,
    pub volume: f64,
    pub muted: bool,
    #[serde(skip)]
    pub heard: Vec<f64>,
    #[serde(skip, default = "never")]
    pub time_since_heard: f64,
}

impl Default for Loudspeaker {
    fn default() -> Self {
        Self {
            sound: Sound::default(),
            volume: 1.0,
            muted: false,
            heard: Vec::new(),
            time_since_heard: never(),
        }
    }
}

pub fn listen(world: &mut World, spikes: &[Spike], time: f64, duration: f64) {
    let mut speakers = world.query_mut::<&mut Loudspeaker>().into_iter().peekable();
    if speakers.peek().is_none() {
        return;
    }
    for (_, speaker) in speakers {
        speaker.time_since_heard += duration;
    }
    let wires: Vec<(Entity, Entity)> = world
        .query::<&Connection>()
        .iter()
        .filter(|(_, connection)| {
            world
                .satisfies::<&Loudspeaker>(connection.to)
                .unwrap_or(false)
        })
        .map(|(_, connection)| (connection.from, connection.to))
        .collect();
    for spike in spikes {
        for &(from, to) in &wires {
            if from != spike.entity {
                continue;
            }
            if let Ok(mut speaker) = world.get::<&mut Loudspeaker>(to) {
                speaker.time_since_heard = time - spike.time * 1e-3;
                if speaker.heard.len() < MAXIMUM_HEARD {
                    speaker.heard.push(spike.time * 1e-3);
                }
            }
        }
    }
}

pub fn take_heard(world: &mut World) -> Vec<(Sound, f32, f64)> {
    let mut heard = Vec::new();
    for (_, speaker) in world.query_mut::<&mut Loudspeaker>() {
        let times = std::mem::take(&mut speaker.heard);
        if !speaker.muted {
            heard.extend(
                times
                    .into_iter()
                    .map(|time| (speaker.sound, speaker.volume as f32, time)),
            );
        }
    }
    heard
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DT;
    use crate::elements::spawn_loudspeaker;
    use glam::Vec3;

    const STEP: f64 = DT * 1e-3;

    fn spike(entity: Entity, time: f64) -> Spike {
        Spike {
            entity,
            time: time * 1e3,
        }
    }

    #[test]
    fn a_loudspeaker_hears_each_spike_of_its_neurons_unless_muted() {
        let mut world = World::new();
        let [wired, unwired] = [(); 2].map(|_| world.spawn((NodeState::default(),)));
        let speaker = spawn_loudspeaker(&mut world, wired, Vec3::ZERO);
        world.get::<&mut Loudspeaker>(speaker).unwrap().sound = Sound::Pop;
        listen(
            &mut world,
            &[spike(wired, 1.0), spike(unwired, 1.0)],
            1.0,
            STEP,
        );
        listen(&mut world, &[], 1.0 + STEP, STEP);
        assert_eq!(take_heard(&mut world), [(Sound::Pop, 1.0, 1.0)]);
        assert!(take_heard(&mut world).is_empty());
        assert_eq!(
            world.get::<&Loudspeaker>(speaker).unwrap().time_since_heard,
            STEP
        );

        world.get::<&mut Loudspeaker>(speaker).unwrap().muted = true;
        listen(&mut world, &[spike(wired, 2.0)], 2.0, STEP);
        assert!(take_heard(&mut world).is_empty());
    }

    #[test]
    fn neuronify_1_sounds_map_to_the_closest_new_ones() {
        assert_eq!(Sound::from_legacy("glass.wav"), Sound::Tone);
        assert_eq!(Sound::from_legacy("drip.wav"), Sound::Drop);
        assert_eq!(Sound::from_legacy("thump.wav"), Sound::Pop);
        assert_eq!(Sound::from_legacy("sonar.wav"), Sound::Tone);
    }
}
