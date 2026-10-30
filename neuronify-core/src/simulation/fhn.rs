use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;

struct OutgoingSpike;

fn spike_excess(compartment: &Compartment) -> f64 {
    (compartment.voltage - BRIDGE_VOLTAGE_THRESHOLD).clamp(0.0, BRIDGE_VOLTAGE_CLAMP)
}

fn current_into_neuron(world: &World, cable: &Connection) -> Option<(Entity, f64)> {
    let (compartment, neuron, sign) = if world.get::<&LeakyDynamics>(cable.to).is_ok() {
        (cable.from, cable.to, neuron_type(world, cable.from).sign())
    } else if world.get::<&OutgoingSpike>(cable.to).is_err() {
        (cable.to, cable.from, 1.0)
    } else {
        return None;
    };
    world.get::<&LeakyDynamics>(neuron).ok()?;
    let excess = spike_excess(&*world.get::<&Compartment>(compartment).ok()?);
    (excess > 0.0).then(|| {
        let current = excess / BRIDGE_VOLTAGE_CLAMP * BRIDGE_CURRENT_SCALE;
        (neuron, sign * current)
    })
}

pub fn fhn_step(world: &mut World, cdt: f64, recently_fired: &HashSet<Entity>) {
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

    let mut new_compartments: HashMap<Entity, Compartment> = world
        .query::<&Compartment>()
        .iter()
        .map(|(entity, &compartment)| (entity, compartment))
        .collect();
    let mut outgoing = Vec::new();

    for (_, (connection, current)) in world.query::<(&Connection, &CompartmentCurrent)>().iter() {
        if let Ok(_compartment_to) = world.get::<&Compartment>(connection.to) {
            if recently_fired.contains(&connection.from) {
                let new_compartment_to = new_compartments
                    .get_mut(&connection.to)
                    .expect("Could not get new compartment");
                new_compartment_to.voltage = COMPARTMENT_FIRE_VOLTAGE;
                outgoing.push(connection.to);
            } else if let Ok(compartment_from) = world.get::<&Compartment>(connection.from) {
                if connection.directional {
                    let sign = neuron_type(world, connection.from).sign();
                    let delta_voltage =
                        sign * spike_excess(&compartment_from) / current.capacitance;
                    let new_compartment_to = new_compartments
                        .get_mut(&connection.to)
                        .expect("Could not get new compartment");
                    new_compartment_to.voltage += delta_voltage * cdt;
                } else {
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
    }

    for (compartment_id, new_compartment) in new_compartments {
        let mut old_compartment = world
            .get::<&mut Compartment>(compartment_id)
            .expect("Could not find compartment");
        *old_compartment = new_compartment;
    }

    for compartment in outgoing {
        world
            .insert_one(compartment, OutgoingSpike)
            .expect("Could not mark compartment");
    }
    let passed: Vec<Entity> = world
        .query::<&Compartment>()
        .with::<&OutgoingSpike>()
        .iter()
        .filter(|(_, compartment)| spike_excess(compartment) == 0.0)
        .map(|(entity, _)| entity)
        .collect();
    for compartment in passed {
        world
            .remove_one::<OutgoingSpike>(compartment)
            .expect("Could not unmark compartment");
    }

    let compartment_to_neuron: Vec<(Entity, f64)> = world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .filter_map(|(_, cable)| current_into_neuron(world, cable))
        .collect();

    for (target, current) in compartment_to_neuron {
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(target) {
            dynamics.received_currents += current;
        }
    }
}

#[cfg(test)]
mod ending_tests {
    use super::*;
    use crate::elements::spawn_cable;
    use crate::simulation::step_simulation;

    fn chain(world: &mut World, neuron_type: NeuronType) -> Vec<Entity> {
        let compartments: Vec<Entity> = (0..4)
            .map(|_| world.spawn((Compartment::default(), neuron_type)))
            .collect();
        for pair in compartments.windows(2) {
            spawn_cable(world, pair[0], pair[1], false);
        }
        compartments
    }

    fn voltage_range_after_exciting(
        world: &mut World,
        excited: Entity,
        watched: Entity,
    ) -> (f64, f64) {
        world.get::<&mut Compartment>(excited).unwrap().voltage = COMPARTMENT_FIRE_VOLTAGE;
        let mut time = 0.0;
        let mut range = (f64::MAX, f64::MIN);
        for _ in 0..800 {
            step_simulation(world, &mut time, 1, None);
            let voltage = world.get::<&Compartment>(watched).unwrap().voltage;
            range = (range.0.min(voltage), range.1.max(voltage));
        }
        range
    }

    fn axon_ending_on_dendrite(axon_type: NeuronType) -> (World, Vec<Entity>, Vec<Entity>) {
        let mut world = World::new();
        let axon = chain(&mut world, axon_type);
        let dendrite = chain(&mut world, NeuronType::Excitatory);
        spawn_cable(&mut world, axon[3], dendrite[0], true);
        (world, axon, dendrite)
    }

    #[test]
    fn an_axon_ending_on_a_compartment_passes_spikes_onwards() {
        let (mut world, axon, dendrite) = axon_ending_on_dendrite(NeuronType::Excitatory);
        let (_, highest) = voltage_range_after_exciting(&mut world, axon[0], dendrite[3]);
        assert!(highest > BRIDGE_VOLTAGE_THRESHOLD, "highest was {highest}");
    }

    #[test]
    fn spikes_do_not_travel_back_into_an_axon_ending() {
        let (mut world, axon, dendrite) = axon_ending_on_dendrite(NeuronType::Excitatory);
        let (_, highest) = voltage_range_after_exciting(&mut world, dendrite[3], axon[0]);
        assert!(highest < BRIDGE_VOLTAGE_THRESHOLD, "highest was {highest}");
    }

    #[test]
    fn an_inhibitory_axon_ending_holds_a_compartment_down() {
        let (mut world, axon, dendrite) = axon_ending_on_dendrite(NeuronType::Inhibitory);
        let rest = Compartment::default().voltage;
        let (lowest, highest) = voltage_range_after_exciting(&mut world, axon[0], dendrite[0]);
        assert!(lowest < rest - 10.0, "lowest was {lowest}");
        assert!(highest < BRIDGE_VOLTAGE_THRESHOLD, "highest was {highest}");
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
                Compartment::default(),
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
