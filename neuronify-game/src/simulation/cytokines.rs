//! Cytokine particle system — mast cell emission, diffusion, and macrophage activation.
//!
//! # Call order each frame
//!
//!   1. `emit_cytokines`       — mast cells emit particles when driver fired recently
//!   2. `tick_cytokines`       — particles drift outward and age; expired ones despawn
//!   3. `activate_macrophages` — macrophages within particle radius become active

use glam::Vec3;

use neuronify_core::{LeakyDynamics, GeneratorDynamics, Position};

use crate::components::{CytokineParticle, MacrophageActivation, MacrophageUnit, MastCell};
use crate::constants::{
    CYTOKINE_LIFETIME, CYTOKINE_SPEED, MAST_CELL_CYTOKINE_RADIUS,
    MAST_CELL_EMIT_INTERVAL, MAST_CELL_PARTICLES_PER_BURST,
};

/// Driver-fired window: a driver neuron counts as "recently fired" if its
/// `time_since_fire` is less than this threshold.
const RECENT_FIRE_WINDOW: f64 = 0.5;

/// Emit cytokine bursts from mast cells whose driver neuron has fired recently.
pub fn emit_cytokines(world: &mut hecs::World, dt: f32) {
    // Collect driver firing state without borrowing world mutably.
    let mast_cells: Vec<(hecs::Entity, Vec3, hecs::Entity, f32)> = world
        .query::<(&MastCell, &Position)>()
        .iter()
        .map(|(e, (mc, pos))| (e, pos.position, mc.driver, mc.emit_timer))
        .collect();

    let mut particles_to_spawn: Vec<(Vec3, Vec3)> = Vec::new(); // (origin, velocity)

    for (entity, origin, driver, timer) in mast_cells {
        // Check if driver fired recently (LeakyDynamics or GeneratorDynamics).
        let driver_fired = {
            let lif = world.get::<&LeakyDynamics>(driver)
                .map(|d| d.time_since_fire < RECENT_FIRE_WINDOW)
                .unwrap_or(false);
            let gen = world.get::<&GeneratorDynamics>(driver)
                .map(|d| d.time_since_fire < RECENT_FIRE_WINDOW)
                .unwrap_or(false);
            lif || gen
        };

        let new_timer = timer - dt;
        if let Ok(mut mc) = world.get::<&mut MastCell>(entity) {
            mc.emit_timer = new_timer;
        }

        if !driver_fired || new_timer > 0.0 {
            continue;
        }

        // Reset timer and emit a burst of particles.
        if let Ok(mut mc) = world.get::<&mut MastCell>(entity) {
            mc.emit_timer = MAST_CELL_EMIT_INTERVAL;
        }

        for i in 0..MAST_CELL_PARTICLES_PER_BURST {
            // Evenly distribute outward directions in the XZ plane.
            let angle = std::f32::consts::TAU * i as f32 / MAST_CELL_PARTICLES_PER_BURST as f32;
            // Small upward component so they drift visibly above the ground plane.
            let velocity = Vec3::new(angle.cos(), 0.08, angle.sin()) * CYTOKINE_SPEED;
            particles_to_spawn.push((origin, velocity));
        }
    }

    for (origin, velocity) in particles_to_spawn {
        world.spawn((
            Position { position: origin },
            CytokineParticle {
                age: 0.0,
                lifetime: CYTOKINE_LIFETIME,
                velocity,
            },
        ));
    }
}

/// Move cytokine particles outward and despawn those that have expired.
pub fn tick_cytokines(world: &mut hecs::World, dt: f32) {
    let particle_entities: Vec<hecs::Entity> = world
        .query::<&CytokineParticle>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    let mut to_despawn = Vec::new();

    for entity in particle_entities {
        let expired = if let Ok(mut p) = world.get::<&mut CytokineParticle>(entity) {
            p.age += dt;
            p.age >= p.lifetime
        } else {
            false
        };

        if expired {
            to_despawn.push(entity);
            continue;
        }

        // Advance position.
        if let Ok(particle) = world.get::<&CytokineParticle>(entity) {
            let vel = particle.velocity;
            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                pos.position += vel * dt;
            }
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

/// Set `MacrophageActivation.active` based on cytokine proximity.
///
/// A macrophage is active if any `CytokineParticle` exists within
/// `MAST_CELL_CYTOKINE_RADIUS` world units of it.  Macrophages without a
/// `MacrophageActivation` component are permanently active (legacy scenarios).
pub fn activate_macrophages(world: &mut hecs::World) {
    // Collect live cytokine positions.
    let cytokine_positions: Vec<Vec3> = world
        .query::<(&CytokineParticle, &Position)>()
        .iter()
        .map(|(_, (_, pos))| pos.position)
        .collect();

    let macrophage_entities: Vec<(hecs::Entity, Vec3)> = world
        .query::<(&MacrophageUnit, &Position)>()
        .with::<&MacrophageActivation>()
        .iter()
        .map(|(e, (_, pos))| (e, pos.position))
        .collect();

    for (entity, mac_pos) in macrophage_entities {
        let active = cytokine_positions
            .iter()
            .any(|&cp| cp.distance(mac_pos) <= MAST_CELL_CYTOKINE_RADIUS);

        if let Ok(mut activation) = world.get::<&mut MacrophageActivation>(entity) {
            activation.active = active;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_core::{LeakyDynamics, Position};

    fn world_with_driver_and_mast_cell() -> (hecs::World, hecs::Entity, hecs::Entity) {
        let mut world = hecs::World::new();

        // Driver neuron that just fired.
        let mut dyn_ = LeakyDynamics::default();
        dyn_.time_since_fire = 0.01; // < RECENT_FIRE_WINDOW
        let driver = world.spawn((Position { position: Vec3::ZERO }, dyn_));

        let mast = crate::spawning::spawn_mast_cell(&mut world, Vec3::new(5.0, 0.0, 0.0), driver);
        (world, driver, mast)
    }

    /// Mast cell emits particles when its driver fired recently.
    #[test]
    fn test_mast_cell_emits_when_driver_fires() {
        let (mut world, _driver, _mast) = world_with_driver_and_mast_cell();

        let before: usize = world.query::<&CytokineParticle>().iter().count();
        emit_cytokines(&mut world, 0.016);
        let after: usize = world.query::<&CytokineParticle>().iter().count();

        assert!(after > before, "mast cell should emit cytokines when driver fires");
        assert_eq!(after - before, MAST_CELL_PARTICLES_PER_BURST,
            "should emit exactly MAST_CELL_PARTICLES_PER_BURST particles");
    }

    /// Mast cell does NOT emit when driver has not fired recently.
    #[test]
    fn test_mast_cell_silent_when_driver_quiet() {
        let mut world = hecs::World::new();
        let mut dyn_ = LeakyDynamics::default();
        dyn_.time_since_fire = 10.0; // > RECENT_FIRE_WINDOW
        let driver = world.spawn((Position { position: Vec3::ZERO }, dyn_));
        crate::spawning::spawn_mast_cell(&mut world, Vec3::new(5.0, 0.0, 0.0), driver);

        emit_cytokines(&mut world, 0.016);

        let count: usize = world.query::<&CytokineParticle>().iter().count();
        assert_eq!(count, 0, "no cytokines when driver has not fired");
    }

    /// Macrophage with MacrophageActivation activates when cytokine is nearby.
    #[test]
    fn test_macrophage_activates_in_cytokine_cloud() {
        let (mut world, _driver, _mast) = world_with_driver_and_mast_cell();

        // Spawn macrophage near the mast cell.
        let mac = world.spawn((
            Position { position: Vec3::new(5.0, 0.0, 0.0) },
            MacrophageUnit,
            MacrophageActivation { active: false },
        ));

        emit_cytokines(&mut world, 0.016);
        tick_cytokines(&mut world, 0.016);
        activate_macrophages(&mut world);

        let active = world.get::<&MacrophageActivation>(mac)
            .map(|a| a.active)
            .unwrap_or(false);
        assert!(active, "macrophage should be active when inside cytokine cloud");
    }

    /// Macrophage goes dormant after cytokines dissipate.
    #[test]
    fn test_macrophage_goes_dormant_when_cytokines_dissipate() {
        let mut world = hecs::World::new();

        // Manually spawn a single cytokine particle near the macrophage.
        let mac_pos = Vec3::new(10.0, 0.0, 0.0);
        let mac = world.spawn((
            Position { position: mac_pos },
            MacrophageUnit,
            MacrophageActivation { active: true },
        ));

        // Emit one cytokine at the macrophage position, then expire it.
        let particle = world.spawn((
            Position { position: mac_pos },
            CytokineParticle { age: 0.0, lifetime: 0.1, velocity: Vec3::ZERO },
        ));

        // Expire the particle.
        tick_cytokines(&mut world, 1.0); // age >> lifetime → despawned
        assert!(!world.contains(particle), "particle should be despawned");

        // Re-check activation: no particles → dormant.
        activate_macrophages(&mut world);
        let active = world.get::<&MacrophageActivation>(mac)
            .map(|a| a.active)
            .unwrap_or(true);
        assert!(!active, "macrophage should be dormant when cytokines dissipate");
    }

    /// Cytokine particles drift outward over time.
    #[test]
    fn test_cytokines_drift_outward() {
        let mut world = hecs::World::new();
        let origin = Vec3::ZERO;
        world.spawn((
            Position { position: origin },
            CytokineParticle {
                age: 0.0,
                lifetime: 10.0,
                velocity: Vec3::new(CYTOKINE_SPEED, 0.0, 0.0),
            },
        ));

        tick_cytokines(&mut world, 1.0);

        let pos = world.query::<(&CytokineParticle, &Position)>()
            .iter()
            .next()
            .map(|(_, (_, p))| p.position)
            .unwrap();
        assert!(pos.x > origin.x, "cytokine should drift away from origin");
    }
}
