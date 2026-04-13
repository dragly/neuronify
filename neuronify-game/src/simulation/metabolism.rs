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
            // Detect firing from either LeakyDynamics or GeneratorDynamics.
            let lif_fired = d.time_since_fire < dt * 2.0;
            let gen_fired = world
                .get::<&neuronify_core::GeneratorDynamics>(e)
                .map(|gd| gd.time_since_fire < dt * 2.0)
                .unwrap_or(false);
            let just_fired = lif_fired || gen_fired;
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
            // Require enough energy for at least one firing event to re-enable.
            let threshold = if dynamics.enabled { 0.0 } else { FIRING_METABOLIC_COST };
            let has_energy = energy > threshold;
            let refractory_done = dynamics.time_since_fire >= dynamics.refractory_period;
            dynamics.enabled = has_energy && refractory_done;
        }
    }
}

pub fn resource_flow(world: &mut hecs::World, dt: f64) {
    let energy_map: HashMap<Entity, f64> = world
        .query::<&MetabolicState>()
        .iter()
        .map(|(e, m)| (e, m.energy))
        .collect();

    // Collect dormant neurons — resource flow should not push energy
    // INTO a dormant neuron. Only external sources (glial lactate) can
    // wake them up. This prevents two exhausted neurons from drip-feeding
    // each other back to life indefinitely.
    let dormant_neurons: std::collections::HashSet<Entity> = world
        .query::<(&LeakyNeuron, &LeakyDynamics)>()
        .iter()
        .filter(|(_, (_, ld))| !ld.enabled)
        .map(|(e, _)| e)
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
                    // Don't flow energy into dormant neurons.
                    let receiver = if diff > 0.0 { n } else { neuron };
                    if dormant_neurons.contains(&receiver) {
                        continue;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use neuronify_core::{
        LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronType, Position,
        RegularSpikeGenerator, GeneratorDynamics, Compartment, CompartmentCurrent,
        Connection, StaticConnectionSource,
    };

    /// Helper: spawn a neuron with standard components.
    fn spawn_test_neuron(world: &mut hecs::World, pos: Vec3, auto_fire_hz: f64) -> hecs::Entity {
        let entity = world.spawn((
            Position { position: pos },
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
            MetabolicState::default(),
            RegularSpikeGenerator { frequency: auto_fire_hz },
            GeneratorDynamics::default(),
        ));
        entity
    }

    /// Helper: connect two neurons with a bridge compartment.
    fn connect_neurons(world: &mut hecs::World, from: hecs::Entity, to: hecs::Entity) {
        let from_pos = world.get::<&Position>(from).unwrap().position;
        let to_pos = world.get::<&Position>(to).unwrap().position;
        let mid = (from_pos + to_pos) * 0.5;
        let comp = world.spawn((
            Position { position: mid },
            NeuronType::Excitatory,
            Compartment {
                voltage: -10.0, m: -0.625, h: 0.0, n: 0.0,
                influence: 0.0, capacitance: 1.0,
                injected_current: 0.0, fire_impulse: 0.0,
            },
            StaticConnectionSource {},
        ));
        // from → comp
        world.spawn((
            Connection { from, to: comp, strength: 1.0, directional: false },
            CompartmentCurrent { capacitance: neuronify_core::COUPLING_CAPACITANCE },
        ));
        // comp → to
        world.spawn((
            Connection { from: comp, to, strength: 1.0, directional: true },
            CompartmentCurrent { capacitance: neuronify_core::COUPLING_CAPACITANCE },
        ));
    }

    /// Run one simulation step: LIF + metabolism + dormancy.
    fn sim_step(world: &mut hecs::World, dt: f64) {
        neuronify_core::lif_step(world, dt, 0.0);
        metabolic_drain(world, dt);
        apply_dormancy(world);
        resource_flow(world, dt);
    }

    /// Two neurons in a loop should exhaust their energy and go dormant.
    #[test]
    fn two_neuron_loop_exhausts_without_glial() {
        let mut world = hecs::World::new();
        let a = spawn_test_neuron(&mut world, Vec3::new(0.0, 0.0, 0.0), 10.0);
        let b = spawn_test_neuron(&mut world, Vec3::new(5.0, 0.0, 0.0), 10.0);
        connect_neurons(&mut world, a, b);
        connect_neurons(&mut world, b, a);

        let dt = 0.001; // 1ms per step
        // Run for 30 simulated seconds (30000 steps).
        for _ in 0..30_000 {
            sim_step(&mut world, dt);
        }

        // Both neurons should be at zero energy (dormant).
        let energy_a = world.get::<&MetabolicState>(a).unwrap().energy;
        let energy_b = world.get::<&MetabolicState>(b).unwrap().energy;
        assert!(
            energy_a <= 0.0,
            "Neuron A should be exhausted, got energy={energy_a}"
        );
        assert!(
            energy_b <= 0.0,
            "Neuron B should be exhausted, got energy={energy_b}"
        );

        // Both should be dormant (disabled).
        let enabled_a = world.get::<&LeakyDynamics>(a).unwrap().enabled;
        let enabled_b = world.get::<&LeakyDynamics>(b).unwrap().enabled;
        assert!(!enabled_a, "Neuron A should be dormant");
        assert!(!enabled_b, "Neuron B should be dormant");
    }

    /// Two plain neurons (no auto-fire) in a loop: one is kicked above threshold.
    /// The reverberating activity should exhaust them.
    #[test]
    fn two_plain_neuron_loop_exhausts() {
        let mut world = hecs::World::new();
        // No auto-fire — just plain leaky neurons.
        let a = world.spawn((
            Position { position: Vec3::ZERO },
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
            MetabolicState::default(),
        ));
        let b = world.spawn((
            Position { position: Vec3::new(5.0, 0.0, 0.0) },
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
            MetabolicState::default(),
        ));
        connect_neurons(&mut world, a, b);
        connect_neurons(&mut world, b, a);

        // Kick neuron A above threshold to start the loop.
        if let Ok(mut d) = world.get::<&mut LeakyDynamics>(a) {
            d.voltage = 100.0; // well above threshold
        }

        let dt = 0.001;
        for _ in 0..60_000 { // 60 seconds
            sim_step(&mut world, dt);
        }

        // Both neurons should be near or below the dormancy threshold.
        // They may oscillate slightly above it due to resource flow, but
        // should be far below their starting energy of 80.
        let energy_a = world.get::<&MetabolicState>(a).unwrap().energy;
        let energy_b = world.get::<&MetabolicState>(b).unwrap().energy;
        assert!(
            energy_a < 10.0 && energy_b < 10.0,
            "Both neurons should be near-exhausted, got a={energy_a}, b={energy_b}"
        );
        // Total energy should be a tiny fraction of starting (160 total).
        let total = energy_a + energy_b;
        assert!(
            total < 15.0,
            "Total energy should be very low, got {total}"
        );
    }

    /// Two neurons in a loop WITH external energy supply should sustain firing.
    /// Simulates glial support by injecting energy each step.
    #[test]
    fn two_neuron_loop_sustains_with_energy_supply() {
        let mut world = hecs::World::new();
        let a = spawn_test_neuron(&mut world, Vec3::new(0.0, 0.0, 0.0), 10.0);
        let b = spawn_test_neuron(&mut world, Vec3::new(5.0, 0.0, 0.0), 10.0);
        connect_neurons(&mut world, a, b);
        connect_neurons(&mut world, b, a);

        let dt = 0.001;
        let energy_injection_per_step = 0.06; // ~60/sec, enough to sustain two firing neurons

        for _ in 0..30_000 {
            sim_step(&mut world, dt);
            // Simulate glial energy supply.
            if let Ok(mut m) = world.get::<&mut MetabolicState>(a) {
                m.energy = (m.energy + energy_injection_per_step).min(m.max_energy);
            }
            if let Ok(mut m) = world.get::<&mut MetabolicState>(b) {
                m.energy = (m.energy + energy_injection_per_step).min(m.max_energy);
            }
        }

        let energy_a = world.get::<&MetabolicState>(a).unwrap().energy;
        let energy_b = world.get::<&MetabolicState>(b).unwrap().energy;
        let total_energy = energy_a + energy_b;
        assert!(
            total_energy > 10.0,
            "Energy-supplied neurons should retain energy, got total={total_energy} (a={energy_a}, b={energy_b})"
        );

        // Both should still be enabled (not dormant).
        let enabled_a = world.get::<&LeakyDynamics>(a).unwrap().enabled;
        let enabled_b = world.get::<&LeakyDynamics>(b).unwrap().enabled;
        assert!(enabled_a || enabled_b, "At least one neuron should still be active");
    }
}
