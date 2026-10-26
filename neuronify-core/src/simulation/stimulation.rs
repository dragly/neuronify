use glam::Vec3;

use crate::components::*;
use crate::constants::*;
use crate::StimulationTool;

pub fn touch_sensor_near(world: &hecs::World, position: Vec3) -> bool {
    world
        .query::<&Position>()
        .with::<&TouchSensor>()
        .iter()
        .any(|(_, sensor)| sensor.position.distance(position) < ERASE_RADIUS)
}

pub fn stimulate_nearby(world: &mut hecs::World, stimulation_tool: &Option<StimulationTool>) {
    let Some(stim) = stimulation_tool else {
        return;
    };

    let touch_entities: Vec<hecs::Entity> = world
        .query::<(&Position, &TouchSensor)>()
        .iter()
        .filter(|(_, (pos, _))| pos.position.distance(stim.position) < ERASE_RADIUS)
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
        .filter(|(_, (pos, _))| pos.position.distance(stim.position) < ERASE_RADIUS)
        .map(|(e, _)| e)
        .collect();
    for entity in neuron_entities {
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
            let neuron = world.get::<&LeakyNeuron>(entity).unwrap();
            dynamics.voltage = neuron.threshold + 0.01;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
