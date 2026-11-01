use glam::Vec3;
use hecs::{Entity, World};

use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::{
    spawn_loudspeaker, spawn_rate_plot, spawn_spike_detector, spawn_synapse, spawn_voltmeter,
    Element,
};
use neuronify_core::measurement::firing_rate::RatePlot;
use neuronify_core::measurement::visual_input::{Pattern, ReceptiveField, VisualInput};
use neuronify_core::rendering::notes::{NOTE_FONT_DATA, NOTE_MARGIN, NOTE_TEXT_HEIGHT};
use neuronify_core::{
    AdaptationCurrent, Annotation, Connection, CurrentSynapse, Label, MeterSize, NeuronType,
    NoteSize, PoissonGenerator, RegularSpikeGenerator,
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

pub fn sheets() -> Vec<(&'static str, World)> {
    vec![("adaptation.neuronify", adaptation())]
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
