use glam::Vec3;

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
}

pub fn stimulate_nearby(world: &mut hecs::World, position: Vec3) {
    for (_, (pos, dynamics)) in world
        .query_mut::<(&Position, &mut LeakyDynamics)>()
        .with::<&LeakyNeuron>()
    {
        if pos.position.distance(position) < ERASE_RADIUS {
            dynamics.received_currents += STIMULATION_CURRENT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;
    use crate::simulation::step_simulation;

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
        let mut time = 0.0;
        let mut spikes = 0;
        while time < 1.0 {
            step_simulation(&mut world, &mut time, 1, Some(touch));
            if world.get::<&LeakyDynamics>(neuron).unwrap().time_since_fire == 0.0 {
                spikes += 1;
            }
        }
        assert!((30..60).contains(&spikes), "{spikes} spikes in a second");
    }
}
