use hecs::{Entity, World};
use neuronify_engine::units::Milliseconds;
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

fn never() -> Milliseconds {
    Milliseconds::INFINITY
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Loudspeaker {
    pub sound: Sound,
    pub volume: f64,
    pub muted: bool,
    #[serde(skip)]
    pub heard: Vec<Milliseconds>,
    #[serde(skip, default = "never")]
    pub time_since_heard: Milliseconds,
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

pub fn listen(world: &mut World, spikes: &[Spike], time: Milliseconds, duration: Milliseconds) {
    let mut speakers = world
        .query_mut::<(Entity, &mut Loudspeaker)>()
        .into_iter()
        .peekable();
    if speakers.peek().is_none() {
        return;
    }
    for (_, speaker) in speakers {
        speaker.time_since_heard += duration;
    }
    let wires: Vec<(Entity, Entity)> = world
        .query::<(Entity, &Connection)>()
        .iter()
        .filter(|(_, connection)| world.satisfies::<&Loudspeaker>(connection.to))
        .map(|(_, connection)| (connection.from, connection.to))
        .collect();
    for spike in spikes {
        for &(from, to) in &wires {
            if from != spike.entity {
                continue;
            }
            if let Ok(mut speaker) = world.get::<&mut Loudspeaker>(to) {
                speaker.time_since_heard = time - spike.time;
                if speaker.heard.len() < MAXIMUM_HEARD {
                    speaker.heard.push(spike.time);
                }
            }
        }
    }
}

pub fn take_heard(world: &mut World) -> Vec<(Sound, f32, Milliseconds)> {
    let mut heard = Vec::new();
    for (_, speaker) in world.query_mut::<(Entity, &mut Loudspeaker)>() {
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

    const SECOND: Milliseconds = Milliseconds(1000.0);

    fn spike(entity: Entity, time: Milliseconds) -> Spike {
        Spike { entity, time }
    }

    #[test]
    fn a_loudspeaker_hears_each_spike_of_its_neurons_unless_muted() {
        let mut world = World::new();
        let [wired, unwired] = [(); 2].map(|_| world.spawn((NodeState::default(),)));
        let speaker = spawn_loudspeaker(&mut world, wired, Vec3::ZERO);
        world.get::<&mut Loudspeaker>(speaker).unwrap().sound = Sound::Pop;
        listen(
            &mut world,
            &[spike(wired, SECOND), spike(unwired, SECOND)],
            SECOND,
            DT,
        );
        listen(&mut world, &[], SECOND + DT, DT);
        assert_eq!(take_heard(&mut world), [(Sound::Pop, 1.0, SECOND)]);
        assert!(take_heard(&mut world).is_empty());
        let since = world.get::<&Loudspeaker>(speaker).unwrap().time_since_heard;
        assert_eq!(since, DT);

        world.get::<&mut Loudspeaker>(speaker).unwrap().muted = true;
        listen(&mut world, &[spike(wired, 2.0 * SECOND)], 2.0 * SECOND, DT);
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
