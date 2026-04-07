use std::collections::{HashMap, HashSet, VecDeque};

use hecs::Entity;

use neuronify_core::{Compartment, CompartmentCurrent, Connection, LeakyNeuron};

use crate::components::*;

/// Propagate ownership from origin neurons through the connection graph via BFS.
pub fn update_ownership(world: &mut hecs::World) {
    let origins: Vec<(Entity, PlayerId)> = world
        .query::<&OriginNeuron>()
        .iter()
        .map(|(e, o)| (e, o.player))
        .collect();

    // Build adjacency from CompartmentCurrent connections (undirected)
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        adjacency.entry(conn.from).or_default().push(conn.to);
        adjacency.entry(conn.to).or_default().push(conn.from);
    }

    // BFS reachability per player
    let mut reachable: HashMap<PlayerId, HashSet<Entity>> = HashMap::new();
    for (origin_entity, player) in &origins {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(*origin_entity);
        visited.insert(*origin_entity);
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if visited.contains(&neighbor) {
                        continue;
                    }
                    let is_neuron = world.get::<&LeakyNeuron>(neighbor).is_ok();
                    let is_compartment = world.get::<&Compartment>(neighbor).is_ok();
                    let is_glial = world.get::<&GlialCell>(neighbor).is_ok();
                    if is_neuron || is_compartment || is_glial {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        reachable.insert(*player, visited);
    }

    // Helper to assign ownership based on reachability
    fn assign_ownership(
        world: &mut hecs::World,
        entity: Entity,
        reachable: &HashMap<PlayerId, HashSet<Entity>>,
    ) {
        let p1_owns = reachable
            .get(&PlayerId::Player1)
            .is_some_and(|s| s.contains(&entity));
        if p1_owns {
            let _ = world.insert_one(
                entity,
                Ownership {
                    player: PlayerId::Player1,
                },
            );
        } else {
            let _ = world.remove_one::<Ownership>(entity);
        }
    }

    // Assign ownership to neurons
    let all_neurons: Vec<Entity> = world
        .query::<&LeakyNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in all_neurons {
        assign_ownership(world, entity, &reachable);
    }

    // Assign ownership to compartments
    let all_compartments: Vec<Entity> = world
        .query::<&Compartment>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in all_compartments {
        assign_ownership(world, entity, &reachable);
    }

    // Assign ownership to glial cells
    let all_glials: Vec<Entity> = world.query::<&GlialCell>().iter().map(|(e, _)| e).collect();
    for entity in all_glials {
        assign_ownership(world, entity, &reachable);
    }
}
