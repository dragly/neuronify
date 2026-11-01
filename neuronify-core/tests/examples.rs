#[allow(dead_code)]
#[path = "../examples/write_examples.rs"]
mod write_examples;

use std::f64::consts::FRAC_PI_2;

use hecs::World;
use neuronify_core::constants::{DEFAULT_SPEED, LIF_DT, SIMULATION_STEPS_PER_SECOND};
use neuronify_core::measurement::visual_input::VisualInput;
use neuronify_core::simulation::step_simulation;
use neuronify_core::{Label, LeakyDynamics};

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

#[test]
fn the_bundled_example_files_match_their_generators() {
    for (file, world) in [
        (
            "listening_to_a_neuron.neuronify",
            write_examples::listening_to_a_neuron(),
        ),
        (
            "adaptive_inhibition.neuronify",
            write_examples::adaptive_inhibition(),
        ),
    ] {
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
