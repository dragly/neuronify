use std::collections::{HashMap, HashSet, VecDeque};

use hecs::Entity;

use neuronify_core::{Compartment, CompartmentCurrent, Connection, LeakyDynamics, LeakyNeuron};

use crate::components::*;
use crate::constants::*;

pub fn metabolic_drain(world: &mut hecs::World, dt: f64) {
    // Count outgoing connections per neuron (exclude dendrite connections — they're free)
    let mut outgoing_count: HashMap<Entity, usize> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        if world.get::<&Dendrite>(conn.to).is_err() {
            *outgoing_count.entry(conn.from).or_insert(0) += 1;
        }
    }

    // Count non-dendrite compartments owned by each neuron
    let mut compartment_counts: HashMap<Entity, usize> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        if world.get::<&LeakyNeuron>(conn.from).is_ok()
            && world.get::<&Compartment>(conn.to).is_ok()
            && world.get::<&Dendrite>(conn.to).is_err()
        {
            *compartment_counts.entry(conn.from).or_insert(0) += 1;
        }
    }

    let neuron_entities: Vec<(Entity, bool, bool)> = world
        .query::<(&LeakyNeuron, &LeakyDynamics, &MetabolicState)>()
        .iter()
        .map(|(e, (_, d, _))| {
            let just_fired = d.time_since_fire < dt * 2.0;
            let is_origin = world.get::<&OriginNeuron>(e).is_ok();
            (e, just_fired, is_origin)
        })
        .collect();

    for (entity, just_fired, is_origin) in neuron_entities {
        if let Ok(mut metab) = world.get::<&mut MetabolicState>(entity) {
            metab.energy -= RESTING_METABOLIC_COST * dt;
            if just_fired {
                metab.energy -= FIRING_METABOLIC_COST;
            }
            let conn_count = outgoing_count.get(&entity).copied().unwrap_or(0);
            metab.energy -= CONNECTION_MAINTENANCE_COST * conn_count as f64 * dt;
            let comp_count = compartment_counts.get(&entity).copied().unwrap_or(0);
            metab.energy -= COMPARTMENT_METABOLIC_COST * comp_count as f64 * dt;

            if is_origin {
                metab.energy = metab.energy.max(ORIGIN_MIN_ENERGY);
            } else {
                metab.energy = metab.energy.max(0.0);
            }
        }
    }
}

/// Disable neurons at 0 energy (dormancy). Re-enable when energy is restored.
pub fn apply_dormancy(world: &mut hecs::World) {
    let entities: Vec<Entity> = world
        .query::<(&LeakyNeuron, &MetabolicState)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in entities {
        let energy = world
            .get::<&MetabolicState>(entity)
            .map(|m| m.energy)
            .unwrap_or(0.0);
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
            dynamics.enabled = energy > 0.0;
        }
    }
}

pub fn resource_flow(world: &mut hecs::World, dt: f64) {
    let energy_map: HashMap<Entity, f64> = world
        .query::<&MetabolicState>()
        .iter()
        .map(|(e, m)| (e, m.energy))
        .collect();

    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (_, conn) in world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
    {
        adjacency.entry(conn.from).or_default().push(conn.to);
        adjacency.entry(conn.to).or_default().push(conn.from);
    }

    let neuron_entities: HashSet<Entity> = energy_map.keys().copied().collect();
    let mut flows: Vec<(Entity, Entity, f64)> = Vec::new();
    let mut visited_pairs: HashSet<(Entity, Entity)> = HashSet::new();

    for &neuron in &neuron_entities {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(neuron);
        if let Some(neighbors) = adjacency.get(&neuron) {
            for &n in neighbors {
                if !neuron_entities.contains(&n) {
                    queue.push_back(n);
                    visited.insert(n);
                } else if !visited_pairs.contains(&(n, neuron)) {
                    visited_pairs.insert((neuron, n));
                    let from_e = energy_map[&neuron];
                    let to_e = energy_map[&n];
                    let diff = from_e - to_e;
                    let max_flow = (from_e * 0.5).min(to_e * 0.5 + from_e * 0.5);
                    let flow = (RESOURCE_FLOW_RATE * diff * dt).clamp(-max_flow, max_flow);
                    flows.push((neuron, n, flow));
                }
            }
        }
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&current) {
                for &n in neighbors {
                    if visited.contains(&n) {
                        continue;
                    }
                    visited.insert(n);
                    if neuron_entities.contains(&n) && !visited_pairs.contains(&(n, neuron)) {
                        visited_pairs.insert((neuron, n));
                        let from_e = energy_map[&neuron];
                        let to_e = energy_map[&n];
                        let diff = from_e - to_e;
                        let max_flow = (from_e * 0.5).min(to_e * 0.5 + from_e * 0.5);
                        let flow = (RESOURCE_FLOW_RATE * diff * dt).clamp(-max_flow, max_flow);
                        flows.push((neuron, n, flow));
                    } else if !neuron_entities.contains(&n) {
                        queue.push_back(n);
                    }
                }
            }
        }
    }

    for (from, to, flow) in flows {
        if let Ok(mut m) = world.get::<&mut MetabolicState>(from) {
            m.energy = (m.energy - flow).max(0.0);
        }
        if let Ok(mut m) = world.get::<&mut MetabolicState>(to) {
            m.energy = (m.energy + flow).min(m.max_energy);
        }
    }
}
