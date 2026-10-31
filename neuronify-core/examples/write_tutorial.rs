use std::path::Path;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::elements::{spawn_synapse, spawn_voltmeter, Element};
use neuronify_core::rendering::notes::{NOTE_FONT_DATA, NOTE_MARGIN, NOTE_TEXT_HEIGHT};
use neuronify_core::serialization;
use neuronify_core::{
    Annotation, Connection, CurrentClamp, CurrentSynapse, LeakyDynamics, Link, NoteSize, Tool,
    VoltmeterSize, DEFAULT_SPEED,
};

const NANOAMPERE: f64 = 1e-9;
const MILLIVOLT: f64 = 1e-3;
const MILLISECOND: f64 = 1e-3;
const CURRENT_DRAWN_AT_FULL_STRENGTH: f64 = 6.0 * NANOAMPERE;

const WIDTH: f32 = 24.0;
const GAP: f32 = 1.2;
const LINK_HEIGHT: f32 = 2.0;

fn choose(tool: Tool) -> String {
    format!("Choose {}", tool.name())
}

fn main() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tutorial");
    std::fs::create_dir_all(&directory).unwrap();
    for (file, world) in [
        ("1_spikes.neuronify", spikes()),
        ("2_leaks.neuronify", leaks()),
        ("3_push_and_pull.neuronify", push_and_pull()),
        ("4_choosing.neuronify", choosing()),
        ("5_sharpening_edges.neuronify", sharpening_edges()),
        ("6_keeping_a_rhythm.neuronify", keeping_a_rhythm()),
    ] {
        let path = directory.join(file);
        std::fs::write(
            &path,
            serialization::save(&world, 0.0, DEFAULT_SPEED).unwrap(),
        )
        .unwrap();
        println!("Wrote {}", path.display());
    }
}

struct Canvas {
    world: World,
    top: f32,
}

impl Canvas {
    fn new() -> Canvas {
        Canvas {
            world: World::new(),
            top: 0.0,
        }
    }

    fn at(right: f32, up: f32) -> Vec3 {
        Vec3::new(up, 0.0, right)
    }

    fn note_size(text: &str, width: f32) -> NoteSize {
        let wrapped = visula::text::tessellate_text_wrapped(
            NOTE_FONT_DATA,
            text,
            NOTE_TEXT_HEIGHT,
            width - 2.0 * NOTE_MARGIN,
        )
        .unwrap();
        NoteSize {
            width,
            height: wrapped.height + 2.0 * NOTE_MARGIN,
        }
    }

    fn note_at(&mut self, text: &str, right: f32, up: f32, width: f32) -> Entity {
        let note = Element::Note.spawn(&mut self.world, Canvas::at(right, up));
        let size = Canvas::note_size(text, width);
        self.world.get::<&mut Annotation>(note).unwrap().text = text.to_string();
        *self.world.get::<&mut NoteSize>(note).unwrap() = size;
        note
    }

    fn text(&mut self, text: &str) {
        let height = Canvas::note_size(text, WIDTH).height;
        self.note_at(text, WIDTH / 2.0, self.top - height / 2.0, WIDTH);
        self.top -= height + GAP;
    }

    fn row(&mut self, height: f32) -> f32 {
        let middle = self.top - height / 2.0;
        self.top -= height + GAP;
        middle
    }

    fn link(&mut self, label: &str, example: &str) {
        let width = visula::text::measure_text_width(NOTE_FONT_DATA, label, NOTE_TEXT_HEIGHT)
            .unwrap()
            + 3.0 * NOTE_MARGIN;
        let note = self.note_at(
            label,
            WIDTH - width / 2.0,
            self.top - LINK_HEIGHT / 2.0,
            width,
        );
        *self.world.get::<&mut NoteSize>(note).unwrap() = NoteSize {
            width,
            height: LINK_HEIGHT,
        };
        self.world
            .insert_one(
                note,
                Link {
                    example: example.to_string(),
                },
            )
            .unwrap();
        self.top -= LINK_HEIGHT + GAP;
    }

    fn next(&mut self, name: &str, file: &str) {
        self.link(&format!("Next: {name} ›"), file);
    }

    fn add(&mut self, element: Element, right: f32, up: f32) -> Entity {
        element.spawn(&mut self.world, Canvas::at(right, up))
    }

    fn connect(&mut self, from: Entity, to: Entity) -> Entity {
        spawn_synapse(&mut self.world, from, to)
    }

    fn synapse(&mut self, from: Entity, to: Entity, maximum_current: f64, tau: f64) -> Entity {
        let synapse = self.connect(from, to);
        let mut current = self.world.get::<&mut CurrentSynapse>(synapse).unwrap();
        current.maximum_current = maximum_current;
        current.tau = tau;
        let charge = maximum_current * tau / (CURRENT_DRAWN_AT_FULL_STRENGTH * 2.0 * MILLISECOND);
        self.world.get::<&mut Connection>(synapse).unwrap().strength = charge.clamp(0.4, 1.5);
        synapse
    }

    fn delay(&mut self, synapse: Entity, delay: f64) {
        self.world
            .get::<&mut CurrentSynapse>(synapse)
            .unwrap()
            .delay = delay;
    }

    fn current(&mut self, to: Entity, current: f64, right: f32, up: f32) {
        let source = self.add(Element::CurrentSource, right, up);
        self.world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = current;
        self.connect(source, to);
    }

    fn voltmeter(&mut self, target: Entity, left: f32, up: f32, size: VoltmeterSize) {
        let voltmeter = spawn_voltmeter(&mut self.world, target, Canvas::at(left, up));
        *self.world.get::<&mut VoltmeterSize>(voltmeter).unwrap() = size;
    }

    fn start_at(&mut self, neuron: Entity, voltage: f64) {
        self.world
            .get::<&mut LeakyDynamics>(neuron)
            .unwrap()
            .voltage = voltage;
    }
}

fn spikes() -> World {
    let mut canvas = Canvas::new();
    canvas.text("Press the hand.");
    let up = canvas.row(2.4);
    let hand = canvas.add(Element::TouchSensor, 2.0, up);
    let first = canvas.add(Element::ExcitatoryNeuron, 12.0, up);
    let second = canvas.add(Element::ExcitatoryNeuron, 21.0, up);
    canvas.connect(hand, first);
    canvas.connect(first, second);
    canvas.text(
        "This is a neuron. It will fire spikes when stimulated. \
         Those are signals that can travel along the axon to other neurons to stimulate them too.",
    );
    canvas.text(
        "The speed is of course much slower here. \
         In your brain, everything is about 100 times faster than this.",
    );
    canvas.next("Leak currents", "tutorial/2_leaks.neuronify");
    canvas.world
}

fn leaks() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "Press the hand once. The neuron charges potential. This is shown as a brighter color. \
         When it gets darker the charge leaks away. \
         For this neuron, one press is not enough to make it fire.",
    );
    let up = canvas.row(5.0);
    let hand = canvas.add(Element::TouchSensor, 1.5, up);
    let neuron = canvas.add(Element::ExcitatoryNeuron, 7.5, up);
    canvas.synapse(hand, neuron, 1.0 * NANOAMPERE, 2.0 * MILLISECOND);
    canvas.voltmeter(
        neuron,
        11.0,
        up,
        VoltmeterSize {
            width: 13.0,
            height: 5.0,
        },
    );
    canvas.text(
        "Try pressing quickly. That will make the incoming current larger than the leak current \
         and the neuron fires.",
    );
    canvas.text(
        "The voltmeter shows the neuron's membrane potential. \
         It rests at −70 mV and fires a spike at −55 mV. \
         When it fires, it will reset below the resting potential.",
    );
    canvas.text(
        "A real spike lasts for about a millisecond, but in this model it lasts only for a moment.",
    );
    canvas.next("Push and pull", "tutorial/3_push_and_pull.neuronify");
    canvas.world
}

fn push_and_pull() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "The current source feeds the middle neuron a steady current to make it fire on its own.",
    );
    let up = canvas.row(12.0);
    let middle = canvas.add(Element::ExcitatoryNeuron, 11.0, up);
    canvas.current(middle, 0.3 * NANOAMPERE, 11.0, up - 4.5);
    let push_hand = canvas.add(Element::TouchSensor, 1.0, up + 4.0);
    let push = canvas.add(Element::ExcitatoryNeuron, 6.0, up + 4.0);
    let pull_hand = canvas.add(Element::TouchSensor, 1.0, up - 4.0);
    let pull = canvas.add(Element::InhibitoryNeuron, 6.0, up - 4.0);
    canvas.connect(push_hand, push);
    canvas.connect(pull_hand, pull);
    canvas.connect(push, middle);
    canvas.connect(pull, middle);
    canvas.voltmeter(
        middle,
        14.0,
        up,
        VoltmeterSize {
            width: 10.0,
            height: 5.0,
        },
    );
    canvas.text(
        "Touching the sensor connected to the blue, excitatory neuron \
         will make the middle one fire more often. The red, inhibitory one will make it fire less.",
    );
    canvas.next("Choosing sides", "tutorial/4_choosing.neuronify");
    canvas.world
}

fn choosing() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "The inhibitory neurons pull the excitatory neuron on each side down. \
         Only one side can keep firing.",
    );
    let up = canvas.row(13.0) + 2.0;
    let left = canvas.add(Element::ExcitatoryNeuron, 4.5, up + 2.5);
    let right = canvas.add(Element::ExcitatoryNeuron, 19.5, up + 2.5);
    let left_partner = canvas.add(Element::InhibitoryNeuron, 4.5, up - 2.5);
    let right_partner = canvas.add(Element::InhibitoryNeuron, 19.5, up - 2.5);
    canvas.current(left, 0.5 * NANOAMPERE, 0.0, up + 2.5);
    canvas.current(right, 0.5 * NANOAMPERE, 24.0, up + 2.5);
    let cross = [
        canvas.connect(left, left_partner),
        canvas.connect(right, right_partner),
        canvas.synapse(left_partner, right, 3.0 * NANOAMPERE, 10.0 * MILLISECOND),
        canvas.synapse(right_partner, left, 3.0 * NANOAMPERE, 10.0 * MILLISECOND),
    ];
    for synapse in cross {
        canvas.delay(synapse, MILLISECOND);
    }
    let left_hand = canvas.add(Element::TouchSensor, 4.5, up - 7.0);
    let right_hand = canvas.add(Element::TouchSensor, 19.5, up - 7.0);
    canvas.synapse(
        left_hand,
        left_partner,
        3.0 * NANOAMPERE,
        10.0 * MILLISECOND,
    );
    canvas.synapse(
        right_hand,
        right_partner,
        3.0 * NANOAMPERE,
        10.0 * MILLISECOND,
    );
    canvas.start_at(left, -56.0 * MILLIVOLT);
    canvas.start_at(right, -80.0 * MILLIVOLT);
    canvas.text(
        "If you press the sensor on either side, \
         its inhibitory neuron will pull the other side down.",
    );
    canvas.text("This is called reciprocal inhibition.");
    canvas.next("Sharpening edges", "tutorial/5_sharpening_edges.neuronify");
    canvas.world
}

const DETECTORS: usize = 6;
const BRIGHT: f64 = 0.7 * NANOAMPERE;
const DIM: f64 = 0.35 * NANOAMPERE;
const SIDEWAYS_PULL: f64 = 1.0 * NANOAMPERE;

fn sharpening_edges() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "Six detectors in a row, like cells in your eye. \
         Light is bright on the left and dim on the right, \
         and each detector's inhibitory neuron pulls on its neighbours.",
    );
    let labels = canvas.row(2.0);
    canvas.note_at("Bright", 6.0, labels, 10.0);
    canvas.note_at("Dim", 18.0, labels, 10.0);
    let up = canvas.row(12.0);
    let spacing = WIDTH / DETECTORS as f32;
    let right_of = |index: usize| spacing * (index as f32 + 0.5);
    let detectors: Vec<Entity> = (0..DETECTORS)
        .map(|index| canvas.add(Element::ExcitatoryNeuron, right_of(index), up))
        .collect();
    let partners: Vec<Entity> = (0..DETECTORS)
        .map(|index| canvas.add(Element::InhibitoryNeuron, right_of(index), up - 4.5))
        .collect();
    for (index, (&detector, &partner)) in detectors.iter().zip(&partners).enumerate() {
        let light = if index < DETECTORS / 2 { BRIGHT } else { DIM };
        canvas.current(detector, light, right_of(index), up + 4.5);
        canvas.connect(detector, partner);
        let neighbours: Vec<usize> = [index.wrapping_sub(1), index + 1]
            .into_iter()
            .filter(|&neighbour| neighbour < DETECTORS)
            .collect();
        for &neighbour in &neighbours {
            let at_the_end = neighbour == 0 || neighbour == DETECTORS - 1;
            let pull = if at_the_end { 2.0 } else { 1.0 } * SIDEWAYS_PULL;
            canvas.synapse(partner, detectors[neighbour], pull, 2.0 * MILLISECOND);
        }
    }
    canvas.text(&format!(
        "{} and drag it along the red row. The edge fades into the rest. \
         Undo (↶) brings it back.",
        choose(Tool::Erase)
    ));
    canvas.text(
        "This sideways pull is called lateral inhibition and is found in for instance the retina.",
    );
    canvas.next("Keeping a rhythm", "tutorial/6_keeping_a_rhythm.neuronify");
    canvas.world
}

fn keeping_a_rhythm() -> World {
    let mut canvas = Canvas::new();
    canvas.text("These four neurons form a ring, but the last connection is missing.");
    let up = canvas.row(9.0);
    let ring = [
        canvas.add(Element::ExcitatoryNeuron, 9.0, up + 3.5),
        canvas.add(Element::ExcitatoryNeuron, 17.0, up + 3.5),
        canvas.add(Element::ExcitatoryNeuron, 17.0, up - 3.5),
        canvas.add(Element::ExcitatoryNeuron, 9.0, up - 3.5),
    ];
    for pair in ring.windows(2) {
        canvas.connect(pair[0], pair[1]);
    }
    let hand = canvas.add(Element::TouchSensor, 2.0, up + 3.5);
    canvas.connect(hand, ring[0]);
    canvas.text("Add a connection and press the hand.");
    canvas.text("A loop is one of the simplest ways a brain keeps something going.");
    canvas.link("More examples ›", "examples");
    canvas.world
}
