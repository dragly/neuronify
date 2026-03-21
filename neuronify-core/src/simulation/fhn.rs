use std::collections::{HashMap, HashSet};

use crate::components::*;
use crate::constants::*;

pub fn fhn_step(
    world: &mut hecs::World,
    cdt: f64,
    recently_fired: &HashSet<hecs::Entity>,
) {
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
            let excess = (compartment.voltage - BRIDGE_VOLTAGE_THRESHOLD).clamp(0.0, BRIDGE_VOLTAGE_CLAMP);
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
}
