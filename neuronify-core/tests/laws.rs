use glam::Vec3;
use hecs::World;
use neuronify_core::dendrites::spawn_dendrites;
use neuronify_core::elements::{spawn_connection, Element};
use neuronify_core::simulation::{step_simulation, Engine};
use neuronify_core::{
    CurrentClamp, Milliseconds, Nanoamperes, Seconds, DEFAULT_SPEED, SIMULATION_STEPS_PER_SECOND,
};

const STEPS_PER_CALL: u32 = 20;
const NANOAMPERES: [f64; 9] = [0.0, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 30.0];

const NEURONS: [(&str, Element, bool); 6] = [
    ("excitatory neuron", Element::ExcitatoryNeuron, false),
    ("inhibitory neuron", Element::InhibitoryNeuron, false),
    ("adaptive neuron", Element::AdaptiveNeuron, false),
    ("burst neuron", Element::BurstNeuron, false),
    (
        "compartmental excitatory neuron",
        Element::ExcitatoryNeuron,
        true,
    ),
    (
        "compartmental inhibitory neuron",
        Element::InhibitoryNeuron,
        true,
    ),
];

fn spikes_in_ten_seconds(neuron: Element, dendrites: bool, nanoamperes: f64) -> usize {
    let mut world = World::new();
    let neuron = neuron.spawn(&mut world, Vec3::ZERO);
    if dendrites {
        spawn_dendrites(&mut world, neuron);
    }
    let source = Element::CurrentSource.spawn(&mut world, Vec3::new(0.0, 0.0, -4.0));
    world
        .get::<&mut CurrentClamp>(source)
        .unwrap()
        .current_output = Nanoamperes(nanoamperes);
    spawn_connection(&mut world, source, neuron);
    let (mut engine, mut time) = (Engine::default(), Milliseconds::ZERO);
    let steps = (Seconds(10.0) * DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND).round() as u32;
    let mut spikes = 0;
    for _ in 0..steps / STEPS_PER_CALL {
        step_simulation(&mut world, &mut engine, &mut time, STEPS_PER_CALL, None);
        spikes += engine
            .spikes()
            .iter()
            .filter(|spike| spike.entity == neuron)
            .count();
    }
    spikes
}

#[test]
fn a_stronger_current_makes_every_kind_of_neuron_fire_harder() {
    for (name, neuron, dendrites) in NEURONS {
        let spikes: Vec<usize> = NANOAMPERES
            .iter()
            .map(|nanoamperes| spikes_in_ten_seconds(neuron, dendrites, *nanoamperes))
            .collect();
        let sweep = NANOAMPERES
            .iter()
            .zip(&spikes)
            .map(|(nanoamperes, spikes)| format!("{nanoamperes} nA: {spikes}"))
            .collect::<Vec<_>>()
            .join(", ");
        assert_eq!(spikes[0], 0, "a {name} fires without current ({sweep})");
        assert!(
            spikes.windows(2).all(|pair| pair[0] <= pair[1]),
            "a {name} fires less with more current ({sweep})"
        );
        assert!(
            spikes[NANOAMPERES.len() - 1] > spikes[3],
            "a {name} fires no harder at 30 nA than at 1 nA ({sweep})"
        );
    }
}
