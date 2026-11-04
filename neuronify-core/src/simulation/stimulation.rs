use glam::Vec3;

use super::fhn::fhn_tau;
use crate::components::*;
use crate::constants::*;

pub fn touch_sensor_near(world: &hecs::World, position: Vec3) -> bool {
    world
        .query::<&Position>()
        .with::<&TouchSensor>()
        .iter()
        .any(|(_, sensor)| sensor.position.distance(position) < ERASE_RADIUS)
}

pub fn fire_nearby(world: &mut hecs::World, position: Vec3) {
    let touch_entities: Vec<hecs::Entity> = world
        .query::<(&Position, &TouchSensor)>()
        .iter()
        .filter(|(_, (pos, _))| pos.position.distance(position) < ERASE_RADIUS)
        .map(|(e, _)| e)
        .collect();
    for entity in touch_entities {
        if let Ok(mut dynamics) = world.get::<&mut GeneratorDynamics>(entity) {
            dynamics.fired = true;
            dynamics.time_since_fire = 0.0;
        }
    }

    let neuron_entities: Vec<hecs::Entity> = world
        .query::<(&Position, &LeakyNeuron)>()
        .iter()
        .filter(|(_, (pos, _))| pos.position.distance(position) < ERASE_RADIUS)
        .map(|(e, _)| e)
        .collect();
    for entity in neuron_entities {
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
            let neuron = world.get::<&LeakyNeuron>(entity).unwrap();
            dynamics.voltage = neuron.threshold + 0.01;
        }
    }

    for (_, (compartment_position, compartment)) in
        world.query_mut::<(&Position, &mut Compartment)>()
    {
        if compartment_position.position.distance(position) < ERASE_RADIUS {
            compartment.voltage = COMPARTMENT_FIRE_VOLTAGE;
        }
    }
}

pub fn stimulate_nearby(world: &mut hecs::World, position: Vec3) {
    let mut touched_neuron = false;
    for (_, (pos, dynamics)) in world
        .query_mut::<(&Position, &mut LeakyDynamics)>()
        .with::<&LeakyNeuron>()
    {
        if pos.position.distance(position) < ERASE_RADIUS {
            dynamics.received_currents += STIMULATION_CURRENT;
            touched_neuron = true;
        }
    }
    if touched_neuron {
        return;
    }
    for (_, (pos, compartment, dendrite)) in
        world.query_mut::<(&Position, &mut Compartment, Option<&Dendrite>)>()
    {
        if pos.position.distance(position) < ERASE_RADIUS {
            let held = STIMULATION_DRIVE * FHN_SCALE * fhn_tau(dendrite);
            compartment.injected_current = compartment.injected_current.max(held);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dendrites::spawn_dendrites;
    use crate::elements::{spawn_cable, spawn_compartment, Element};
    use crate::simulation::step_simulation;

    fn spikes_in_a_second(world: &mut hecs::World, neuron: hecs::Entity, touch: Vec3) -> usize {
        let mut time = 0.0;
        let mut spikes = 0;
        while time < 1.0 {
            step_simulation(world, &mut time, 1, Some(touch));
            if world.get::<&LeakyDynamics>(neuron).unwrap().time_since_fire == 0.0 {
                spikes += 1;
            }
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
        let touch = world.get::<&Position>(neuron).unwrap().position;
        let spikes = spikes_in_a_second(&mut world, neuron, touch);
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
        let spikes = spikes_in_a_second(&mut world, neuron, tip);
        assert!((30..70).contains(&spikes), "{spikes} spikes in a second");
    }

    #[test]
    fn a_held_axon_keeps_firing_its_neuron() {
        let mut world = hecs::World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let mut previous = neuron;
        for step in 1..=10 {
            let position = -(step as f32) * SEGMENT_LENGTH * Vec3::Z;
            let compartment = spawn_compartment(&mut world, position, NeuronType::Excitatory);
            spawn_cable(&mut world, previous, compartment, false);
            previous = compartment;
        }
        let middle = -5.0 * SEGMENT_LENGTH * Vec3::Z;
        let spikes = spikes_in_a_second(&mut world, neuron, middle);
        assert!(spikes >= 30, "{spikes} spikes in a second");
    }

    #[test]
    fn holding_a_neuron_leaves_its_compartments_alone() {
        let mut world = hecs::World::new();
        Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let compartment = spawn_compartment(&mut world, Vec3::X, NeuronType::Excitatory);

        stimulate_nearby(&mut world, Vec3::ZERO);

        let held = world
            .get::<&Compartment>(compartment)
            .unwrap()
            .injected_current;
        assert_eq!(held, 0.0);
    }

    #[test]
    fn touching_a_compartment_starts_a_spike() {
        let mut world = hecs::World::new();
        let compartment_at = |world: &mut hecs::World, position: Vec3| {
            world.spawn((Position { position }, Compartment::default()))
        };
        let touched = compartment_at(&mut world, Vec3::ZERO);
        let far = compartment_at(&mut world, 10.0 * Vec3::X);

        fire_nearby(&mut world, 0.5 * Vec3::Z);

        let voltage = |entity| world.get::<&Compartment>(entity).unwrap().voltage;
        assert_eq!(voltage(touched), COMPARTMENT_FIRE_VOLTAGE);
        assert_eq!(voltage(far), Compartment::default().voltage);
    }
}
