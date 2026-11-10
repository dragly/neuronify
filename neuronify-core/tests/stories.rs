use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use glam::Vec3;
use hecs::{Entity, World};
use indexmap::IndexMap;
use neuronify_core::measurement::firing_rate::RatePlot;
use neuronify_core::measurement::loudspeaker::Loudspeaker;
use neuronify_core::measurement::spike_detector::SpikeDetector;
use neuronify_core::measurement::visual_input::VisualInput;
use neuronify_core::measurement::voltmeter::Voltmeter;
use neuronify_core::serialization::load_from_bytes;
use neuronify_core::simulation::{fire_nearby, step_simulation, Engine};
use neuronify_core::{
    neuron_type, AdaptationCurrent, AlternatingCurrent, Annotation, BurstCurrent, Compartment,
    ConductanceSynapse, CurrentClamp, CurrentSynapse, Dendrite, Label, LeakyNeuron, Link,
    NeuronType, PoissonGenerator, Position, RegularSpikeGenerator, Seconds, TouchSensor,
    DEFAULT_SPEED, SIMULATION_STEPS_PER_SECOND,
};
use serde::{Deserialize, Serialize};

const FOLDERS: [&str; 2] = ["examples", "src/legacy/fixtures"];
const SECONDS: f64 = 10.0;
const PRESS_AT: f64 = 2.0;
const STEPS_PER_CALL: usize = 20;

fn steps(seconds: f64) -> usize {
    (Seconds(seconds) * DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND).round() as usize
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn stories() -> PathBuf {
    crate_root().join("tests/stories")
}

fn example_files() -> Vec<String> {
    fn walk(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, files);
            } else if matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("nfy" | "neuronify")
            ) {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    for folder in FOLDERS {
        walk(&crate_root().join(folder), &mut files);
    }
    let mut names: Vec<String> = files
        .iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

fn open(file: &str) -> World {
    let bytes = std::fs::read(crate_root().join(file)).unwrap();
    load_from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("{file} does not open: {error}"))
        .world
}

fn reading_order(a: Vec3, b: Vec3) -> std::cmp::Ordering {
    b.x.total_cmp(&a.x).then(a.z.total_cmp(&b.z))
}

fn in_reading_order<Q: hecs::Component>(world: &World) -> Vec<(Entity, Vec3)> {
    let mut found: Vec<(Entity, Vec3)> = world
        .query::<(Entity, &Position)>()
        .with::<&Q>()
        .iter()
        .map(|(entity, position)| (entity, position.position))
        .collect();
    found.sort_by(|a, b| reading_order(a.1, b.1));
    found
}

fn element(world: &World, entity: Entity) -> Option<&'static str> {
    Some(if world.get::<&LeakyNeuron>(entity).is_ok() {
        if world.get::<&AdaptationCurrent>(entity).is_ok() {
            "adaptive neuron"
        } else if world.get::<&BurstCurrent>(entity).is_ok() {
            "burst neuron"
        } else if neuron_type(world, entity) == NeuronType::Inhibitory {
            "inhibitory neuron"
        } else {
            "excitatory neuron"
        }
    } else if world.get::<&Dendrite>(entity).is_ok() {
        "dendrite compartment"
    } else if world.get::<&Compartment>(entity).is_ok() {
        "compartment"
    } else if world.get::<&CurrentSynapse>(entity).is_ok()
        || world.get::<&ConductanceSynapse>(entity).is_ok()
    {
        "synapse"
    } else if world.get::<&AlternatingCurrent>(entity).is_ok() {
        "alternating current source"
    } else if world.get::<&CurrentClamp>(entity).is_ok() {
        "current source"
    } else if world.get::<&TouchSensor>(entity).is_ok() {
        "touch sensor"
    } else if world.get::<&RegularSpikeGenerator>(entity).is_ok() {
        "regular spike generator"
    } else if world.get::<&PoissonGenerator>(entity).is_ok() {
        "Poisson generator"
    } else if world.get::<&VisualInput>(entity).is_ok() {
        "visual input"
    } else if world.get::<&Voltmeter>(entity).is_ok() {
        "voltmeter"
    } else if world.get::<&SpikeDetector>(entity).is_ok() {
        "spike detector"
    } else if world.get::<&RatePlot>(entity).is_ok() {
        "rate plot"
    } else if world.get::<&Loudspeaker>(entity).is_ok() {
        "loudspeaker"
    } else if world.get::<&Link>(entity).is_ok() {
        "link"
    } else if world.get::<&Annotation>(entity).is_ok() {
        "note"
    } else {
        return None;
    })
}

fn elements(world: &World) -> IndexMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entity in world.iter().map(|entity| entity.entity()) {
        if let Some(name) = element(world, entity) {
            *counts.entry((name.to_lowercase(), name)).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .map(|((_, name), count)| (name.to_string(), count))
        .collect()
}

fn spikes(file: &str, press: Option<Vec3>) -> Vec<usize> {
    let mut world = open(file);
    let mut engine = Engine::default();
    let neurons = in_reading_order::<LeakyNeuron>(&world);
    let mut counts = vec![0; neurons.len()];
    let mut time = Default::default();
    let end = steps(SECONDS);
    let press_step = steps(PRESS_AT);
    let mut step = 0;
    while step < end {
        if step == press_step {
            if let Some(hand) = press {
                fire_nearby(&mut world, &mut engine, hand);
            }
        }
        let next = [end, press_step]
            .into_iter()
            .filter(|event| *event > step)
            .min()
            .unwrap();
        let chunk = (next - step).min(STEPS_PER_CALL);
        step_simulation(&mut world, &mut engine, &mut time, chunk as u32, None);
        step += chunk;
        for (count, (neuron, _)) in counts.iter_mut().zip(&neurons) {
            let voltage = engine.voltage(*neuron).unwrap();
            assert!(
                voltage.is_finite(),
                "{file}: a neuron's voltage became {voltage}"
            );
            *count += engine
                .spikes()
                .iter()
                .filter(|spike| spike.entity == *neuron)
                .count();
        }
    }
    counts
}

#[derive(Deserialize, Serialize)]
struct Story {
    seconds: f64,
    runs: Vec<String>,
    elements: IndexMap<String, usize>,
    spikes: IndexMap<String, Vec<usize>>,
}

impl Story {
    fn of(file: &str) -> Story {
        let world = open(file);
        let hands = in_reading_order::<TouchSensor>(&world);
        let names: Vec<String> = in_reading_order::<LeakyNeuron>(&world)
            .iter()
            .enumerate()
            .map(|(index, (neuron, _))| {
                let label = world
                    .get::<&Label>(*neuron)
                    .map(|label| label.text.trim().to_string())
                    .unwrap_or_default();
                format!("{} {label}", index + 1).trim().to_string()
            })
            .collect();
        let mut runs = vec!["alone".to_string()];
        runs.extend((1..=hands.len()).map(|sensor| format!("sensor {sensor}")));
        let presses = std::iter::once(None).chain(hands.iter().map(|(_, hand)| Some(*hand)));
        let counts: Vec<Vec<usize>> = std::thread::scope(|scope| {
            let runs: Vec<_> = presses
                .map(|press| scope.spawn(move || spikes(file, press)))
                .collect();
            runs.into_iter()
                .map(|run| {
                    run.join()
                        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
                })
                .collect()
        });
        let spikes = names
            .into_iter()
            .enumerate()
            .map(|(row, name)| (name, counts.iter().map(|run| run[row]).collect()))
            .collect();
        Story {
            seconds: SECONDS,
            runs,
            elements: elements(&world),
            spikes,
        }
    }

    fn differences(&self, now: &Story) -> Vec<String> {
        let mut differences = Vec::new();
        let mut names: Vec<&String> = self.elements.keys().chain(now.elements.keys()).collect();
        names.sort_by_key(|name| name.to_lowercase());
        names.dedup();
        for name in names {
            let count = |story: &Story| story.elements.get(name).copied().unwrap_or(0);
            if count(self) != count(now) {
                differences.push(format!(
                    "had {}, now {}",
                    counted(count(self), name),
                    counted(count(now), name)
                ));
            }
        }
        if self.seconds != now.seconds || self.runs != now.runs {
            differences.push(format!(
                "had runs {:?} over {} s, now {:?} over {} s",
                self.runs, self.seconds, now.runs, now.seconds
            ));
            return differences;
        }
        if !self.spikes.keys().eq(now.spikes.keys()) {
            differences.push(format!(
                "had neurons {:?}, now {:?}",
                self.spikes.keys().collect::<Vec<_>>(),
                now.spikes.keys().collect::<Vec<_>>()
            ));
            return differences;
        }
        for ((name, before), after) in self.spikes.iter().zip(now.spikes.values()) {
            for ((run, before), after) in self.runs.iter().zip(before).zip(after) {
                if !similar(*before, *after) {
                    let run = match run.as_str() {
                        "alone" => "alone".to_string(),
                        sensor => format!("after pressing {sensor}"),
                    };
                    differences.push(format!(
                        "neuron {name} fired {} {run} instead of {}",
                        times(*after),
                        times(*before)
                    ));
                }
            }
        }
        differences
    }
}

fn counted(count: usize, name: &str) -> String {
    match count {
        1 => format!("1 {name}"),
        _ => format!("{count} {name}s"),
    }
}

fn times(count: usize) -> String {
    match count {
        0 => "never".to_string(),
        1 => "once".to_string(),
        _ => format!("{count} times"),
    }
}

fn similar(before: usize, after: usize) -> bool {
    (before == 0) == (after == 0) && after <= 2 * before + 1 && before <= 2 * after + 1
}

fn at_most_three(context: &str, differences: Vec<String>) -> Vec<String> {
    let mut lines: Vec<String> = differences
        .iter()
        .take(3)
        .map(|difference| format!("{context}: {difference}"))
        .collect();
    if differences.len() > 3 {
        lines.push(format!("{context}: and {} more", differences.len() - 3));
    }
    lines
}

fn story_path(file: &str) -> PathBuf {
    stories().join(format!("{file}.toml"))
}

#[test]
fn every_example_tells_the_story_it_told_before() {
    let mut failures = Vec::new();
    for file in example_files() {
        let now = Story::of(&file);
        match std::fs::read_to_string(story_path(&file)) {
            Ok(text) => match toml::from_str::<Story>(&text) {
                Ok(before) => failures.extend(at_most_three(&file, before.differences(&now))),
                Err(error) => failures.push(format!("{file}: the story does not read: {error}")),
            },
            Err(_) => failures.push(format!("{file} has no recorded story")),
        }
    }
    assert!(
        failures.is_empty(),
        "\n{}\n\nIf the change is intended, record the stories again with\n\
         cargo test -p neuronify-core --test stories write_stories -- --ignored\n",
        failures.join("\n")
    );
}

#[test]
#[ignore]
fn write_stories() {
    for file in example_files() {
        let path = story_path(&file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, toml::to_string(&Story::of(&file)).unwrap()).unwrap();
    }
}
