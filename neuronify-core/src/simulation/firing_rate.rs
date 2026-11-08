use hecs::{Entity, World};
use neuronify_engine::units::Milliseconds;

use super::Spike;
use crate::components::*;
use crate::constants::*;

pub fn update_firing_rates(
    world: &mut World,
    spikes: &[Spike],
    duration: Milliseconds,
    now: Milliseconds,
) {
    let new_neurons: Vec<Entity> = world
        .query::<Entity>()
        .with::<&LeakyNeuron>()
        .without::<&FiringRate>()
        .iter()
        .collect();
    for entity in new_neurons {
        world.insert_one(entity, FiringRate::default()).unwrap();
    }
    let decay = (-duration / FIRING_RATE_TIME_CONSTANT).exp();
    for firing_rate in world.query_mut::<&mut FiringRate>() {
        firing_rate.rate *= decay;
    }
    for spike in spikes {
        if let Ok(mut firing_rate) = world.get::<&mut FiringRate>(spike.entity) {
            let age = (now - spike.time).max(Milliseconds::ZERO);
            let rise = (-age / FIRING_RATE_TIME_CONSTANT).exp() / FIRING_RATE_TIME_CONSTANT;
            firing_rate.rate += rise.to_hertz();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Milliseconds = Milliseconds(0.4);

    #[test]
    fn a_neuron_firing_at_50_hz_settles_near_50_hz() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyNeuron::default(),));
        let frames_between_spikes = 50;
        let rates: Vec<Hertz> = (0..5000)
            .map(|frame_index| {
                let now = (frame_index + 1) as f64 * FRAME;
                let spikes: Vec<Spike> = (frame_index % frames_between_spikes == 0)
                    .then_some(Spike {
                        entity: neuron,
                        time: now - FRAME / 2.0,
                    })
                    .into_iter()
                    .collect();
                update_firing_rates(&mut world, &spikes, FRAME, now);
                world.get::<&FiringRate>(neuron).unwrap().rate
            })
            .collect();
        let last_period = &rates[rates.len() - frames_between_spikes..];
        let rate = last_period.iter().sum::<Hertz>() / frames_between_spikes as f64;
        assert!((Hertz(45.0)..Hertz(55.0)).contains(&rate), "{rate}");
    }

    #[test]
    fn a_silent_neuron_decays_towards_zero() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyNeuron::default(), FiringRate { rate: Hertz(100.0) }));
        for frame in 0..2500 {
            update_firing_rates(&mut world, &[], FRAME, frame as f64 * FRAME);
        }
        assert!(world.get::<&FiringRate>(neuron).unwrap().rate < Hertz(1.0));
    }
}
