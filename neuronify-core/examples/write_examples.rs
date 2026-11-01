use glam::Vec3;
use hecs::{Entity, World};

use neuronify_core::constants::DEFAULT_SPEED;
use neuronify_core::elements::{
    spawn_loudspeaker, spawn_rate_plot, spawn_spike_detector, spawn_synapse, spawn_voltmeter,
    Element,
};
use neuronify_core::measurement::firing_rate::RatePlot;
use neuronify_core::measurement::visual_input::{Pattern, ReceptiveField, VisualInput};
use neuronify_core::{
    AdaptationCurrent, Annotation, CurrentSynapse, Label, NeuronType, NoteSize, PoissonGenerator,
    RegularSpikeGenerator,
};

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

fn main() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    for (file, world) in [
        ("listening_to_a_neuron.neuronify", listening_to_a_neuron()),
        ("adaptive_inhibition.neuronify", adaptive_inhibition()),
    ] {
        let bytes = neuronify_core::serialization::save(&world, 0.0, DEFAULT_SPEED).unwrap();
        std::fs::write(format!("{folder}/{file}"), bytes).unwrap();
        println!("Wrote {file}");
    }
}
