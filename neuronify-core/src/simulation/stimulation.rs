use glam::Vec3;
use hecs::{Entity, World};

use super::Engine;
use crate::components::*;
use crate::constants::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Touch {
    pub position: Vec3,
    pub current: f64,
}

pub fn touch_sensor_near(world: &World, position: Vec3) -> bool {
    world
        .query::<&Position>()
        .with::<&TouchSensor>()
        .iter()
        .any(|(_, sensor)| sensor.position.distance(position) < ERASE_RADIUS)
}

fn near<Q: hecs::Query>(world: &World, position: Vec3) -> Vec<Entity> {
    world
        .query::<&Position>()
        .with::<Q>()
        .iter()
        .filter(|(_, at)| at.position.distance(position) < ERASE_RADIUS)
        .map(|(entity, _)| entity)
        .collect()
}

pub fn fire_nearby(world: &mut World, engine: &mut Engine, position: Vec3) {
    engine.sync(world);
    let touched = [
        near::<&TouchSensor>(world, position),
        near::<&LeakyNeuron>(world, position),
        near::<&Compartment>(world, position),
    ];
    for entity in touched.into_iter().flatten() {
        engine.fire(world, entity);
    }
}

pub fn touched(world: &World, touch: Option<Touch>) -> Vec<(Entity, f64)> {
    let Some(touch) = touch else {
        return Vec::new();
    };
    let neurons = near::<&LeakyNeuron>(world, touch.position);
    if !neurons.is_empty() {
        return neurons
            .into_iter()
            .map(|neuron| (neuron, touch.current))
            .collect();
    }
    near::<&Compartment>(world, touch.position)
        .into_iter()
        .map(|compartment| (compartment, touch.current))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dendrites::spawn_dendrites;
    use crate::elements::{spawn_cable, spawn_compartment, Element};
    use crate::simulation::{step_simulation, Engine};

    fn held(position: Vec3) -> Touch {
        Touch {
            position,
            current: STIMULATION_CURRENT,
        }
    }

    fn spikes_in_a_second(world: &mut hecs::World, neuron: hecs::Entity, touch: Touch) -> usize {
        let (mut engine, mut time) = (Engine::default(), 0.0);
        let mut spikes = 0;
        while time < 1.0 {
            step_simulation(world, &mut engine, &mut time, 40, Some(touch));
            spikes += engine
                .spikes()
                .iter()
                .filter(|spike| spike.entity == neuron)
                .count();
        }
        spikes
    }

    #[test]
    fn finds_touch_sensors_within_reach() {
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            TouchSensor,
            GeneratorDynamics::default(),
        ));
        world.spawn((
            Position {
                position: 10.0 * Vec3::X,
            },
            LeakyNeuron::default(),
        ));
        assert!(touch_sensor_near(&world, 0.5 * ERASE_RADIUS * Vec3::Z));
        assert!(!touch_sensor_near(&world, 2.0 * ERASE_RADIUS * Vec3::Z));
        assert!(!touch_sensor_near(&world, 10.0 * Vec3::X));
    }

    #[test]
    fn a_held_neuron_fires_about_as_often_as_the_examples() {
        let mut world = hecs::World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let soma = world.get::<&Position>(neuron).unwrap().position;
        let spikes = spikes_in_a_second(&mut world, neuron, held(soma));
        assert!((30..60).contains(&spikes), "{spikes} spikes in a second");
    }

    #[test]
    fn a_held_dendrite_fires_its_neuron_about_as_often_as_a_held_neuron() {
        let mut world = hecs::World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        spawn_dendrites(&mut world, neuron);
        let tip = world
            .query::<&Position>()
            .with::<&Dendrite>()
            .iter()
            .map(|(_, dendrite)| dendrite.position)
            .max_by(|a, b| a.length().total_cmp(&b.length()))
            .unwrap();
        let spikes = spikes_in_a_second(&mut world, neuron, held(tip));
        assert!((30..70).contains(&spikes), "{spikes} spikes in a second");
    }

    fn neuron_with_axon(world: &mut hecs::World) -> hecs::Entity {
        let neuron = Element::ExcitatoryNeuron.spawn(world, Vec3::ZERO);
        let mut previous = neuron;
        for step in 1..=10 {
            let position = -(step as f32) * SEGMENT_LENGTH * Vec3::Z;
            let compartment = spawn_compartment(world, position, NeuronType::Excitatory);
            spawn_cable(world, previous, compartment, false);
            previous = compartment;
        }
        neuron
    }

    #[test]
    fn a_held_axon_fires_its_neuron_faster_than_a_held_neuron() {
        let mut world = hecs::World::new();
        let neuron = neuron_with_axon(&mut world);
        let axon = spikes_in_a_second(&mut world, neuron, held(-5.0 * SEGMENT_LENGTH * Vec3::Z));
        let mut world = hecs::World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let soma = spikes_in_a_second(&mut world, neuron, held(Vec3::ZERO));
        assert!(axon > 2 * soma, "axon {axon}, soma {soma}");
    }

    #[test]
    fn moving_a_held_finger_along_an_axon_adds_no_spikes() {
        let spikes = |moving: bool| {
            let mut world = hecs::World::new();
            let neuron = neuron_with_axon(&mut world);
            let (mut engine, mut time) = (Engine::default(), 0.0);
            let mut spikes = 0;
            for frame in 0..500 {
                let node = if moving { 3 + (frame / 3) % 4 } else { 4 };
                let position = -(node as f32) * SEGMENT_LENGTH * Vec3::Z;
                step_simulation(&mut world, &mut engine, &mut time, 8, Some(held(position)));
                spikes += engine
                    .spikes()
                    .iter()
                    .filter(|spike| spike.entity == neuron)
                    .count();
            }
            spikes
        };
        assert_eq!(spikes(true), spikes(false));
    }

    #[test]
    fn a_held_neuron_fires_faster_the_stronger_the_current() {
        let spikes = |current| {
            let mut world = hecs::World::new();
            let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
            let touch = Touch {
                position: Vec3::ZERO,
                current,
            };
            spikes_in_a_second(&mut world, neuron, touch)
        };
        assert_eq!(spikes(0.1), 0);
        assert!(spikes(STIMULATION_CURRENT) < spikes(1.0));
    }

    #[test]
    fn a_held_dendrite_keeps_firing_however_strong_the_current() {
        let mut world = hecs::World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        spawn_dendrites(&mut world, neuron);
        let mut dendrites: Vec<Vec3> = world
            .query::<&Position>()
            .with::<&Dendrite>()
            .iter()
            .map(|(_, dendrite)| dendrite.position)
            .collect();
        dendrites.sort_by(|a, b| a.length().total_cmp(&b.length()));
        let touch = Touch {
            position: dendrites[dendrites.len() / 2],
            current: 2.0,
        };
        let spikes = spikes_in_a_second(&mut world, neuron, touch);
        assert!(spikes >= 30, "{spikes} spikes in a second");
    }

    #[test]
    fn holding_a_neuron_leaves_its_compartments_alone() {
        let mut world = hecs::World::new();
        Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let compartment = spawn_compartment(&mut world, Vec3::X, NeuronType::Excitatory);

        let touched = touched(&world, Some(held(Vec3::ZERO)));

        assert!(touched.iter().all(|&(entity, _)| entity != compartment));
        assert_eq!(touched.len(), 1);
    }

    #[test]
    fn touching_a_compartment_starts_a_spike() {
        let mut world = hecs::World::new();
        let chain_at = |world: &mut hecs::World, position: Vec3| {
            let start = spawn_compartment(world, position, NeuronType::Excitatory);
            let end = spawn_compartment(world, position + 2.0 * Vec3::Z, NeuronType::Excitatory);
            spawn_cable(world, start, end, false);
            start
        };
        let touched = chain_at(&mut world, Vec3::ZERO);
        let far = chain_at(&mut world, 10.0 * Vec3::X);

        let (mut engine, mut time) = (Engine::default(), 0.0);
        fire_nearby(&mut world, &mut engine, 0.5 * Vec3::Z);
        step_simulation(&mut world, &mut engine, &mut time, 8, None);

        let voltage = |entity| world.get::<&NodeState>(entity).unwrap().voltage;
        assert!(voltage(touched) > 0.0, "{}", voltage(touched));
        assert!(
            (voltage(far) - RESTING_POTENTIAL).abs() < 1.0,
            "{}",
            voltage(far)
        );
    }
}
