use super::convert::spawn_legacy_simulation;
use super::*;
use crate::components::*;
use crate::simulation::{step_simulation, Engine};
use hecs::Entity;

struct SpikeRecord {
    entity_index: usize,
    time: Milliseconds,
}

fn run_headless(world: &mut hecs::World, steps: usize, dt: f64) -> Vec<SpikeRecord> {
    let neurons: Vec<hecs::Entity> = world
        .query::<(Entity, &LeakyNeuron)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    crate::simulation::run_headless(world, Seconds(steps as f64 * dt).to_milliseconds())
        .into_iter()
        .filter_map(|spike| {
            Some(SpikeRecord {
                entity_index: neurons.iter().position(|&entity| entity == spike.entity)?,
                time: spike.time,
            })
        })
        .collect()
}

const EMPTY_NFY: &str = r#"{"nodes": [], "edges": []}"#;

const TUTORIAL_1_INTRO_NFY: &str = include_str!("fixtures/tutorial_1_intro.nfy");

const TWO_NEURON_OSCILLATOR_NFY: &str = include_str!("fixtures/two_neuron_oscillator.nfy");

const LEAKY_NFY: &str = include_str!("fixtures/leaky.nfy");

const ADAPTATION_NFY: &str = include_str!("fixtures/adaptation.nfy");

const BURST_NFY: &str = include_str!("fixtures/burst.nfy");

const INHIBITORY_NFY: &str = include_str!("fixtures/inhibitory.nfy");

const TUTORIAL_2_CIRCUITS_NFY: &str = include_str!("fixtures/tutorial_2_circuits.nfy");

const GENERATORS_NFY: &str = include_str!("fixtures/generators.nfy");

const LATERAL_INHIBITION_1_NFY: &str = include_str!("fixtures/lateral_inhibition_1.nfy");

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
                current_output: Nanoamperes(5.0),
            },
        )
        .unwrap();

    let dt = 0.0001;
    let total_steps = 20_000;

    let neuron_entities: Vec<hecs::Entity> = world
        .query::<(Entity, &LeakyNeuron)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    let all_spikes = run_headless(&mut world, total_steps, dt);
    let time = total_steps as f64 * dt;

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
        let mid_time = Seconds(time / 2.0).to_milliseconds();
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
    assert_eq!(neuron_type(&world, c_entity), NeuronType::Excitatory);

    let b_entity = entities[3];
    assert_eq!(neuron_type(&world, b_entity), NeuronType::Inhibitory);

    let dt = 0.0001;
    run_headless(&mut world, 100, dt);
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
        NodeState::at(Millivolts(-70.0)),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));

    let clamp = world.spawn((
        CurrentClamp {
            current_output: Nanoamperes(5.0),
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
        ImmediateFireSynapse,
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
        NodeState::at(Millivolts(-70.0)),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));

    let clamp = world.spawn((
        CurrentClamp {
            current_output: Nanoamperes(5.0),
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
        ImmediateFireSynapse,
    ));

    let inhibitor = world.spawn((
        LeakyNeuron::default(),
        NodeState::at(Millivolts(-70.0)),
        LeakCurrent::default(),
        NeuronType::Inhibitory,
        Position {
            position: Vec3::new(0.0, 0.0, 1.0),
        },
    ));

    let inhib_clamp = world.spawn((
        CurrentClamp {
            current_output: Nanoamperes(10.0),
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
        ImmediateFireSynapse,
    ));

    world.spawn((
        Connection {
            from: inhibitor,
            to: target,
            strength: 1.0,
            directional: true,
        },
        CurrentSynapse {
            maximum_current: Nanoamperes(20.0),
            ..CurrentSynapse::default()
        },
    ));

    let dt = 0.0001;
    let spikes = run_headless(&mut world, 10_000, dt);

    let mut world_no_inhib = hecs::World::new();
    let neuron_alone = world_no_inhib.spawn((
        LeakyNeuron::default(),
        NodeState::at(Millivolts(-70.0)),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        Position {
            position: Vec3::ZERO,
        },
    ));
    let clamp_alone = world_no_inhib.spawn((
        CurrentClamp {
            current_output: Nanoamperes(5.0),
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
        ImmediateFireSynapse,
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

#[test]
fn test_annotation_loads_with_size() {
    let sim = parse_legacy_nfy(LEAKY_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);

    let mut annotation_count = 0;
    for (_entity, (annotation, size)) in world.query::<(Entity, (&Annotation, &NoteSize))>().iter()
    {
        annotation_count += 1;
        assert!(!annotation.text.is_empty());
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }
    assert!(
        annotation_count > 0,
        "leaky.nfy should produce at least one annotation"
    );
}

#[test]
fn test_burst_neuron_loads_with_burst_current() {
    let sim = parse_legacy_nfy(BURST_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);
    assert_eq!(world.query::<(Entity, &BurstCurrent)>().iter().count(), 1);
}

#[test]
fn the_next_tutorial_button_becomes_a_link_to_the_next_tutorial() {
    let sim = parse_legacy_nfy(TUTORIAL_1_INTRO_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);
    let links: Vec<(String, String)> = world
        .query::<(&Link, &Annotation)>()
        .iter()
        .map(|(link, note)| (link.example.clone(), note.text.clone()))
        .collect();
    assert_eq!(
        links,
        [(
            "tutorial_2_circuits.nfy".to_string(),
            "Next tutorial ›".to_string()
        )]
    );
}

#[test]
fn loads_regular_and_irregular_spike_generators() {
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &parse_legacy_nfy(GENERATORS_NFY).unwrap());

    let frequencies: Vec<Hertz> = world
        .query::<&RegularSpikeGenerator>()
        .iter()
        .map(|generator| generator.frequency)
        .collect();
    let rates: Vec<Hertz> = world
        .query::<&PoissonGenerator>()
        .iter()
        .map(|generator| generator.rate)
        .collect();
    assert_eq!(frequencies, [Hertz(200.0)]);
    assert_eq!(rates, [Hertz(200.0)]);
    for (generator, _) in world.query::<(Entity, &GeneratorDynamics)>().iter() {
        let synapses = world
            .query::<(&Connection, &CurrentSynapse)>()
            .iter()
            .filter(|(connection, _)| connection.from == generator)
            .count();
        assert_eq!(synapses, 1);
    }
}

fn generator_driving_a_neuron(inhibitory: bool) -> String {
    format!(
        r#"{{"fileFormatVersion": 4, "nodes": [
            {{"filename": "generators/RegularSpikeGenerator.qml",
              "savedProperties": {{"engine": {{"rate": 100}}, "inhibitory": {inhibitory}, "x": 0, "y": 0}}}},
            {{"filename": "neurons/LeakyNeuron.qml",
              "savedProperties": {{"engine": {{"voltage": -0.07}}, "x": 100, "y": 0}}}}
        ], "edges": [
            {{"filename": "edges/CurrentSynapse.qml", "from": 0, "to": 1,
              "savedProperties": {{"engine": {{"maximumCurrent": 3e-10, "tau": 0.002, "delay": 0}}}}}}
        ]}}"#
    )
}

#[test]
fn spikes_from_an_inhibitory_generator_push_the_voltage_down() {
    for inhibitory in [false, true] {
        let simulation = parse_legacy_nfy(&generator_driving_a_neuron(inhibitory)).unwrap();
        let mut world = hecs::World::new();
        let entities = spawn_legacy_simulation(&mut world, &simulation);
        let expected = if inhibitory {
            NeuronType::Inhibitory
        } else {
            NeuronType::Excitatory
        };
        assert_eq!(neuron_type(&world, entities[0]), expected);

        let resting = world
            .get::<&LeakyNeuron>(entities[1])
            .unwrap()
            .resting_potential;
        let mut furthest = Millivolts::ZERO;
        let (mut engine, mut time) = (Engine::default(), Milliseconds::ZERO);
        for _ in 0..600 {
            step_simulation(&mut world, &mut engine, &mut time, 1, None);
            let change = world.get::<&NodeState>(entities[1]).unwrap().voltage - resting;
            if change.abs() > furthest.abs() {
                furthest = change;
            }
        }
        assert!(furthest.abs() > Millivolts(1.0));
        assert_eq!(furthest < Millivolts::ZERO, inhibitory);
    }
}

#[test]
fn loads_inhibitory_adaptive_neurons() {
    let file = r#"{"fileFormatVersion": 4, "edges": [], "nodes": [
        {"filename": "neurons/AdaptationInhibitoryNeuron.qml",
         "savedProperties": {"engine": {"adaptation": 2e-08}, "x": 0, "y": 0}}
    ]}"#;
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &parse_legacy_nfy(file).unwrap());
    assert_eq!(neuron_type(&world, entities[0]), NeuronType::Inhibitory);
    assert_eq!(
        world
            .get::<&AdaptationCurrent>(entities[0])
            .unwrap()
            .adaptation,
        Microsiemens(0.02)
    );
}

#[test]
fn current_sources_load_without_a_synapse() {
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &parse_legacy_nfy(GENERATORS_NFY).unwrap());
    let (source, _) = world
        .query::<(Entity, &CurrentClamp)>()
        .iter()
        .next()
        .unwrap();
    let mut connections = world.query::<(Entity, (&Connection, Option<&CurrentSynapse>))>();
    let from_source: Vec<bool> = connections
        .iter()
        .filter(|(_, (connection, _))| connection.from == source)
        .map(|(_, (_, synapse))| synapse.is_some())
        .collect();
    assert_eq!(from_source, [false]);
}

#[test]
fn a_voltmeter_measures_every_neuron_wired_to_it_from_the_highest_down() {
    let sim = parse_legacy_nfy(LATERAL_INHIBITION_1_NFY).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);
    let measured = |voltmeter: usize| -> Vec<hecs::Entity> {
        crate::measurement::voltmeter::measured(&world, entities[voltmeter])
            .into_iter()
            .map(|(_, neuron)| neuron)
            .collect()
    };
    assert_eq!(measured(6), [entities[9], entities[8], entities[0]]);
    assert_eq!(measured(7), [entities[1], entities[3], entities[2]]);

    let size = world.get::<&MeterSize>(entities[6]).unwrap();
    assert_eq!((size.width, size.height), (384.0 / 25.0, 288.0 / 25.0));
}

#[test]
fn a_voltmeter_keeps_its_range_and_sits_where_neuronify_1_drew_it() {
    let file = r#"{"fileFormatVersion": 4, "edges": [], "nodes": [
        {"filename": "meters/Voltmeter.qml",
         "savedProperties": {"x": 960, "y": 540, "width": 200, "height": 100,
                             "minimumValue": -200, "maximumValue": 20}}
    ]}"#;
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &parse_legacy_nfy(file).unwrap());
    let range = world
        .get::<&crate::measurement::voltmeter::VoltmeterRange>(entities[0])
        .unwrap()
        .clone();
    assert_eq!(
        (range.minimum, range.maximum),
        (Millivolts(-200.0), Millivolts(20.0))
    );
    let frame = crate::measurement::voltmeter::Frame::of(&world, entities[0]).unwrap();
    let top_left = frame.corner(crate::measurement::voltmeter::Corner::TopLeft);
    assert!(top_left.abs_diff_eq(glam::Vec3::ZERO, 1e-5), "{top_left}");
}

#[test]
fn neurons_keep_their_labels() {
    let sim = parse_legacy_nfy(TUTORIAL_2_CIRCUITS_NFY).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);
    let mut labels: Vec<String> = world
        .query::<&Label>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|label| label.text.clone())
        .filter(|text| !text.is_empty())
        .collect();
    labels.sort();
    assert_eq!(labels, ["A", "B", "C"]);
    assert_eq!(
        world.query::<(Entity, &Label)>().iter().count(),
        world.query::<(Entity, &LeakyNeuron)>().iter().count()
    );
}

#[test]
fn labels_lose_the_spaces_around_them() {
    let file = r#"{"fileFormatVersion": 4, "edges": [], "nodes": [
        {"filename": "neurons/LeakyNeuron.qml",
         "savedProperties": {"label": "with refractory period ", "x": 0, "y": 0}}
    ]}"#;
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &parse_legacy_nfy(file).unwrap());
    assert_eq!(
        world.get::<&Label>(entities[0]).unwrap().text,
        "with refractory period"
    );
}

#[test]
fn loads_alternating_current_sources() {
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &parse_legacy_nfy(GENERATORS_NFY).unwrap());
    let sources: Vec<(hecs::Entity, Nanoamperes, Hertz)> = world
        .query::<(Entity, &AlternatingCurrent)>()
        .with::<&CurrentClamp>()
        .iter()
        .map(|(source, alternating)| (source, alternating.amplitude, alternating.frequency))
        .collect();
    let [(source, amplitude, frequency)] = sources[..] else {
        panic!("expected one alternating current source, got {sources:?}");
    };
    assert_eq!((amplitude, frequency), (Nanoamperes(1.0), Hertz(34.0)));
    let targets = world
        .query::<&Connection>()
        .without::<&CurrentSynapse>()
        .iter()
        .filter(|connection| connection.from == source)
        .count();
    assert_eq!(targets, 1);
}

#[test]
fn spike_detectors_keep_their_rows_time_range_and_size() {
    let sim = parse_legacy_nfy(GENERATORS_NFY).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);
    let detector = entities[4];
    let time_range = world
        .get::<&crate::measurement::spike_detector::SpikeDetector>(detector)
        .unwrap()
        .time_range;
    assert_eq!(time_range, Seconds(0.101).to_milliseconds());
    let rows: Vec<hecs::Entity> = crate::measurement::voltmeter::measured(&world, detector)
        .into_iter()
        .map(|(_, neuron)| neuron)
        .collect();
    assert_eq!(rows, [entities[2], entities[0], entities[3], entities[1]]);
    let size = world.get::<&MeterSize>(detector).unwrap();
    assert_eq!((size.width, size.height), (448.0 / 25.0, 352.0 / 25.0));
}

#[test]
fn rate_plots_keep_their_window_smoothing_and_range() {
    let sim = parse_legacy_nfy(include_str!("fixtures/types_of_inhibition.nfy")).unwrap();
    let mut world = hecs::World::new();
    spawn_legacy_simulation(&mut world, &sim);
    let plots: Vec<(hecs::Entity, crate::measurement::firing_rate::RatePlot)> = world
        .query::<(Entity, &crate::measurement::firing_rate::RatePlot)>()
        .iter()
        .map(|(plot, settings)| (plot, settings.clone()))
        .collect();
    assert_eq!(plots.len(), 2);
    for (plot, settings) in plots {
        assert_eq!(
            (settings.window, settings.kernel_width, settings.maximum),
            (
                Seconds(0.4).to_milliseconds(),
                Seconds(0.3).to_milliseconds(),
                Hertz(50.0)
            )
        );
        assert!(!crate::measurement::voltmeter::measured(&world, plot).is_empty());
    }
}

#[test]
fn loudspeakers_play_the_closest_sound_for_each_neuron_wired_to_them() {
    let file = r#"{"fileFormatVersion": 4, "nodes": [
        {"filename": "meters/Loudspeaker.qml", "savedProperties": {"source": "drip.wav", "x": 0, "y": 0}},
        {"filename": "neurons/LeakyNeuron.qml", "savedProperties": {"x": 100, "y": 0}}
    ], "edges": [
        {"filename": "edges/MeterEdge.qml", "from": 0, "to": 1, "savedProperties": {}}
    ]}"#;
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &parse_legacy_nfy(file).unwrap());
    let speaker = world
        .get::<&crate::measurement::loudspeaker::Loudspeaker>(entities[0])
        .unwrap()
        .clone();
    assert_eq!(speaker.sound, crate::measurement::loudspeaker::Sound::Drop);
    let wired: Vec<hecs::Entity> = crate::measurement::voltmeter::measured(&world, entities[0])
        .into_iter()
        .map(|(_, neuron)| neuron)
        .collect();
    assert_eq!(wired, [entities[1]]);
}

#[test]
fn retinas_become_visual_inputs_with_their_receptive_fields() {
    use crate::measurement::visual_input::{ReceptiveField, VisualInput};
    let sim = parse_legacy_nfy(include_str!("fixtures/visualInput.nfy")).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);
    let edge = world.get::<&VisualInput>(entities[0]).unwrap().clone();
    let orientation = world.get::<&VisualInput>(entities[6]).unwrap().clone();
    assert_eq!(
        edge.field,
        ReceptiveField::Edge {
            angle: 4.71238898038469
        }
    );
    assert_eq!((edge.sensitivity, edge.show_field), (Hertz(1000.0), true));
    assert_eq!(
        orientation.field,
        ReceptiveField::Orientation {
            angle: 0.0,
            x: 0.0,
            y: 0.0
        }
    );
    assert_eq!(orientation.sensitivity, Hertz(1600.0));
    for input in [entities[0], entities[6]] {
        let synapses = world
            .query::<(&Connection, &CurrentSynapse)>()
            .iter()
            .filter(|(connection, _)| connection.from == input && connection.to == entities[1])
            .count();
        assert_eq!(synapses, 1);
    }
}

#[test]
fn a_visual_input_drives_the_neuron_it_connects_to() {
    let sim = parse_legacy_nfy(include_str!("fixtures/visualInput.nfy")).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);
    let spikes = run_headless(&mut world, 20_000, 0.0001);
    let neuron = world
        .query::<(Entity, &LeakyNeuron)>()
        .iter()
        .position(|(entity, _)| entity == entities[1])
        .unwrap();
    assert!(spikes.iter().any(|spike| spike.entity_index == neuron));
}

fn bundled_neuronify_1_files() -> Vec<(String, String)> {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/src/legacy/fixtures");
    let mut files: Vec<(String, String)> = std::fs::read_dir(fixtures)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "nfy"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(path).unwrap())
        })
        .collect();
    files.sort();
    files
}

fn converted(world: &hecs::World, entity: hecs::Entity) -> bool {
    use crate::measurement::{is_meter, visual_input::VisualInput};
    world.satisfies::<hecs::Or<
        &LeakyNeuron,
        hecs::Or<&CurrentClamp, hecs::Or<&GeneratorDynamics, hecs::Or<&VisualInput, &Annotation>>>,
    >>(entity)
        || is_meter(world, entity)
}

#[test]
fn every_bundled_neuronify_1_file_loads_with_nothing_dropped() {
    let files = bundled_neuronify_1_files();
    assert_eq!(files.len(), 23);
    for (name, json) in files {
        let sim = parse_legacy_nfy(&json).unwrap();
        let mut world = hecs::World::new();
        let entities = spawn_legacy_simulation(&mut world, &sim);
        for (node, &entity) in sim.nodes.iter().zip(&entities) {
            assert!(
                converted(&world, entity),
                "{name}: {} was dropped",
                node.filename
            );
        }
        let connections: Vec<(hecs::Entity, hecs::Entity)> = world
            .query::<&Connection>()
            .iter()
            .map(|connection| (connection.from, connection.to))
            .collect();
        for edge in &sim.edges {
            let (from, to) = (entities[edge.from], entities[edge.to]);
            let kept = connections.contains(&(from, to))
                || (crate::measurement::is_meter(&world, from)
                    && connections.contains(&(to, from)));
            assert!(
                kept,
                "{name}: {} from {} to {} was dropped",
                edge.filename, edge.from, edge.to
            );
        }
        assert_eq!(connections.len(), sim.edges.len(), "{name}");
    }
}

#[test]
fn a_negative_legacy_fire_output_makes_a_leaky_neuron_inhibitory() {
    let sim = parse_legacy_nfy(TWO_NEURON_OSCILLATOR_NFY).unwrap();
    let mut world = hecs::World::new();
    let entities = spawn_legacy_simulation(&mut world, &sim);
    for neuron in &entities[..2] {
        assert_eq!(neuron_type(&world, *neuron), NeuronType::Inhibitory);
    }
}

#[test]
fn a_linked_file_is_only_looked_up_beside_the_open_one() {
    assert_eq!(
        sibling_file("tutorial_2_circuits.nfy"),
        Some("tutorial_2_circuits.nfy")
    );
    for name in [
        "",
        ".",
        "..",
        "../secret.nfy",
        "a/b.nfy",
        "..\\b.nfy",
        "C:b.nfy",
    ] {
        assert_eq!(sibling_file(name), None, "{name}");
    }
}
