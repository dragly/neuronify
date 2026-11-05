use hecs::{Entity, World};

use super::Spike;
use crate::components::*;
use crate::constants::*;

pub fn update_firing_rates(world: &mut World, spikes: &[Spike], duration: f64, now: f64) {
    let new_neurons: Vec<Entity> = world
        .query::<()>()
        .with::<&LeakyNeuron>()
        .without::<&FiringRate>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in new_neurons {
        world.insert_one(entity, FiringRate::default()).unwrap();
    }
    let decay = (-duration / FIRING_RATE_TIME_CONSTANT).exp();
    for (_, firing_rate) in world.query_mut::<&mut FiringRate>() {
        firing_rate.rate *= decay;
    }
    for spike in spikes {
        if let Ok(mut firing_rate) = world.get::<&mut FiringRate>(spike.entity) {
            let age = (now - spike.time * 1e-3).max(0.0);
            firing_rate.rate +=
                (-age / FIRING_RATE_TIME_CONSTANT).exp() / FIRING_RATE_TIME_CONSTANT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f64 = 0.0004;

    #[test]
    fn a_neuron_firing_at_50_hz_settles_near_50_hz() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyNeuron::default(),));
        let frames_between_spikes = 50;
        let rates: Vec<f64> = (0..5000)
            .map(|frame_index| {
                let now = (frame_index + 1) as f64 * FRAME;
                let spikes: Vec<Spike> = (frame_index % frames_between_spikes == 0)
                    .then_some(Spike {
                        entity: neuron,
                        time: (now - FRAME / 2.0) * 1e3,
                    })
                    .into_iter()
                    .collect();
                update_firing_rates(&mut world, &spikes, FRAME, now);
                world.get::<&FiringRate>(neuron).unwrap().rate
            })
            .collect();
        let last_period = &rates[rates.len() - frames_between_spikes..];
        let rate = last_period.iter().sum::<f64>() / frames_between_spikes as f64;
        assert!((45.0..55.0).contains(&rate), "{rate} Hz");
    }

    #[test]
    fn a_silent_neuron_decays_towards_zero() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyNeuron::default(), FiringRate { rate: 100.0 }));
        for frame in 0..2500 {
            update_firing_rates(&mut world, &[], FRAME, frame as f64 * FRAME);
        }
        assert!(world.get::<&FiringRate>(neuron).unwrap().rate < 1.0);
    }
}
