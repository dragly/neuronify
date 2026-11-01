use glam::Vec3;
use hecs::{Entity, World};

use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::{
    spawn_loudspeaker, spawn_rate_plot, spawn_spike_detector, spawn_spike_detector_wire,
    spawn_synapse, spawn_voltmeter, Element,
};
use neuronify_core::measurement::firing_rate::RatePlot;
use neuronify_core::measurement::visual_input::{Pattern, ReceptiveField, VisualInput};
use neuronify_core::rendering::notes::{NOTE_FONT_DATA, NOTE_MARGIN, NOTE_TEXT_HEIGHT};
use neuronify_core::{
    AdaptationCurrent, Annotation, Connection, CurrentClamp, CurrentSynapse, Label, LeakyDynamics,
    MeterSize, NeuronDisplay, NeuronType, NoteSize, PoissonGenerator, RegularSpikeGenerator,
    Setting,
};

const NANOAMPERE: f64 = 1e-9;
const MILLISECOND: f64 = 1e-3;
const CURRENT_DRAWN_AT_FULL_STRENGTH: f64 = 6.0 * NANOAMPERE;
const WIDTH: f32 = 24.0;
const GAP: f32 = 1.2;

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
            WIDTH - 2.0 * NOTE_MARGIN,
        )
        .unwrap();
        let height = wrapped.height + 2.0 * NOTE_MARGIN;
        note(
            &mut self.world,
            Sheet::at(WIDTH / 2.0, self.top - height / 2.0),
            WIDTH,
            height,
            text,
        );
        self.top -= height + GAP;
    }

    fn row(&mut self, height: f32) -> f32 {
        let middle = self.top - height / 2.0;
        self.top -= height + GAP;
        middle
    }

    fn add(&mut self, element: Element, right: f32, up: f32) -> Entity {
        element.spawn(&mut self.world, Sheet::at(right, up))
    }

    fn label(&mut self, entity: Entity, text: &str) {
        label(&mut self.world, entity, text);
    }

    fn inhibitory(&mut self, neuron: Entity) {
        *self.world.get::<&mut NeuronType>(neuron).unwrap() = NeuronType::Inhibitory;
    }

    fn lasting_inhibition(&mut self, from: Entity, to: Entity) -> Entity {
        let synapse = self.synapse(from, to, 3.0 * NANOAMPERE);
        let mut current = self.world.get::<&mut CurrentSynapse>(synapse).unwrap();
        current.tau = 10.0 * MILLISECOND;
        current.delay = MILLISECOND;
        drop(current);
        synapse
    }

    fn connect(&mut self, from: Entity, to: Entity) -> Entity {
        spawn_synapse(&mut self.world, from, to)
    }

    fn synapse(&mut self, from: Entity, to: Entity, maximum_current: f64) -> Entity {
        let synapse = self.connect(from, to);
        let mut current = self.world.get::<&mut CurrentSynapse>(synapse).unwrap();
        current.maximum_current = maximum_current;
        let charge =
            maximum_current * current.tau / (CURRENT_DRAWN_AT_FULL_STRENGTH * 2.0 * MILLISECOND);
        self.world.get::<&mut Connection>(synapse).unwrap().strength = charge.clamp(0.4, 1.5);
        synapse
    }

    fn current(&mut self, to: Entity, current: f64, right: f32, up: f32) -> Entity {
        let source = self.add(Element::CurrentSource, right, up);
        self.world
            .get::<&mut CurrentClamp>(source)
            .unwrap()
            .current_output = current;
        self.connect(source, to);
        source
    }

    fn refractory_period(&mut self, neuron: Entity, seconds: f64) {
        self.world
            .get::<&mut LeakyDynamics>(neuron)
            .unwrap()
            .refractory_period = seconds;
    }

    fn rate_plot(&mut self, target: Entity, left: f32, up: f32, maximum: f64) {
        let plot = spawn_rate_plot(&mut self.world, target, Sheet::at(left, up));
        self.world.get::<&mut RatePlot>(plot).unwrap().maximum = maximum;
        *self.world.get::<&mut MeterSize>(plot).unwrap() = MeterSize {
            width: 14.5,
            height: 4.0,
        };
    }

    fn spike_detector(&mut self, targets: &[Entity], left: f32, up: f32, width: f32, height: f32) {
        let detector = spawn_spike_detector(&mut self.world, targets[0], Sheet::at(left, up));
        for &target in &targets[1..] {
            spawn_spike_detector_wire(&mut self.world, target, detector);
        }
        *self.world.get::<&mut MeterSize>(detector).unwrap() = MeterSize { width, height };
    }

    fn voltmeter(&mut self, target: Entity, left: f32, up: f32, width: f32, height: f32) {
        let voltmeter = spawn_voltmeter(&mut self.world, target, Sheet::at(left, up));
        *self.world.get::<&mut MeterSize>(voltmeter).unwrap() = MeterSize { width, height };
    }
}

fn at(up: f32, right: f32) -> Vec3 {
    Vec3::new(up, 0.0, right)
}

fn note(world: &mut World, position: Vec3, width: f32, height: f32, text: &str) {
    let note = Element::Note.spawn(world, position);
    world.get::<&mut Annotation>(note).unwrap().text = text.to_string();
    *world.get::<&mut NoteSize>(note).unwrap() = NoteSize { width, height };
}

fn label(world: &mut World, entity: Entity, text: &str) {
    world.get::<&mut Label>(entity).unwrap().text = text.to_string();
}

fn synapse(world: &mut World, from: Entity, to: Entity, maximum_current: f64) {
    let synapse = spawn_synapse(world, from, to);
    world
        .get::<&mut CurrentSynapse>(synapse)
        .unwrap()
        .maximum_current = maximum_current;
}

pub fn listening_to_a_neuron() -> World {
    let mut world = World::new();
    let eye = Element::VisualInput.spawn(&mut world, at(0.0, -9.0));
    *world.get::<&mut VisualInput>(eye).unwrap() = VisualInput {
        field: ReceptiveField::Orientation {
            angle: 0.0,
            x: 0.0,
            y: 0.0,
        },
        pattern: Pattern::Grating,
        sensitivity: 10000.0,
        phase: 0.75,
        ..Default::default()
    };
    let neuron = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
    label(&mut world, neuron, "Cortex");
    synapse(&mut world, eye, neuron, 1.5e-9);
    spawn_loudspeaker(&mut world, neuron, at(3.0, 4.0));
    spawn_spike_detector(&mut world, neuron, at(-5.0, 3.0));
    note(
        &mut world,
        at(9.0, -1.0),
        22.0,
        5.0,
        "Hubel and Wiesel found the neurons that see edges by listening to their spikes \
         as clicks. This neuron fires as the stripes pass when they lie at the angle \
         its receptive field prefers. Turn the picture angle and listen to the clicks fade.",
    );
    world
}

pub fn adaptive_inhibition() -> World {
    let mut world = World::new();
    let drive = Element::RegularSpikeGenerator.spawn(&mut world, at(0.0, -11.0));
    world
        .get::<&mut RegularSpikeGenerator>(drive)
        .unwrap()
        .frequency = 400.0;
    let output = Element::ExcitatoryNeuron.spawn(&mut world, at(0.0, 0.0));
    label(&mut world, output, "Output");
    let adaptive = Element::AdaptiveNeuron.spawn(&mut world, at(5.0, -5.0));
    *world.get::<&mut NeuronType>(adaptive).unwrap() = NeuronType::Inhibitory;
    *world.get::<&mut AdaptationCurrent>(adaptive).unwrap() = AdaptationCurrent {
        adaptation: 8e-9,
        time_constant: 0.4,
        ..Default::default()
    };
    label(&mut world, adaptive, "Adaptive");
    let noise = Element::PoissonGenerator.spawn(&mut world, at(-6.0, -6.0));
    *world.get::<&mut NeuronType>(noise).unwrap() = NeuronType::Inhibitory;
    world.get::<&mut PoissonGenerator>(noise).unwrap().rate = 30.0;
    synapse(&mut world, drive, output, 3e-9);
    synapse(&mut world, drive, adaptive, 6e-9);
    synapse(&mut world, adaptive, output, 4e-9);
    synapse(&mut world, noise, output, 2e-9);
    spawn_voltmeter(&mut world, output, at(-2.0, 5.0));
    let plot = spawn_rate_plot(&mut world, output, at(-9.0, 5.0));
    *world.get::<&mut RatePlot>(plot).unwrap() = RatePlot {
        window: 0.15,
        kernel_width: 0.05,
        maximum: 200.0,
        ..Default::default()
    };
    note(
        &mut world,
        at(12.5, 2.0),
        22.0,
        5.0,
        "The adaptive neuron is inhibitory. At first it holds the output back, \
         but as it adapts the output fires more and more. The random spikes from the \
         inhibitory generator below keep the output from firing like clockwork.",
    );
    world
}

pub fn adaptation() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "The neuron on the right adapts: each spike makes the next one harder. \
         Press the touch sensor a few times in a row and watch it start to skip.",
    );
    let up = sheet.row(5.0);
    let sensor = sheet.add(Element::TouchSensor, 1.5, up);
    let regular = sheet.add(Element::ExcitatoryNeuron, 6.5, up);
    let adaptive = sheet.add(Element::AdaptiveNeuron, 11.5, up);
    sheet.label(adaptive, "Adaptive");
    sheet.connect(sensor, regular);
    sheet.synapse(regular, adaptive, 3.0 * NANOAMPERE);
    sheet.voltmeter(adaptive, 14.5, up, 9.5, 5.0);
    sheet.text(
        "Leave it alone for a while and it recovers, \
         firing on every press again.",
    );
    sheet.world
}

pub fn bursting() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "A bursting neuron answers a single spike with a quick run of them. \
         Press the touch sensor once.",
    );
    let up = sheet.row(5.0);
    let sensor = sheet.add(Element::TouchSensor, 1.5, up);
    let bursting = sheet.add(Element::BurstNeuron, 7.0, up);
    sheet.label(bursting, "Bursting");
    sheet.connect(sensor, bursting);
    sheet.voltmeter(bursting, 10.5, up, 13.5, 5.0);
    sheet.text(
        "The first spike switches on a current inside the neuron. \
         It keeps the neuron firing until it drains away a moment later.",
    );
    sheet.world
}

pub fn firing_threshold() -> World {
    let mut sheet = Sheet::new();
    sheet.text("Each neuron gets a steady current from its source, the weakest at the top.");
    for current in [0.14, 0.2, 0.4] {
        let up = sheet.row(4.0);
        let neuron = sheet.add(Element::ExcitatoryNeuron, 6.5, up);
        sheet.current(neuron, current * NANOAMPERE, 1.5, up);
        sheet.voltmeter(neuron, 9.5, up, 14.5, 4.0);
    }
    sheet.text(
        "The weakest current never lifts the top neuron to its threshold at −55 mV, \
         so it never fires. Above the threshold, a stronger current makes a neuron fire faster.",
    );
    sheet.text(
        "Select a current source and change its current \
         to find the weakest one that makes the neuron fire.",
    );
    sheet.text(
        "Based on figure 8.5a in Principles of Computational Modelling in Neuroscience \
         by David Sterratt and others (Cambridge University Press, 2011).",
    );
    sheet.world
}

pub fn refractory_period() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "Both neurons get the same current. After each spike, the top neuron rests for 2 ms \
         and the bottom one for 10 ms before it can fire again.",
    );
    let top = sheet.row(4.0);
    let bottom = sheet.row(4.0);
    let short = sheet.add(Element::ExcitatoryNeuron, 6.5, top);
    let long = sheet.add(Element::ExcitatoryNeuron, 6.5, bottom);
    sheet.label(short, "2 ms");
    sheet.label(long, "10 ms");
    sheet.refractory_period(short, 2.0 * MILLISECOND);
    sheet.refractory_period(long, 10.0 * MILLISECOND);
    let source = sheet.current(short, 1.0 * NANOAMPERE, 1.5, (top + bottom) / 2.0);
    sheet.connect(source, long);
    sheet.rate_plot(short, 9.5, top, 300.0);
    sheet.rate_plot(long, 9.5, bottom, 300.0);
    sheet.text(
        "Select the current source and raise the current. The top neuron fires faster and faster, \
         but the bottom one never passes 100 spikes per second, one every 10 ms.",
    );
    sheet.text(
        "Based on figure 8.5b in Principles of Computational Modelling in Neuroscience \
         by David Sterratt and others (Cambridge University Press, 2011).",
    );
    sheet.world
}

pub fn summation() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "Each neuron on the left fires once when you press its touch sensor. \
         One spike alone is too weak to make the output neuron fire.",
    );
    let middle = sheet.row(12.0);
    let output = sheet.add(Element::ExcitatoryNeuron, 13.5, middle);
    sheet.label(output, "Output");
    for offset in [4.5, 0.0, -4.5] {
        let up = middle + offset;
        let sensor = sheet.add(Element::TouchSensor, 1.5, up);
        let input = sheet.add(Element::ExcitatoryNeuron, 6.5, up);
        sheet.connect(sensor, input);
        sheet.synapse(input, output, 1.4 * NANOAMPERE);
    }
    sheet.voltmeter(output, 16.0, middle, 8.0, 5.0);
    sheet.text(
        "Press two sensors right after each other: \
         the two inputs add up and lift the output over its threshold. \
         Pressing one sensor twice in quick succession works too, \
         because the second spike arrives before the first has leaked away.",
    );
    sheet.world
}

pub fn reverberation() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "These four neurons form a loop, as in tutorial 6, but the one on the left adapts. \
         Press the touch sensor to send a spike around the loop.",
    );
    let middle = sheet.row(10.0);
    let sensor = sheet.add(Element::TouchSensor, 1.5, middle);
    let adaptive = sheet.add(Element::AdaptiveNeuron, 6.0, middle);
    sheet.label(adaptive, "Adaptive");
    let top = sheet.add(Element::ExcitatoryNeuron, 10.0, middle + 4.0);
    let right = sheet.add(Element::ExcitatoryNeuron, 14.0, middle);
    let bottom = sheet.add(Element::ExcitatoryNeuron, 10.0, middle - 4.0);
    sheet.connect(sensor, adaptive);
    let ring = [adaptive, top, right, bottom];
    for (index, &from) in ring.iter().enumerate() {
        sheet.synapse(from, ring[(index + 1) % ring.len()], 3.0 * NANOAMPERE);
    }
    sheet.spike_detector(&[right], 16.5, middle, 7.5, 4.0);
    sheet.text(
        "The spikes keep circling long after your press, until the adaptive neuron \
         grows too tired to pass them on. Activity that outlasts its input like this \
         is called reverberation.",
    );
    sheet.world
}

pub fn half_center_oscillator() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "Each inhibitory neuron gets a steady current and holds the other one back, \
         so only one of them fires at a time. Both adapt, so the one that fires slowly tires \
         until the other breaks free.",
    );
    let middle = sheet.row(9.0);
    let top = sheet.add(Element::AdaptiveNeuron, 7.0, middle + 3.0);
    let bottom = sheet.add(Element::AdaptiveNeuron, 7.0, middle - 3.0);
    for (neuron, up) in [(top, middle + 3.0), (bottom, middle - 3.0)] {
        sheet.inhibitory(neuron);
        sheet
            .world
            .get::<&mut AdaptationCurrent>(neuron)
            .unwrap()
            .time_constant = 0.15;
        sheet.current(neuron, 0.5 * NANOAMPERE, 1.5, up);
    }
    sheet.lasting_inhibition(top, bottom);
    sheet.lasting_inhibition(bottom, top);
    sheet
        .world
        .get::<&mut LeakyDynamics>(bottom)
        .unwrap()
        .voltage = -0.075;
    sheet.spike_detector(&[top, bottom], 10.5, middle, 13.5, 9.0);
    sheet.text(
        "Pairs like this take turns in the circuits that drive walking and swimming, \
         and in the heartbeat of leeches. Such a pair is called a half-center oscillator.",
    );
    sheet.world
}

pub fn feedback_loops() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "Both chains get the same steady input. In the top chain the output excites a partner \
         that excites it back. In the bottom chain the partner is inhibitory and holds it back.",
    );
    for (name, partner_element) in [
        ("Faster", Element::ExcitatoryNeuron),
        ("Slower", Element::InhibitoryNeuron),
    ] {
        let up = sheet.row(9.0);
        let input = sheet.add(Element::ExcitatoryNeuron, 5.5, up + 2.5);
        sheet.current(input, 0.3 * NANOAMPERE, 1.5, up + 2.5);
        let output = sheet.add(Element::ExcitatoryNeuron, 9.5, up - 0.5);
        let partner = sheet.add(partner_element, 9.5, up - 4.0);
        sheet.label(input, "Input");
        sheet.label(output, name);
        sheet.synapse(input, output, 3.0 * NANOAMPERE);
        sheet.synapse(output, partner, 3.0 * NANOAMPERE);
        sheet.synapse(partner, output, 3.0 * NANOAMPERE);
        sheet.spike_detector(&[input, output], 13.0, up + 1.0, 11.0, 6.0);
    }
    sheet.text(
        "The excitatory loop keeps itself going and fires much faster than its input. \
         The inhibitory loop slows its output down. \
         These are positive and negative feedback.",
    );
    sheet.world
}

pub fn disinhibition() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "The output neuron gets a steady current and would fire on its own, \
         but the inhibitory neuron next to it holds it back like a brake.",
    );
    let up = sheet.row(8.0) + 1.5;
    let sensor = sheet.add(Element::TouchSensor, 1.5, up);
    let release = sheet.add(Element::BurstNeuron, 5.5, up);
    let brake = sheet.add(Element::InhibitoryNeuron, 10.0, up);
    let output = sheet.add(Element::ExcitatoryNeuron, 14.5, up);
    sheet.inhibitory(release);
    sheet.label(brake, "Brake");
    sheet.label(output, "Output");
    sheet.current(brake, 0.3 * NANOAMPERE, 10.0, up - 4.5);
    sheet.current(output, 0.3 * NANOAMPERE, 14.5, up - 4.5);
    sheet.connect(sensor, release);
    sheet.lasting_inhibition(release, brake);
    sheet.lasting_inhibition(brake, output);
    sheet.voltmeter(output, 17.0, up, 7.0, 4.0);
    sheet.text(
        "Press the touch sensor. Its neuron fires a burst that holds back the brake, \
         and the output fires for a while. Holding back a neuron that holds back another \
         is called disinhibition.",
    );
    sheet.text(
        "Disinhibition is common in the brain. In the cortex it helps with learning \
         (Letzkus, Wolff and Lüthi, Neuron, 2015).",
    );
    sheet.world
}

pub fn feedback_inhibition() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "A steady current makes the input neuron fire, \
         and the signal passes down the chain to the output.",
    );
    let middle = sheet.row(13.0) - 0.5;
    let input = sheet.add(Element::ExcitatoryNeuron, 6.0, middle + 5.0);
    sheet.current(input, 0.3 * NANOAMPERE, 1.5, middle + 5.0);
    let relay = sheet.add(Element::ExcitatoryNeuron, 6.0, middle);
    let output = sheet.add(Element::ExcitatoryNeuron, 6.0, middle - 5.0);
    let feedback = sheet.add(Element::InhibitoryNeuron, 10.0, middle);
    sheet.label(input, "Input");
    sheet.synapse(input, relay, 3.0 * NANOAMPERE);
    sheet.synapse(relay, output, 3.0 * NANOAMPERE);
    sheet.synapse(output, feedback, 3.0 * NANOAMPERE);
    sheet.synapse(feedback, relay, 3.0 * NANOAMPERE);
    sheet.spike_detector(&[input, output], 13.0, middle, 11.0, 8.0);
    sheet.text(
        "Each time the output fires, it wakes the inhibitory neuron, \
         which reaches back and holds the chain down for a moment. \
         The output fires about half as often as the input. \
         This is called feedback inhibition.",
    );
    sheet.world
}

const PRESSURE: [f64; 5] = [0.18, 0.25, 0.4, 0.25, 0.18];
const SIDEWAYS_PULL: f64 = 1.5 * NANOAMPERE;

pub fn lateral_inhibition() -> World {
    let mut sheet = Sheet::new();
    NeuronDisplay {
        voltage: true,
        firing_rate: true,
    }
    .set_in(&mut sheet.world);
    sheet.text(
        "A pencil pressing on the skin pushes hardest under its tip. \
         The current sources stand for the pressure on five spots of skin, \
         strongest in the middle.",
    );
    let up = sheet.row(17.0) - 0.5;
    let spacing = WIDTH / PRESSURE.len() as f32;
    let right_of = |index: usize| spacing * (index as f32 + 0.5);
    let skin: Vec<Entity> = (0..PRESSURE.len())
        .map(|index| sheet.add(Element::ExcitatoryNeuron, right_of(index), up + 3.5))
        .collect();
    let partners: Vec<Entity> = (0..PRESSURE.len())
        .map(|index| sheet.add(Element::InhibitoryNeuron, right_of(index), up - 1.5))
        .collect();
    let outputs: Vec<Entity> = (0..PRESSURE.len())
        .map(|index| sheet.add(Element::ExcitatoryNeuron, right_of(index), up - 6.5))
        .collect();
    for (index, &pressure) in PRESSURE.iter().enumerate() {
        sheet.current(
            skin[index],
            pressure * NANOAMPERE,
            right_of(index),
            up + 8.0,
        );
        sheet.synapse(skin[index], outputs[index], 3.0 * NANOAMPERE);
        sheet.synapse(skin[index], partners[index], 3.0 * NANOAMPERE);
        for neighbour in [index.wrapping_sub(1), index + 1] {
            if neighbour < PRESSURE.len() {
                sheet.synapse(partners[index], outputs[neighbour], SIDEWAYS_PULL);
            }
        }
    }
    sheet.text(
        "Each sensory neuron passes its spikes to the output neuron below it, \
         and through an inhibitory neuron it holds back the outputs next to it. \
         The glow under each neuron shows how often it fires. \
         The middle output stands out much more than the middle of the skin.",
    );
    sheet.text(
        "This is called lateral inhibition, and it makes it easier to tell where the pencil is. \
         Select a current source and change its current to press somewhere else.",
    );
    sheet.world
}

const DIRECTION_RELAY_TO_OUTPUT: f64 = 1.2 * NANOAMPERE;

pub fn direction_selectivity() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "The five touch sensors stand for light falling on a row of cells in the eye. \
         Press them one after another from right to left, and then from left to right.",
    );
    let up = sheet.row(23.0) + 0.5;
    let columns = 5;
    let spacing = WIDTH / columns as f32;
    let right_of = |index: usize| spacing * (index as f32 + 0.5);
    let output = sheet.add(Element::ExcitatoryNeuron, 7.5, up - 10.0);
    sheet.label(output, "Output");
    let mut relays = Vec::new();
    let mut partners = Vec::new();
    for index in 0..columns {
        let sensor = sheet.add(Element::TouchSensor, right_of(index), up + 10.0);
        let receptor = sheet.add(Element::ExcitatoryNeuron, right_of(index), up + 5.5);
        let partner = sheet.add(Element::InhibitoryNeuron, right_of(index) + 1.6, up + 1.0);
        let relay = sheet.add(Element::ExcitatoryNeuron, right_of(index), up - 3.5);
        sheet.connect(sensor, receptor);
        let partner_synapse = sheet.synapse(receptor, partner, 3.0 * NANOAMPERE);
        sheet
            .world
            .get::<&mut CurrentSynapse>(partner_synapse)
            .unwrap()
            .delay = MILLISECOND;
        let relay_synapse = sheet.synapse(receptor, relay, 3.0 * NANOAMPERE);
        sheet
            .world
            .get::<&mut CurrentSynapse>(relay_synapse)
            .unwrap()
            .delay = 2.0 * MILLISECOND;
        sheet.synapse(relay, output, DIRECTION_RELAY_TO_OUTPUT);
        relays.push(relay);
        partners.push(partner);
    }
    for index in 0..columns - 1 {
        let synapse = sheet.lasting_inhibition(partners[index], relays[index + 1]);
        sheet.world.get::<&mut CurrentSynapse>(synapse).unwrap().tau = 20.0 * MILLISECOND;
    }
    sheet.voltmeter(output, 11.0, up - 10.0, 13.0, 4.0);
    sheet.text(
        "From right to left, the spikes reach the output close together and it fires. \
         From left to right, each inhibitory neuron has already silenced \
         the next neuron in the middle row, so the output stays quiet.",
    );
    sheet.text(
        "Cells in the retina sense movement in one direction like this. \
         This is called direction selectivity.",
    );
    sheet.world
}

const RECEPTIVE_FIELD_SENSITIVITY: f64 = 10000.0;
const RECEPTIVE_FIELD_SYNAPSE: f64 = 2.0 * NANOAMPERE;

pub fn receptive_fields() -> World {
    let mut sheet = Sheet::new();
    sheet.text(
        "Both visual inputs watch the same picture, a dot moving in a circle. \
         Each one's receptive field, drawn over the picture, is a small spot inside a ring: \
         light on the spot excites it, and light on the ring holds it back.",
    );
    let mut neurons = Vec::new();
    let mut middle = 0.0;
    for spot in [0.25, -0.25] {
        let up = sheet.row(8.0);
        middle += up / 2.0;
        let eye = sheet.add(Element::VisualInput, 5.0, up);
        *sheet.world.get::<&mut VisualInput>(eye).unwrap() = VisualInput {
            field: ReceptiveField::CenterSurround {
                off_center: false,
                x: spot,
                y: 0.0,
            },
            pattern: Pattern::Dot,
            sensitivity: RECEPTIVE_FIELD_SENSITIVITY,
            ..Default::default()
        };
        let neuron = sheet.add(Element::ExcitatoryNeuron, 12.5, up);
        sheet.synapse(eye, neuron, RECEPTIVE_FIELD_SYNAPSE);
        neurons.push(neuron);
    }
    sheet.spike_detector(&neurons, 15.5, middle, 8.5, 9.0);
    sheet.text(
        "Each neuron fires when the dot crosses its own spot, \
         so the two take turns. Cells like these in the retina are the first step of seeing.",
    );
    sheet.text("Select a visual input to change its picture, its speed or its field.");
    sheet.world
}

pub fn sheets() -> Vec<(&'static str, World)> {
    vec![
        ("adaptation.neuronify", adaptation()),
        ("bursting.neuronify", bursting()),
        ("firing_threshold.neuronify", firing_threshold()),
        ("refractory_period.neuronify", refractory_period()),
        ("summation.neuronify", summation()),
        ("reverberation.neuronify", reverberation()),
        ("half_center_oscillator.neuronify", half_center_oscillator()),
        ("feedback_loops.neuronify", feedback_loops()),
        ("disinhibition.neuronify", disinhibition()),
        ("feedback_inhibition.neuronify", feedback_inhibition()),
        ("lateral_inhibition.neuronify", lateral_inhibition()),
        ("direction_selectivity.neuronify", direction_selectivity()),
        ("receptive_fields.neuronify", receptive_fields()),
    ]
}

pub fn examples() -> Vec<(&'static str, World)> {
    let mut examples = vec![
        ("listening_to_a_neuron.neuronify", listening_to_a_neuron()),
        ("adaptive_inhibition.neuronify", adaptive_inhibition()),
    ];
    examples.extend(sheets());
    examples
}

fn main() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    for (file, world) in examples() {
        let bytes = neuronify_core::serialization::save(&world, 0.0, DEFAULT_SPEED).unwrap();
        std::fs::write(format!("{folder}/{file}"), bytes).unwrap();
        println!("Wrote {file}");
    }
}
