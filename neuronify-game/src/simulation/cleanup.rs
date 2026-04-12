use std::collections::HashSet;

use neuronify_core::{Compartment, Connection};

use crate::components::Ownership;

/// Clean up orphaned connections (where `from` or `to` no longer exists) and
/// orphaned compartments (not referenced by any surviving connection).
/// Player-owned orphan compartments are kept so the player can erase them manually.
pub fn cleanup_orphans(world: &mut hecs::World) {
    // First pass: remove Connection entities whose endpoints have been despawned.
    let broken_connections: Vec<hecs::Entity> = world
        .query::<&Connection>()
        .iter()
        .filter(|(_, c)| !world.contains(c.from) || !world.contains(c.to))
        .map(|(e, _)| e)
        .collect();
    for entity in broken_connections {
        let _ = world.despawn(entity);
    }

    // Second pass: remove orphaned compartments not referenced by any connection.
    // Player-owned compartments are kept — the player can erase them manually.
    let mut connected_entities: HashSet<hecs::Entity> = HashSet::new();
    for (_, conn) in world.query::<&Connection>().iter() {
        connected_entities.insert(conn.from);
        connected_entities.insert(conn.to);
    }
    let orphan_compartments: Vec<hecs::Entity> = world
        .query::<&Compartment>()
        .iter()
        .filter(|(e, _)| {
            if connected_entities.contains(e) {
                return false; // still connected, not an orphan
            }
            // Keep player-owned orphans for manual cleanup.
            world.get::<&Ownership>(*e).is_err()
        })
        .map(|(e, _)| e)
        .collect();
    for entity in orphan_compartments {
        let _ = world.despawn(entity);
    }
}
