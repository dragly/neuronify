use glam::Vec3;

use neuronify_core::{LeakyDynamics, SpatialDynamics};

use crate::components::*;

/// Apply motor cilia forces: neurons with MotorCilia that recently fired get thrust.
pub fn apply_motor_forces(world: &mut hecs::World, dt: f64) {
    let entities: Vec<hecs::Entity> = world
        .query::<(&MotorCilia, &LeakyDynamics)>()
        .without::<&Anchored>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in entities {
        let fired = world
            .get::<&LeakyDynamics>(entity)
            .map(|d| d.time_since_fire < dt * 2.0)
            .unwrap_or(false);

        if fired {
            if let Ok(cilia) = world.get::<&MotorCilia>(entity) {
                let force = cilia.direction * cilia.strength;
                if let Ok(mut dynamics) = world.get::<&mut SpatialDynamics>(entity) {
                    dynamics.acceleration += force;
                }
            }
        }
    }
}

/// Enforce anchored entities stay put.
pub fn enforce_anchored(world: &mut hecs::World) {
    for (_, (dynamics, _)) in world.query_mut::<(&mut SpatialDynamics, &Anchored)>() {
        dynamics.velocity = Vec3::ZERO;
        dynamics.acceleration = Vec3::ZERO;
    }
}
