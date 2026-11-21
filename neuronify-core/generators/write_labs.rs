use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::Element;
use neuronify_core::measurement::frame::RowOrder;
use neuronify_core::measurement::spike_detector::SpikeDetector;
use neuronify_core::measurement::visual_input::{
    spawn_picture_wire, Pattern, ReceptiveField, ReceptiveFieldCell, VisualInput,
};
use neuronify_core::measurement::Meter;
use neuronify_core::rendering::notes::{NOTE_FONT_DATA, NOTE_MARGIN, NOTE_TEXT_HEIGHT};
use neuronify_core::{
    Annotation, Connection, ConnectionView, CurrentClamp, CurrentSynapse, Deletable, Hertz, Label,
    MeterSize, Milliseconds, Nanoamperes, NeuronDisplay, NeuronType, NoteSize, PoissonGenerator,
    Selected, Setting,
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
const DIRECTION_SELECTIVITY_SPEED: f64 = 8.0;

pub fn labs() -> [(&'static str, World, f64); 6] {
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
        ("hopfield.neuronify", hopfield(&STORED), DEFAULT_SPEED),
        (
            "hopfield_overloaded.neuronify",
            hopfield(&OVERLOADED),
            DEFAULT_SPEED,
        ),
        (
            "direction_selectivity.neuronify",
            direction_selectivity(),
            DIRECTION_SELECTIVITY_SPEED,
        ),
    ]
}

fn main() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/labs");
    std::fs::create_dir_all(folder).expect("the labs folder should be writable");
    for (file, world, speed) in labs() {
        let bytes = neuronify_core::serialization::save(
            &world,
            Milliseconds::ZERO,
            speed,
            &neuronify_core::serialization::Meta::default(),
        )
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
    let detector = Meter::SpikeDetector.spawn_measuring(world, ring[0], position);
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
        Meter::SpikeDetector.wire(world, cell, detector);
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

const SHEET_WIDTH: f32 = 24.0;
const SHEET_GAP: f32 = 1.2;

struct Sheet {
    world: World,
    top: f32,
}

impl Sheet {
    fn new() -> Sheet {
        Sheet {
            world: World::new(),
            top: 0.0,
        }
    }

    fn at(right: f32, up: f32) -> Vec3 {
        Vec3::new(up, 0.0, right)
    }

    fn text(&mut self, text: &str) {
        let wrapped = visula::text::tessellate_text_wrapped(
            NOTE_FONT_DATA,
            text,
            NOTE_TEXT_HEIGHT,
            SHEET_WIDTH - 2.0 * NOTE_MARGIN,
        )
        .expect("the note font should lay out text");
        let height = wrapped.height + 2.0 * NOTE_MARGIN;
        let note = Element::Note.spawn(
            &mut self.world,
            Sheet::at(SHEET_WIDTH / 2.0, self.top - height / 2.0),
        );
        self.world
            .get::<&mut Annotation>(note)
            .expect("notes spawn with an Annotation")
            .text = text.to_string();
        *self
            .world
            .get::<&mut NoteSize>(note)
            .expect("notes spawn with a NoteSize") = NoteSize {
            width: SHEET_WIDTH,
            height,
        };
        self.top -= height + SHEET_GAP;
    }

    fn row(&mut self, height: f32) -> f32 {
        let top = self.top;
        self.top -= height + SHEET_GAP;
        top
    }

    fn add(&mut self, element: Element, right: f32, up: f32) -> Entity {
        element.spawn(&mut self.world, Sheet::at(right, up))
    }

    fn neuron(&mut self, neuron_type: NeuronType, right: f32, up: f32, label: &str) -> Entity {
        let neuron = neuron(&mut self.world, Sheet::at(right, up), neuron_type);
        self.world
            .get::<&mut Label>(neuron)
            .expect("neurons spawn with a Label")
            .text = label.to_string();
        neuron
    }

    fn synapse(
        &mut self,
        from: Entity,
        to: Entity,
        current: Nanoamperes,
        tau: Milliseconds,
        delay: Milliseconds,
    ) {
        self.world.spawn((
            Connection {
                from,
                to,
                strength: (current * tau / (CURRENT_DRAWN_AT_FULL_STRENGTH * Milliseconds(2.0)))
                    .clamp(0.3, 1.5),
                directional: true,
            },
            CurrentSynapse {
                maximum_current: current,
                tau,
                delay,
                ..Default::default()
            },
            Deletable {},
        ));
    }
}

const RELAY: Nanoamperes = Nanoamperes(3.0);

pub const GRID: usize = 5;
pub const STORED: [char; 3] = ['T', 'L', 'X'];
pub const OVERLOADED: [char; 5] = ['T', 'L', 'X', 'I', 'V'];
pub const RESET: &str = "Reset";

const GRID_SPACING: f32 = 5.4;
const PARTNER_SPACING: f32 = 4.0;
const HEBBIAN_GAIN: Nanoamperes = Nanoamperes(1.3);
const HOPFIELD_CUE_CURRENT: Nanoamperes = Nanoamperes(2.0);
const HOPFIELD_CUE_TIME_CONSTANT: Milliseconds = Milliseconds(50.0);
const RESET_CURRENT: Nanoamperes = Nanoamperes(6.0);
const RESET_TIME_CONSTANT: Milliseconds = Milliseconds(100.0);

pub fn lit(letter: char, row: usize, column: usize) -> bool {
    let last = GRID - 1;
    match letter {
        'T' => row == 0 || column == 2,
        'L' => column == 0 || row == last,
        'X' => row == column || row + column == last,
        'I' => ((row == 0 || row == last) && (1..last).contains(&column)) || column == 2,
        'V' => matches!(
            (row, column),
            (0, 0) | (1, 0) | (2, 1) | (3, 1) | (4, 2) | (3, 3) | (2, 3) | (1, 4) | (0, 4)
        ),
        _ => false,
    }
}

fn hopfield(letters: &[char]) -> World {
    let delay = CurrentSynapse::default().delay;
    let fast = Milliseconds(2.0);
    let mut sheet = Sheet::new();
    sheet.text(&format!(
        "The connections store the letters {}. \
         Neurons lit or dark together in most of them excite each other, \
         and the rest hold each other back through the partners below the grid.",
        listed(letters)
    ));
    let top = sheet.row(GRID as f32 * GRID_SPACING - 1.0) - 1.0;
    let place = |row: usize, column: usize| {
        (
            1.0 + column as f32 * GRID_SPACING,
            top - row as f32 * GRID_SPACING,
        )
    };
    let mut cells = Vec::new();
    let mut partners = Vec::new();
    let mut pads = Vec::new();
    for row in 0..GRID {
        for column in 0..GRID {
            let (right, up) = place(row, column);
            cells.push(sheet.neuron(NeuronType::Excitatory, right, up, ""));
            pads.push(sheet.add(Element::TouchSensor, right - 2.1, up - 2.1));
        }
    }
    let partner_top = sheet.row(GRID as f32 * PARTNER_SPACING + 1.0) - 1.0;
    for row in 0..GRID {
        for column in 0..GRID {
            let right = SHEET_WIDTH / 2.0 + (column as f32 - 2.0) * PARTNER_SPACING;
            let up = partner_top - row as f32 * PARTNER_SPACING;
            partners.push(sheet.neuron(NeuronType::Inhibitory, right, up, ""));
        }
    }
    let patterns: Vec<Vec<f64>> = letters
        .iter()
        .map(|&letter| {
            (0..GRID * GRID)
                .map(|cell| f64::from(u8::from(lit(letter, cell / GRID, cell % GRID))))
                .collect()
        })
        .collect();
    let activity = patterns.iter().flatten().sum::<f64>() / (patterns.len() * GRID * GRID) as f64;
    for cell in 0..GRID * GRID {
        sheet.synapse(cells[cell], partners[cell], RELAY, fast, Milliseconds(1.0));
        sheet.synapse(
            pads[cell],
            cells[cell],
            HOPFIELD_CUE_CURRENT,
            HOPFIELD_CUE_TIME_CONSTANT,
            Milliseconds::ZERO,
        );
    }
    for from in 0..GRID * GRID {
        for to in 0..GRID * GRID {
            if from == to {
                continue;
            }
            let weight = patterns
                .iter()
                .map(|pattern| (pattern[from] - activity) * (pattern[to] - activity))
                .sum::<f64>()
                * HEBBIAN_GAIN;
            if weight > Nanoamperes::ZERO {
                sheet.synapse(cells[from], cells[to], weight, fast, delay);
            } else if weight < Nanoamperes::ZERO {
                sheet.synapse(
                    partners[from],
                    cells[to],
                    -weight,
                    fast,
                    delay - Milliseconds(1.0),
                );
            }
        }
    }
    let reset_row = sheet.row(4.0) - 2.0;
    let reset_pad = sheet.add(Element::TouchSensor, SHEET_WIDTH / 2.0 - 3.0, reset_row);
    let reset = sheet.neuron(
        NeuronType::Inhibitory,
        SHEET_WIDTH / 2.0 + 1.0,
        reset_row,
        RESET,
    );
    sheet.synapse(reset_pad, reset, RELAY, fast, Milliseconds::ZERO);
    for &cell in &cells {
        sheet.synapse(
            reset,
            cell,
            RESET_CURRENT,
            RESET_TIME_CONSTANT,
            Milliseconds(1.0),
        );
    }
    sheet.text(
        "Press the sensors of two neurons in one letter, \
         and the rest of the letter lights up and stays lit. \
         Reset silences the grid.",
    );
    sheet.text(
        "Try one sensor alone, two from different letters, or the corners L and X share. \
         Select a neuron to ring the ones it connects to: \
         a left arrow means it sends to them, a right arrow that they send to it.",
    );
    sheet.text(if letters.len() > STORED.len() {
        "With five letters stored, the cues for T and L land on other letters \
         or on blends never stored. A grid of 25 neurons holds only about three patterns."
    } else {
        "Hopfield showed in 1982 that such connections make each stored pattern a resting place, \
         and the hippocampus may recall memories this way. \
         Here the connections are written in, not learned, \
         and the neurons fire far faster than real ones."
    });
    ConnectionView::Rings.set_in(&mut sheet.world);
    NeuronDisplay {
        voltage: true,
        firing_rate: true,
        ..Default::default()
    }
    .set_in(&mut sheet.world);
    Selected::set(&mut sheet.world, Some(cells[GRID * GRID - 3]));
    sheet.world
}

fn listed(letters: &[char]) -> String {
    let names: Vec<String> = letters.iter().map(char::to_string).collect();
    match names.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

pub const RECEPTORS: [&str; 5] = ["A", "B", "C", "D", "E"];
pub const PAIRS: [&str; 4] = ["A-B", "B-C", "C-D", "D-E"];
pub const OUTPUT: &str = "Output";
pub const PICTURE_SPEED: Hertz = Hertz(3.0);

const PICTURE_SIZE: f32 = 8.0;
const FIELD_SIZE: f32 = 4.0;
const FIELD_STEP: f32 = 5.0;
const RECEPTOR_DROP: f32 = 6.5;
const LAYER_DROP: f32 = 6.0;
const PARTNER_SHIFT: f32 = 1.0;
const RECEPTIVE_FIELD_SPACING: f64 = 0.15;
const RECEPTIVE_FIELD_SENSITIVITY: Hertz = Hertz(50_000.0);
const VETO_EXCITATION: Nanoamperes = Nanoamperes(1.5);
const VETO_INHIBITION: Nanoamperes = Nanoamperes(2.0);
const VETO_INHIBITION_TIME_CONSTANT: Milliseconds = Milliseconds(30.0);
const OUTPUT_EXCITATION: Nanoamperes = Nanoamperes(1.5);
const DETECTOR_WIDTH: f32 = 16.0;
const DETECTOR_HEIGHT: f32 = 12.0;
const DETECTOR_TIME_RANGE: Milliseconds = Milliseconds(1000.0);

fn direction_selectivity() -> World {
    let fast = Milliseconds(2.0);
    let mut sheet = Sheet::new();
    sheet.text(
        "A bar of light sweeps left to right across the picture. \
         Five receptive fields, A to E, each drive the receptor neuron below them.",
    );
    let picture_row = sheet.row(PICTURE_SIZE) - PICTURE_SIZE / 2.0;
    let picture = sheet.add(Element::VisualInput, SHEET_WIDTH / 2.0, picture_row);
    *sheet
        .world
        .get::<&mut VisualInput>(picture)
        .expect("visual inputs spawn with a VisualInput") = VisualInput {
        pattern: Pattern::Bar,
        speed: PICTURE_SPEED,
        ..Default::default()
    };
    let field_row = sheet.row(FIELD_SIZE) - FIELD_SIZE / 2.0;
    let receptor_row = field_row - RECEPTOR_DROP;
    let partner_row = receptor_row - LAYER_DROP;
    let pair_row = partner_row - LAYER_DROP;
    let output_row = pair_row - LAYER_DROP;
    sheet.row(field_row - output_row - SHEET_GAP);
    let middle = (RECEPTORS.len() - 1) as f32 / 2.0;
    let column = |index: usize| SHEET_WIDTH / 2.0 + (index as f32 - middle) * FIELD_STEP;
    let mut receptors = Vec::new();
    let mut partners = Vec::new();
    for (index, name) in RECEPTORS.into_iter().enumerate() {
        let field = sheet.add(Element::ReceptiveField, column(index), field_row);
        *sheet
            .world
            .get::<&mut ReceptiveFieldCell>(field)
            .expect("receptive fields spawn with a ReceptiveFieldCell") = ReceptiveFieldCell {
            field: ReceptiveField::CenterSurround {
                off_center: false,
                x: (index as f64 - middle as f64) * RECEPTIVE_FIELD_SPACING,
                y: 0.0,
            },
            sensitivity: RECEPTIVE_FIELD_SENSITIVITY,
            ..Default::default()
        };
        spawn_picture_wire(&mut sheet.world, picture, field);
        let receptor = sheet.neuron(NeuronType::Excitatory, column(index), receptor_row, name);
        let partner = sheet.neuron(
            NeuronType::Inhibitory,
            column(index) - PARTNER_SHIFT,
            partner_row,
            "",
        );
        sheet.synapse(field, receptor, RELAY, fast, Milliseconds::ZERO);
        sheet.synapse(receptor, partner, RELAY, fast, Milliseconds(1.0));
        receptors.push(receptor);
        partners.push(partner);
    }
    let output = sheet.neuron(
        NeuronType::Excitatory,
        SHEET_WIDTH / 2.0,
        output_row,
        OUTPUT,
    );
    let mut rightward = Vec::new();
    for (index, pair) in PAIRS.into_iter().enumerate() {
        let between = column(index) + FIELD_STEP / 2.0;
        let right = sheet.neuron(NeuronType::Excitatory, between, pair_row, pair);
        sheet.synapse(
            receptors[index],
            right,
            VETO_EXCITATION,
            fast,
            Milliseconds(1.0),
        );
        sheet.synapse(
            partners[index + 1],
            right,
            VETO_INHIBITION,
            VETO_INHIBITION_TIME_CONSTANT,
            Milliseconds(1.0),
        );
        sheet.synapse(right, output, OUTPUT_EXCITATION, fast, Milliseconds(1.0));
        rightward.push(right);
    }
    let detector_row = sheet.row(DETECTOR_HEIGHT) - DETECTOR_HEIGHT / 2.0;
    let detector = Meter::SpikeDetector.spawn_measuring(
        &mut sheet.world,
        receptors[0],
        Sheet::at((SHEET_WIDTH - DETECTOR_WIDTH) / 2.0, detector_row),
    );
    for &cell in receptors[1..].iter().chain(&rightward).chain([&output]) {
        Meter::SpikeDetector.wire(&mut sheet.world, cell, detector);
    }
    *sheet
        .world
        .get::<&mut MeterSize>(detector)
        .expect("meters spawn with a MeterSize") = MeterSize {
        width: DETECTOR_WIDTH,
        height: DETECTOR_HEIGHT,
    };
    *sheet
        .world
        .get::<&mut RowOrder>(detector)
        .expect("spike detectors spawn with a RowOrder") = RowOrder::Connection;
    sheet
        .world
        .get::<&mut SpikeDetector>(detector)
        .expect("spike detectors spawn with a SpikeDetector")
        .time_range = DETECTOR_TIME_RANGE;
    sheet.text(
        "A-B to D-E are each excited by their first receptor \
         and inhibited by the partner of their second. \
         Moving right, the bar excites them before the inhibition arrives, and the output fires. \
         Tap the arrow on the picture: now the inhibition comes first and vetoes them. \
         At 1 Hz the inhibition fades before the excitation comes.",
    );
    sheet.text(
        "Barlow and Levick found this veto in the rabbit retina in 1965, \
         repeated across small subunits of one cell. \
         There the subunits are dendrite branches, inhibited by starburst amacrine cells. \
         Here each one is a neuron, and Output stands for the whole cell.",
    );
    sheet.world
}
