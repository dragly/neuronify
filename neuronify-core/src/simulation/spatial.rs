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

    // ── Cylinder-to-point repulsion ──────────────────────────────────────
    // Push compartments away from nearby cylinder segments they don't belong to.
    // This prevents axon/dendrite chains from overlapping visually.

    // Collect all connection segments with positions.
    let segments: Vec<(Entity, Entity, Vec3, Vec3)> = connections
        .iter()
        .filter_map(|(_, c)| {
            let from = world.get::<&Position>(c.from).ok()?.position;
            let to = world.get::<&Position>(c.to).ok()?.position;
            Some((c.from, c.to, from, to))
        })
        .collect();

    let cylinder_repulsion = 0.8;
    let min_sep = NODE_RADIUS * 1.2;
    let min_sep2 = min_sep * min_sep;

    // Build adjacency: which entities are directly connected to each entity.
    let mut neighbors: std::collections::HashSet<(Entity, Entity)> = std::collections::HashSet::new();
    for &(seg_from, seg_to, _, _) in &segments {
        neighbors.insert((seg_from, seg_to));
        neighbors.insert((seg_to, seg_from));
    }

    for (id, (position, dynamics)) in world.query_mut::<(&Position, &mut SpatialDynamics)>() {
        let p = position.position;
        for &(seg_from, seg_to, a, b) in &segments {
            // Skip segments that share this entity or a neighbor as endpoint.
            if id == seg_from || id == seg_to {
                continue;
            }
            // Skip if this entity is directly connected to either segment endpoint
            // (would fight the spring forces in the same chain).
            if neighbors.contains(&(id, seg_from)) || neighbors.contains(&(id, seg_to)) {
                continue;
            }
            // Closest point on segment AB to point P.
            let ab = b - a;
            let len2 = ab.length_squared();
            if len2 < 1e-6 {
                continue;
            }
            let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
            let closest = a + ab * t;
            let diff = p - closest;
            let dist2 = diff.length_squared();
            // Use XZ distance only to detect overlap (cylinders live in the ground plane).
            let diff_xz = Vec3::new(diff.x, 0.0, diff.z);
            let dist_xz2 = diff_xz.length_squared();
            if dist_xz2 < min_sep2 && dist_xz2 > 1e-6 {
                // Push in Y direction: up if above the segment, down if below,
                // with a bias upward so crossing axons bridge over each other.
                let y_sign = if diff.y >= 0.0 { 1.0 } else { -1.0 };
                let push = cylinder_repulsion * (min_sep2 - dist_xz2);
                dynamics.acceleration.y += y_sign * push;
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
        // Ground plane: prevent entities from going below y=0.
        if position.position.y < 0.0 {
            position.position.y = 0.0;
            dynamics.velocity.y = dynamics.velocity.y.max(0.0);
        }
        if !position.position.is_finite() {
            position.position = Vec3::ZERO;
        }
    }
}
