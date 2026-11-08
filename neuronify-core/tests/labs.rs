#[allow(dead_code)]
#[path = "../generators/write_labs.rs"]
mod write_labs;

use std::f32::consts::TAU;
use std::sync::OnceLock;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::{DEFAULT_SPEED, DT};
use neuronify_core::editor::Editor;
use neuronify_core::measurement::spike_detector::SpikeDetector;
use neuronify_core::measurement::voltmeter::measured;
use neuronify_core::serialization::{load_from_bytes, save};
use neuronify_core::simulation::{fire_nearby, step_simulation, Engine};
use neuronify_core::{
    neuron_type, ConnectionView, LeakyNeuron, NeuronType, Position, Selected, Setting, TouchSensor,
};

const STEPS_PER_CALL: u32 = 40;
const STEP_DURATION: f64 = STEPS_PER_CALL as f64 * DT * 1e-3;
const TAP_DURATION: f64 = STEP_DURATION;
const BEFORE_THE_FIRST_SYNAPTIC_DELAY: f64 = 0.004;

struct Lab {
    world: World,
    engine: Engine,
    editor: Editor,
    time: f64,
    spikes: Vec<(Entity, f64)>,
}

impl Lab {
    fn load(name: &str) -> Lab {
        let path = format!("{}/examples/labs/{name}", env!("CARGO_MANIFEST_DIR"));
        let world = load_from_bytes(&std::fs::read(path).unwrap())
            .unwrap()
            .world;
        Lab {
            world,
            engine: Engine::default(),
            editor: Editor::default(),
            time: 0.0,
            spikes: Vec::new(),
        }
    }

    fn neurons(&self) -> Vec<Entity> {
        self.world
            .query::<&LeakyNeuron>()
            .iter()
            .map(|(entity, _)| entity)
            .collect()
    }

    fn run_until(&mut self, until: f64) {
        while self.time < until - STEP_DURATION / 2.0 {
            step_simulation(
                &mut self.world,
                &mut self.engine,
                &mut self.time,
                STEPS_PER_CALL,
                self.editor.touch(),
            );
            self.spikes.extend(
                self.engine
                    .spikes()
                    .iter()
                    .map(|spike| (spike.entity, spike.time * 1e-3)),
            );
        }
    }

    fn press_until(&mut self, finger: Vec3, until: f64) {
        self.editor.view(&mut self.world, finger);
        for position in self.editor.take_presses() {
            fire_nearby(&mut self.world, &mut self.engine, position);
        }
        self.run_until(until);
        self.editor.release(&self.world);
    }

    fn spike_counts(&self, cells: &[Entity], from: f64, to: f64) -> Vec<usize> {
        let mut counts = vec![0; cells.len()];
        for &(neuron, time) in &self.spikes {
            if time < from || time >= to {
                continue;
            }
            if let Some(cell) = cells.iter().position(|&c| c == neuron) {
                counts[cell] += 1;
            }
        }
        counts
    }

    fn spikes_between(&self, from: f64, to: f64) -> usize {
        self.spikes
            .iter()
            .filter(|&&(_, time)| time >= from && time < to)
            .count()
    }

    fn spike_times(&self, cell: Entity) -> Vec<f64> {
        self.spikes
            .iter()
            .filter(|&&(neuron, _)| neuron == cell)
            .map(|&(_, time)| time)
            .collect()
    }

    fn position(&self, cell: Entity) -> Vec3 {
        self.world.get::<&Position>(cell).unwrap().position
    }
}

struct RingLab {
    lab: Lab,
    ring: Vec<Entity>,
    pool: Vec<Entity>,
    touch_sensors: Vec<Entity>,
}

impl RingLab {
    fn load(name: &str) -> RingLab {
        let lab = Lab::load(name);
        let (mut ring, pool): (Vec<Entity>, Vec<Entity>) = lab
            .neurons()
            .iter()
            .partition(|&&cell| neuron_type(&lab.world, cell) != NeuronType::Inhibitory);
        let angle = |cell: Entity| {
            let position = lab.position(cell);
            position.z.atan2(position.x).rem_euclid(TAU)
        };
        ring.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)));
        let mut touch_sensors: Vec<Entity> = lab
            .world
            .query::<&TouchSensor>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        touch_sensors.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)));
        RingLab {
            lab,
            ring,
            pool,
            touch_sensors,
        }
    }

    fn tap(&mut self, cell: usize, at: f64) {
        let finger = self.lab.position(self.touch_sensors[cell]);
        self.lab.run_until(at);
        self.lab.press_until(finger, at + TAP_DURATION);
    }

    fn ring_counts(&self, from: f64, to: f64) -> Vec<usize> {
        self.lab.spike_counts(&self.ring, from, to)
    }
}

const FIRST_CUE: f64 = 0.2;
const SECOND_CUE: f64 = 1.2;
const END: f64 = 1.9;
const SETTLED_WINDOW: f64 = 0.5;
const FIRST_CUED_CELL: usize = 8;
const SECOND_CUED_CELL: usize = 24;

fn ring_cued_twice() -> &'static RingLab {
    static RUN: OnceLock<RingLab> = OnceLock::new();
    RUN.get_or_init(|| {
        let mut ring = RingLab::load("ring_attractor.neuronify");
        ring.tap(FIRST_CUED_CELL, FIRST_CUE);
        ring.tap(SECOND_CUED_CELL, SECOND_CUE);
        ring.lab.run_until(END);
        ring
    })
}

fn noisy_ring_cued_once() -> &'static RingLab {
    static RUN: OnceLock<RingLab> = OnceLock::new();
    RUN.get_or_init(|| {
        let mut ring = RingLab::load("ring_attractor_noise.neuronify");
        ring.tap(FIRST_CUED_CELL, FIRST_CUE);
        ring.lab.run_until(3.0);
        ring
    })
}

fn cells_firing_at_10_hz_or_more(counts: &[usize], duration: f64) -> usize {
    counts
        .iter()
        .filter(|&&count| count as f64 >= 10.0 * duration)
        .count()
}

fn bump_centre_in_cells(counts: &[usize]) -> f32 {
    let cells = counts.len() as f32;
    let (sin, cos) = counts
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(sin, cos), (cell, &count)| {
            let angle = TAU * cell as f32 / cells;
            (
                sin + count as f32 * angle.sin(),
                cos + count as f32 * angle.cos(),
            )
        });
    (sin.atan2(cos) / TAU * cells).rem_euclid(cells)
}

fn distance_round_ring(a: f32, b: f32, cells: usize) -> f32 {
    let cells = cells as f32;
    let difference = (a - b).rem_euclid(cells);
    difference.min(cells - difference)
}

#[test]
fn ring_attractor_finger_on_a_touch_sensor_fires_only_its_cell() {
    let mut ring = RingLab::load("ring_attractor.neuronify");
    let finger = ring.lab.position(ring.touch_sensors[FIRST_CUED_CELL]);
    assert_eq!(cells_fired_by_a_poke(&mut ring, finger), [FIRST_CUED_CELL]);
}

#[test]
fn ring_attractor_finger_between_two_touch_sensors_fires_both_cells() {
    let mut ring = RingLab::load("ring_attractor.neuronify");
    let finger = (ring.lab.position(ring.touch_sensors[FIRST_CUED_CELL])
        + ring.lab.position(ring.touch_sensors[FIRST_CUED_CELL + 1]))
        / 2.0;
    assert_eq!(
        cells_fired_by_a_poke(&mut ring, finger),
        [FIRST_CUED_CELL, FIRST_CUED_CELL + 1]
    );
}

#[test]
fn ring_attractor_finger_on_a_cell_fires_nothing_in_view_mode() {
    let mut ring = RingLab::load("ring_attractor.neuronify");
    let finger = ring.lab.position(ring.ring[FIRST_CUED_CELL]);
    assert!(cells_fired_by_a_poke(&mut ring, finger).is_empty());
}

#[test]
fn ring_attractor_poke_on_a_touch_sensor_starts_a_bump_at_its_cell() {
    let mut ring = RingLab::load("ring_attractor.neuronify");
    let finger = ring.lab.position(ring.touch_sensors[FIRST_CUED_CELL]);
    ring.lab.press_until(finger, TAP_DURATION);
    ring.lab.run_until(SETTLED_WINDOW * 2.0);
    let counts = ring.ring_counts(SETTLED_WINDOW, SETTLED_WINDOW * 2.0);
    let centre = bump_centre_in_cells(&counts);
    assert!(
        cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW) >= 9,
        "{counts:?}"
    );
    assert!(distance_round_ring(centre, FIRST_CUED_CELL as f32, ring.ring.len()) < 0.5);
}

fn cells_fired_by_a_poke(ring: &mut RingLab, finger: Vec3) -> Vec<usize> {
    ring.lab.press_until(finger, TAP_DURATION);
    ring.lab.run_until(BEFORE_THE_FIRST_SYNAPTIC_DELAY);
    ring.ring_counts(0.0, BEFORE_THE_FIRST_SYNAPTIC_DELAY)
        .iter()
        .enumerate()
        .filter(|(_, &count)| count > 0)
        .map(|(cell, _)| cell)
        .collect()
}

#[test]
fn ring_attractors_open_showing_only_the_connections_of_the_top_cell() {
    for name in ["ring_attractor.neuronify", "ring_attractor_noise.neuronify"] {
        let ring = RingLab::load(name);
        assert_eq!(
            ConnectionView::of(&ring.lab.world),
            ConnectionView::LastSelected
        );
        assert_eq!(Selected::entity(&ring.lab.world), Some(ring.ring[0]));
    }
}

#[test]
fn ring_attractors_have_a_spike_detector_with_one_row_per_cell_clockwise_from_the_top() {
    for name in ["ring_attractor.neuronify", "ring_attractor_noise.neuronify"] {
        let ring = RingLab::load(name);
        let world = &ring.lab.world;
        let detectors: Vec<Entity> = world
            .query::<&SpikeDetector>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        assert_eq!(detectors.len(), 1);
        let rows: Vec<Entity> = measured(world, detectors[0])
            .into_iter()
            .map(|(_, neuron)| neuron)
            .collect();
        assert_eq!(rows, ring.ring);
    }
}

#[test]
fn the_bundled_lab_files_match_their_generators() {
    for (file, world) in write_labs::labs() {
        let path = format!("{}/examples/labs/{file}", env!("CARGO_MANIFEST_DIR"));
        let bundled = std::fs::read(path).unwrap();
        let written = save(&world, 0.0, DEFAULT_SPEED).unwrap();
        assert_eq!(bundled, written, "{file} is out of date");
    }
}

#[test]
fn ring_attractor_is_silent_before_the_cue() {
    let ring = ring_cued_twice();
    assert_eq!(ring.lab.spikes_between(0.0, FIRST_CUE), 0);
}

#[test]
fn ring_attractor_cue_leaves_a_bump_of_nine_cells_at_the_cue() {
    let ring = ring_cued_twice();
    let counts = ring.ring_counts(SECOND_CUE - SETTLED_WINDOW, SECOND_CUE);
    let peak_rate = *counts.iter().max().unwrap() as f64 / SETTLED_WINDOW;
    let centre = bump_centre_in_cells(&counts);
    assert_eq!(
        cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW),
        9,
        "{counts:?}"
    );
    assert!(distance_round_ring(centre, FIRST_CUED_CELL as f32, ring.ring.len()) < 0.5);
    assert!((140.0..=170.0).contains(&peak_rate), "{peak_rate} Hz");
}

#[test]
fn ring_attractor_second_cue_moves_the_bump_and_silences_the_first() {
    let ring = ring_cued_twice();
    let counts = ring.ring_counts(END - SETTLED_WINDOW, END);
    let centre = bump_centre_in_cells(&counts);
    assert_eq!(
        cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW),
        9,
        "{counts:?}"
    );
    assert!(distance_round_ring(centre, SECOND_CUED_CELL as f32, ring.ring.len()) < 0.5);
    assert_eq!(counts[FIRST_CUED_CELL], 0);
}

#[test]
fn ring_attractor_inhibitory_cells_fire_together() {
    let ring = ring_cued_twice();
    let first = ring.lab.spike_times(ring.pool[0]);
    assert!(!first.is_empty());
    for &cell in &ring.pool[1..] {
        assert_eq!(ring.lab.spike_times(cell), first);
    }
}

#[test]
fn ring_attractor_noise_alone_lights_no_bump() {
    let ring = noisy_ring_cued_once();
    let counts = ring.ring_counts(0.0, FIRST_CUE);
    assert_eq!(cells_firing_at_10_hz_or_more(&counts, FIRST_CUE), 0);
}

#[test]
fn ring_attractor_bump_survives_three_seconds_of_noise() {
    let ring = noisy_ring_cued_once();
    for second in [1.0, 2.0, 3.0] {
        let counts = ring.ring_counts(second - SETTLED_WINDOW, second);
        assert!(
            cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW) >= 5,
            "{second} s: {counts:?}"
        );
    }
}
