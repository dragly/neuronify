use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;

pub fn update_firing_rates(world: &mut World, duration: f64) {
    let new_neurons: Vec<Entity> = world
        .query::<()>()
        .with::<&LeakyDynamics>()
        .without::<&FiringRate>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in new_neurons {
        world.insert_one(entity, FiringRate::default()).unwrap();
    }
    let decay = (-duration / FIRING_RATE_TIME_CONSTANT).exp();
    for (_, (dynamics, firing_rate)) in world.query_mut::<(&LeakyDynamics, &mut FiringRate)>() {
        firing_rate.rate *= decay;
        if dynamics.time_since_fire < duration {
            firing_rate.rate += (-dynamics.time_since_fire / FIRING_RATE_TIME_CONSTANT).exp()
                / FIRING_RATE_TIME_CONSTANT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_neuron_firing_at_50_hz_settles_near_50_hz() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyDynamics::default(),));
        let frame = 4.0 * LIF_DT;
        let frames_between_spikes = 50;
        let rates: Vec<f64> = (0..5000)
            .map(|frame_index| {
                world
                    .get::<&mut LeakyDynamics>(neuron)
                    .unwrap()
                    .time_since_fire = (frame_index % frames_between_spikes) as f64 * frame;
                update_firing_rates(&mut world, frame);
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
        let neuron = world.spawn((LeakyDynamics::default(), FiringRate { rate: 100.0 }));
        for _ in 0..2500 {
            update_firing_rates(&mut world, 4.0 * LIF_DT);
        }
        assert!(world.get::<&FiringRate>(neuron).unwrap().rate < 1.0);
    }
}
