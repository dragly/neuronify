use std::collections::{HashMap, HashSet};

use crate::components::*;
use crate::constants::*;

pub fn fhn_step(world: &mut hecs::World, cdt: f64, recently_fired: &HashSet<hecs::Entity>) {
    for (_, compartment) in world.query_mut::<&mut Compartment>() {
        let v = (compartment.voltage - FHN_OFFSET) / FHN_SCALE;
        let w = compartment.m;
        let dv = FHN_TAU * (v - v * v * v / 3.0 - w);
        let dw = FHN_TAU * FHN_EPS * (v + FHN_A - FHN_B * w);
        let new_v = v + dv * cdt;
        let new_w = w + dw * cdt;
        compartment.voltage = new_v * FHN_SCALE + FHN_OFFSET;
        compartment.m = new_w;
    }

    let mut new_compartments: HashMap<hecs::Entity, Compartment> = world
        .query::<&Compartment>()
        .iter()
        .map(|(entity, &compartment)| (entity, compartment))
        .collect();

    for (_, (connection, current)) in world.query::<(&Connection, &CompartmentCurrent)>().iter() {
        if let Ok(_compartment_to) = world.get::<&Compartment>(connection.to) {
            if recently_fired.contains(&connection.from) {
                let new_compartment_to = new_compartments
                    .get_mut(&connection.to)
                    .expect("Could not get new compartment");
                new_compartment_to.voltage = FHN_FIRE_VOLTAGE * FHN_SCALE + FHN_OFFSET;
            } else if let Ok(compartment_from) = world.get::<&Compartment>(connection.from) {
                let voltage_diff = compartment_from.voltage - _compartment_to.voltage;
                let delta_voltage = voltage_diff / current.capacitance;
                let new_compartment_to = new_compartments
                    .get_mut(&connection.to)
                    .expect("Could not get new compartment");
                new_compartment_to.voltage += delta_voltage * cdt;
                let new_compartment_from = new_compartments
                    .get_mut(&connection.from)
                    .expect("Could not get new compartment");
                new_compartment_from.voltage -= delta_voltage * cdt;
            }
        }
    }

    for (compartment_id, new_compartment) in new_compartments {
        let mut old_compartment = world
            .get::<&mut Compartment>(compartment_id)
            .expect("Could not find compartment");
        *old_compartment = new_compartment;
    }

    let compartment_to_neuron: Vec<(hecs::Entity, f64)> = world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .filter_map(|(_, conn)| {
            let compartment = world.get::<&Compartment>(conn.from).ok()?;
            let excess =
                (compartment.voltage - BRIDGE_VOLTAGE_THRESHOLD).clamp(0.0, BRIDGE_VOLTAGE_CLAMP);
            if excess == 0.0 {
                return None;
            }
            let current = excess / BRIDGE_VOLTAGE_CLAMP * BRIDGE_CURRENT_SCALE;
            world.get::<&LeakyDynamics>(conn.to).ok()?;
            let sign = match world.get::<&NeuronType>(conn.from) {
                Ok(nt) => match *nt {
                    NeuronType::Excitatory => 1.0,
                    NeuronType::Inhibitory => -1.0,
                },
                Err(_) => 1.0,
            };
            Some((conn.to, sign * current))
        })
        .collect();

    for (target, current) in compartment_to_neuron {
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(target) {
            dynamics.received_currents += current;
        }
    }
}

#[cfg(test)]
mod axon_tests {
    use super::*;
    use crate::simulation::lif::lif_step;
    use glam::Vec3;

    #[test]
    fn test_axon_action_potential_triggers_target_neuron() {
        let mut world = hecs::World::new();

        let neuron_a = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 0.0),
            },
            NeuronType::Excitatory,
        ));

        let clamp = world.spawn((
            CurrentClamp {
                current_output: 5e-9,
            },
            Position {
                position: Vec3::new(0.0, 0.0, -1.0),
            },
        ));
        world.spawn((
            Connection {
                from: clamp,
                to: neuron_a,
                strength: 1.0,
                directional: true,
            },
            ImmediateFireSynapse::default(),
        ));

        let neuron_b = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 10.0),
            },
            NeuronType::Excitatory,
        ));

        let num_compartments = 5;
        let mut compartment_entities = Vec::new();
        for i in 0..num_compartments {
            let comp = world.spawn((
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                Position {
                    position: Vec3::new(0.0, 0.0, 2.0 * (i + 1) as f32),
                },
                NeuronType::Excitatory,
            ));
            compartment_entities.push(comp);
        }

        world.spawn((
            Connection {
                from: neuron_a,
                to: compartment_entities[0],
                strength: 1.0,
                directional: false,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        for i in 0..num_compartments - 1 {
            world.spawn((
                Connection {
                    from: compartment_entities[i],
                    to: compartment_entities[i + 1],
                    strength: 1.0,
                    directional: false,
                },
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));
        }

        world.spawn((
            Connection {
                from: *compartment_entities.last().unwrap(),
                to: neuron_b,
                strength: 1.0,
                directional: false,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        let lif_dt = LIF_DT;
        let cdt = FHN_CDT;
        let lif_steps_per_frame = 10;
        let total_fhn_steps = 2000;

        let mut time = 0.0;
        let mut neuron_b_fired = false;

        for _ in 0..total_fhn_steps {
            for _ in 0..lif_steps_per_frame {
                lif_step(&mut world, lif_dt, time);
                time += lif_dt;
            }

            let recently_fired: std::collections::HashSet<hecs::Entity> = world
                .query::<&LeakyDynamics>()
                .iter()
                .filter(|(_, d)| d.time_since_fire < lif_steps_per_frame as f64 * lif_dt)
                .map(|(e, _)| e)
                .collect();

            fhn_step(&mut world, cdt, &recently_fired);

            if let Ok(dynamics) = world.get::<&LeakyDynamics>(neuron_b) {
                if dynamics.time_since_fire < lif_steps_per_frame as f64 * lif_dt {
                    neuron_b_fired = true;
                }
            }
        }

        assert!(
            neuron_b_fired,
            "Target neuron should fire after action potential propagates through axon compartment chain"
        );
    }

    /// Helper: build recently_fired set the same way the game does — from LeakyDynamics only.
    /// This is the BUGGY version that misses generator neurons.
    fn recently_fired_leaky_only(
        world: &hecs::World,
        window: f64,
    ) -> std::collections::HashSet<hecs::Entity> {
        world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < window)
            .map(|(e, _)| e)
            .collect()
    }

    /// Helper: build recently_fired set including both LeakyDynamics and GeneratorDynamics.
    fn recently_fired_all(
        world: &hecs::World,
        window: f64,
    ) -> std::collections::HashSet<hecs::Entity> {
        let mut set: std::collections::HashSet<hecs::Entity> = world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < window)
            .map(|(e, _)| e)
            .collect();
        for (e, d) in world.query::<&GeneratorDynamics>().iter() {
            if d.time_since_fire < window {
                set.insert(e);
            }
        }
        set
    }

    #[test]
    fn test_generator_neuron_fires_through_axon() {
        let mut world = hecs::World::new();

        // Origin-style neuron: has both LeakyNeuron and a RegularSpikeGenerator
        let origin = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            RegularSpikeGenerator { frequency: 5.0 },
            GeneratorDynamics::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 0.0),
            },
            NeuronType::Excitatory,
        ));

        // Target neuron
        let target = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 10.0),
            },
            NeuronType::Excitatory,
        ));

        // Axon: 3 compartments connecting origin -> target
        let num_compartments = 3;
        let mut compartments = Vec::new();
        for i in 0..num_compartments {
            let comp = world.spawn((
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                Position {
                    position: Vec3::new(0.0, 0.0, 2.5 * (i + 1) as f32),
                },
                NeuronType::Excitatory,
            ));
            compartments.push(comp);
        }

        // Wire: origin -> comp[0] -> comp[1] -> comp[2] -> target
        world.spawn((
            Connection {
                from: origin,
                to: compartments[0],
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));
        for i in 0..num_compartments - 1 {
            world.spawn((
                Connection {
                    from: compartments[i],
                    to: compartments[i + 1],
                    strength: 1.0,
                    directional: true,
                },
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));
        }
        world.spawn((
            Connection {
                from: *compartments.last().unwrap(),
                to: target,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        let lif_dt = LIF_DT;
        let cdt = FHN_CDT;
        let lif_steps_per_frame = 10;
        let window = lif_steps_per_frame as f64 * lif_dt;
        // Run for enough time for the generator to fire (0.2s at 5Hz) and signal to propagate
        let total_fhn_steps = 3000;

        let mut time = 0.0;
        let mut target_fired = false;

        for _ in 0..total_fhn_steps {
            for _ in 0..lif_steps_per_frame {
                lif_step(&mut world, lif_dt, time);
                time += lif_dt;
            }

            // Use the CORRECT recently_fired set that includes generators
            let fired = recently_fired_all(&world, window);
            fhn_step(&mut world, cdt, &fired);

            if let Ok(dynamics) = world.get::<&LeakyDynamics>(target) {
                if dynamics.time_since_fire < window {
                    target_fired = true;
                }
            }
        }

        assert!(
            target_fired,
            "Target neuron must fire when origin generator neuron fires through an axon"
        );
    }

    /// Two-hop chain: origin (generator) → B → C, each connected via a bridge compartment.
    /// Verifies that a neuron excited by an upstream AP can itself propagate downstream.
    #[test]
    fn test_two_hop_chain_propagates() {
        let mut world = hecs::World::new();

        let origin = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            RegularSpikeGenerator { frequency: 20.0 },
            GeneratorDynamics::default(),
            Position { position: Vec3::new(0.0, 0.0, 0.0) },
            NeuronType::Excitatory,
        ));
        let neuron_b = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position { position: Vec3::new(0.0, 0.0, 5.0) },
            NeuronType::Excitatory,
        ));
        let neuron_c = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position { position: Vec3::new(0.0, 0.0, 10.0) },
            NeuronType::Excitatory,
        ));

        let bridge_b = world.spawn((
            Compartment { voltage: -10.0, m: -0.625, h: 0.0, n: 0.0, influence: 0.0, capacitance: 1.0, injected_current: 0.0, fire_impulse: 0.0 },
            Position { position: Vec3::new(0.0, 0.0, 3.5) },
            NeuronType::Excitatory,
        ));
        let bridge_c = world.spawn((
            Compartment { voltage: -10.0, m: -0.625, h: 0.0, n: 0.0, influence: 0.0, capacitance: 1.0, injected_current: 0.0, fire_impulse: 0.0 },
            Position { position: Vec3::new(0.0, 0.0, 8.5) },
            NeuronType::Excitatory,
        ));

        // origin → bridge_b → neuron_b
        world.spawn((Connection { from: origin, to: bridge_b, strength: 1.0, directional: true },
                     CompartmentCurrent { capacitance: COUPLING_CAPACITANCE }));
        world.spawn((Connection { from: bridge_b, to: neuron_b, strength: 1.0, directional: true },
                     CompartmentCurrent { capacitance: COUPLING_CAPACITANCE }));
        // neuron_b → bridge_c → neuron_c
        world.spawn((Connection { from: neuron_b, to: bridge_c, strength: 1.0, directional: true },
                     CompartmentCurrent { capacitance: COUPLING_CAPACITANCE }));
        world.spawn((Connection { from: bridge_c, to: neuron_c, strength: 1.0, directional: true },
                     CompartmentCurrent { capacitance: COUPLING_CAPACITANCE }));

        let lif_dt = LIF_DT;
        let cdt = FHN_CDT;
        let iterations: u32 = 4;
        let fire_window = iterations as f64 * lif_dt;
        // 0.5s is enough: origin fires 10x at 20Hz, C should fire at least once
        let total_frames = (0.5 / (iterations as f64 * lif_dt)) as usize;

        let mut time = 0.0;
        let mut c_fired = false;

        for _ in 0..total_frames {
            for _ in 0..iterations {
                lif_step(&mut world, lif_dt, time);
                time += lif_dt;
            }
            let mut fired: std::collections::HashSet<hecs::Entity> = world
                .query::<&LeakyDynamics>()
                .iter()
                .filter(|(_, d)| d.time_since_fire < fire_window)
                .map(|(e, _)| e)
                .collect();
            for (e, d) in world.query::<&GeneratorDynamics>().iter() {
                if d.time_since_fire < fire_window { fired.insert(e); }
            }
            for _ in 0..iterations {
                fhn_step(&mut world, cdt, &fired);
            }
            if let Ok(d) = world.get::<&LeakyDynamics>(neuron_c) {
                if d.time_since_fire < fire_window { c_fired = true; break; }
            }
        }

        assert!(c_fired, "Neuron C must fire when excited through origin→B→C two-hop chain");
    }

    /// Mirrors the exact game loop: 4 LIF steps, build recently_fired (LIF + Generator),
    /// then 4 FHN steps.  Origin (20 Hz generator) connects directly to target via a single
    /// bridge compartment — the minimal topology the Axon tool produces.
    #[test]
    fn test_bridge_compartment_excites_target_game_loop() {
        let mut world = hecs::World::new();

        // Origin neuron with RegularSpikeGenerator at 20 Hz
        let origin = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            RegularSpikeGenerator { frequency: 20.0 },
            GeneratorDynamics::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 0.0),
            },
            NeuronType::Excitatory,
        ));

        // Target neuron — no generator, pure receiver
        let target = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 5.0),
            },
            NeuronType::Excitatory,
        ));

        // Bridge compartment — placed just outside target, no SpatialDynamics
        let bridge = world.spawn((
            Compartment {
                voltage: -10.0,
                m: -0.625,
                h: 0.0,
                n: 0.0,
                influence: 0.0,
                capacitance: 1.0,
                injected_current: 0.0,
                fire_impulse: 0.0,
            },
            Position {
                position: Vec3::new(0.0, 0.0, 3.5),
            },
            NeuronType::Excitatory,
        ));

        // origin → bridge
        world.spawn((
            Connection {
                from: origin,
                to: bridge,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        // bridge → target (bridge mechanism: compartment.voltage > threshold → inject current)
        world.spawn((
            Connection {
                from: bridge,
                to: target,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        let lif_dt = LIF_DT;
        let cdt = FHN_CDT;
        let iterations: u32 = 4; // matches GameApp::iterations default
        let fire_window = iterations as f64 * lif_dt;

        // 0.5s is enough: origin fires 10x at 20Hz; with correct synaptic strength, target fires fast
        let total_frames = (0.5 / (iterations as f64 * lif_dt)) as usize;

        let mut time = 0.0;
        let mut target_fired = false;

        for _ in 0..total_frames {
            // LIF substeps
            for _ in 0..iterations {
                lif_step(&mut world, lif_dt, time);
                time += lif_dt;
            }

            // Build recently_fired exactly as the game does
            let mut fired: std::collections::HashSet<hecs::Entity> = world
                .query::<&LeakyDynamics>()
                .iter()
                .filter(|(_, d)| d.time_since_fire < fire_window)
                .map(|(e, _)| e)
                .collect();
            for (e, d) in world.query::<&GeneratorDynamics>().iter() {
                if d.time_since_fire < fire_window {
                    fired.insert(e);
                }
            }

            // FHN substeps
            for _ in 0..iterations {
                fhn_step(&mut world, cdt, &fired);
            }

            if let Ok(dynamics) = world.get::<&LeakyDynamics>(target) {
                if dynamics.time_since_fire < fire_window {
                    target_fired = true;
                    break;
                }
            }
        }

        assert!(
            target_fired,
            "Target neuron must fire when origin generator fires through a bridge compartment \
             (game loop: {} LIF + {} FHN steps per frame)",
            iterations, iterations
        );
    }

    /// This test demonstrates the bug: using only LeakyDynamics for recently_fired
    /// means generator neurons never trigger axon compartments.
    #[test]
    fn test_generator_not_in_recently_fired_leaky_only() {
        let mut world = hecs::World::new();

        let origin = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            RegularSpikeGenerator { frequency: 5.0 },
            GeneratorDynamics::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 0.0),
            },
            NeuronType::Excitatory,
        ));

        let _target = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::new(0.0, 0.0, 10.0),
            },
            NeuronType::Excitatory,
        ));

        let comp = world.spawn((
            Compartment {
                voltage: -10.0,
                m: -0.625,
                h: 0.0,
                n: 0.0,
                influence: 0.0,
                capacitance: 1.0,
                injected_current: 0.0,
                fire_impulse: 0.0,
            },
            Position {
                position: Vec3::new(0.0, 0.0, 5.0),
            },
            NeuronType::Excitatory,
        ));

        world.spawn((
            Connection {
                from: origin,
                to: comp,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));
        world.spawn((
            Connection {
                from: comp,
                to: _target,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        let lif_dt = LIF_DT;
        let cdt = FHN_CDT;
        let lif_steps_per_frame = 10;
        let window = lif_steps_per_frame as f64 * lif_dt;
        let total_fhn_steps = 3000;

        let mut time = 0.0;

        // Verify the generator fires but is NOT included in leaky-only recently_fired
        let mut generator_fired = false;
        let mut generator_in_leaky_set = false;

        for _ in 0..total_fhn_steps {
            for _ in 0..lif_steps_per_frame {
                lif_step(&mut world, lif_dt, time);
                time += lif_dt;
            }

            if let Ok(gd) = world.get::<&GeneratorDynamics>(origin) {
                if gd.time_since_fire < window {
                    generator_fired = true;
                }
            }

            let leaky_set = recently_fired_leaky_only(&world, window);
            if leaky_set.contains(&origin) {
                generator_in_leaky_set = true;
            }

            fhn_step(&mut world, cdt, &leaky_set);
        }

        assert!(
            generator_fired,
            "Generator should have fired during the simulation"
        );
        // The generator neuron has LeakyDynamics but its time_since_fire is managed by
        // the LIF refractory system, not by the generator. Without external current input,
        // the LIF neuron never fires, so time_since_fire grows monotonically.
        assert!(
            !generator_in_leaky_set,
            "Bug confirmed: generator neuron is NOT in the leaky-only recently_fired set"
        );
    }
}
