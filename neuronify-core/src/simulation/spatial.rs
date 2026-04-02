use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;

pub fn apply_spatial_forces(world: &mut hecs::World) {
    let positions: Vec<(Entity, Position)> = world
        .query::<&Position>()
        .iter()
        .map(|(e, p)| (e.to_owned(), p.to_owned()))
        .collect();
    for (id, (position, dynamics)) in world.query_mut::<(&Position, &mut SpatialDynamics)>() {
        for (other_id, other_position) in &positions {
            if id == *other_id {
                continue;
            }
            let from = position.position;
            let to = other_position.position;
            let r2 = from.distance_squared(to);
            let target2 = (2.0 * NODE_RADIUS).powi(2);
            let d = (to - from).normalize_or_zero();
            let force = REPULSION_STRENGTH * (r2 - target2).min(0.0) * d;
            dynamics.acceleration += force;
        }
    }

    let connections: Vec<(Entity, Connection)> = world
        .query::<&Connection>()
        .iter()
        .map(|(e, c)| (e.to_owned(), c.to_owned()))
        .collect();

    for (connection_id_1, connection_1) in &connections {
        for (connection_id_2, connection_2) in &connections {
            if connection_id_1 == connection_id_2 {
                continue;
            }
            if connection_1.to != connection_2.from {
                continue;
            }
            let Some(to_1) = world.get::<&Position>(connection_1.to).ok().map(|p| p.position) else { continue };
            let Some(from_1) = world.get::<&Position>(connection_1.from).ok().map(|p| p.position) else { continue };
            let Some(to_2) = world.get::<&Position>(connection_2.to).ok().map(|p| p.position) else { continue };
            let Some(from_2) = world.get::<&Position>(connection_2.from).ok().map(|p| p.position) else { continue };
            let target = 1.0;
            let dir_ab = (to_1 - from_1).normalize_or_zero();
            let dir_bc = (to_2 - from_2).normalize_or_zero();
            let dot = dir_ab.dot(dir_bc);
            let diff = target - dot;
            let p_a = (dir_ab.cross((dir_ab).cross(dir_bc))).normalize();
            let p_c = (dir_bc.cross((dir_ab).cross(dir_bc))).normalize();
            let k = ANGLE_ALIGNMENT_STRENGTH;
            let f_a = k * diff / dir_ab.length() * p_a;
            let f_c = k * diff / dir_bc.length() * p_c;
            let f_b = -f_a - f_c;
            if f_a.is_nan() || f_b.is_nan() || f_c.is_nan() {
                continue;
            }
            if let Ok(mut dynamics_a) = world.get::<&mut SpatialDynamics>(connection_1.from) {
                dynamics_a.acceleration += f_a;
            }
            if let Ok(mut dynamics_b) = world.get::<&mut SpatialDynamics>(connection_1.to) {
                dynamics_b.acceleration += f_b;
            }
            if let Ok(mut dynamics_c) = world.get::<&mut SpatialDynamics>(connection_2.to) {
                dynamics_c.acceleration += f_c;
            }
        }
    }

    for (_, connection) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        if let (Ok(from), Ok(to)) = (
            world.get::<&Position>(connection.from),
            world.get::<&Position>(connection.to),
        ) {
            let r2 = from.position.distance_squared(to.position);
            let d = to.position - from.position;
            let target_length = 2.0 * NODE_RADIUS;
            let force = SPRING_STRENGTH * (r2 - target_length.powi(2)) * d.normalize_or_zero();
            if let Ok(mut dynamics_from) = world.get::<&mut SpatialDynamics>(connection.from) {
                dynamics_from.acceleration += force;
            }
            if let Ok(mut dynamics_to) = world.get::<&mut SpatialDynamics>(connection.to) {
                dynamics_to.acceleration -= force;
            }
        }
    }
}

pub fn integrate_motion(world: &mut hecs::World, dt: f64) {
    for (_, (position, dynamics)) in world.query_mut::<(&mut Position, &mut SpatialDynamics)>() {
        // Guard against NaN/Inf from force calculations
        if !dynamics.acceleration.is_finite() {
            dynamics.acceleration = Vec3::ZERO;
        }
        if !dynamics.velocity.is_finite() {
            dynamics.velocity = Vec3::ZERO;
        }
        let gravity = -position.position.y;
        dynamics.acceleration += Vec3::new(0.0, gravity, 0.0);
        dynamics.velocity += dynamics.acceleration * dt as f32;
        position.position += dynamics.velocity * dt as f32;
        dynamics.acceleration = Vec3::ZERO;
        dynamics.velocity -= dynamics.velocity * dt as f32;
        if !position.position.is_finite() {
            position.position = Vec3::ZERO;
        }
    }
}
