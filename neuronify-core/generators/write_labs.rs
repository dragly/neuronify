use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::{spawn_spike_detector, spawn_spike_detector_wire, Element};
use neuronify_core::measurement::spike_detector::{RowOrder, SpikeDetector};
use neuronify_core::{
    Connection, ConnectionView, CurrentClamp, CurrentSynapse, Deletable, Hertz, Label, MeterSize,
    Milliseconds, Nanoamperes, NeuronDisplay, NeuronType, PoissonGenerator, Selected, Setting,
};

const CURRENT_DRAWN_AT_FULL_STRENGTH: Nanoamperes = Nanoamperes(2.0);
const BIAS_CONNECTION_STRENGTH: f64 = 0.15;
const CUE_CURRENT: Nanoamperes = Nanoamperes(6.0);
const CUE_TIME_CONSTANT: Milliseconds = Milliseconds(100.0);
const SPIKE_DETECTOR_GAP: f32 = 5.0;
const SPIKE_DETECTOR_SIZE: MeterSize = MeterSize {
    width: 40.0,
    height: 16.0,
};
const SPIKE_DETECTOR_TIME_RANGE: Milliseconds = Milliseconds(2000.0);
const COMPASS_SPEED: f64 = 8.0;

pub fn labs() -> [(&'static str, World, f64); 3] {
    [
        (
            "ring_attractor.neuronify",
            ring_attractor(RingInput::Bias),
            DEFAULT_SPEED,
        ),
        (
            "ring_attractor_noise.neuronify",
            ring_attractor(RingInput::Noise),
            DEFAULT_SPEED,
        ),
        ("compass.neuronify", compass(), COMPASS_SPEED),
    ]
}

fn main() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/labs");
    std::fs::create_dir_all(folder).expect("the labs folder should be writable");
    for (file, world, speed) in labs() {
        let bytes = neuronify_core::serialization::save(&world, Milliseconds::ZERO, speed)
            .expect("the lab should save");
        std::fs::write(format!("{folder}/{file}"), bytes)
            .expect("the labs folder should be writable");
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

fn synapse(
    world: &mut World,
    from: Entity,
    to: Entity,
    maximum_current: Nanoamperes,
    delay: Milliseconds,
) {
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

fn circle_of_cells(
    world: &mut World,
    count: usize,
    radius: f32,
    offset: f32,
    neuron_type: NeuronType,
) -> Vec<Entity> {
    (0..count)
        .map(|i| {
            let position = clockwise_from_top(radius, (i as f32 + offset) / count as f32);
            neuron(world, position, neuron_type)
        })
        .collect()
}

fn connect_neighbours(world: &mut World, ring: &[Entity], currents: &[Nanoamperes]) {
    let delay = CurrentSynapse::default().delay;
    let cells = ring.len();
    for i in 0..cells {
        for (distance, &current) in (1..).zip(currents) {
            for j in [i + distance, i + cells - distance] {
                synapse(world, ring[i], ring[j % cells], current, delay);
            }
        }
    }
}

fn connect_pool(
    world: &mut World,
    ring: &[Entity],
    pool: &[Entity],
    excitatory_to_inhibitory: Nanoamperes,
    inhibitory_to_excitatory: Nanoamperes,
) {
    let delay = CurrentSynapse::default().delay;
    for &excitatory in ring {
        for &inhibitory in pool {
            synapse(
                world,
                excitatory,
                inhibitory,
                excitatory_to_inhibitory,
                delay,
            );
            synapse(
                world,
                inhibitory,
                excitatory,
                inhibitory_to_excitatory,
                delay,
            );
        }
    }
}

fn current_source(
    world: &mut World,
    position: Vec3,
    current: Nanoamperes,
    cells: &[Entity],
    strength: f64,
) -> Entity {
    let source = Element::CurrentSource.spawn(world, position);
    world
        .get::<&mut CurrentClamp>(source)
        .expect("current sources spawn with a CurrentClamp")
        .current_output = current;
    for &cell in cells {
        world.spawn((
            Connection {
                from: source,
                to: cell,
                strength,
                directional: true,
            },
            CurrentSynapse::default(),
            Deletable {},
        ));
    }
    source
}

fn cue_sensors(world: &mut World, ring: &[Entity], radius: f32) {
    for (i, &cell) in ring.iter().enumerate() {
        let position = clockwise_from_top(radius, i as f32 / ring.len() as f32);
        let sensor = Element::TouchSensor.spawn(world, position);
        world.spawn((
            Connection {
                from: sensor,
                to: cell,
                strength: 1.0,
                directional: true,
            },
            CurrentSynapse {
                tau: CUE_TIME_CONSTANT,
                maximum_current: CUE_CURRENT,
                delay: Milliseconds::ZERO,
                ..Default::default()
            },
            Deletable {},
        ));
    }
}

fn ring_detector(world: &mut World, ring: &[Entity], outermost: f32) {
    let position = Vec3::new(
        -(outermost + SPIKE_DETECTOR_GAP + SPIKE_DETECTOR_SIZE.height * 0.5),
        0.0,
        -SPIKE_DETECTOR_SIZE.width * 0.5,
    );
    let detector = spawn_spike_detector(world, ring[0], position);
    world
        .get::<&mut SpikeDetector>(detector)
        .expect("spike detectors spawn with a SpikeDetector")
        .time_range = SPIKE_DETECTOR_TIME_RANGE;
    *world
        .get::<&mut MeterSize>(detector)
        .expect("meters spawn with a MeterSize") = SPIKE_DETECTOR_SIZE;
    *world
        .get::<&mut RowOrder>(detector)
        .expect("spike detectors spawn with a RowOrder") = RowOrder::Connection;
    for &cell in &ring[1..] {
        spawn_spike_detector_wire(world, cell, detector);
    }
}

fn open_on_the_top_cell(world: &mut World, ring: &[Entity]) {
    ConnectionView::LastSelected.set_in(world);
    NeuronDisplay {
        firing_rate: true,
        ..Default::default()
    }
    .set_in(world);
    Selected::set(world, Some(ring[0]));
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
    const NEIGHBOUR_CURRENTS: [Nanoamperes; 3] =
        [Nanoamperes(2.0), Nanoamperes(1.32), Nanoamperes(0.66)];
    const EXCITATORY_TO_INHIBITORY: Nanoamperes = Nanoamperes(0.3);
    const INHIBITORY_TO_EXCITATORY: Nanoamperes = Nanoamperes(1.0);
    const BIAS: Nanoamperes = Nanoamperes(0.1);
    const NOISE_RATE: Hertz = Hertz(100.0);
    const NOISE_CURRENT: Nanoamperes = Nanoamperes(0.3);
    const NOISE_DELAY: Milliseconds = Milliseconds(1.0);

    let mut world = World::new();
    let ring = circle_of_cells(
        &mut world,
        EXCITATORY,
        RING_RADIUS,
        0.0,
        NeuronType::Excitatory,
    );
    let pool = circle_of_cells(
        &mut world,
        INHIBITORY,
        POOL_RADIUS,
        0.5,
        NeuronType::Inhibitory,
    );
    connect_neighbours(&mut world, &ring, &NEIGHBOUR_CURRENTS);
    connect_pool(
        &mut world,
        &ring,
        &pool,
        EXCITATORY_TO_INHIBITORY,
        INHIBITORY_TO_EXCITATORY,
    );
    match input {
        RingInput::Bias => {
            current_source(
                &mut world,
                Vec3::ZERO,
                BIAS,
                &ring,
                BIAS_CONNECTION_STRENGTH,
            );
        }
        RingInput::Noise => {
            for (i, &excitatory) in ring.iter().enumerate() {
                let position =
                    clockwise_from_top(NOISE_RADIUS, (i as f32 + 0.5) / EXCITATORY as f32);
                let generator = Element::PoissonGenerator.spawn(&mut world, position);
                world
                    .get::<&mut PoissonGenerator>(generator)
                    .expect("Poisson generators spawn with a PoissonGenerator")
                    .rate = NOISE_RATE;
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
    cue_sensors(&mut world, &ring, TOUCH_SENSOR_RADIUS);
    let outermost = match input {
        RingInput::Bias => TOUCH_SENSOR_RADIUS,
        RingInput::Noise => NOISE_RADIUS,
    };
    ring_detector(&mut world, &ring, outermost);
    open_on_the_top_cell(&mut world, &ring);
    world
}

fn compass() -> World {
    const EXCITATORY: usize = 16;
    const INHIBITORY: usize = 4;
    const RING_RADIUS: f32 = 12.0;
    const POOL_RADIUS: f32 = 5.0;
    const ROTATION_RADIUS: f32 = 16.0;
    const ROTATION_OFFSET: f32 = 0.25;
    const TOUCH_SENSOR_RADIUS: f32 = 20.0;
    const TURN_GAP: f32 = 6.0;
    const TURN_SPREAD: f32 = 6.0;
    const NEIGHBOUR_CURRENTS: [Nanoamperes; 2] = [Nanoamperes(2.858), Nanoamperes(0.344)];
    const EXCITATORY_TO_INHIBITORY: Nanoamperes = Nanoamperes(0.246);
    const INHIBITORY_TO_EXCITATORY: Nanoamperes = Nanoamperes(1.154);
    const BIAS: Nanoamperes = Nanoamperes(0.093);
    const RING_TO_ROTATION: Nanoamperes = Nanoamperes(1.471);
    const INHIBITORY_TO_ROTATION: Nanoamperes = Nanoamperes(0.706);
    const ROTATION_TO_RING: [Nanoamperes; 2] = [Nanoamperes(1.029), Nanoamperes(0.231)];
    const TURNING_TO_ROTATION: Nanoamperes = Nanoamperes(0.536);

    let delay = CurrentSynapse::default().delay;
    let mut world = World::new();
    let ring = circle_of_cells(
        &mut world,
        EXCITATORY,
        RING_RADIUS,
        0.0,
        NeuronType::Excitatory,
    );
    let pool = circle_of_cells(
        &mut world,
        INHIBITORY,
        POOL_RADIUS,
        0.5,
        NeuronType::Inhibitory,
    );
    let clockwise = circle_of_cells(
        &mut world,
        EXCITATORY,
        ROTATION_RADIUS,
        ROTATION_OFFSET,
        NeuronType::Excitatory,
    );
    let anticlockwise = circle_of_cells(
        &mut world,
        EXCITATORY,
        ROTATION_RADIUS,
        -ROTATION_OFFSET,
        NeuronType::Excitatory,
    );
    connect_neighbours(&mut world, &ring, &NEIGHBOUR_CURRENTS);
    connect_pool(
        &mut world,
        &ring,
        &pool,
        EXCITATORY_TO_INHIBITORY,
        INHIBITORY_TO_EXCITATORY,
    );
    for (rotation, step) in [(&clockwise, 1), (&anticlockwise, EXCITATORY - 1)] {
        for (i, &cell) in rotation.iter().enumerate() {
            synapse(&mut world, ring[i], cell, RING_TO_ROTATION, delay);
            for &inhibitory in &pool {
                synapse(&mut world, inhibitory, cell, INHIBITORY_TO_ROTATION, delay);
            }
            for (distance, &current) in (1..).zip(&ROTATION_TO_RING) {
                let ahead = ring[(i + distance * step) % EXCITATORY];
                synapse(&mut world, cell, ahead, current, delay);
            }
        }
    }

    current_source(
        &mut world,
        Vec3::ZERO,
        BIAS,
        &ring,
        BIAS_CONNECTION_STRENGTH,
    );
    let turning_x = TOUCH_SENSOR_RADIUS + TURN_GAP;
    for (side, opposite, text) in [
        (1.0, &anticlockwise, "Turn clockwise"),
        (-1.0, &clockwise, "Turn anticlockwise"),
    ] {
        let turning = neuron(
            &mut world,
            Vec3::new(turning_x, 0.0, side * TURN_SPREAD),
            NeuronType::Inhibitory,
        );
        for &cell in opposite {
            synapse(&mut world, turning, cell, TURNING_TO_ROTATION, delay);
        }
        let source = current_source(
            &mut world,
            Vec3::new(turning_x + TURN_GAP, 0.0, side * TURN_SPREAD),
            Nanoamperes::ZERO,
            &[turning],
            1.0,
        );
        world
            .insert_one(
                source,
                Label {
                    text: text.to_string(),
                },
            )
            .expect("the turn source was just spawned");
    }
    cue_sensors(&mut world, &ring, TOUCH_SENSOR_RADIUS);
    ring_detector(&mut world, &ring, TOUCH_SENSOR_RADIUS);
    open_on_the_top_cell(&mut world, &ring);
    world
}
