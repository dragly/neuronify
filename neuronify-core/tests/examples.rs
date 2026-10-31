#[allow(dead_code)]
#[path = "../examples/write_examples.rs"]
mod write_examples;

use std::f64::consts::FRAC_PI_2;

use hecs::World;
use neuronify_core::measurement::visual_input::VisualInput;
use neuronify_core::simulation::step_simulation;
use neuronify_core::{Label, LeakyDynamics};

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
fn the_listened_neuron_clicks_for_a_bar_at_its_angle_and_not_across_it() {
    let mut aligned = write_examples::listening_to_a_neuron();
    let mut across = write_examples::listening_to_a_neuron();
    for (_, input) in across.query_mut::<&mut VisualInput>() {
        input.pattern_angle = FRAC_PI_2;
    }
    assert!(spikes(&mut aligned, "Cortex", 0.0, 4.0) > 20);
    assert_eq!(spikes(&mut across, "Cortex", 0.0, 4.0), 0);
}

#[test]
fn the_output_fires_more_as_the_inhibitory_neuron_tires() {
    let early = spikes(&mut write_examples::tiring_inhibition(), "Output", 0.0, 2.0);
    let late = spikes(&mut write_examples::tiring_inhibition(), "Output", 2.0, 4.0);
    assert!(late > 2 * early, "{early} early and {late} late");
}

#[test]
fn the_bundled_example_files_match_their_generators() {
    for (file, world) in [
        (
            "listening_to_a_neuron.neuronify",
            write_examples::listening_to_a_neuron(),
        ),
        (
            "tiring_inhibition.neuronify",
            write_examples::tiring_inhibition(),
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
