use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::{spawn_spike_detector, spawn_spike_detector_wire, Element};
use neuronify_core::measurement::spike_detector::{RowOrder, SpikeDetector};
use neuronify_core::{
    Connection, ConnectionView, CurrentClamp, CurrentSynapse, Deletable, MeterSize, NeuronDisplay,
    NeuronType, PoissonGenerator, Selected, Setting,
};

const NANOAMPERE: f64 = 1.0;
const MILLISECOND: f64 = 1.0;
const CURRENT_DRAWN_AT_FULL_STRENGTH: f64 = 2.0 * NANOAMPERE;

pub fn labs() -> [(&'static str, World); 2] {
    [
        ("ring_attractor.neuronify", ring_attractor(RingInput::Bias)),
        (
            "ring_attractor_noise.neuronify",
            ring_attractor(RingInput::Noise),
        ),
    ]
}

fn main() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/labs");
    std::fs::create_dir_all(folder).unwrap();
    for (file, world) in labs() {
        let bytes = neuronify_core::serialization::save(&world, 0.0, DEFAULT_SPEED).unwrap();
        std::fs::write(format!("{folder}/{file}"), bytes).unwrap();
        println!("Wrote {file}");
    }
}

fn neuron(world: &mut World, position: Vec3, neuron_type: NeuronType) -> Entity {
    match neuron_type {
        NeuronType::Excitatory => Element::ExcitatoryNeuron,
        NeuronType::Inhibitory => Element::InhibitoryNeuron,
    }
    .spawn(world, position)
}

fn synapse(world: &mut World, from: Entity, to: Entity, maximum_current: f64, delay: f64) {
    world.spawn((
        Connection {
            from,
            to,
            strength: maximum_current / CURRENT_DRAWN_AT_FULL_STRENGTH,
            directional: true,
        },
        CurrentSynapse {
            maximum_current,
            delay,
            ..Default::default()
        },
        Deletable {},
    ));
}

fn clockwise_from_top(radius: f32, fraction_of_turn: f32) -> Vec3 {
    let angle = TAU * fraction_of_turn;
    Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
}

enum RingInput {
    Bias,
    Noise,
}

fn ring_attractor(input: RingInput) -> World {
    const EXCITATORY: usize = 32;
    const INHIBITORY: usize = 4;
    const RING_RADIUS: f32 = 15.0;
    const POOL_RADIUS: f32 = 7.0;
    const TOUCH_SENSOR_RADIUS: f32 = 19.0;
    const NOISE_RADIUS: f32 = 23.0;
    const NEIGHBOUR_CURRENTS: [f64; 3] = [2.0 * NANOAMPERE, 1.32 * NANOAMPERE, 0.66 * NANOAMPERE];
    const EXCITATORY_TO_INHIBITORY: f64 = 0.3 * NANOAMPERE;
    const INHIBITORY_TO_EXCITATORY: f64 = 1.0 * NANOAMPERE;
    const BIAS: f64 = 0.1 * NANOAMPERE;
    const BIAS_CONNECTION_STRENGTH: f64 = 0.15;
    const NOISE_RATE: f64 = 100.0;
    const NOISE_CURRENT: f64 = 0.3 * NANOAMPERE;
    const NOISE_DELAY: f64 = 1.0 * MILLISECOND;
    const CUE_CURRENT: f64 = 6.0 * NANOAMPERE;
    const CUE_TIME_CONSTANT: f64 = 100.0 * MILLISECOND;
    const SPIKE_DETECTOR_GAP: f32 = 5.0;
    const SPIKE_DETECTOR_SIZE: MeterSize = MeterSize {
        width: 40.0,
        height: 16.0,
    };
    const SPIKE_DETECTOR_TIME_RANGE: f64 = 2.0;

    let delay = CurrentSynapse::default().delay;
    let mut world = World::new();
    let ring: Vec<Entity> = (0..EXCITATORY)
        .map(|i| {
            let position = clockwise_from_top(RING_RADIUS, i as f32 / EXCITATORY as f32);
            neuron(&mut world, position, NeuronType::Excitatory)
        })
        .collect();
    let pool: Vec<Entity> = (0..INHIBITORY)
        .map(|i| {
            let position = clockwise_from_top(POOL_RADIUS, (i as f32 + 0.5) / INHIBITORY as f32);
            neuron(&mut world, position, NeuronType::Inhibitory)
        })
        .collect();

    for i in 0..EXCITATORY {
        for (distance, &current) in (1..).zip(&NEIGHBOUR_CURRENTS) {
            for j in [i + distance, i + EXCITATORY - distance] {
                synapse(&mut world, ring[i], ring[j % EXCITATORY], current, delay);
            }
        }
    }
    for &excitatory in &ring {
        for &inhibitory in &pool {
            synapse(
                &mut world,
                excitatory,
                inhibitory,
                EXCITATORY_TO_INHIBITORY,
                delay,
            );
            synapse(
                &mut world,
                inhibitory,
                excitatory,
                INHIBITORY_TO_EXCITATORY,
                delay,
            );
        }
    }

    match input {
        RingInput::Bias => {
            let bias = Element::CurrentSource.spawn(&mut world, Vec3::ZERO);
            world.get::<&mut CurrentClamp>(bias).unwrap().current_output = BIAS;
            for &excitatory in &ring {
                world.spawn((
                    Connection {
                        from: bias,
                        to: excitatory,
                        strength: BIAS_CONNECTION_STRENGTH,
                        directional: true,
                    },
                    CurrentSynapse::default(),
                    Deletable {},
                ));
            }
        }
        RingInput::Noise => {
            for (i, &excitatory) in ring.iter().enumerate() {
                let position =
                    clockwise_from_top(NOISE_RADIUS, (i as f32 + 0.5) / EXCITATORY as f32);
                let generator = Element::PoissonGenerator.spawn(&mut world, position);
                world.get::<&mut PoissonGenerator>(generator).unwrap().rate = NOISE_RATE;
                synapse(
                    &mut world,
                    generator,
                    excitatory,
                    NOISE_CURRENT,
                    NOISE_DELAY,
                );
            }
        }
    }
    for (i, &excitatory) in ring.iter().enumerate() {
        let position = clockwise_from_top(TOUCH_SENSOR_RADIUS, i as f32 / EXCITATORY as f32);
        let sensor = Element::TouchSensor.spawn(&mut world, position);
        world.spawn((
            Connection {
                from: sensor,
                to: excitatory,
                strength: 1.0,
                directional: true,
            },
            CurrentSynapse {
                tau: CUE_TIME_CONSTANT,
                maximum_current: CUE_CURRENT,
                delay: 0.0,
                ..Default::default()
            },
            Deletable {},
        ));
    }
    let outermost = match input {
        RingInput::Bias => TOUCH_SENSOR_RADIUS,
        RingInput::Noise => NOISE_RADIUS,
    };
    let detector_position = Vec3::new(
        -(outermost + SPIKE_DETECTOR_GAP + SPIKE_DETECTOR_SIZE.height * 0.5),
        0.0,
        -SPIKE_DETECTOR_SIZE.width * 0.5,
    );
    let detector = spawn_spike_detector(&mut world, ring[0], detector_position);
    world
        .get::<&mut SpikeDetector>(detector)
        .unwrap()
        .time_range = SPIKE_DETECTOR_TIME_RANGE;
    *world.get::<&mut MeterSize>(detector).unwrap() = SPIKE_DETECTOR_SIZE;
    *world.get::<&mut RowOrder>(detector).unwrap() = RowOrder::Connection;
    for &excitatory in &ring[1..] {
        spawn_spike_detector_wire(&mut world, excitatory, detector);
    }
    ConnectionView::LastSelected.set_in(&mut world);
    NeuronDisplay {
        firing_rate: true,
        ..Default::default()
    }
    .set_in(&mut world);
    Selected::set(&mut world, Some(ring[0]));
    world
}
