#[allow(dead_code)]
#[path = "../generators/write_labs.rs"]
mod write_labs;

use std::f32::consts::TAU;
use std::sync::OnceLock;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::DT;
use neuronify_core::editor::Editor;
use neuronify_core::measurement::frame::measured;
use neuronify_core::measurement::spike_detector::SpikeDetector;
use neuronify_core::rendering::halos::collect_halos;
use neuronify_core::rendering::{canvas_bounds, Theme};
use neuronify_core::serialization::{load_from_bytes, save, Meta};
use neuronify_core::simulation::{fire_nearby, step_simulation, Engine};
use neuronify_core::stimuli::receptive_field::{watched_picture, ReceptiveFieldCell};
use neuronify_core::stimuli::touch_sensor::TouchSensor;
use neuronify_core::stimuli::visual_input::{direction_badge, VisualInput};
use neuronify_core::stimuli::CurrentClamp;
use neuronify_core::{
    neuron_type, Connection, ConnectionView, Hertz, Label, LeakyNeuron, Milliseconds, Nanoamperes,
    NeuronType, NoteSize, Position, Selected, Setting, NODE_RADIUS,
};

const STEPS_PER_CALL: u32 = 40;
const STEP_DURATION: Milliseconds = Milliseconds(STEPS_PER_CALL as f64 * DT.0);
const TAP_DURATION: Milliseconds = STEP_DURATION;
const BEFORE_THE_FIRST_SYNAPTIC_DELAY: Milliseconds = Milliseconds(4.0);

struct Lab {
    world: World,
    engine: Engine,
    editor: Editor,
    time: Milliseconds,
    spikes: Vec<(Entity, Milliseconds)>,
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
            time: Milliseconds::ZERO,
            spikes: Vec::new(),
        }
    }

    fn neurons(&self) -> Vec<Entity> {
        self.world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .map(|(entity, _)| entity)
            .collect()
    }

    fn run_until(&mut self, until: Milliseconds) {
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
                    .map(|spike| (spike.entity, spike.time)),
            );
        }
    }

    fn press_until(&mut self, finger: Vec3, until: Milliseconds) {
        self.editor.view(&mut self.world, finger);
        for position in self.editor.take_presses() {
            fire_nearby(&mut self.world, &mut self.engine, position);
        }
        self.run_until(until);
        self.editor.release(&self.world);
    }

    fn spike_counts(&self, cells: &[Entity], from: Milliseconds, to: Milliseconds) -> Vec<usize> {
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

    fn spikes_between(&self, from: Milliseconds, to: Milliseconds) -> usize {
        self.spikes
            .iter()
            .filter(|&&(_, time)| time >= from && time < to)
            .count()
    }

    fn spike_times(&self, cell: Entity) -> Vec<Milliseconds> {
        self.spikes
            .iter()
            .filter(|&&(neuron, _)| neuron == cell)
            .map(|&(_, time)| time)
            .collect()
    }

    fn position(&self, cell: Entity) -> Vec3 {
        self.world.get::<&Position>(cell).unwrap().position
    }

    fn targets(&self, from: Entity) -> Vec<Entity> {
        self.world
            .query::<&Connection>()
            .iter()
            .filter(|connection| connection.from == from)
            .map(|connection| connection.to)
            .collect()
    }

    fn labelled(&self, text: &str) -> Entity {
        self.world
            .query::<(Entity, &Label)>()
            .iter()
            .find(|(_, label)| label.text == text)
            .unwrap()
            .0
    }

    fn set_current(&mut self, source: Entity, current: Nanoamperes) {
        self.world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = current;
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
        let angle = |cell: Entity| {
            let position = lab.position(cell);
            position.z.atan2(position.x).rem_euclid(TAU)
        };
        let mut touch_sensors: Vec<Entity> = lab
            .world
            .query::<(Entity, &TouchSensor)>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        touch_sensors.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)));
        let ring: Vec<Entity> = touch_sensors
            .iter()
            .map(|&sensor| lab.targets(sensor)[0])
            .collect();
        let pool: Vec<Entity> = lab
            .targets(ring[0])
            .into_iter()
            .filter(|&cell| neuron_type(&lab.world, cell) == NeuronType::Inhibitory)
            .collect();
        RingLab {
            lab,
            ring,
            pool,
            touch_sensors,
        }
    }

    fn tap(&mut self, cell: usize, at: Milliseconds) {
        let finger = self.lab.position(self.touch_sensors[cell]);
        self.lab.run_until(at);
        self.lab.press_until(finger, at + TAP_DURATION);
    }

    fn ring_counts(&self, from: Milliseconds, to: Milliseconds) -> Vec<usize> {
        self.lab.spike_counts(&self.ring, from, to)
    }

    fn steps_of_the_bump(&self, from: Milliseconds, to: Milliseconds) -> Vec<f32> {
        let cells = self.ring.len() as f32;
        let mut centres = Vec::new();
        let mut start = from;
        while start < to {
            centres.push(bump_centre_in_cells(
                &self.ring_counts(start, start + TRACKING_WINDOW),
            ));
            start += TRACKING_WINDOW;
        }
        centres
            .windows(2)
            .map(|pair| (pair[1] - pair[0] + cells / 2.0).rem_euclid(cells) - cells / 2.0)
            .collect()
    }
}

const TRACKING_WINDOW: Milliseconds = Milliseconds(100.0);
const FIRST_CUE: Milliseconds = Milliseconds(200.0);
const SECOND_CUE: Milliseconds = Milliseconds(1200.0);
const END: Milliseconds = Milliseconds(1900.0);
const SETTLED_WINDOW: Milliseconds = Milliseconds(500.0);
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
        ring.lab.run_until(Milliseconds(3000.0));
        ring
    })
}

fn cells_firing_at_10_hz_or_more(counts: &[usize], duration: Milliseconds) -> usize {
    counts
        .iter()
        .filter(|&&count| count as f64 >= Hertz(10.0).cycles_in(duration))
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
    ring.ring_counts(Milliseconds::ZERO, BEFORE_THE_FIRST_SYNAPTIC_DELAY)
        .iter()
        .enumerate()
        .filter(|(_, &count)| count > 0)
        .map(|(cell, _)| cell)
        .collect()
}

const RING_LABS: [&str; 3] = [
    "ring_attractor.neuronify",
    "ring_attractor_noise.neuronify",
    "compass.neuronify",
];

#[test]
fn ring_labs_open_showing_only_the_connections_of_the_top_cell() {
    for name in RING_LABS {
        let ring = RingLab::load(name);
        assert_eq!(
            ConnectionView::of(&ring.lab.world),
            ConnectionView::LastSelected
        );
        assert_eq!(Selected::entity(&ring.lab.world), Some(ring.ring[0]));
    }
}

#[test]
fn ring_labs_have_a_spike_detector_with_one_row_per_cell_clockwise_from_the_top() {
    for name in RING_LABS {
        let ring = RingLab::load(name);
        let world = &ring.lab.world;
        let detectors: Vec<Entity> = world
            .query::<(Entity, &SpikeDetector)>()
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
    for (file, world, speed) in write_labs::labs() {
        let path = format!("{}/examples/labs/{file}", env!("CARGO_MANIFEST_DIR"));
        let bundled = std::fs::read(path).unwrap();
        let written = save(&world, Milliseconds::ZERO, speed, &Meta::default()).unwrap();
        assert_eq!(bundled, written, "{file} is out of date");
    }
}

#[test]
fn ring_attractor_is_silent_before_the_cue() {
    let ring = ring_cued_twice();
    assert_eq!(ring.lab.spikes_between(Milliseconds::ZERO, FIRST_CUE), 0);
}

#[test]
fn ring_attractor_cue_leaves_a_bump_of_nine_cells_at_the_cue() {
    let ring = ring_cued_twice();
    let counts = ring.ring_counts(SECOND_CUE - SETTLED_WINDOW, SECOND_CUE);
    let peak_rate = (*counts.iter().max().unwrap() as f64 / SETTLED_WINDOW).to_hertz();
    let centre = bump_centre_in_cells(&counts);
    assert_eq!(
        cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW),
        9,
        "{counts:?}"
    );
    assert!(distance_round_ring(centre, FIRST_CUED_CELL as f32, ring.ring.len()) < 0.5);
    assert!(
        (Hertz(140.0)..=Hertz(170.0)).contains(&peak_rate),
        "{peak_rate}"
    );
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
    let counts = ring.ring_counts(Milliseconds::ZERO, FIRST_CUE);
    assert_eq!(cells_firing_at_10_hz_or_more(&counts, FIRST_CUE), 0);
}

#[test]
fn ring_attractor_bump_survives_three_seconds_of_noise() {
    let ring = noisy_ring_cued_once();
    for second in [1000.0, 2000.0, 3000.0].map(Milliseconds) {
        let counts = ring.ring_counts(second - SETTLED_WINDOW, second);
        assert!(
            cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW) >= 5,
            "{second}: {counts:?}"
        );
    }
}

const COMPASS_CUED_CELL: usize = 4;
const TURN_CURRENT: Nanoamperes = Nanoamperes(3.0);
const TURN: Milliseconds = Milliseconds(1000.0);
const TURN_BACK: Milliseconds = Milliseconds(2500.0);
const TURN_DURATION: Milliseconds = Milliseconds(500.0);
const COMPASS_END: Milliseconds = Milliseconds(4000.0);
const TURN_CLOCKWISE: &str = "Turn clockwise";
const TURN_ANTICLOCKWISE: &str = "Turn anticlockwise";

fn compass_turned_there_and_back() -> &'static RingLab {
    static RUN: OnceLock<RingLab> = OnceLock::new();
    RUN.get_or_init(|| {
        let mut compass = RingLab::load("compass.neuronify");
        let clockwise = compass.lab.labelled(TURN_CLOCKWISE);
        let anticlockwise = compass.lab.labelled(TURN_ANTICLOCKWISE);
        compass.tap(COMPASS_CUED_CELL, FIRST_CUE);
        for (source, start) in [(clockwise, TURN), (anticlockwise, TURN_BACK)] {
            compass.lab.run_until(start);
            compass.lab.set_current(source, TURN_CURRENT);
            compass.lab.run_until(start + TURN_DURATION);
            compass.lab.set_current(source, Nanoamperes::ZERO);
        }
        compass.lab.run_until(COMPASS_END);
        compass
    })
}

#[test]
fn compass_opens_at_8x_with_both_turn_currents_off() {
    let path = format!(
        "{}/examples/labs/compass.neuronify",
        env!("CARGO_MANIFEST_DIR")
    );
    let saved = load_from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(saved.speed, 8.0);
    let compass = RingLab::load("compass.neuronify");
    for text in [TURN_CLOCKWISE, TURN_ANTICLOCKWISE] {
        let source = compass.lab.labelled(text);
        let current = compass
            .lab
            .world
            .get::<&CurrentClamp>(source)
            .unwrap()
            .current_output;
        assert_eq!(current, Nanoamperes::ZERO);
    }
}

#[test]
fn compass_is_silent_before_the_cue() {
    let compass = compass_turned_there_and_back();
    assert_eq!(compass.lab.spikes_between(Milliseconds::ZERO, FIRST_CUE), 0);
}

#[test]
fn compass_cue_leaves_a_bump_that_holds_still() {
    let compass = compass_turned_there_and_back();
    let counts = compass.ring_counts(TURN - SETTLED_WINDOW, TURN);
    let centre = bump_centre_in_cells(&counts);
    let width = cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW);
    assert!((5..=9).contains(&width), "{counts:?}");
    assert!(
        distance_round_ring(centre, COMPASS_CUED_CELL as f32, compass.ring.len()) < 0.5,
        "{centre}"
    );
    let steps = compass.steps_of_the_bump(TURN - SETTLED_WINDOW, TURN);
    assert!(steps.iter().all(|step| step.abs() < 0.25), "{steps:?}");
}

#[test]
fn compass_turning_clockwise_runs_the_bump_clockwise() {
    let compass = compass_turned_there_and_back();
    let steps = compass.steps_of_the_bump(TURN, TURN + TURN_DURATION);
    let moved: f32 = steps.iter().sum();
    assert!((6.0..=12.0).contains(&moved), "{steps:?}");
    assert!(steps.iter().all(|&step| step > -0.25), "{steps:?}");
}

#[test]
fn compass_turning_anticlockwise_runs_the_bump_back() {
    let compass = compass_turned_there_and_back();
    let steps = compass.steps_of_the_bump(TURN_BACK, TURN_BACK + TURN_DURATION);
    let moved: f32 = steps.iter().sum();
    assert!((-12.0..=-6.0).contains(&moved), "{steps:?}");
    assert!(steps.iter().all(|&step| step < 0.25), "{steps:?}");
}

#[test]
fn compass_bump_stays_where_each_turn_left_it() {
    let compass = compass_turned_there_and_back();
    let settle = Milliseconds(200.0);
    for (stop, next) in [
        (TURN + TURN_DURATION, TURN_BACK),
        (TURN_BACK + TURN_DURATION, COMPASS_END),
    ] {
        let steps = compass.steps_of_the_bump(stop + settle, next);
        let drift: f32 = steps.iter().sum();
        assert!(drift.abs() < 0.5, "{stop}: {steps:?}");
        let counts = compass.ring_counts(next - SETTLED_WINDOW, next);
        let width = cells_firing_at_10_hz_or_more(&counts, SETTLED_WINDOW);
        assert!((5..=9).contains(&width), "{stop}: {counts:?}");
    }
}

#[test]
fn compass_turn_currents_light_nothing_without_a_bump() {
    let mut compass = RingLab::load("compass.neuronify");
    for text in [TURN_CLOCKWISE, TURN_ANTICLOCKWISE] {
        let source = compass.lab.labelled(text);
        compass.lab.set_current(source, Nanoamperes(30.0));
    }
    compass.lab.run_until(Milliseconds(1000.0));
    let excitatory = compass
        .lab
        .spikes
        .iter()
        .filter(|&&(cell, _)| neuron_type(&compass.lab.world, cell) == NeuronType::Excitatory)
        .count();
    assert_eq!(excitatory, 0);
}

const PHONE_WIDTH: f32 = 27.0;
const SHEETS: [&str; 3] = [
    "hopfield.neuronify",
    "hopfield_overloaded.neuronify",
    "direction_selectivity.neuronify",
];

#[test]
fn every_lab_written_as_a_sheet_fits_an_upright_phone() {
    for file in SHEETS {
        let world = Lab::load(file).world;
        let bounds = canvas_bounds(&world).unwrap();
        let width = bounds.max.z - bounds.min.z;
        assert!(width <= PHONE_WIDTH, "{file} is {width} wide");
    }
}

#[test]
fn no_note_covers_a_lab_circuit() {
    for file in SHEETS {
        let world = Lab::load(file).world;
        let notes: Vec<(Vec3, NoteSize)> = world
            .query::<(&Position, &NoteSize)>()
            .iter()
            .map(|(position, size)| (position.position, size.clone()))
            .collect();
        for (entity, position) in world
            .query::<(Entity, &Position)>()
            .without::<&NoteSize>()
            .iter()
        {
            for (center, size) in &notes {
                let offset = position.position - *center;
                let inside = offset.x.abs() < size.height / 2.0 + NODE_RADIUS
                    && offset.z.abs() < size.width / 2.0 + NODE_RADIUS;
                assert!(!inside, "{file}: {entity:?} is under a note");
            }
        }
    }
}

fn labelled(world: &World, label: &str) -> Entity {
    world
        .query::<(Entity, &Label)>()
        .iter()
        .find(|(_, neuron)| neuron.text == label)
        .unwrap_or_else(|| panic!("no neuron labelled {label}"))
        .0
}

const HOPFIELD: &str = "hopfield.neuronify";
const HOPFIELD_OVERLOADED: &str = "hopfield_overloaded.neuronify";
const CELLS: usize = write_labs::GRID * write_labs::GRID;
const FIRST_PRESS: Milliseconds = Milliseconds(50.0);
const BETWEEN_PRESSES: Milliseconds = Milliseconds(4.0);
const HELD_FROM: Milliseconds = Milliseconds(500.0);
const HELD_UNTIL: Milliseconds = Milliseconds(800.0);
const RESET_AT: Milliseconds = Milliseconds(800.0);
const QUIET_FROM: Milliseconds = Milliseconds(1000.0);
const QUIET_UNTIL: Milliseconds = Milliseconds(1200.0);

struct Recall {
    held: Vec<usize>,
    rate: Hertz,
    spikes_after_reset: usize,
}

fn row_major(a: Vec3, b: Vec3) -> std::cmp::Ordering {
    b.x.total_cmp(&a.x).then(a.z.total_cmp(&b.z))
}

fn recall(file: &str, pressed: &[usize]) -> Recall {
    let mut lab = Lab::load(file);
    let mut cells: Vec<Entity> = lab
        .neurons()
        .into_iter()
        .filter(|&cell| neuron_type(&lab.world, cell) == NeuronType::Excitatory)
        .collect();
    cells.sort_by(|&a, &b| row_major(lab.position(a), lab.position(b)));
    let mut pads: Vec<Vec3> = lab
        .world
        .query::<&Position>()
        .with::<&TouchSensor>()
        .iter()
        .map(|position| position.position)
        .collect();
    pads.sort_by(|&a, &b| row_major(a, b));
    let reset = *pads.last().unwrap();
    for (press, &cell) in pressed.iter().enumerate() {
        let at = FIRST_PRESS + BETWEEN_PRESSES * press as f64;
        lab.run_until(at);
        lab.press_until(pads[cell], at + TAP_DURATION);
    }
    lab.run_until(RESET_AT);
    lab.press_until(reset, RESET_AT + TAP_DURATION);
    lab.run_until(QUIET_UNTIL);
    let counts = lab.spike_counts(&cells, HELD_FROM, HELD_UNTIL);
    let window = HELD_UNTIL - HELD_FROM;
    let held: Vec<usize> = (0..CELLS)
        .filter(|&cell| counts[cell] as f64 >= Hertz(10.0).cycles_in(window))
        .collect();
    let spikes_per_cell =
        held.iter().map(|&cell| counts[cell] as f64).sum::<f64>() / held.len().max(1) as f64;
    let rate = (spikes_per_cell / window).to_hertz();
    let spikes_after_reset = lab
        .spike_counts(&cells, QUIET_FROM, QUIET_UNTIL)
        .iter()
        .sum();
    Recall {
        held,
        rate,
        spikes_after_reset,
    }
}

fn letter_cells(letter: char) -> Vec<usize> {
    (0..CELLS)
        .filter(|&cell| write_labs::lit(letter, cell / write_labs::GRID, cell % write_labs::GRID))
        .collect()
}

fn two_cells_only_in(letter: char) -> [usize; 2] {
    let only: Vec<usize> = letter_cells(letter)
        .into_iter()
        .filter(|cell| {
            write_labs::STORED
                .iter()
                .filter(|&&other| other != letter)
                .all(|&other| !letter_cells(other).contains(cell))
        })
        .collect();
    [only[only.len() - 2], only[only.len() - 1]]
}

#[test]
fn hopfield_one_pressed_neuron_brings_back_nothing() {
    let recalled = recall(HOPFIELD, &[letter_cells('T')[8]]);
    assert!(recalled.held.is_empty(), "{:?}", recalled.held);
}

#[test]
fn hopfield_two_pressed_neurons_bring_back_their_whole_letter_until_reset() {
    for letter in write_labs::STORED {
        let recalled = recall(HOPFIELD, &two_cells_only_in(letter));
        assert_eq!(recalled.held, letter_cells(letter), "{letter}");
        assert!(
            (Hertz(80.0)..=Hertz(200.0)).contains(&recalled.rate),
            "{letter}: {}",
            recalled.rate
        );
        assert_eq!(recalled.spikes_after_reset, 0, "{letter}");
    }
}

#[test]
fn hopfield_a_wrong_extra_neuron_does_not_stop_the_letter_coming_back() {
    let wrong = (0..CELLS)
        .find(|&cell| {
            write_labs::STORED
                .iter()
                .all(|&letter| !letter_cells(letter).contains(&cell))
        })
        .unwrap();
    let [first, second] = two_cells_only_in('T');
    let recalled = recall(HOPFIELD, &[first, second, wrong]);
    assert_eq!(recalled.held, letter_cells('T'));
}

#[test]
fn hopfield_corners_shared_by_two_letters_bring_back_one_of_them() {
    let shared: Vec<usize> = letter_cells('L')
        .into_iter()
        .filter(|cell| letter_cells('X').contains(cell) && !letter_cells('T').contains(cell))
        .collect();
    let recalled = recall(HOPFIELD, &shared);
    assert!(
        recalled.held == letter_cells('L') || recalled.held == letter_cells('X'),
        "{:?}",
        recalled.held
    );
}

#[test]
fn hopfield_overloaded_cues_land_on_other_patterns() {
    for letter in ['T', 'L'] {
        let recalled = recall(HOPFIELD_OVERLOADED, &two_cells_only_in(letter));
        assert!(!recalled.held.is_empty(), "{letter}");
        assert_ne!(recalled.held, letter_cells(letter), "{letter}");
    }
}

#[test]
fn hopfield_neurons_keep_dales_law_with_one_inhibitory_partner_each() {
    let lab = Lab::load(HOPFIELD);
    let world = &lab.world;
    for synapse in world.query::<&neuronify_core::CurrentSynapse>().iter() {
        assert!(synapse.maximum_current > neuronify_core::Nanoamperes::ZERO);
    }
    let excitatory: Vec<Entity> = lab
        .neurons()
        .into_iter()
        .filter(|&cell| neuron_type(world, cell) == NeuronType::Excitatory)
        .collect();
    assert_eq!(excitatory.len(), CELLS);
    for cell in excitatory {
        let partners = world
            .query::<&Connection>()
            .iter()
            .filter(|connection| {
                connection.from == cell
                    && neuron_type(world, connection.to) == NeuronType::Inhibitory
            })
            .count();
        assert_eq!(partners, 1);
    }
}

#[test]
fn hopfield_labs_open_with_rings_around_the_neurons_one_neuron_is_connected_to() {
    for file in [HOPFIELD, HOPFIELD_OVERLOADED] {
        let world = Lab::load(file).world;
        assert_eq!(ConnectionView::of(&world), ConnectionView::Rings);
        let selected = Selected::entity(&world).unwrap();
        assert_eq!(neuron_type(&world, selected), NeuronType::Excitatory);
    }
}

#[test]
fn hopfield_reset_is_labelled() {
    let world = Lab::load(HOPFIELD).world;
    let reset = labelled(&world, write_labs::RESET);
    assert_eq!(neuron_type(&world, reset), NeuronType::Inhibitory);
}

const DIRECTION_SELECTIVITY: &str = "direction_selectivity.neuronify";

struct DirectionLab {
    world: World,
    receptors: Vec<Entity>,
    rightward: Vec<Entity>,
    output: Entity,
}

fn direction_lab() -> DirectionLab {
    let world = Lab::load(DIRECTION_SELECTIVITY).world;
    DirectionLab {
        receptors: write_labs::RECEPTORS
            .iter()
            .map(|&label| labelled(&world, label))
            .collect(),
        rightward: write_labs::PAIRS
            .iter()
            .map(|&label| labelled(&world, label))
            .collect(),
        output: labelled(&world, write_labs::OUTPUT),
        world,
    }
}

fn visual_inputs(world: &World) -> Vec<Entity> {
    world
        .query::<(Entity, &VisualInput)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect()
}

fn at_speed(mut world: World, speed: Hertz) -> World {
    for input in world.query_mut::<&mut VisualInput>() {
        input.speed = Hertz(speed.0.copysign(input.speed.0));
    }
    world
}

fn reversed(mut world: World) -> World {
    let picture = world
        .query::<&Position>()
        .with::<&VisualInput>()
        .iter()
        .map(|picture| picture.position)
        .next()
        .expect("the lab has a picture");
    let mut editor = Editor::default();
    assert!(editor.reverse_picture_at(&mut world, direction_badge(picture)));
    world
}

struct Sweep {
    lab: Lab,
    passes: usize,
}

impl Sweep {
    fn run(world: World, passes: usize) -> Sweep {
        let picture = visual_inputs(&world)[0];
        let speed = world.get::<&VisualInput>(picture).unwrap().speed.abs();
        let mut lab = Lab {
            world,
            engine: Engine::default(),
            editor: Editor::default(),
            time: Milliseconds::ZERO,
            spikes: Vec::new(),
        };
        lab.run_until((passes as f64 / speed).to_milliseconds());
        Sweep { lab, passes }
    }

    fn per_pass(&self, cells: &[Entity]) -> Vec<f64> {
        self.lab
            .spike_counts(cells, Milliseconds::ZERO, self.lab.time)
            .into_iter()
            .map(|count| count as f64 / self.passes as f64)
            .collect()
    }

    fn first_spikes(&self, cells: &[Entity]) -> Vec<Milliseconds> {
        cells
            .iter()
            .map(|&cell| {
                self.lab
                    .spike_times(cell)
                    .first()
                    .copied()
                    .unwrap_or(Milliseconds(f64::NAN))
            })
            .collect()
    }
}

#[test]
fn direction_selectivity_opens_at_the_fastest_playback_with_five_fields_on_one_picture() {
    let path = format!(
        "{}/examples/labs/{DIRECTION_SELECTIVITY}",
        env!("CARGO_MANIFEST_DIR")
    );
    let saved = load_from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(saved.speed, 8.0);
    let pictures = visual_inputs(&saved.world);
    assert_eq!(pictures.len(), 1);
    let fields: Vec<Entity> = saved
        .world
        .query::<(Entity, &ReceptiveFieldCell)>()
        .iter()
        .map(|(cell, _)| cell)
        .collect();
    assert_eq!(fields.len(), 5);
    for field in fields {
        assert_eq!(watched_picture(&saved.world, field), Some(pictures[0]));
    }
    let picture = saved.world.get::<&VisualInput>(pictures[0]).unwrap();
    assert_eq!(picture.speed, write_labs::PICTURE_SPEED);
}

#[test]
fn direction_selectivity_has_one_right_neuron_between_each_two_receptors_all_sending_to_the_output()
{
    let lab = direction_lab();
    let position = |cell: Entity| lab.world.get::<&Position>(cell).unwrap().position;
    assert_eq!(lab.rightward.len(), lab.receptors.len() - 1);
    for (index, &cell) in lab.rightward.iter().enumerate() {
        let between =
            (position(lab.receptors[index]).z + position(lab.receptors[index + 1]).z) / 2.0;
        assert!((position(cell).z - between).abs() < 1e-3);
        assert!(position(lab.output).x < position(cell).x);
        let to_output = lab
            .world
            .query::<&Connection>()
            .with::<&neuronify_core::CurrentSynapse>()
            .iter()
            .filter(|connection| connection.from == cell && connection.to == lab.output)
            .count();
        assert_eq!(to_output, 1);
    }
    let others: Vec<Entity> = lab
        .world
        .query::<(Entity, &LeakyNeuron)>()
        .iter()
        .map(|(cell, _)| cell)
        .filter(|&cell| neuron_type(&lab.world, cell) == NeuronType::Excitatory)
        .filter(|cell| !lab.receptors.contains(cell) && !lab.rightward.contains(cell))
        .collect();
    assert_eq!(others, [lab.output]);
}

#[test]
fn direction_selectivity_opens_showing_all_connections_with_nothing_selected() {
    let lab = direction_lab();
    assert_eq!(ConnectionView::of(&lab.world), ConnectionView::All);
    assert_eq!(Selected::entity(&lab.world), None);
}

#[test]
fn direction_selectivity_draws_no_firing_rate_halos() {
    let sweep = Sweep::run(direction_lab().world, 1);
    assert!(collect_halos(&sweep.lab.world, &Theme::Dark.palette()).is_empty());
}

#[test]
fn direction_selectivity_spike_detector_lists_the_receptors_then_the_right_neurons_then_the_output()
{
    let lab = direction_lab();
    let detector = lab
        .world
        .query::<(Entity, &SpikeDetector)>()
        .iter()
        .map(|(entity, _)| entity)
        .next()
        .unwrap();
    let rows: Vec<Entity> = measured(&lab.world, detector)
        .into_iter()
        .map(|(_, neuron)| neuron)
        .collect();
    let expected: Vec<Entity> = [&lab.receptors, &lab.rightward, &vec![lab.output]]
        .into_iter()
        .flatten()
        .copied()
        .collect();
    assert_eq!(rows, expected);
}

#[test]
fn the_right_neurons_fire_one_after_another_as_the_bar_moves_right_and_drive_the_output() {
    let lab = direction_lab();
    let sweep = Sweep::run(lab.world, 4);
    let right = sweep.per_pass(&lab.rightward);
    let output = sweep.per_pass(&[lab.output])[0];
    assert!(
        right.iter().all(|&spikes| spikes >= 1.0),
        "{right:?} {output}"
    );
    assert!(output >= 3.0, "{right:?} {output}");
    let first = sweep.first_spikes(&lab.rightward);
    assert!(first.windows(2).all(|pair| pair[0] < pair[1]), "{first:?}");
}

#[test]
fn a_bar_moving_left_leaves_the_right_neurons_and_the_output_silent() {
    let lab = direction_lab();
    let sweep = Sweep::run(reversed(lab.world), 4);
    let right = sweep.per_pass(&lab.rightward);
    let output = sweep.per_pass(&[lab.output])[0];
    assert!(right.iter().sum::<f64>() < 0.5, "{right:?} {output}");
    assert_eq!(output, 0.0, "{right:?}");
}

#[test]
fn a_slow_bar_moving_left_fires_the_output_too_once_the_inhibition_wears_off() {
    let lab = direction_lab();
    let sweep = Sweep::run(at_speed(reversed(lab.world), Hertz(1.0)), 3);
    let right: f64 = sweep.per_pass(&lab.rightward).iter().sum();
    let output = sweep.per_pass(&[lab.output])[0];
    assert!(right >= 4.0, "{right} {output}");
    assert!(output >= 1.0, "{right} {output}");
}

#[test]
fn without_the_inhibitory_partner_of_c_the_right_neuron_between_b_and_c_fires_for_a_leftward_bar() {
    let mut lab = direction_lab();
    let c = lab.receptors[2];
    let world = &mut lab.world;
    let partner = world
        .query::<&Connection>()
        .iter()
        .map(|connection| connection.to)
        .find(|&to| {
            world
                .query::<&Connection>()
                .iter()
                .any(|connection| connection.from == c && connection.to == to)
                && neuron_type(world, to) == NeuronType::Inhibitory
        })
        .unwrap();
    let touching: Vec<Entity> = world
        .query::<(Entity, &Connection)>()
        .iter()
        .filter(|(_, connection)| connection.from == partner || connection.to == partner)
        .map(|(connection, _)| connection)
        .collect();
    for connection in touching {
        world.despawn(connection).unwrap();
    }
    world.despawn(partner).unwrap();
    let sweep = Sweep::run(reversed(lab.world), 4);
    let right = sweep.per_pass(&lab.rightward);
    assert!(right[1] >= 1.0, "{right:?}");
    for others in [right[0], right[2], right[3]] {
        assert!(10.0 * others <= right[1], "{right:?}");
    }
}
