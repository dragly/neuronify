use std::path::Path;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_core::dendrites::{compartment_tree, spawn_dendrites};
use neuronify_core::elements::{
    spawn_cable, spawn_compartment, spawn_internode, spawn_synapse, spawn_voltmeter, Element,
};
use neuronify_core::rendering::notes::{NOTE_FONT_DATA, NOTE_MARGIN, NOTE_TEXT_HEIGHT};
use neuronify_core::serialization;
use neuronify_core::{
    Annotation, Connection, CurrentClamp, CurrentSynapse, Link, MeterSize, Milliseconds,
    Millivolts, Nanoamperes, NeuronDisplay, NeuronType, NodeState, NoteSize, Position, Setting,
    Tool, ToolGroup, DEFAULT_SPEED, INTERNODE_LENGTH, SEGMENT_LENGTH,
};

const CURRENT_DRAWN_AT_FULL_STRENGTH: Nanoamperes = Nanoamperes(6.0);

const WIDTH: f32 = 24.0;
const GAP: f32 = 1.2;
const LINK_HEIGHT: f32 = 2.0;

fn choose(tool: Tool) -> String {
    format!("Choose {}", tool.name())
}

fn choose_in(tool: Tool, group: ToolGroup) -> String {
    format!("Choose {} under {}", tool.name(), group.label())
}

pub fn canvases() -> Vec<(&'static str, World)> {
    vec![
        ("1_spikes.neuronify", spikes()),
        ("2_leaks.neuronify", leaks()),
        ("3_push_and_pull.neuronify", push_and_pull()),
        ("4_choosing.neuronify", choosing()),
        ("5_sharpening_edges.neuronify", sharpening_edges()),
        ("6_keeping_a_rhythm.neuronify", keeping_a_rhythm()),
        ("7_axons.neuronify", axons()),
        ("8_myelin.neuronify", myelin()),
        ("9_dendrites.neuronify", dendrites()),
        ("10_coincidence.neuronify", coincidence()),
        (
            "11_inhibition_on_a_branch.neuronify",
            inhibition_on_a_branch(),
        ),
    ]
}

fn main() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tutorial");
    std::fs::create_dir_all(&directory).unwrap();
    for (file, world) in canvases() {
        let path = directory.join(file);
        std::fs::write(
            &path,
            serialization::save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap(),
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

    fn synapse(
        &mut self,
        from: Entity,
        to: Entity,
        maximum_current: Nanoamperes,
        tau: Milliseconds,
    ) -> Entity {
        let synapse = self.connect(from, to);
        let mut current = self.world.get::<&mut CurrentSynapse>(synapse).unwrap();
        current.maximum_current = maximum_current;
        current.tau = tau;
        let charge = maximum_current * tau / (CURRENT_DRAWN_AT_FULL_STRENGTH * Milliseconds(2.0));
        self.world.get::<&mut Connection>(synapse).unwrap().strength = charge.clamp(0.4, 1.5);
        synapse
    }

    fn delay(&mut self, synapse: Entity, delay: Milliseconds) {
        self.world
            .get::<&mut CurrentSynapse>(synapse)
            .unwrap()
            .delay = delay;
    }

    fn current(&mut self, to: Entity, current: Nanoamperes, right: f32, up: f32) {
        let source = self.add(Element::CurrentSource, right, up);
        self.world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = current;
        self.connect(source, to);
    }

    fn voltmeter(&mut self, target: Entity, left: f32, up: f32, size: MeterSize) {
        let voltmeter = spawn_voltmeter(&mut self.world, target, Canvas::at(left, up));
        *self.world.get::<&mut MeterSize>(voltmeter).unwrap() = size;
    }

    fn axon(&mut self, from: Entity, waypoints: &[(f32, f32)], to: Entity) -> Entity {
        let position =
            |world: &World, entity: Entity| world.get::<&Position>(entity).unwrap().position;
        let neuron_type = *self.world.get::<&NeuronType>(from).unwrap();
        let mut points = vec![position(&self.world, from)];
        points.extend(waypoints.iter().map(|&(right, up)| Canvas::at(right, up)));
        points.push(position(&self.world, to));
        let mut along: Vec<Vec3> = Vec::new();
        for pair in points.windows(2) {
            let pieces = (pair[0].distance(pair[1]) / SEGMENT_LENGTH)
                .round()
                .max(1.0) as usize;
            along.extend(
                (1..=pieces).map(|piece| pair[0].lerp(pair[1], piece as f32 / pieces as f32)),
            );
        }
        along.pop();
        let mut previous = from;
        for point in along {
            let piece = spawn_compartment(&mut self.world, point, neuron_type);
            spawn_cable(&mut self.world, previous, piece, false);
            previous = piece;
        }
        spawn_cable(&mut self.world, previous, to, true)
    }

    fn myelinated_axon(&mut self, from: Entity, to: Entity) -> Entity {
        let position =
            |world: &World, entity: Entity| world.get::<&Position>(entity).unwrap().position;
        let neuron_type = *self.world.get::<&NeuronType>(from).unwrap();
        let start = position(&self.world, from);
        let end = position(&self.world, to);
        let internodes = ((start.distance(end) - SEGMENT_LENGTH) / INTERNODE_LENGTH).floor();
        let mut previous = from;
        for internode in 1..=internodes as usize {
            let along = (end - start).normalize() * internode as f32 * INTERNODE_LENGTH;
            let node = spawn_compartment(&mut self.world, start + along, neuron_type);
            spawn_internode(&mut self.world, previous, node);
            previous = node;
        }
        self.axon(previous, &[], to)
    }

    fn weak_axon(&mut self, from: Entity, to: Entity, strength: f64) {
        let ending = self.axon(from, &[], to);
        self.world.get::<&mut Connection>(ending).unwrap().strength = strength;
    }

    fn branch_near(&self, neuron: Entity, right: f32, up: f32) -> Entity {
        let target = Canvas::at(right, up);
        let distance = |entity: &Entity| {
            self.world
                .get::<&Position>(*entity)
                .unwrap()
                .position
                .distance(target)
        };
        compartment_tree(&self.world, neuron)
            .into_iter()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .unwrap()
    }

    fn start_at(&mut self, neuron: Entity, voltage: Millivolts) {
        self.world.get::<&mut NodeState>(neuron).unwrap().voltage = voltage;
    }
}

fn spikes() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "The blue spheres are excitatory neurons. They will fire spikes when stimulated, \
         signals that can travel along the connection to other neurons to stimulate them too.",
    );
    let up = canvas.row(2.4);
    let hand = canvas.add(Element::TouchSensor, 2.0, up);
    let first = canvas.add(Element::ExcitatoryNeuron, 12.0, up);
    let second = canvas.add(Element::ExcitatoryNeuron, 21.0, up);
    canvas.connect(hand, first);
    canvas.connect(first, second);
    canvas.text("Press the touch sensor to trigger a spike.");
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
        "Press the touch sensor once. The neuron charges potential. \
         This is shown as a brighter color. When it gets darker the charge leaks away. \
         For this neuron, one press is not enough to make it fire.",
    );
    let up = canvas.row(5.0);
    let hand = canvas.add(Element::TouchSensor, 1.5, up);
    let neuron = canvas.add(Element::ExcitatoryNeuron, 7.5, up);
    canvas.synapse(hand, neuron, Nanoamperes(1.0), Milliseconds(2.0));
    canvas.voltmeter(
        neuron,
        11.0,
        up,
        MeterSize {
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
    let up = canvas.row(15.0) + 1.5;
    let middle = canvas.add(Element::ExcitatoryNeuron, 11.0, up);
    canvas.current(middle, Nanoamperes(0.3), 11.0, up - 7.0);
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
        MeterSize {
            width: 10.0,
            height: 5.0,
        },
    );
    canvas.text(
        "Pressing the touch sensor connected to the blue, excitatory neuron \
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
    canvas.current(left, Nanoamperes(0.5), 0.0, up + 2.5);
    canvas.current(right, Nanoamperes(0.5), 24.0, up + 2.5);
    let cross = [
        canvas.connect(left, left_partner),
        canvas.connect(right, right_partner),
        canvas.synapse(left_partner, right, Nanoamperes(3.0), Milliseconds(10.0)),
        canvas.synapse(right_partner, left, Nanoamperes(3.0), Milliseconds(10.0)),
    ];
    for synapse in cross {
        canvas.delay(synapse, Milliseconds(1.0));
    }
    let left_hand = canvas.add(Element::TouchSensor, 4.5, up - 7.0);
    let right_hand = canvas.add(Element::TouchSensor, 19.5, up - 7.0);
    canvas.synapse(
        left_hand,
        left_partner,
        Nanoamperes(3.0),
        Milliseconds(10.0),
    );
    canvas.synapse(
        right_hand,
        right_partner,
        Nanoamperes(3.0),
        Milliseconds(10.0),
    );
    canvas.start_at(left, Millivolts(-56.0));
    canvas.start_at(right, Millivolts(-80.0));
    canvas.text(
        "If you press the touch sensor on either side, \
         its inhibitory neuron will pull the other side down.",
    );
    canvas.text("This is called reciprocal inhibition.");
    canvas.next("Sharpening edges", "tutorial/5_sharpening_edges.neuronify");
    canvas.world
}

const DETECTORS: usize = 6;
const BRIGHT: Nanoamperes = Nanoamperes(0.7);
const DIM: Nanoamperes = Nanoamperes(0.35);
const SIDEWAYS_PULL: Nanoamperes = Nanoamperes(1.0);

fn sharpening_edges() -> World {
    let mut canvas = Canvas::new();
    NeuronDisplay {
        firing_rate: true,
        ..Default::default()
    }
    .set_in(&mut canvas.world);
    canvas.text(
        "Six detectors in a row, like cells in your eye. \
         Light is bright on the left and dim on the right, \
         and each detector's inhibitory neuron pulls on its neighbors.",
    );
    canvas.text(
        "The firing rate of each neuron is shown as a glow below it. \
         Purple means low firing rate and yellow means high firing rate. \
         Visualizing firing rate can be toggled on or off. \
         In this simulation, you can see that the last bright detector glows the most \
         and the first dim one the least, meaning we have enhanced contrast.",
    );
    let labels = canvas.row(2.0);
    canvas.note_at("Bright", 6.0, labels, 10.0);
    canvas.note_at("Dim", 18.0, labels, 10.0);
    let up = canvas.row(15.0) - 1.25;
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
        canvas.current(detector, light, right_of(index), up + 7.0);
        canvas.connect(detector, partner);
        let neighbours: Vec<usize> = [index.wrapping_sub(1), index + 1]
            .into_iter()
            .filter(|&neighbour| neighbour < DETECTORS)
            .collect();
        for &neighbour in &neighbours {
            let at_the_end = neighbour == 0 || neighbour == DETECTORS - 1;
            let pull = if at_the_end { 2.0 } else { 1.0 } * SIDEWAYS_PULL;
            canvas.synapse(partner, detectors[neighbour], pull, Milliseconds(2.0));
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
    canvas.text("Add a connection and press the touch sensor.");
    canvas.text("A loop is one of the simplest ways a brain keeps something going.");
    canvas.next("Axons", "tutorial/7_axons.neuronify");
    canvas.world
}

fn axons() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "In real neurons, spikes travel as action potentials along an axon. \
         That can be modeled as a long thin cable.",
    );
    let up = canvas.row(15.0) + 2.0;
    let hand = canvas.add(Element::TouchSensor, 1.0, up + 2.0);
    let sender = canvas.add(Element::ExcitatoryNeuron, 6.0, up + 2.0);
    let near = canvas.add(Element::ExcitatoryNeuron, 13.0, up + 5.0);
    let far = canvas.add(Element::ExcitatoryNeuron, 22.0, up + 5.0);
    canvas.connect(hand, sender);
    canvas.axon(sender, &[], near);
    canvas.axon(
        sender,
        &[(8.0, up - 5.0), (14.0, up - 8.0), (20.0, up - 4.0)],
        far,
    );
    canvas.text(
        "Press the touch sensor to see how the action potentials travel along each axon. \
         See how the distance affects the timing.",
    );
    canvas.next("Myelination", "tutorial/8_myelin.neuronify");
    canvas.world
}

fn myelin() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "Many axons are wrapped in myelin, a fatty sheath with small gaps along it. \
         Myelination makes spikes jump from gap to gap instead of creeping along the whole cable.",
    );
    let up = canvas.row(28.0);
    let hand = canvas.add(Element::TouchSensor, 8.0, up + 12.5);
    let sender = canvas.add(Element::ExcitatoryNeuron, 12.0, up + 12.5);
    let plain = canvas.add(Element::ExcitatoryNeuron, 5.5, up - 13.2);
    let myelinated = canvas.add(Element::ExcitatoryNeuron, 18.5, up - 13.2);
    canvas.connect(hand, sender);
    canvas.axon(sender, &[], plain);
    canvas.myelinated_axon(sender, myelinated);
    canvas.text(
        "Press the touch sensor. The two axons are the same length, \
         but the spike on the myelinated one reaches its neuron first.",
    );
    canvas.next("Dendrites", "tutorial/9_dendrites.neuronify");
    canvas.world
}

fn dendrites() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "Dendrites in the right neuron can receive signals from the left neuron, \
         but you need to draw an axon first.",
    );
    let up = canvas.row(25.0);
    let hand = canvas.add(Element::TouchSensor, 1.0, up + 2.0);
    let sender = canvas.add(Element::ExcitatoryNeuron, 5.0, up + 2.0);
    canvas.connect(hand, sender);
    let receiver = canvas.add(Element::ExcitatoryNeuron, 17.0, up - 4.5);
    spawn_dendrites(&mut canvas.world, receiver);
    canvas.text(&format!(
        "{}, then drag from the left neuron onto one of the branches. \
         Now press the touch sensor: the spike travels from the axon to the dendrites, \
         runs to the cell body and makes it fire.",
        choose_in(Tool::Axon, ToolGroup::Connections)
    ));
    canvas.next("Coincidence", "tutorial/10_coincidence.neuronify");
    canvas.world
}

fn coincidence() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "The synaptic connections from these two axons are weak. \
         One alone will not trigger the rightmost neuron to fire.",
    );
    let up = canvas.row(25.0);
    let receiver = canvas.add(Element::ExcitatoryNeuron, 18.0, up - 4.5);
    spawn_dendrites(&mut canvas.world, receiver);
    let branch = canvas.branch_near(receiver, 12.0, up);
    let meeting = canvas.world.get::<&Position>(branch).unwrap().position;
    let (right, up) = (meeting.z, meeting.x);
    let upper = canvas.add(Element::ExcitatoryNeuron, right - 6.0, up + 4.5);
    let lower = canvas.add(Element::ExcitatoryNeuron, right - 6.0, up - 4.5);
    canvas.weak_axon(upper, branch, 0.25);
    canvas.weak_axon(lower, branch, 0.25);
    let top = canvas.add(Element::TouchSensor, 1.0, up + 4.5);
    let middle = canvas.add(Element::TouchSensor, 1.0, up);
    let bottom = canvas.add(Element::TouchSensor, 1.0, up - 4.5);
    canvas.connect(top, upper);
    canvas.connect(middle, upper);
    canvas.connect(middle, lower);
    canvas.connect(bottom, lower);
    canvas.text(
        "Pressing the top or bottom touch sensors will not fire the rightmost neuron, \
         while pressing the middle will. The two spikes arrive close in time and add up.",
    );
    canvas.text(
        "A neuron that fires only when its inputs arrive together is called \
         a coincidence detector.",
    );
    canvas.next(
        "Inhibition on a branch",
        "tutorial/11_inhibition_on_a_branch.neuronify",
    );
    canvas.world
}

fn inhibition_on_a_branch() -> World {
    let mut canvas = Canvas::new();
    canvas.text(
        "The neuron in the middle is inhibitory and fires all the time. \
         Its red axon ends partway down the dendrites, \
         between the top branches and the cell body.",
    );
    let up = canvas.row(25.0);
    let receiver = canvas.add(Element::ExcitatoryNeuron, 18.0, up - 4.5);
    spawn_dendrites(&mut canvas.world, receiver);
    let tuft = canvas.branch_near(receiver, 15.5, up + 11.0);
    let trunk = canvas.branch_near(receiver, 18.0, up + 3.5);
    let basal = canvas.branch_near(receiver, 12.0, up);
    let position = |canvas: &Canvas, entity: Entity| {
        let position = canvas.world.get::<&Position>(entity).unwrap().position;
        (position.z, position.x)
    };
    let (tuft_right, tuft_up) = position(&canvas, tuft);
    let (basal_right, basal_up) = position(&canvas, basal);
    let upper = canvas.add(Element::ExcitatoryNeuron, tuft_right - 7.0, tuft_up);
    let lower = canvas.add(Element::ExcitatoryNeuron, basal_right - 6.0, basal_up);
    canvas.axon(upper, &[], tuft);
    canvas.axon(lower, &[], basal);
    let top = canvas.add(Element::TouchSensor, 1.0, tuft_up);
    let bottom = canvas.add(Element::TouchSensor, 1.0, basal_up);
    canvas.connect(top, upper);
    canvas.connect(bottom, lower);
    let (_, trunk_up) = position(&canvas, trunk);
    let inhibitor = canvas.add(Element::InhibitoryNeuron, 10.0, trunk_up - 1.0);
    canvas.axon(inhibitor, &[], trunk);
    canvas.current(inhibitor, Nanoamperes(0.6), 4.0, trunk_up - 1.0);
    canvas.text(
        "Press the top touch sensor: the spike runs down the dendrites \
         but is stopped where the red axon ends. \
         Press the bottom one: that spike takes another way and makes the neuron fire.",
    );
    canvas.text(
        "Inhibition on a dendrite works locally. \
         It silences the inputs beyond it and leaves the other branches alone.",
    );
    canvas.link("More examples ›", "examples");
    canvas.world
}
