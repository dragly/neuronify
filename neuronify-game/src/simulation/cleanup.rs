use std::collections::HashSet;

use neuronify_core::{Compartment, Connection};

/// Clean up orphaned connections (where `from` or `to` no longer exists) and
/// orphaned compartments (not referenced by any surviving connection).
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

    // Second pass: remove compartments not referenced by any surviving connection.
    let mut connected_entities: HashSet<hecs::Entity> = HashSet::new();
    for (_, conn) in world.query::<&Connection>().iter() {
        connected_entities.insert(conn.from);
        connected_entities.insert(conn.to);
    }
    let orphan_compartments: Vec<hecs::Entity> = world
        .query::<&Compartment>()
        .iter()
        .filter(|(e, _)| !connected_entities.contains(e))
        .map(|(e, _)| e)
        .collect();
    for entity in orphan_compartments {
        let _ = world.despawn(entity);
    }
}
