use super::convert::spawn_legacy_simulation;
use super::*;
use crate::components::*;
use crate::simulation::{lif_step, run_headless, SpikeRecord};

const EMPTY_NFY: &str = r#"{"nodes": [], "edges": []}"#;

const TUTORIAL_1_INTRO_NFY: &str = include_str!("../../examples/tutorial_1_intro.nfy");

const TWO_NEURON_OSCILLATOR_NFY: &str = include_str!("../../examples/two_neuron_oscillator.nfy");

const LEAKY_NFY: &str = include_str!("../../examples/leaky.nfy");

const ADAPTATION_NFY: &str = include_str!("../../examples/adaptation.nfy");

const INHIBITORY_NFY: &str = include_str!("../../examples/inhibitory.nfy");

const TUTORIAL_2_CIRCUITS_NFY: &str = include_str!("../../examples/tutorial_2_circuits.nfy");

#[test]
fn test_parse_empty() {
    let sim = parse_legacy_nfy(EMPTY_NFY).unwrap();
    assert_eq!(sim.nodes.len(), 0);
    assert_eq!(sim.edges.len(), 0);
}

#[test]
fn test_parse_tutorial_1_intro() {
    let sim = parse_legacy_nfy(TUTORIAL_1_INTRO_NFY).unwrap();
    assert_eq!(sim.file_format_version, Some(4));
    assert_eq!(sim.nodes.len(), 8);
    assert_eq!(sim.edges.len(), 2);

    assert_eq!(sim.nodes[0].filename, "neurons/LeakyNeuron.qml");
    assert!((sim.nodes[0].engine.capacitance.unwrap() - 2e-10).abs() < 1e-20);

    assert_eq!(sim.nodes[1].filename, "generators/CurrentClamp.qml");
    assert!((sim.nodes[1].engine.current_output.unwrap() - 3e-10).abs() < 1e-20);

    assert_eq!(sim.nodes[2].filename, "meters/Voltmeter.qml");
}

#[test]
fn test_parse_v2_format() {
    let sim = parse_legacy_nfy(TWO_NEURON_OSCILLATOR_NFY).unwrap();
    assert_eq!(sim.file_format_version, Some(2));
    assert_eq!(sim.nodes.len(), 6);
    assert_eq!(sim.edges.len(), 6);

    assert_eq!(sim.nodes[0].filename, "neurons/LeakyNeuron.qml");
    assert_eq!(sim.edges[0].filename, "Edge.qml");
}

#[test]
fn test_tutorial_1_intro_simulation() {
    let sim = parse_legacy_nfy(TUTORIAL_1_INTRO_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);

    let dt = 0.0001;
    let steps = 10_000;
    let spikes = run_headless(&mut world, steps, dt);

    assert!(
        spikes.len() > 20,
        "Expected at least 20 spikes in 1 second, got {}",
        spikes.len()
    );
    assert!(
        spikes.len() < 100,
        "Expected fewer than 100 spikes in 1 second, got {}",
        spikes.len()
    );

    assert!(spikes.iter().all(|s| s.entity_index == 0));
}

#[test]
fn test_tutorial_2_circuits_simulation() {
    let sim = parse_legacy_nfy(TUTORIAL_2_CIRCUITS_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);

    let dt = 0.0001;
    let steps = 10_000;
    let spikes = run_headless(&mut world, steps, dt);

    let neuron_0_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 0).collect();
    let neuron_1_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 1).collect();

    assert!(
        neuron_0_spikes.len() > 5,
        "Neuron 0 should fire, got {} spikes",
        neuron_0_spikes.len()
    );
    assert!(
        neuron_1_spikes.len() > 2,
        "Neuron 1 should fire from synaptic input, got {} spikes",
        neuron_1_spikes.len()
    );
}

#[test]
fn test_adaptation_decreasing_rate() {
    let sim = parse_legacy_nfy(ADAPTATION_NFY).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);

    let adapt_neuron_idx = sim
        .nodes
        .iter()
        .position(|n| n.filename == "neurons/AdaptationNeuron.qml")
        .unwrap();

    let leaky_idx = sim
        .nodes
        .iter()
        .position(|n| n.filename == "neurons/LeakyNeuron.qml")
        .unwrap();

    let leaky_entity = entities[leaky_idx];
    world
        .insert_one(
            leaky_entity,
            CurrentClamp {
                current_output: 5e-9,
            },
        )
        .unwrap();

    let dt = 0.0001;
    let total_steps = 20_000;

    let mut all_spikes = Vec::new();
    let mut time = 0.0;
    let neuron_entities: Vec<hecs::Entity> = world
        .query::<&LeakyNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for _ in 0..total_steps {
        lif_step(&mut world, dt, time);
        for (idx, entity) in neuron_entities.iter().enumerate() {
            if let Ok(dynamics) = world.get::<&LeakyDynamics>(*entity) {
                if dynamics.time_since_fire == 0.0 {
                    all_spikes.push(SpikeRecord {
                        entity_index: idx,
                        time,
                    });
                }
            }
        }
        time += dt;
    }

    let adapt_entity = entities[adapt_neuron_idx];
    let adapt_neuron_query_idx = neuron_entities
        .iter()
        .position(|&e| e == adapt_entity)
        .unwrap();

    let adapt_spikes: Vec<_> = all_spikes
        .iter()
        .filter(|s| s.entity_index == adapt_neuron_query_idx)
        .collect();

    if adapt_spikes.len() >= 4 {
        let mid_time = time / 2.0;
        let first_half_count = adapt_spikes.iter().filter(|s| s.time < mid_time).count();
        let second_half_count = adapt_spikes.iter().filter(|s| s.time >= mid_time).count();

        assert!(
            first_half_count > second_half_count,
            "Adaptation neuron firing rate should decrease over time. \
             First half: {}, Second half: {}",
            first_half_count,
            second_half_count
        );
    }
}

#[test]
fn test_two_neuron_oscillator_v2() {
    let sim = parse_legacy_nfy(TWO_NEURON_OSCILLATOR_NFY).unwrap();
    assert_eq!(sim.file_format_version, Some(2));

    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);

    let dt = 0.0001;
    let steps = 10_000;
    let spikes = run_headless(&mut world, steps, dt);

    let n0_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 0).collect();
    let n1_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 1).collect();

    assert!(
        n0_spikes.len() > 3,
        "Neuron 0 should fire in oscillator, got {}",
        n0_spikes.len()
    );
    assert!(
        n1_spikes.len() > 3,
        "Neuron 1 should fire in oscillator, got {}",
        n1_spikes.len()
    );
}

#[test]
fn test_inhibitory_simulation() {
    let sim = parse_legacy_nfy(INHIBITORY_NFY).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);

    assert_eq!(sim.nodes.len(), 9);
    assert_eq!(sim.edges.len(), 5);

    let c_entity = entities[0];
    assert!(world.get::<&Inhibitory>(c_entity).is_err());

    let b_entity = entities[3];
    assert!(world.get::<&Inhibitory>(b_entity).is_ok());

    let dt = 0.0001;
    for i in 0..100 {
        lif_step(&mut world, dt, i as f64 * dt);
    }
}

#[test]
fn test_leaky_simulation() {
    let sim = parse_legacy_nfy(LEAKY_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);

    let dt = 0.0001;
    let steps = 10_000;
    let _spikes = run_headless(&mut world, steps, dt);

    assert!(!sim.nodes.is_empty());
}

#[test]
fn test_current_clamp_drives_neuron() {
    use crate::{Connection, NeuronType, Position};
    use glam::Vec3;

    let mut world = hecs::World::new();

    let neuron = world.spawn((
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));

    let clamp = world.spawn((
        CurrentClamp {
            current_output: 5e-9,
        },
        Position {
            position: Vec3::new(0.0, 0.0, -1.0),
        },
    ));

    world.spawn((
        Connection {
            from: clamp,
            to: neuron,
            strength: 1.0,
            directional: true,
        },
        ImmediateFireSynapse::default(),
    ));

    let dt = 0.0001;
    let spikes = run_headless(&mut world, 10_000, dt);

    let neuron_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 0).collect();
    assert!(
        neuron_spikes.len() > 10,
        "Current clamp should drive neuron to fire repeatedly, got {} spikes",
        neuron_spikes.len()
    );
}

#[test]
fn test_inhibitory_suppresses_firing() {
    use crate::{Connection, NeuronType, Position};
    use glam::Vec3;

    let mut world = hecs::World::new();

    let target = world.spawn((
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));

    let clamp = world.spawn((
        CurrentClamp {
            current_output: 5e-9,
        },
        Position {
            position: Vec3::new(0.0, 0.0, -2.0),
        },
    ));

    world.spawn((
        Connection {
            from: clamp,
            to: target,
            strength: 1.0,
            directional: true,
        },
        ImmediateFireSynapse::default(),
    ));

    let inhibitor = world.spawn((
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Inhibitory,
        Inhibitory,
        Position {
            position: Vec3::new(0.0, 0.0, 1.0),
        },
    ));

    let inhib_clamp = world.spawn((
        CurrentClamp {
            current_output: 10e-9,
        },
        Position {
            position: Vec3::new(0.0, 0.0, 2.0),
        },
    ));

    world.spawn((
        Connection {
            from: inhib_clamp,
            to: inhibitor,
            strength: 1.0,
            directional: true,
        },
        ImmediateFireSynapse::default(),
    ));

    world.spawn((
        Connection {
            from: inhibitor,
            to: target,
            strength: 1.0,
            directional: true,
        },
        CurrentSynapse {
            maximum_current: 20e-9,
            ..CurrentSynapse::default()
        },
    ));

    let dt = 0.0001;
    let spikes = run_headless(&mut world, 10_000, dt);

    let mut world_no_inhib = hecs::World::new();
    let neuron_alone = world_no_inhib.spawn((
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));
    let clamp_alone = world_no_inhib.spawn((
        CurrentClamp {
            current_output: 5e-9,
        },
        Position {
            position: Vec3::new(0.0, 0.0, -2.0),
        },
    ));
    world_no_inhib.spawn((
        Connection {
            from: clamp_alone,
            to: neuron_alone,
            strength: 1.0,
            directional: true,
        },
        ImmediateFireSynapse::default(),
    ));
    let spikes_no_inhib = run_headless(&mut world_no_inhib, 10_000, dt);

    let target_spikes: Vec<_> = spikes.iter().filter(|s| s.entity_index == 0).collect();
    let alone_spikes: Vec<_> = spikes_no_inhib
        .iter()
        .filter(|s| s.entity_index == 0)
        .collect();

    assert!(
        target_spikes.len() < alone_spikes.len(),
        "Inhibition should reduce firing: {} with inhibition vs {} without",
        target_spikes.len(),
        alone_spikes.len()
    );
}
