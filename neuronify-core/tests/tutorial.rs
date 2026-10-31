use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::elements::spawn_synapse;
use neuronify_core::examples::{find, EXAMPLE_CATEGORIES};
use neuronify_core::rendering::canvas_bounds;
use neuronify_core::simulation::{fire_nearby, step_simulation};
use neuronify_core::{
    Annotation, LeakyDynamics, LeakyNeuron, Link, NeuronDisplay, NeuronType, Position, Setting,
    TouchSensor, DEFAULT_SPEED, LIF_DT, SIMULATION_STEPS_PER_SECOND,
};

const STEPS_PER_SECOND: f64 = DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND;
const SIMULATED_PER_REAL_SECOND: f64 = STEPS_PER_SECOND * LIF_DT;
const PHONE_WIDTH: f32 = 27.0;

struct Canvas {
    world: World,
    time: f64,
    spikes: Vec<(Entity, f64)>,
}

impl Canvas {
    fn open(file: &str) -> Canvas {
        Canvas {
            world: find(file).unwrap().spawn().unwrap(),
            time: 0.0,
            spikes: Vec::new(),
        }
    }

    fn wait(&mut self, seconds: f64) {
        for _ in 0..(seconds * STEPS_PER_SECOND).round() as usize {
            step_simulation(&mut self.world, &mut self.time, 1, None);
            let fired = self
                .world
                .query::<&LeakyDynamics>()
                .iter()
                .filter(|(_, dynamics)| dynamics.time_since_fire == 0.0)
                .map(|(entity, _)| entity)
                .collect::<Vec<_>>();
            self.spikes
                .extend(fired.into_iter().map(|entity| (entity, self.time)));
        }
    }

    fn now(&self) -> f64 {
        self.time
    }

    fn wait_for_spike(&mut self, neuron: Entity) -> f64 {
        let since = self.now();
        while self.spikes_since(neuron, since).is_empty() {
            self.wait(1.0 / STEPS_PER_SECOND);
        }
        self.spikes_since(neuron, since)[0]
    }

    fn press(&mut self, hand: Vec3) {
        fire_nearby(&mut self.world, hand);
    }

    fn spikes_since(&self, neuron: Entity, since: f64) -> Vec<f64> {
        self.spikes
            .iter()
            .filter(|(entity, time)| *entity == neuron && *time >= since)
            .map(|(_, time)| *time)
            .collect()
    }

    fn voltage(&self, neuron: Entity) -> f64 {
        self.world.get::<&LeakyDynamics>(neuron).unwrap().voltage
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
    assert_eq!(files.len(), 6);
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
fn each_canvas_fits_an_upright_phone() {
    for file in tutorial() {
        let bounds = canvas_bounds(&Canvas::open(file).world).unwrap();
        let width = bounds.max.z - bounds.min.z;
        assert!(width <= PHONE_WIDTH, "{file} is {width} wide");
    }
}

#[test]
fn notes_name_tools_that_exist() {
    let tools: Vec<&str> = [neuronify_core::Tool::Erase]
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
        canvas.spikes_since(neuron, 0.0).len()
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
    let alone = canvas.spikes_since(middle, 0.0);
    let period = alone[alone.len() - 1] - alone[alone.len() - 2];
    let real_period = period / SIMULATED_PER_REAL_SECOND;
    assert!(
        (1.0..2.5).contains(&real_period),
        "fires every {real_period} s"
    );

    let last = canvas.wait_for_spike(middle);
    canvas.press(push_hand);
    canvas.wait(2.0);
    let next = canvas.spikes_since(middle, last + 1e-9)[0];
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
    let next = canvas.spikes_since(middle, last + 1e-9)[0];
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
