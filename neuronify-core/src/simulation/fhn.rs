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
        (neuron, sign * cable.strength * current)
    })
}

pub fn fhn_step(world: &mut World, cdt: f64, recently_fired: &HashSet<Entity>) {
    for (_, (compartment, dendrite)) in world.query_mut::<(&mut Compartment, Option<&Dendrite>)>() {
        let tau = match dendrite {
            Some(_) => FHN_TAU / DENDRITE_SLOWDOWN,
            None => FHN_TAU,
        };
        let v = (compartment.voltage - FHN_OFFSET) / FHN_SCALE;
        let w = compartment.m;
        let dv = tau * (v - v * v * v / 3.0 - w);
        let dw = tau * FHN_EPS * (v + FHN_A - FHN_B * w);
        let new_v = v + dv * cdt;
        let new_w = w + dw * cdt;
        let shunt = (compartment.inhibition * cdt).min(1.0)
            * (COMPARTMENT_RESTING_VOLTAGE - compartment.voltage);
        compartment.voltage =
            new_v * FHN_SCALE + FHN_OFFSET + compartment.injected_current * cdt + shunt;
        compartment.m = new_w;
        compartment.injected_current -= compartment.injected_current * cdt / ENDING_TIME_CONSTANT;
        compartment.inhibition -= compartment.inhibition * cdt / INHIBITION_TIME_CONSTANT;
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
                    let charge =
                        connection.strength * spike_excess(&compartment_from) / current.capacitance;
                    let new_compartment_to = new_compartments
                        .get_mut(&connection.to)
                        .expect("Could not get new compartment");
                    match neuron_type(world, connection.from) {
                        NeuronType::Excitatory => {
                            new_compartment_to.injected_current +=
                                ENDING_STRENGTH * charge / ENDING_TIME_CONSTANT * cdt;
                        }
                        NeuronType::Inhibitory => {
                            new_compartment_to.inhibition +=
                                INHIBITION_STRENGTH * charge / INHIBITION_TIME_CONSTANT * cdt;
                        }
                    }
                } else {
                    let voltage_diff = compartment_from.voltage - _compartment_to.voltage;
                    let delta_voltage = voltage_diff / current.capacitance;
                    let spread = (compartment_from.injected_current
                        - _compartment_to.injected_current)
                        * ENDING_SPREAD
                        * cdt;
                    let inhibition_spread = (compartment_from.inhibition
                        - _compartment_to.inhibition)
                        * ENDING_SPREAD
                        * cdt;
                    let new_compartment_to = new_compartments
                        .get_mut(&connection.to)
                        .expect("Could not get new compartment");
                    new_compartment_to.voltage += delta_voltage * cdt;
                    new_compartment_to.injected_current += spread;
                    new_compartment_to.inhibition += inhibition_spread;
                    let new_compartment_from = new_compartments
                        .get_mut(&connection.from)
                        .expect("Could not get new compartment");
                    new_compartment_from.voltage -= delta_voltage * cdt;
                    new_compartment_from.injected_current -= spread;
                    new_compartment_from.inhibition -= inhibition_spread;
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

    const DENDRITE_LENGTH: usize = 8;
    const MILLISECOND: usize = 10;

    fn chain(world: &mut World, neuron_type: NeuronType, length: usize) -> Vec<Entity> {
        let compartments: Vec<Entity> = (0..length)
            .map(|_| world.spawn((Compartment::default(), neuron_type)))
            .collect();
        for pair in compartments.windows(2) {
            spawn_cable(world, pair[0], pair[1], false);
        }
        compartments
    }

    fn dendrite(world: &mut World) -> Vec<Entity> {
        let dendrite = chain(world, NeuronType::Excitatory, DENDRITE_LENGTH);
        for &compartment in &dendrite {
            world.insert_one(compartment, Dendrite).unwrap();
        }
        dendrite
    }

    fn voltage_ranges(
        world: &mut World,
        fired: &[(Entity, usize)],
        watched: &[Entity],
    ) -> Vec<(f64, f64)> {
        let mut ranges = vec![(f64::MAX, f64::MIN); watched.len()];
        let mut time = 0.0;
        for step in 0..800 {
            for &(compartment, at) in fired {
                if step == at {
                    world.get::<&mut Compartment>(compartment).unwrap().voltage =
                        COMPARTMENT_FIRE_VOLTAGE;
                }
            }
            step_simulation(world, &mut time, 1, None);
            for (range, &compartment) in ranges.iter_mut().zip(watched) {
                let voltage = world.get::<&Compartment>(compartment).unwrap().voltage;
                *range = (range.0.min(voltage), range.1.max(voltage));
            }
        }
        ranges
    }

    fn dendrite_receiving(
        strength: f64,
        endings: &[(NeuronType, usize, usize)],
    ) -> Vec<(f64, f64)> {
        let mut world = World::new();
        let dendrite = dendrite(&mut world);
        let fired: Vec<(Entity, usize)> = endings
            .iter()
            .map(|&(axon_type, site, at)| {
                let axon = chain(&mut world, axon_type, 3);
                let ending = spawn_cable(&mut world, axon[2], dendrite[site], true);
                world.get::<&mut Connection>(ending).unwrap().strength = strength;
                (axon[0], at)
            })
            .collect();
        voltage_ranges(&mut world, &fired, &dendrite)
    }

    fn spiked_along(ranges: &[(f64, f64)]) -> bool {
        ranges
            .iter()
            .all(|range| range.1 > BRIDGE_VOLTAGE_THRESHOLD)
    }

    fn stayed_below_threshold(ranges: &[(f64, f64)]) -> bool {
        ranges
            .iter()
            .all(|range| range.1 < BRIDGE_VOLTAGE_THRESHOLD)
    }

    const EXCITATORY: NeuronType = NeuronType::Excitatory;
    const INHIBITORY: NeuronType = NeuronType::Inhibitory;
    const WEAK: f64 = 0.25;

    #[test]
    fn one_axon_ending_starts_a_dendritic_spike() {
        let ranges = dendrite_receiving(1.0, &[(EXCITATORY, 0, 0)]);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn a_weak_axon_ending_only_stirs_the_dendrite_where_it_ends() {
        let rest = Compartment::default().voltage;
        let ranges = dendrite_receiving(WEAK, &[(EXCITATORY, 0, 0)]);
        assert!(
            ranges[0].1 > rest + 15.0,
            "the ending reached {}",
            ranges[0].1
        );
        assert!(stayed_below_threshold(&ranges), "{ranges:?}");
    }

    #[test]
    fn two_weak_axon_endings_arriving_together_start_a_dendritic_spike() {
        let ranges = dendrite_receiving(WEAK, &[(EXCITATORY, 0, 0), (EXCITATORY, 0, 0)]);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn two_weak_axon_endings_a_few_compartments_apart_add_up() {
        let endings = [(EXCITATORY, 0, 0), (EXCITATORY, 2, 0)];
        let ranges = dendrite_receiving(WEAK, &endings);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn weak_axon_endings_far_apart_do_not_add_up() {
        let endings = [(EXCITATORY, 1, 0), (EXCITATORY, DENDRITE_LENGTH - 2, 0)];
        let ranges = dendrite_receiving(WEAK, &endings);
        assert!(stayed_below_threshold(&ranges), "{ranges:?}");
    }

    #[test]
    fn weak_axon_endings_two_milliseconds_apart_add_up() {
        let endings = [(EXCITATORY, 0, 0), (EXCITATORY, 0, 2 * MILLISECOND)];
        let ranges = dendrite_receiving(WEAK, &endings);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn weak_axon_endings_ten_milliseconds_apart_do_not_add_up() {
        let endings = [(EXCITATORY, 0, 0), (EXCITATORY, 0, 10 * MILLISECOND)];
        let ranges = dendrite_receiving(WEAK, &endings);
        assert!(stayed_below_threshold(&ranges), "{ranges:?}");
    }

    fn soma_fired_by_an_ending_of_strength(strength: f64) -> bool {
        let mut world = World::new();
        let soma = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
        ));
        let axon = chain(&mut world, EXCITATORY, 3);
        let ending = spawn_cable(&mut world, axon[2], soma, true);
        world.get::<&mut Connection>(ending).unwrap().strength = strength;
        world.get::<&mut Compartment>(axon[0]).unwrap().voltage = COMPARTMENT_FIRE_VOLTAGE;
        let mut time = 0.0;
        (0..400).any(|_| {
            step_simulation(&mut world, &mut time, 1, None);
            world.get::<&LeakyDynamics>(soma).unwrap().time_since_fire == 0.0
        })
    }

    #[test]
    fn a_weak_axon_ending_on_a_soma_no_longer_fires_it_alone() {
        assert!(soma_fired_by_an_ending_of_strength(1.0));
        assert!(!soma_fired_by_an_ending_of_strength(WEAK));
    }

    #[test]
    fn an_inhibitory_axon_ending_never_pulls_a_compartment_below_rest() {
        let rest = Compartment::default().voltage;
        let ranges = dendrite_receiving(1.0, &[(INHIBITORY, 0, 0)]);
        assert!(
            ranges.iter().all(|range| range.0 > rest - 0.5),
            "{ranges:?}"
        );
    }

    #[test]
    fn an_inhibitory_axon_ending_blocks_spikes_passing_it_for_several_milliseconds() {
        let passing = |after: usize| {
            let endings = [
                (INHIBITORY, DENDRITE_LENGTH / 2, 0),
                (EXCITATORY, 0, after * MILLISECOND),
            ];
            dendrite_receiving(1.0, &endings)[DENDRITE_LENGTH - 1].1 > BRIDGE_VOLTAGE_THRESHOLD
        };
        assert!(!passing(10));
        assert!(passing(40));
    }

    #[test]
    fn spikes_cross_a_dendrite_more_slowly_than_an_axon() {
        let crossing_time = |world: &mut World, compartments: &[Entity]| {
            world
                .get::<&mut Compartment>(compartments[0])
                .unwrap()
                .voltage = COMPARTMENT_FIRE_VOLTAGE;
            let mut time = 0.0;
            (0..800)
                .position(|_| {
                    step_simulation(world, &mut time, 1, None);
                    let far = compartments[compartments.len() - 1];
                    world.get::<&Compartment>(far).unwrap().voltage > BRIDGE_VOLTAGE_THRESHOLD
                })
                .unwrap()
        };
        let mut world = World::new();
        let axon = chain(&mut world, EXCITATORY, DENDRITE_LENGTH);
        let dendrite = dendrite(&mut world);
        let through_axon = crossing_time(&mut world, &axon);
        let through_dendrite = crossing_time(&mut world, &dendrite);
        assert!(
            2 * through_dendrite > 3 * through_axon,
            "{through_dendrite} against {through_axon} steps"
        );
    }

    #[test]
    fn an_inhibitory_axon_ending_vetoes_an_excitatory_one_arriving_with_it() {
        let ranges = dendrite_receiving(1.0, &[(EXCITATORY, 0, 0), (INHIBITORY, 0, 0)]);
        assert!(stayed_below_threshold(&ranges), "{ranges:?}");
    }

    #[test]
    fn spikes_do_not_travel_back_into_an_axon_ending() {
        let mut world = World::new();
        let axon = chain(&mut world, EXCITATORY, 3);
        let dendrite = dendrite(&mut world);
        spawn_cable(&mut world, axon[2], dendrite[0], true);
        let fired = [(dendrite[DENDRITE_LENGTH - 1], 0)];
        let ranges = voltage_ranges(&mut world, &fired, &[axon[0]]);
        assert!(
            ranges[0].1 < BRIDGE_VOLTAGE_THRESHOLD,
            "the axon reached {}",
            ranges[0].1
        );
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
