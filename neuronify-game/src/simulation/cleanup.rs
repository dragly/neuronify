use std::collections::HashSet;

use neuronify_core::{Compartment, Connection};

/// Clean up orphaned compartments (not connected to anything).
pub fn cleanup_orphans(world: &mut hecs::World) {
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
