#[allow(dead_code)]
#[path = "../generators/write_examples.rs"]
mod write_examples;

use std::f64::consts::FRAC_PI_2;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::{DEFAULT_SPEED, DT, SIMULATION_STEPS_PER_SECOND};
use neuronify_core::measurement::visual_input::VisualInput;
use neuronify_core::measurement::voltmeter::VoltageSeries;
use neuronify_core::rendering::canvas_bounds;
use neuronify_core::simulation::model::prepare;
use neuronify_core::simulation::{fire_nearby, step_simulation, Engine};
use neuronify_core::{
    Celsius, Connection, CurrentClamp, Label, LeakyNeuron, Milliseconds, Millivolts, Nanoamperes,
    NoteSize, Position, Preparation, Seconds, Setting, Temperature, TouchSensor, NODE_RADIUS,
};

const PHONE_WIDTH: f32 = 27.0;

fn on_screen(seconds: f64) -> Milliseconds {
    let steps = Seconds(seconds) * DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND;
    steps * DT
}

const STEPS_PER_CALL: u32 = 40;

fn fired(engine: &Engine, neuron: Entity) -> usize {
    engine
        .spikes()
        .iter()
        .filter(|spike| spike.entity == neuron)
        .count()
}

fn spikes(world: &mut World, label: &str, from: Milliseconds, to: Milliseconds) -> usize {
    let neuron = world
        .query::<&Label>()
        .iter()
        .find(|(_, neuron)| neuron.text == label)
        .unwrap()
        .0;
    let (mut engine, mut time) = (Engine::default(), Milliseconds::ZERO);
    let mut spikes = 0;
    while time < to {
        step_simulation(world, &mut engine, &mut time, STEPS_PER_CALL, None);
        if time >= from {
            spikes += fired(&engine, neuron);
        }
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
                spikes(&mut world, "Cortex", Milliseconds::ZERO, on_screen(seconds))
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
        Milliseconds::ZERO,
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
    engine: Engine,
    time: Milliseconds,
}

impl Experiment {
    fn new(world: World) -> Experiment {
        Experiment {
            world,
            engine: Engine::default(),
            time: Milliseconds::ZERO,
        }
    }

    fn step(&mut self) {
        step_simulation(
            &mut self.world,
            &mut self.engine,
            &mut self.time,
            STEPS_PER_CALL,
            None,
        );
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

    fn neurons_from_left(&self) -> Vec<Entity> {
        let mut neurons: Vec<(Entity, Vec3)> = self
            .world
            .query::<&Position>()
            .with::<&LeakyNeuron>()
            .iter()
            .map(|(entity, position)| (entity, position.position))
            .collect();
        neurons.sort_by(|a, b| a.1.z.total_cmp(&b.1.z));
        neurons.into_iter().map(|(entity, _)| entity).collect()
    }

    fn meter_peaks_from_left(&self) -> Vec<Millivolts> {
        let mut wires: Vec<(f32, Millivolts)> = self
            .world
            .query::<(&Connection, &VoltageSeries)>()
            .iter()
            .map(|(_, (wire, series))| {
                let meter = self.world.get::<&Position>(wire.to).unwrap().position.z;
                let peak = series
                    .measurements
                    .iter()
                    .map(|measurement| measurement.voltage)
                    .fold(Millivolts::NEG_INFINITY, Millivolts::max);
                (meter, peak)
            })
            .collect();
        wires.sort_by(|a, b| a.0.total_cmp(&b.0));
        wires.into_iter().map(|(_, peak)| peak).collect()
    }

    fn neurons_from_top(&self) -> Vec<Entity> {
        let mut neurons: Vec<(Entity, Vec3)> = self
            .world
            .query::<&Position>()
            .with::<&LeakyNeuron>()
            .iter()
            .map(|(entity, position)| (entity, position.position))
            .collect();
        neurons.sort_by(|a, b| b.1.x.total_cmp(&a.1.x).then(a.1.z.total_cmp(&b.1.z)));
        neurons.into_iter().map(|(entity, _)| entity).collect()
    }

    fn spike_counts(&mut self, duration: Milliseconds, neurons: &[Entity]) -> Vec<usize> {
        let end = self.time + duration;
        let mut counts = vec![0; neurons.len()];
        while self.time < end {
            self.step();
            for (count, &neuron) in counts.iter_mut().zip(neurons) {
                *count += fired(&self.engine, neuron);
            }
        }
        counts
    }

    fn press(&mut self, sensor: Vec3) {
        fire_nearby(&mut self.world, &mut self.engine, sensor);
    }

    fn spikes_during(&mut self, duration: Milliseconds, label: &str) -> usize {
        let neuron = self.neuron(label);
        let end = self.time + duration;
        let mut spikes = 0;
        while self.time < end {
            self.step();
            spikes += fired(&self.engine, neuron);
        }
        spikes
    }

    fn presses(
        &mut self,
        sensor: Vec3,
        count: usize,
        interval: Milliseconds,
        label: &str,
    ) -> usize {
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
    assert_eq!(
        experiment.presses(sensor, 3, Milliseconds(20.0), "Adaptive"),
        3
    );
    assert!(experiment.presses(sensor, 9, Milliseconds(20.0), "Adaptive") < 6);
}

#[test]
fn the_adaptive_neuron_recovers_after_a_rest() {
    let mut experiment = Experiment::new(write_examples::adaptation());
    let sensor = experiment.sensors_from_left()[0];
    experiment.presses(sensor, 12, Milliseconds(20.0), "Adaptive");
    experiment.spikes_during(Milliseconds(500.0), "Adaptive");
    assert_eq!(
        experiment.presses(sensor, 3, Milliseconds(20.0), "Adaptive"),
        3
    );
}

#[test]
fn one_press_sets_off_a_burst_that_ends_on_its_own() {
    let mut experiment = Experiment::new(write_examples::bursting());
    let sensor = experiment.sensors_from_left()[0];
    experiment.press(sensor);
    let burst = experiment.spikes_during(Milliseconds(250.0), "Bursting");
    assert!((10..40).contains(&burst), "{burst} spikes");
    assert_eq!(experiment.spikes_during(Milliseconds(300.0), "Bursting"), 0);
}

#[test]
fn only_currents_above_the_threshold_fire_and_stronger_ones_fire_faster() {
    let mut experiment = Experiment::new(write_examples::firing_threshold());
    let neurons = experiment.neurons_from_top();
    let [weak, medium, strong] = experiment.spike_counts(Milliseconds(1000.0), &neurons)[..] else {
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
            source.current_output = Nanoamperes(nanoamperes);
        }
        let neurons = [experiment.neuron("2 ms"), experiment.neuron("10 ms")];
        let [short, long] = experiment.spike_counts(Milliseconds(1000.0), &neurons)[..] else {
            unreachable!()
        };
        assert!(short >= short_at_least, "{nanoamperes} nA: {short} spikes");
        assert!(long < long_below, "{nanoamperes} nA: {long} spikes");
    }
}

struct Press {
    axon: Millivolts,
    dendrite: Millivolts,
    fired: bool,
}

fn press_when_warmed(preparation: Preparation, celsius: f64) -> Press {
    let mut world = write_examples::body_temperature();
    prepare(&mut world, preparation);
    Temperature {
        celsius: Celsius(celsius),
    }
    .set_in(&mut world);
    let mut experiment = Experiment::new(world);
    let far = *experiment.neurons_from_left().last().unwrap();
    experiment.spike_counts(on_screen(1.0), &[]);
    experiment.press(experiment.sensors_from_left()[0]);
    let fired = experiment.spike_counts(on_screen(2.0), &[far])[0] > 0;
    let [axon, dendrite] = experiment.meter_peaks_from_left()[..] else {
        panic!("expected two voltmeters");
    };
    Press {
        axon,
        dendrite,
        fired,
    }
}

#[test]
fn both_kinds_carry_a_spike_to_the_far_neuron_at_their_own_temperature() {
    for (preparation, celsius) in [(Preparation::Squid, 16.3), (Preparation::Mammal, 37.0)] {
        let press = press_when_warmed(preparation, celsius);
        assert!(
            press.axon > Millivolts::ZERO && press.dendrite > Millivolts::ZERO && press.fired,
            "{preparation:?}"
        );
    }
}

#[test]
fn squid_axons_fail_when_warm_and_mammalian_ones_keep_going() {
    assert!(press_when_warmed(Preparation::Squid, 35.0).axon < Millivolts::ZERO);
    for celsius in [20.0, 42.0] {
        assert!(
            press_when_warmed(Preparation::Mammal, celsius).axon > Millivolts::ZERO,
            "{celsius} °C"
        );
    }
}

#[test]
fn the_mammalian_dendritic_spike_shrinks_as_it_warms_until_the_neuron_stays_quiet() {
    let peaks = [20.0, 30.0, 40.0].map(|celsius| press_when_warmed(Preparation::Mammal, celsius));
    assert!(
        peaks
            .windows(2)
            .all(|pair| pair[0].dendrite > pair[1].dendrite + Millivolts(5.0)),
        "{:?}",
        peaks.map(|press| press.dendrite)
    );
    assert!(!press_when_warmed(Preparation::Mammal, 42.0).fired);
}

fn output_spikes_after_presses(presses: &[(usize, f64)]) -> usize {
    let mut experiment = Experiment::new(write_examples::summation());
    let mut sensors = experiment.sensors_from_left();
    sensors.sort_by(|a, b| b.x.total_cmp(&a.x));
    let mut spikes = 0;
    for &(sensor, wait) in presses {
        experiment.press(sensors[sensor]);
        spikes += experiment.spikes_during(Milliseconds(wait), "Output");
    }
    spikes + experiment.spikes_during(Milliseconds(100.0), "Output")
}

#[test]
fn one_input_alone_does_not_fire_the_output() {
    for sensor in 0..3 {
        assert_eq!(output_spikes_after_presses(&[(sensor, 0.0)]), 0);
    }
}

#[test]
fn two_inputs_close_together_fire_the_output() {
    assert_eq!(output_spikes_after_presses(&[(0, 5.0), (2, 0.0)]), 1);
    assert_eq!(output_spikes_after_presses(&[(1, 5.0), (1, 0.0)]), 1);
}

#[test]
fn two_inputs_far_apart_leak_away() {
    assert_eq!(output_spikes_after_presses(&[(0, 50.0), (2, 0.0)]), 0);
    assert_eq!(output_spikes_after_presses(&[(1, 50.0), (1, 0.0)]), 0);
}

#[test]
fn one_press_keeps_the_loop_going_until_the_adaptive_neuron_tires() {
    let mut experiment = Experiment::new(write_examples::reverberation());
    let sensor = experiment.sensors_from_left()[0];
    experiment.press(sensor);
    let early = experiment.spikes_during(Milliseconds(50.0), "Adaptive");
    let later = experiment.spikes_during(Milliseconds(50.0), "Adaptive");
    experiment.spikes_during(Milliseconds(300.0), "Adaptive");
    let at_rest = experiment.spikes_during(Milliseconds(200.0), "Adaptive");
    assert!(early >= 2 && later >= 1, "{early} and {later} spikes");
    assert_eq!(at_rest, 0);
}

#[test]
fn the_two_neurons_take_turns() {
    let mut experiment = Experiment::new(write_examples::half_center_oscillator());
    let neurons = experiment.neurons_from_top();
    experiment.spike_counts(Milliseconds(200.0), &neurons);
    let mut last = None;
    let mut turns = 0;
    let mut spikes = [0, 0];
    for _ in 0..10_000 {
        let fired = experiment.spike_counts(Milliseconds(0.1), &neurons);
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
    let [input, faster, slower] = experiment.spike_counts(Milliseconds(1000.0), &neurons)[..]
    else {
        unreachable!()
    };
    assert!(faster > 2 * input, "{faster} against {input}");
    assert!(
        slower > 0 && 5 * slower < 4 * input,
        "{slower} against {input}"
    );
}

#[test]
fn the_brake_holds_the_output_until_a_press_releases_it() {
    let mut experiment = Experiment::new(write_examples::disinhibition());
    let sensor = experiment.sensors_from_left()[0];
    experiment.spikes_during(Milliseconds(50.0), "Output");
    assert_eq!(experiment.spikes_during(Milliseconds(300.0), "Output"), 0);
    experiment.press(sensor);
    assert!(experiment.spikes_during(Milliseconds(200.0), "Output") >= 4);
    experiment.spikes_during(Milliseconds(300.0), "Output");
    assert_eq!(experiment.spikes_during(Milliseconds(300.0), "Output"), 0);
}

#[test]
fn feedback_inhibition_halves_the_output_rate() {
    let mut experiment = Experiment::new(write_examples::feedback_inhibition());
    let from_top = experiment.neurons_from_top();
    let neurons = [from_top[0], from_top[3]];
    let [input, output] = experiment.spike_counts(Milliseconds(1000.0), &neurons)[..] else {
        unreachable!()
    };
    assert!(
        (input * 2 / 5..input * 3 / 5).contains(&output),
        "{output} against {input}"
    );
}

#[test]
fn lateral_inhibition_sharpens_the_middle_of_the_touch() {
    let mut experiment = Experiment::new(write_examples::lateral_inhibition());
    let neurons = experiment.neurons_from_top();
    let counts = experiment.spike_counts(Milliseconds(1000.0), &neurons);
    let (skin, outputs) = (&counts[..5], &counts[10..]);
    assert!(skin[2] < 2 * skin[1], "skin {skin:?}");
    assert!(outputs[2] > 4 * outputs[1], "outputs {outputs:?}");
    assert_eq!(outputs[1], outputs[3]);
}

fn output_spikes_after_a_sweep(right_to_left: bool, interval: Milliseconds) -> usize {
    let mut experiment = Experiment::new(write_examples::direction_selectivity());
    let mut sensors = experiment.sensors_from_left();
    if right_to_left {
        sensors.reverse();
    }
    let presses: usize = sensors
        .into_iter()
        .map(|sensor| experiment.presses(sensor, 1, interval, "Output"))
        .sum();
    presses + experiment.spikes_during(Milliseconds(100.0), "Output")
}

#[test]
fn the_output_fires_for_a_sweep_from_right_to_left_but_not_from_left_to_right() {
    for interval in [1.0, 3.0, 8.0, 15.0].map(Milliseconds) {
        assert!(
            output_spikes_after_a_sweep(true, interval) >= 1,
            "{interval}"
        );
        assert_eq!(
            output_spikes_after_a_sweep(false, interval),
            0,
            "{interval}"
        );
    }
}

#[test]
fn a_single_press_does_not_fire_the_direction_detector() {
    let mut experiment = Experiment::new(write_examples::direction_selectivity());
    for sensor in experiment.sensors_from_left() {
        experiment.press(sensor);
        assert_eq!(experiment.spikes_during(Milliseconds(100.0), "Output"), 0);
    }
}

#[test]
fn each_neuron_fires_when_the_dot_crosses_its_own_field_and_never_on_the_far_side() {
    let mut experiment = Experiment::new(write_examples::receptive_fields());
    let neurons = experiment.neurons_from_top();
    let mut near = [0, 0];
    let mut far = [0, 0];
    for _ in 0..20_000 {
        let fired = experiment.spike_counts(Milliseconds(0.1), &neurons);
        let phase = experiment
            .world
            .query_mut::<&VisualInput>()
            .into_iter()
            .next()
            .unwrap()
            .1
            .phase;
        for (index, &count) in fired.iter().enumerate() {
            let crossing = index as f64 * 0.5;
            let distance = (phase - crossing).rem_euclid(1.0);
            if distance.min(1.0 - distance) < 0.25 {
                near[index] += count;
            } else {
                far[index] += count;
            }
        }
    }
    for index in 0..2 {
        assert!(near[index] >= 15, "{near:?} {far:?}");
        assert_eq!(far[index], 0, "{near:?} {far:?}");
    }
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
            Milliseconds::ZERO,
            neuronify_core::constants::DEFAULT_SPEED,
        )
        .unwrap();
        assert_eq!(bundled, written, "{file} is out of date");
    }
}
