use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;

struct OutgoingSpike;

fn spike_excess(compartment: &Compartment) -> f64 {
    (compartment.voltage - BRIDGE_VOLTAGE_THRESHOLD).clamp(0.0, BRIDGE_VOLTAGE_CLAMP)
}

struct Saltated;

struct Arrival(f64);

fn firing(world: &World, node: Entity) -> bool {
    world
        .get::<&Compartment>(node)
        .is_ok_and(|compartment| compartment.voltage >= COMPARTMENT_FIRE_VOLTAGE)
}

fn ready(world: &World, node: Entity) -> bool {
    !world.satisfies::<&Saltated>(node).unwrap_or(true)
        && world.get::<&Compartment>(node).is_ok_and(|compartment| {
            compartment.voltage < COMPARTMENT_FIRE_VOLTAGE
                && compartment.m < NODE_RECOVERY_THRESHOLD
        })
}

fn internode_delays(world: &World) -> HashMap<Entity, Vec<(Entity, f64)>> {
    let mut delays: HashMap<Entity, Vec<(Entity, f64)>> = HashMap::new();
    for (_, internode) in world.query::<&Connection>().with::<&Myelin>().iter() {
        let position = |node| world.get::<&Position>(node).map(|p| p.position);
        if let (Ok(from), Ok(to)) = (position(internode.from), position(internode.to)) {
            let delay = from.distance(to) as f64 / SALTATORY_SPEED;
            delays
                .entry(internode.from)
                .or_default()
                .push((internode.to, delay));
            delays
                .entry(internode.to)
                .or_default()
                .push((internode.from, delay));
        }
    }
    delays
}

fn saltate(world: &mut World) {
    let delays = internode_delays(world);
    let mut fired: Vec<(Entity, f64)> = Vec::new();
    for (node, arrival) in world.query_mut::<&mut Arrival>() {
        arrival.0 -= LIF_DT;
        if arrival.0 <= 0.0 {
            fired.push((node, arrival.0));
        }
    }
    for &(node, _) in &fired {
        world
            .remove_one::<Arrival>(node)
            .expect("Could not clear arrival");
    }
    fired.extend(
        delays
            .keys()
            .filter(|&&node| {
                firing(world, node) && !world.satisfies::<&Saltated>(node).unwrap_or(true)
            })
            .map(|&node| (node, 0.0)),
    );
    while let Some(earliest) = (0..fired.len()).min_by(|&a, &b| fired[a].1.total_cmp(&fired[b].1)) {
        let (node, time) = fired.swap_remove(earliest);
        if !firing(world, node) {
            if !ready(world, node) {
                continue;
            }
            world.get::<&mut Compartment>(node).unwrap().voltage = COMPARTMENT_FIRE_VOLTAGE;
        } else if world.satisfies::<&Saltated>(node).unwrap_or(true) {
            continue;
        }
        world
            .insert_one(node, Saltated)
            .expect("Could not mark node");
        for &(next, delay) in delays.get(&node).into_iter().flatten() {
            let arrival = time + delay;
            if !ready(world, next) {
                continue;
            }
            if arrival <= 0.0 {
                fired.push((next, arrival));
            } else if !world
                .get::<&Arrival>(next)
                .is_ok_and(|pending| pending.0 <= arrival)
            {
                world
                    .insert_one(next, Arrival(arrival))
                    .expect("Could not schedule arrival");
            }
        }
    }
    let passed: Vec<Entity> = world
        .query::<&Compartment>()
        .with::<&Saltated>()
        .iter()
        .filter(|(_, compartment)| compartment.voltage < COMPARTMENT_FIRE_VOLTAGE)
        .map(|(node, _)| node)
        .collect();
    for node in passed {
        world
            .remove_one::<Saltated>(node)
            .expect("Could not unmark node");
    }
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
        let shunted = (compartment.inhibition * cdt).min(1.0);
        let shunt = shunted * (COMPARTMENT_RESTING_VOLTAGE - compartment.voltage);
        compartment.voltage =
            new_v * FHN_SCALE + FHN_OFFSET + compartment.injected_current * cdt + shunt;
        compartment.m = new_w;
        compartment.injected_current -= shunted * compartment.injected_current;
        compartment.injected_current -= compartment.injected_current * cdt / ENDING_TIME_CONSTANT;
        compartment.inhibition -= compartment.inhibition * cdt / INHIBITION_TIME_CONSTANT;
    }

    let mut new_compartments: HashMap<Entity, Compartment> = world
        .query::<&Compartment>()
        .iter()
        .map(|(entity, &compartment)| (entity, compartment))
        .collect();
    let mut outgoing = Vec::new();

    for (_, (connection, current, myelin)) in world
        .query::<(&Connection, &CompartmentCurrent, Option<&Myelin>)>()
        .iter()
    {
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
                } else if myelin.is_none() {
                    let voltage_diff = compartment_from.voltage - _compartment_to.voltage;
                    let delta_voltage = voltage_diff / current.capacitance;
                    let spread = (compartment_from.injected_current
                        - _compartment_to.injected_current)
                        * ENDING_SPREAD
                        * cdt;
                    let new_compartment_to = new_compartments
                        .get_mut(&connection.to)
                        .expect("Could not get new compartment");
                    new_compartment_to.voltage += delta_voltage * cdt;
                    new_compartment_to.injected_current += spread;
                    let new_compartment_from = new_compartments
                        .get_mut(&connection.from)
                        .expect("Could not get new compartment");
                    new_compartment_from.voltage -= delta_voltage * cdt;
                    new_compartment_from.injected_current -= spread;
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
    saltate(world);

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
    fn a_steadily_firing_inhibitory_ending_stops_spikes_where_it_ends() {
        let site = DENDRITE_LENGTH / 2;
        let mut endings: Vec<(NeuronType, usize, usize)> = (0..8)
            .map(|spike| (INHIBITORY, site, spike * 10 * MILLISECOND))
            .collect();
        endings.push((EXCITATORY, 0, 45 * MILLISECOND));
        let ranges = dendrite_receiving(1.0, &endings);
        assert!(spiked_along(&ranges[..site]), "{ranges:?}");
        assert!(stayed_below_threshold(&ranges[site + 1..]), "{ranges:?}");
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

#[cfg(test)]
mod node_tests {
    use super::*;
    use crate::elements::spawn_internode;
    use crate::simulation::step_simulation;
    use glam::Vec3;

    fn node(world: &mut World, x: f32, z: f32) -> Entity {
        world.spawn((
            Compartment::default(),
            NeuronType::Excitatory,
            Position {
                position: Vec3::new(x, 0.0, z),
            },
        ))
    }

    fn spikes_after_firing(world: &mut World, first: Entity) -> HashMap<Entity, Vec<usize>> {
        world.get::<&mut Compartment>(first).unwrap().voltage = COMPARTMENT_FIRE_VOLTAGE;
        let mut firing = HashSet::new();
        let mut spikes: HashMap<Entity, Vec<usize>> = HashMap::new();
        let mut time = 0.0;
        for step in 0..1000 {
            step_simulation(world, &mut time, 1, None);
            for (node, compartment) in world.query::<&Compartment>().iter() {
                assert!(compartment.voltage.is_finite());
                if compartment.voltage < COMPARTMENT_FIRE_VOLTAGE {
                    firing.remove(&node);
                } else if firing.insert(node) {
                    spikes.entry(node).or_default().push(step);
                }
            }
        }
        spikes
    }

    #[test]
    fn spikes_reach_each_node_after_its_distance_over_the_saltatory_speed() {
        let mut world = World::new();
        let xs = [-20.0, -7.0, -4.0, 0.0, 3.0, 12.0, 14.0, 27.0, 50.0];
        let nodes: Vec<Entity> = xs.iter().map(|&x| node(&mut world, x, 0.0)).collect();
        for pair in nodes.windows(2) {
            spawn_internode(&mut world, pair[0], pair[1]);
        }
        let spikes = spikes_after_firing(&mut world, nodes[3]);
        for (&x, node) in xs.iter().zip(&nodes) {
            let expected = x.abs() as f64 / SALTATORY_SPEED / LIF_DT;
            let step = spikes[node][0] as f64;
            assert!(
                (expected..expected + 2.0).contains(&step),
                "node at {x} fired at step {step}, expected about {expected}"
            );
        }
    }

    #[test]
    fn spikes_fire_every_node_once_however_many_internodes_branch_from_one() {
        for branches in 2..8 {
            let mut world = World::new();
            let hub = node(&mut world, 0.0, 0.0);
            for branch in 0..branches {
                let mut previous = hub;
                for step in 1..5 {
                    let next = node(&mut world, step as f32 * 5.0, branch as f32);
                    spawn_internode(&mut world, previous, next);
                    previous = next;
                }
            }
            let spikes = spikes_after_firing(&mut world, hub);
            assert_eq!(spikes.len(), 1 + 4 * branches);
            assert!(spikes.values().all(|steps| steps.len() == 1), "{spikes:?}");
        }
    }
}
