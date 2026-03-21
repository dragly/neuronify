use crate::components::*;
use crate::constants::*;
use crate::StimulationTool;

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
