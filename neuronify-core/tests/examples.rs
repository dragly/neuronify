#[allow(dead_code)]
#[path = "../examples/write_examples.rs"]
mod write_examples;

use std::f64::consts::FRAC_PI_2;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::{DEFAULT_SPEED, LIF_DT, SIMULATION_STEPS_PER_SECOND};
use neuronify_core::measurement::visual_input::VisualInput;
use neuronify_core::rendering::canvas_bounds;
use neuronify_core::simulation::{fire_nearby, step_simulation};
use neuronify_core::{
    CurrentClamp, Label, LeakyDynamics, NoteSize, Position, TouchSensor, NODE_RADIUS,
};

const PHONE_WIDTH: f32 = 27.0;

fn on_screen(seconds: f64) -> f64 {
    seconds * DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND * LIF_DT
}

fn spikes(world: &mut World, label: &str, from: f64, to: f64) -> usize {
    let neuron = world
        .query::<&Label>()
        .iter()
        .find(|(_, neuron)| neuron.text == label)
        .unwrap()
        .0;
    let mut time = 0.0;
    let mut spikes = 0;
    while time < to {
        step_simulation(world, &mut time, 1, None);
        let fired = world.get::<&LeakyDynamics>(neuron).unwrap().time_since_fire == 0.0;
        spikes += usize::from(fired && time >= from);
    }
    spikes
}

#[test]
fn the_listened_neuron_clicks_within_seconds_for_stripes_at_its_angle_and_not_across_them() {
    let clicks = |pattern_angle: f64, seconds: f64| {
        (0..5)
            .map(|_| {
                let mut world = write_examples::listening_to_a_neuron();
                for (_, input) in world.query_mut::<&mut VisualInput>() {
                    input.pattern_angle = pattern_angle;
                }
                spikes(&mut world, "Cortex", 0.0, on_screen(seconds))
            })
            .sum::<usize>()
    };
    let aligned = clicks(0.0, 24.0);
    let across = clicks(FRAC_PI_2, 24.0);
    assert!(clicks(0.0, 2.0) >= 5);
    assert!(aligned >= 40, "{aligned}");
    assert!(
        aligned > 10 * across.max(1),
        "{aligned} aligned and {across} across"
    );
}

#[test]
fn the_output_fires_more_within_seconds_as_the_inhibitory_neuron_adapts() {
    let early = spikes(
        &mut write_examples::adaptive_inhibition(),
        "Output",
        0.0,
        on_screen(8.0),
    );
    let late = spikes(
        &mut write_examples::adaptive_inhibition(),
        "Output",
        on_screen(16.0),
        on_screen(24.0),
    );
    assert!(late > 3 * early.max(1), "{early} early and {late} late");
}

struct Experiment {
    world: World,
    time: f64,
}

impl Experiment {
    fn new(world: World) -> Experiment {
        Experiment { world, time: 0.0 }
    }

    fn neuron(&self, label: &str) -> Entity {
        self.world
            .query::<&Label>()
            .iter()
            .find(|(_, neuron)| neuron.text == label)
            .unwrap_or_else(|| panic!("no neuron labelled {label}"))
            .0
    }

    fn sensors_from_left(&self) -> Vec<Vec3> {
        let mut sensors: Vec<Vec3> = self
            .world
            .query::<&Position>()
            .with::<&TouchSensor>()
            .iter()
            .map(|(_, position)| position.position)
            .collect();
        sensors.sort_by(|a, b| a.z.total_cmp(&b.z));
        sensors
    }

    fn neurons_from_top(&self) -> Vec<Entity> {
        let mut neurons: Vec<(Entity, Vec3)> = self
            .world
            .query::<&Position>()
            .with::<&LeakyDynamics>()
            .iter()
            .map(|(entity, position)| (entity, position.position))
            .collect();
        neurons.sort_by(|a, b| b.1.x.total_cmp(&a.1.x).then(a.1.z.total_cmp(&b.1.z)));
        neurons.into_iter().map(|(entity, _)| entity).collect()
    }

    fn spike_counts(&mut self, seconds: f64, neurons: &[Entity]) -> Vec<usize> {
        let end = self.time + seconds;
        let mut counts = vec![0; neurons.len()];
        while self.time < end {
            step_simulation(&mut self.world, &mut self.time, 1, None);
            for (count, &neuron) in counts.iter_mut().zip(neurons) {
                let dynamics = self.world.get::<&LeakyDynamics>(neuron).unwrap();
                *count += usize::from(dynamics.time_since_fire == 0.0);
            }
        }
        counts
    }

    fn press(&mut self, sensor: Vec3) {
        fire_nearby(&mut self.world, sensor);
    }

    fn spikes_during(&mut self, seconds: f64, label: &str) -> usize {
        let neuron = self.neuron(label);
        let end = self.time + seconds;
        let mut spikes = 0;
        while self.time < end {
            step_simulation(&mut self.world, &mut self.time, 1, None);
            let fired = self
                .world
                .get::<&LeakyDynamics>(neuron)
                .unwrap()
                .time_since_fire
                == 0.0;
            spikes += usize::from(fired);
        }
        spikes
    }

    fn presses(&mut self, sensor: Vec3, count: usize, interval: f64, label: &str) -> usize {
        (0..count)
            .map(|_| {
                self.press(sensor);
                self.spikes_during(interval, label)
            })
            .sum()
    }
}

#[test]
fn the_adaptive_neuron_starts_to_skip_presses() {
    let mut experiment = Experiment::new(write_examples::adaptation());
    let sensor = experiment.sensors_from_left()[0];
    assert_eq!(experiment.presses(sensor, 3, 0.02, "Adaptive"), 3);
    assert!(experiment.presses(sensor, 9, 0.02, "Adaptive") < 6);
}

#[test]
fn the_adaptive_neuron_recovers_after_a_rest() {
    let mut experiment = Experiment::new(write_examples::adaptation());
    let sensor = experiment.sensors_from_left()[0];
    experiment.presses(sensor, 12, 0.02, "Adaptive");
    experiment.spikes_during(0.5, "Adaptive");
    assert_eq!(experiment.presses(sensor, 3, 0.02, "Adaptive"), 3);
}

#[test]
fn one_press_sets_off_a_burst_that_ends_on_its_own() {
    let mut experiment = Experiment::new(write_examples::bursting());
    let sensor = experiment.sensors_from_left()[0];
    experiment.press(sensor);
    let burst = experiment.spikes_during(0.25, "Bursting");
    assert!((10..40).contains(&burst), "{burst} spikes");
    assert_eq!(experiment.spikes_during(0.3, "Bursting"), 0);
}

#[test]
fn only_currents_above_the_threshold_fire_and_stronger_ones_fire_faster() {
    let mut experiment = Experiment::new(write_examples::firing_threshold());
    let neurons = experiment.neurons_from_top();
    let [weak, medium, strong] = experiment.spike_counts(1.0, &neurons)[..] else {
        panic!("expected three neurons");
    };
    assert_eq!(weak, 0);
    assert!((10..40).contains(&medium), "{medium} spikes");
    assert!(strong > 2 * medium, "{strong} and {medium} spikes");
}

#[test]
fn a_long_refractory_period_caps_the_firing_rate() {
    for (nanoamperes, short_at_least, long_below) in [(1.0, 100, 70), (5.0, 250, 100)] {
        let mut experiment = Experiment::new(write_examples::refractory_period());
        for (_, source) in experiment.world.query_mut::<&mut CurrentClamp>() {
            source.current_output = nanoamperes * 1e-9;
        }
        let neurons = [experiment.neuron("2 ms"), experiment.neuron("10 ms")];
        let [short, long] = experiment.spike_counts(1.0, &neurons)[..] else {
            unreachable!()
        };
        assert!(short >= short_at_least, "{nanoamperes} nA: {short} spikes");
        assert!(long < long_below, "{nanoamperes} nA: {long} spikes");
    }
}

fn output_spikes_after_presses(presses: &[(usize, f64)]) -> usize {
    let mut experiment = Experiment::new(write_examples::summation());
    let mut sensors = experiment.sensors_from_left();
    sensors.sort_by(|a, b| b.x.total_cmp(&a.x));
    let mut spikes = 0;
    for &(sensor, wait) in presses {
        experiment.press(sensors[sensor]);
        spikes += experiment.spikes_during(wait, "Output");
    }
    spikes + experiment.spikes_during(0.1, "Output")
}

#[test]
fn one_input_alone_does_not_fire_the_output() {
    for sensor in 0..3 {
        assert_eq!(output_spikes_after_presses(&[(sensor, 0.0)]), 0);
    }
}

#[test]
fn two_inputs_close_together_fire_the_output() {
    assert_eq!(output_spikes_after_presses(&[(0, 0.005), (2, 0.0)]), 1);
    assert_eq!(output_spikes_after_presses(&[(1, 0.005), (1, 0.0)]), 1);
}

#[test]
fn two_inputs_far_apart_leak_away() {
    assert_eq!(output_spikes_after_presses(&[(0, 0.05), (2, 0.0)]), 0);
    assert_eq!(output_spikes_after_presses(&[(1, 0.05), (1, 0.0)]), 0);
}

#[test]
fn one_press_keeps_the_loop_going_until_the_adaptive_neuron_tires() {
    let mut experiment = Experiment::new(write_examples::reverberation());
    let sensor = experiment.sensors_from_left()[0];
    experiment.press(sensor);
    let early = experiment.spikes_during(0.05, "Adaptive");
    let later = experiment.spikes_during(0.05, "Adaptive");
    experiment.spikes_during(0.3, "Adaptive");
    let at_rest = experiment.spikes_during(0.2, "Adaptive");
    assert!(early >= 2 && later >= 1, "{early} and {later} spikes");
    assert_eq!(at_rest, 0);
}

#[test]
fn the_two_neurons_take_turns() {
    let mut experiment = Experiment::new(write_examples::half_center_oscillator());
    let neurons = experiment.neurons_from_top();
    experiment.spike_counts(0.2, &neurons);
    let mut last = None;
    let mut turns = 0;
    let mut spikes = [0, 0];
    for _ in 0..10_000 {
        let fired = experiment.spike_counts(0.0001, &neurons);
        for (side, &count) in fired.iter().enumerate() {
            spikes[side] += count;
            if count > 0 && last != Some(side) {
                turns += 1;
                last = Some(side);
            }
        }
    }
    assert!(turns >= 10, "{turns} turns");
    assert!(
        spikes.iter().all(|&count| count >= 2 * turns / 2),
        "{spikes:?}"
    );
}

#[test]
fn excitatory_feedback_speeds_the_output_up_and_inhibitory_feedback_slows_it_down() {
    let mut experiment = Experiment::new(write_examples::feedback_loops());
    let neurons = [
        experiment.neurons_from_top()[0],
        experiment.neuron("Faster"),
        experiment.neuron("Slower"),
    ];
    let [input, faster, slower] = experiment.spike_counts(1.0, &neurons)[..] else {
        unreachable!()
    };
    assert!(faster > 2 * input, "{faster} against {input}");
    assert!(
        slower > 0 && 5 * slower < 4 * input,
        "{slower} against {input}"
    );
}

#[test]
fn no_note_covers_the_circuit() {
    for (file, world) in write_examples::sheets() {
        let notes: Vec<(Vec3, NoteSize)> = world
            .query::<(&Position, &NoteSize)>()
            .iter()
            .map(|(_, (position, size))| (position.position, size.clone()))
            .collect();
        for (entity, position) in world.query::<&Position>().without::<&NoteSize>().iter() {
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
fn each_example_fits_an_upright_phone() {
    for (file, world) in write_examples::sheets() {
        let bounds = canvas_bounds(&world).unwrap();
        let width = bounds.max.z - bounds.min.z;
        assert!(width <= PHONE_WIDTH, "{file} is {width} wide");
    }
}

#[test]
fn the_bundled_example_files_match_their_generators() {
    for (file, world) in write_examples::examples() {
        let path = format!("{}/examples/{file}", env!("CARGO_MANIFEST_DIR"));
        let bundled = std::fs::read(path).unwrap();
        let written = neuronify_core::serialization::save(
            &world,
            0.0,
            neuronify_core::constants::DEFAULT_SPEED,
        )
        .unwrap();
        assert_eq!(bundled, written, "{file} is out of date");
    }
}
