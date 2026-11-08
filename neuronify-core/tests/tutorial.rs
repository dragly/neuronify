#[allow(dead_code)]
#[path = "../generators/write_tutorial.rs"]
mod write_tutorial;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::dendrites::compartment_tree;
use neuronify_core::elements::{spawn_cable, spawn_compartment, spawn_synapse};
use neuronify_core::examples::{find, EXAMPLE_CATEGORIES};
use neuronify_core::rendering::canvas_bounds;
use neuronify_core::simulation::model::prepare;
use neuronify_core::simulation::{fire_nearby, step_simulation, Engine};
use neuronify_core::{
    Annotation, Connection, CurrentClamp, LeakyNeuron, Link, Milliseconds, Millivolts, Nanoamperes,
    NeuronDisplay, NeuronType, NodeState, NoteSize, Position, Preparation, Setting, TouchSensor,
    DEFAULT_SPEED, DT, NODE_RADIUS, SEGMENT_LENGTH, SIMULATION_STEPS_PER_SECOND,
    TERMINAL_THRESHOLD,
};

const STEPS_PER_SECOND: f64 = DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND.0;
const SIMULATED_PER_REAL_SECOND: Milliseconds = Milliseconds(STEPS_PER_SECOND * DT.0);
const PHONE_WIDTH: f32 = 27.0;

struct Canvas {
    world: World,
    engine: Engine,
    time: Milliseconds,
    spikes: Vec<(Entity, Milliseconds)>,
}

const STEPS_PER_FRAME: u32 = 8;

impl Canvas {
    fn open(file: &str) -> Canvas {
        Canvas {
            world: find(file).unwrap().spawn().unwrap(),
            engine: Engine::default(),
            time: Milliseconds::ZERO,
            spikes: Vec::new(),
        }
    }

    fn prepared(file: &str, preparation: Preparation) -> Canvas {
        let mut canvas = Canvas::open(file);
        prepare(&mut canvas.world, preparation);
        canvas
    }

    fn wait(&mut self, seconds: f64) {
        let mut remaining = (seconds * STEPS_PER_SECOND).round() as u32;
        while remaining > 0 {
            let steps = remaining.min(STEPS_PER_FRAME);
            remaining -= steps;
            step_simulation(
                &mut self.world,
                &mut self.engine,
                &mut self.time,
                steps,
                None,
            );
            self.spikes.extend(
                self.engine
                    .spikes()
                    .iter()
                    .map(|spike| (spike.entity, spike.time)),
            );
        }
    }

    fn now(&self) -> Milliseconds {
        self.time
    }

    fn wait_for_spike(&mut self, neuron: Entity) -> Milliseconds {
        let since = self.now();
        while self.spikes_since(neuron, since).is_empty() {
            self.wait(STEPS_PER_FRAME as f64 / STEPS_PER_SECOND);
        }
        self.spikes_since(neuron, since)[0]
    }

    fn press(&mut self, hand: Vec3) {
        fire_nearby(&mut self.world, &mut self.engine, hand);
    }

    fn spikes_since(&self, neuron: Entity, since: Milliseconds) -> Vec<Milliseconds> {
        self.spikes
            .iter()
            .filter(|(entity, time)| *entity == neuron && *time >= since)
            .map(|(_, time)| *time)
            .collect()
    }

    fn voltage(&self, neuron: Entity) -> Millivolts {
        self.world.get::<&NodeState>(neuron).unwrap().voltage
    }

    fn neurons(&self, neuron_type: NeuronType) -> Vec<(Entity, Vec3)> {
        let mut neurons: Vec<(Entity, Vec3)> = self
            .world
            .query::<(&Position, &NeuronType)>()
            .with::<&LeakyNeuron>()
            .iter()
            .filter(|(_, (_, kind))| **kind == neuron_type)
            .map(|(entity, (position, _))| (entity, position.position))
            .collect();
        neurons.sort_by(|a, b| reading_order(a.1, b.1));
        neurons
    }

    fn hands(&self) -> Vec<Vec3> {
        let mut hands: Vec<Vec3> = self
            .world
            .query::<&Position>()
            .with::<&TouchSensor>()
            .iter()
            .map(|(_, position)| position.position)
            .collect();
        hands.sort_by(|a, b| reading_order(*a, *b));
        hands
    }
}

fn reading_order(a: Vec3, b: Vec3) -> std::cmp::Ordering {
    b.x.partial_cmp(&a.x)
        .unwrap()
        .then(a.z.partial_cmp(&b.z).unwrap())
}

fn tutorial() -> Vec<&'static str> {
    EXAMPLE_CATEGORIES[0]
        .examples
        .iter()
        .map(|example| example.file())
        .collect()
}

#[test]
fn each_canvas_links_to_the_next_and_the_last_to_the_examples() {
    let files = tutorial();
    assert_eq!(files.len(), 11);
    for (index, file) in files.iter().enumerate() {
        let canvas = Canvas::open(file);
        let links: Vec<String> = canvas
            .world
            .query::<&Link>()
            .iter()
            .map(|(_, link)| link.example.clone())
            .collect();
        assert_eq!(links.len(), 1, "{file}");
        match files.get(index + 1) {
            Some(next) => assert_eq!(links[0], *next),
            None => assert!(find(&links[0]).is_none()),
        }
    }
}

#[test]
fn only_the_sharpening_edges_canvas_shows_firing_rates() {
    for file in tutorial() {
        let display = NeuronDisplay::of(&Canvas::open(file).world);
        assert_eq!(display.firing_rate, file.contains("sharpening"), "{file}");
        assert!(display.voltage, "{file}");
    }
}

#[test]
fn no_note_covers_the_circuit() {
    for file in tutorial() {
        let canvas = Canvas::open(file);
        let notes: Vec<(Vec3, NoteSize)> = canvas
            .world
            .query::<(&Position, &NoteSize)>()
            .iter()
            .map(|(_, (position, size))| (position.position, size.clone()))
            .collect();
        for (entity, position) in canvas
            .world
            .query::<&Position>()
            .without::<&NoteSize>()
            .iter()
        {
            for (center, size) in &notes {
                let offset = position.position - *center;
                let inside = offset.x.abs() < size.height / 2.0 + NODE_RADIUS
                    && offset.z.abs() < size.width / 2.0 + NODE_RADIUS;
                assert!(
                    !inside,
                    "{file}: {entity:?} at {:?} is under a note",
                    position.position
                );
            }
        }
    }
}

#[test]
fn each_canvas_fits_an_upright_phone() {
    for file in tutorial() {
        let bounds = canvas_bounds(&Canvas::open(file).world).unwrap();
        let width = bounds.max.z - bounds.min.z;
        assert!(width <= PHONE_WIDTH, "{file} is {width} wide");
    }
}

#[test]
fn notes_name_tools_that_exist() {
    let tools: Vec<&str> = [neuronify_core::Tool::Erase, neuronify_core::Tool::Axon]
        .map(|tool| tool.name())
        .to_vec();
    let text: String = tutorial()
        .iter()
        .flat_map(|file| {
            let canvas = Canvas::open(file);
            let notes: Vec<String> = canvas
                .world
                .query::<&Annotation>()
                .iter()
                .map(|(_, note)| note.text.clone())
                .collect();
            notes
        })
        .collect();
    for tool in tools {
        assert!(text.contains(&format!("Choose {tool}")), "{tool}");
    }
}

#[test]
fn a_press_sends_a_spike_down_the_chain() {
    let mut canvas = Canvas::open("tutorial/1_spikes.neuronify");
    let neurons = canvas.neurons(NeuronType::Excitatory);
    let hand = canvas.hands()[0];
    canvas.wait(2.0);
    canvas.press(hand);
    let pressed = canvas.now();
    canvas.wait(1.0);
    for (neuron, _) in &neurons {
        assert_eq!(canvas.spikes_since(*neuron, pressed).len(), 1);
    }
    canvas.wait(4.0);
    for (neuron, _) in &neurons {
        assert_eq!(canvas.spikes_since(*neuron, pressed).len(), 1);
    }
}

#[test]
fn quick_presses_add_up_and_slow_ones_leak_away() {
    let presses = |every: f64, count: usize| {
        let mut canvas = Canvas::open("tutorial/2_leaks.neuronify");
        let (neuron, _) = canvas.neurons(NeuronType::Excitatory)[0];
        let hand = canvas.hands()[0];
        for _ in 0..count {
            canvas.press(hand);
            canvas.wait(every);
        }
        canvas.wait(1.0);
        canvas.spikes_since(neuron, Milliseconds::ZERO).len()
    };
    assert_eq!(presses(3.0, 1), 0);
    assert!(presses(0.25, 3) >= 1);
    assert!(presses(0.4, 4) >= 1);
    assert_eq!(presses(1.5, 6), 0);
}

#[test]
fn the_blue_side_pushes_and_the_red_side_pulls() {
    let mut canvas = Canvas::open("tutorial/3_push_and_pull.neuronify");
    let middle = canvas.neurons(NeuronType::Excitatory)[1].0;
    let [push_hand, pull_hand] = canvas.hands()[..] else {
        panic!("expected two hands");
    };
    canvas.wait(5.0);
    let alone = canvas.spikes_since(middle, Milliseconds::ZERO);
    let period = alone[alone.len() - 1] - alone[alone.len() - 2];
    let real_period = period / SIMULATED_PER_REAL_SECOND;
    assert!(
        (1.0..2.5).contains(&real_period),
        "fires every {real_period} s"
    );

    let last = canvas.wait_for_spike(middle);
    canvas.press(push_hand);
    canvas.wait(2.0);
    let next = canvas.spikes_since(middle, last + Milliseconds(1e-6))[0];
    assert!(
        next - last < 0.7 * period,
        "fired after {} instead of {period}",
        next - last
    );

    canvas.wait(1.0);
    let last = canvas.wait_for_spike(middle);
    canvas.press(pull_hand);
    canvas.wait(0.6);
    let rest = canvas
        .world
        .get::<&LeakyNeuron>(middle)
        .unwrap()
        .resting_potential;
    assert!(canvas.voltage(middle) < rest);
    canvas.wait(3.0);
    let next = canvas.spikes_since(middle, last + Milliseconds(1e-6))[0];
    assert!(
        next - last > 1.5 * period,
        "waited {} instead of {period}",
        next - last
    );
}

#[test]
fn one_side_wins_until_the_quiet_side_is_pressed() {
    let mut canvas = Canvas::open("tutorial/4_choosing.neuronify");
    let blue = canvas.neurons(NeuronType::Excitatory);
    let (left, right) = (blue[0].0, blue[1].0);
    let hands = canvas.hands();
    let (left_hand, right_hand) = if hands[0].z < hands[1].z {
        (hands[0], hands[1])
    } else {
        (hands[1], hands[0])
    };
    let winner_after = |canvas: &mut Canvas, settling: f64| {
        canvas.wait(settling);
        let settled = canvas.now();
        canvas.wait(8.0);
        (
            canvas.spikes_since(left, settled).len(),
            canvas.spikes_since(right, settled).len(),
        )
    };
    let (left_spikes, right_spikes) = winner_after(&mut canvas, 1.0);
    assert!(left_spikes > 4 && right_spikes == 0);

    canvas.press(right_hand);
    let (left_spikes, right_spikes) = winner_after(&mut canvas, 6.0);
    assert!(left_spikes == 0 && right_spikes > 4);

    canvas.press(left_hand);
    let (left_spikes, right_spikes) = winner_after(&mut canvas, 6.0);
    assert!(left_spikes > 4 && right_spikes == 0);
}

fn detector_counts(canvas: &mut Canvas, seconds: f64) -> Vec<usize> {
    let detectors = canvas.neurons(NeuronType::Excitatory);
    canvas.wait(2.0);
    let start = canvas.now();
    canvas.wait(seconds);
    detectors
        .iter()
        .map(|(neuron, _)| canvas.spikes_since(*neuron, start).len())
        .collect()
}

#[test]
fn sideways_pull_makes_the_edge_stand_out() {
    let mut canvas = Canvas::open("tutorial/5_sharpening_edges.neuronify");
    let with_pull = detector_counts(&mut canvas, 20.0);
    assert!(with_pull[2] > with_pull[1]);
    assert!(with_pull[0] < with_pull[2] && with_pull[5] <= with_pull[4] + 1);
    assert!(with_pull[3] < with_pull[4]);
    assert!(with_pull[3] * 2 < with_pull[4]);

    let partners: Vec<Entity> = canvas
        .neurons(NeuronType::Inhibitory)
        .into_iter()
        .map(|(entity, _)| entity)
        .collect();
    for partner in partners {
        canvas.world.despawn(partner).unwrap();
    }
    let without = detector_counts(&mut canvas, 20.0);
    assert!(without[1].abs_diff(without[2]) <= 1);
    assert!(without[3].abs_diff(without[4]) <= 1);
}

#[test]
fn closing_the_ring_keeps_the_beat() {
    let mut canvas = Canvas::open("tutorial/6_keeping_a_rhythm.neuronify");
    let ring = canvas.neurons(NeuronType::Excitatory);
    let hand = canvas.hands()[0];
    canvas.press(hand);
    canvas.wait(3.0);
    let open = canvas.now();
    canvas.wait(3.0);
    assert!(ring
        .iter()
        .all(|(neuron, _)| canvas.spikes_since(*neuron, open).is_empty()));

    let bottom_left = ring[2].0;
    let top_left = ring[0].0;
    spawn_synapse(&mut canvas.world, bottom_left, top_left);
    canvas.press(hand);
    canvas.wait(10.0);
    let late = canvas.now() - 2.0 * SIMULATED_PER_REAL_SECOND;
    assert!(ring
        .iter()
        .all(|(neuron, _)| !canvas.spikes_since(*neuron, late).is_empty()));
}

fn the_spike_reaches_the_near_neuron_before_the_far_one(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/7_axons.neuronify", preparation);
    let mut neurons = canvas.neurons(NeuronType::Excitatory);
    neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
    let (near, far) = (neurons[1].0, neurons[2].0);
    canvas.press(canvas.hands()[0]);
    canvas.wait(3.0);
    let near_spike = canvas.spikes_since(near, Milliseconds::ZERO)[0];
    let far_spike = canvas.spikes_since(far, Milliseconds::ZERO)[0];
    let delay = (far_spike - near_spike) / SIMULATED_PER_REAL_SECOND;
    assert!(delay > 0.25, "the far neuron fired only {delay} s later");
}

fn axon_length(world: &World, from: Entity, to: Entity) -> f32 {
    let position = |entity: Entity| world.get::<&Position>(entity).unwrap().position;
    let mut length = 0.0;
    let mut current = to;
    while current != from {
        let previous = world
            .query::<&Connection>()
            .iter()
            .find(|(_, connection)| connection.to == current)
            .map(|(_, connection)| connection.from)
            .unwrap();
        length += position(previous).distance(position(current));
        current = previous;
    }
    length
}

fn the_spike_on_the_myelinated_axon_arrives_first(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/8_myelin.neuronify", preparation);
    let neurons = canvas.neurons(NeuronType::Excitatory);
    let (sender, plain, myelinated) = (neurons[0].0, neurons[1].0, neurons[2].0);
    let lengths = [plain, myelinated].map(|target| axon_length(&canvas.world, sender, target));
    assert!(
        (lengths[0] - lengths[1]).abs() < 0.05 * lengths[0],
        "{lengths:?}"
    );
    canvas.press(canvas.hands()[0]);
    canvas.wait(3.0);
    let sent = canvas.spikes_since(sender, Milliseconds::ZERO)[0];
    let arrivals = [plain, myelinated].map(|target| {
        (canvas.spikes_since(target, Milliseconds::ZERO)[0] - sent) / SIMULATED_PER_REAL_SECOND
    });
    assert!(arrivals[0] - arrivals[1] > 0.25, "{arrivals:?}");
}

fn an_axon_grown_onto_a_branch_fires_the_neuron(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/9_dendrites.neuronify", preparation);
    let mut neurons = canvas.neurons(NeuronType::Excitatory);
    neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
    let (sender, receiver) = (neurons[0].0, neurons[1].0);
    let hand = canvas.hands()[0];
    canvas.press(hand);
    canvas.wait(3.0);
    assert!(canvas.spikes_since(receiver, Milliseconds::ZERO).is_empty());

    let start = canvas.world.get::<&Position>(sender).unwrap().position;
    let branch = compartment_tree(&canvas.world, receiver)
        .into_iter()
        .min_by(|a, b| {
            let distance = |entity: &Entity| {
                canvas
                    .world
                    .get::<&Position>(*entity)
                    .unwrap()
                    .position
                    .distance(start)
            };
            distance(a).total_cmp(&distance(b))
        })
        .unwrap();
    let end = canvas.world.get::<&Position>(branch).unwrap().position;
    let pieces = (start.distance(end) / SEGMENT_LENGTH).round() as usize;
    let mut previous = sender;
    for piece in 1..pieces {
        let compartment = spawn_compartment(
            &mut canvas.world,
            start.lerp(end, piece as f32 / pieces as f32),
            NeuronType::Excitatory,
        );
        spawn_cable(&mut canvas.world, previous, compartment, false);
        previous = compartment;
    }
    spawn_cable(&mut canvas.world, previous, branch, true);

    let pressed = canvas.now();
    canvas.press(hand);
    canvas.wait(3.0);
    assert!(!canvas.spikes_since(receiver, pressed).is_empty());
}

fn weak_endings_fire_the_neuron_only_together(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/10_coincidence.neuronify", preparation);
    let mut neurons = canvas.neurons(NeuronType::Excitatory);
    neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
    let receiver = neurons.last().unwrap().0;
    let hands = canvas.hands();
    let (top, middle, bottom) = (hands[0], hands[1], hands[2]);
    for hand in [top, bottom] {
        let pressed = canvas.now();
        canvas.press(hand);
        canvas.wait(3.0);
        assert!(canvas.spikes_since(receiver, pressed).is_empty());
    }

    let pressed = canvas.now();
    canvas.press(top);
    canvas.wait(0.3);
    canvas.press(bottom);
    canvas.wait(3.0);
    assert!(canvas.spikes_since(receiver, pressed).is_empty());

    let pressed = canvas.now();
    canvas.press(middle);
    canvas.wait(3.0);
    assert!(!canvas.spikes_since(receiver, pressed).is_empty());
}

fn the_red_axon_blocks_the_top_input_and_lets_the_bottom_one_through(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/11_inhibition_on_a_branch.neuronify", preparation);
    let mut neurons = canvas.neurons(NeuronType::Excitatory);
    neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
    let receiver = neurons.last().unwrap().0;
    let hands = canvas.hands();
    let (top, bottom) = (hands[0], hands[1]);
    let fires = |canvas: &mut Canvas, hand: Vec3| {
        let pressed = canvas.now();
        canvas.press(hand);
        canvas.wait(3.0);
        !canvas.spikes_since(receiver, pressed).is_empty()
    };
    for pause in [1.0, 1.2, 1.4, 1.6, 1.8] {
        canvas.wait(pause);
        assert!(!fires(&mut canvas, top));
        assert!(fires(&mut canvas, bottom));
    }

    for (_, clamp) in canvas.world.query::<&mut CurrentClamp>().iter() {
        clamp.current_output = Nanoamperes::ZERO;
    }
    canvas.wait(3.0);
    assert!(fires(&mut canvas, top));
}

fn the_top_spike_runs_down_to_the_red_ending_and_stops_there(preparation: Preparation) {
    let mut canvas = Canvas::prepared("tutorial/11_inhibition_on_a_branch.neuronify", preparation);
    let mut neurons = canvas.neurons(NeuronType::Excitatory);
    neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
    let receiver = neurons.last().unwrap().0;
    let tree = compartment_tree(&canvas.world, receiver);
    let ending = |canvas: &Canvas, neuron_type: NeuronType| {
        canvas
            .world
            .query::<&Connection>()
            .iter()
            .filter(|(_, connection)| {
                connection.directional
                    && tree.contains(&connection.to)
                    && *canvas.world.get::<&NeuronType>(connection.from).unwrap() == neuron_type
            })
            .map(|(_, connection)| connection.to)
            .max_by(|a, b| {
                let up =
                    |entity: &Entity| canvas.world.get::<&Position>(*entity).unwrap().position.x;
                up(a).total_cmp(&up(b))
            })
            .unwrap()
    };
    let (tip, red) = (
        ending(&canvas, NeuronType::Excitatory),
        ending(&canvas, NeuronType::Inhibitory),
    );
    let parent = |canvas: &Canvas, child: Entity| {
        canvas
            .world
            .query::<&Connection>()
            .iter()
            .find(|(_, connection)| !connection.directional && connection.to == child)
            .map(|(_, connection)| connection.from)
            .unwrap()
    };
    let mut path = vec![tip];
    while *path.last().unwrap() != receiver {
        path.push(parent(&canvas, *path.last().unwrap()));
    }
    path.pop();
    let red_at = path
        .iter()
        .position(|&compartment| compartment == red)
        .unwrap();
    assert!(
        red_at >= 3,
        "the red ending is only {red_at} compartments down"
    );

    canvas.wait(3.0);
    canvas.press(canvas.hands()[0]);
    let mut peaks = vec![Millivolts::NEG_INFINITY; path.len()];
    for _ in 0..(3.0 * STEPS_PER_SECOND) as usize {
        canvas.wait(1.0 / STEPS_PER_SECOND);
        for (peak, &compartment) in peaks.iter_mut().zip(&path) {
            let voltage = canvas.world.get::<&NodeState>(compartment).unwrap().voltage;
            *peak = peak.max(voltage);
        }
    }
    assert!(
        peaks[..red_at]
            .iter()
            .all(|&peak| peak > TERMINAL_THRESHOLD),
        "{peaks:?}"
    );
    assert!(
        peaks[red_at + 1..]
            .iter()
            .all(|&peak| peak < TERMINAL_THRESHOLD),
        "{peaks:?}"
    );
}

macro_rules! with_either_channels {
    ($($check:ident),* $(,)?) => {
        mod squid {
            $(#[test]
            fn $check() {
                super::$check(neuronify_core::Preparation::Squid)
            })*
        }

        mod mammal {
            $(#[test]
            fn $check() {
                super::$check(neuronify_core::Preparation::Mammal)
            })*
        }
    };
}

with_either_channels!(
    the_spike_reaches_the_near_neuron_before_the_far_one,
    the_spike_on_the_myelinated_axon_arrives_first,
    an_axon_grown_onto_a_branch_fires_the_neuron,
    weak_endings_fire_the_neuron_only_together,
    the_red_axon_blocks_the_top_input_and_lets_the_bottom_one_through,
    the_top_spike_runs_down_to_the_red_ending_and_stops_there,
);

#[test]
fn the_bundled_tutorial_files_match_their_generator() {
    for (file, world) in write_tutorial::canvases() {
        let path = format!("{}/examples/tutorial/{file}", env!("CARGO_MANIFEST_DIR"));
        let bundled = std::fs::read(path).unwrap();
        let written = neuronify_core::serialization::save(
            &world,
            Milliseconds::ZERO,
            neuronify_core::DEFAULT_SPEED,
        )
        .unwrap();
        assert_eq!(bundled, written, "{file} is out of date");
    }
}
