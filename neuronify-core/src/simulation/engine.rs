use std::collections::HashMap;

use hecs::{Entity, World};
use neuronify_engine::{
    Adaptation, Alternating, BuildError, Burst, ConductanceSynapse as Conductance,
    CurrentSynapse as Current, Membrane, Morphology, Network, NodeMembrane, Rhythm, Site, Spiking,
    Target, Tree,
};

use super::model::*;
use crate::components::*;
use crate::constants::*;
use crate::measurement::visual_input::{VisualInput, FRAME_INTERVAL};
use crate::measurement::voltmeter::VoltageSeries;

const D_LAMBDA: f64 = 0.3;
const PROBE_EVERY: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spike {
    pub entity: Entity,
    pub time: f64,
}

pub struct ProbeSamples {
    pub wire: Entity,
    pub target: Entity,
    pub samples: Vec<(f64, f64)>,
}

#[derive(Clone, Copy, Debug)]
struct Place {
    tree: usize,
    point: usize,
    soma: Option<usize>,
}

impl Place {
    fn site(self) -> Site {
        Site {
            tree: self.tree,
            point: self.point,
        }
    }
}

struct CableLink {
    entity: Entity,
    from: Entity,
    to: Entity,
    cable: Cable,
    membrane: Membrane,
    myelinated: bool,
}

pub struct Engine {
    network: Network,
    built: bool,
    fingerprint: u64,
    preparation: Preparation,
    places: Vec<Option<(Entity, Place)>>,
    nodes: Vec<(Entity, Place)>,
    cables: Vec<(Entity, usize, usize)>,
    sources: Vec<Entity>,
    source_index: HashMap<Entity, usize>,
    generators: Vec<(Entity, usize)>,
    conductances: Vec<(Entity, usize)>,
    currents: Vec<(Entity, usize)>,
    adaptations: Vec<(Entity, usize)>,
    bursts: Vec<(Entity, usize)>,
    clamps: HashMap<(usize, usize), f64>,
    cable_conductances: HashMap<Entity, f64>,
    probes: Vec<(Entity, Entity, usize)>,
    touched: Vec<Site>,
    spikes: Vec<Spike>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::starting_at(0.0)
    }
}

fn soma(world: &World, entity: Entity) -> bool {
    world.satisfies::<&LeakyNeuron>(entity).unwrap_or(false)
}

fn compartment(world: &World, entity: Entity) -> bool {
    world.satisfies::<&Compartment>(entity).unwrap_or(false)
}

fn node(world: &World, entity: Entity) -> bool {
    soma(world, entity) || compartment(world, entity)
}

fn dendritic(world: &World, entity: Entity) -> bool {
    world.satisfies::<&Dendrite>(entity).unwrap_or(false)
}

fn root(parents: &mut [usize], mut index: usize) -> usize {
    while parents[index] != index {
        parents[index] = parents[parents[index]];
        index = parents[index];
    }
    index
}

fn node_membrane(world: &World, entity: Entity, links: &[&CableLink]) -> NodeMembrane {
    let touching = || {
        links
            .iter()
            .filter(move |link| link.from == entity || link.to == entity)
    };
    let Ok(neuron) = world.get::<&LeakyNeuron>(entity) else {
        if touching().any(|link| link.myelinated) {
            let diameter = touching()
                .filter(|link| link.myelinated)
                .map(|link| link.cable.diameter)
                .fold(0.0, f64::max);
            return NodeMembrane::Patch {
                area: std::f64::consts::PI * diameter * RANVIER_LENGTH,
                membrane: Preparation::of(world).ranvier_membrane(),
            };
        }
        return NodeMembrane::Point;
    };
    if touching().any(|link| dendritic(world, link.from) || dendritic(world, link.to)) {
        return NodeMembrane::Patch {
            area: std::f64::consts::PI * SOMA_DIAMETER * SOMA_DIAMETER,
            membrane: soma_membrane(neuron.resting_potential),
        };
    }
    let conductance = world
        .get::<&LeakCurrent>(entity)
        .map_or(0.0, |leak| 1.0 / leak.resistance);
    NodeMembrane::Lumped {
        capacitance: neuron.capacitance,
        conductance,
        reversal: neuron.resting_potential,
    }
}

struct Fingerprint {
    total: u64,
    entry: u64,
    kind: u64,
}

impl Fingerprint {
    fn add(&mut self, value: u64) {
        self.entry = (self.entry ^ value)
            .wrapping_mul(0x100_0000_01b3)
            .rotate_left(29);
    }

    fn float(&mut self, value: f64) {
        self.add(value.to_bits());
    }

    fn entity(&mut self, entity: Entity) {
        self.add(entity.to_bits().get());
    }

    fn components<T: hecs::Component>(
        &mut self,
        world: &World,
        mut each: impl FnMut(&mut Self, &T),
    ) {
        self.kind += 1;
        for (entity, component) in world.query::<&T>().iter() {
            self.entry = 0xcbf2_9ce4_8422_2325 ^ self.kind;
            self.entity(entity);
            each(self, component);
            self.total = self.total.wrapping_add(self.entry);
        }
    }

    fn of(world: &World) -> u64 {
        let mut hash = Fingerprint {
            total: 0,
            entry: 0,
            kind: 0,
        };
        hash.components(world, |hash, connection: &Connection| {
            hash.entity(connection.from);
            hash.entity(connection.to);
            hash.add(connection.directional as u64);
            hash.float(connection.strength);
        });
        hash.components(world, |hash, cable: &Cable| {
            hash.float(cable.length);
            hash.float(cable.diameter);
        });
        hash.components(world, |hash, membrane: &Membrane| {
            hash.float(membrane.capacitance);
            hash.float(membrane.axial_resistivity);
            hash.float(membrane.leak);
            hash.float(membrane.leak_reversal);
            for density in &membrane.channels {
                hash.add(density.channel.index() as u64);
                hash.float(density.conductance);
                hash.float(density.reversal);
            }
        });
        hash.components(world, |_, _: &Myelin| {});
        hash.components(world, |hash, preparation: &Preparation| {
            hash.add(*preparation as u64)
        });
        hash.components(world, |hash, neuron: &LeakyNeuron| {
            for value in [
                neuron.capacitance,
                neuron.resting_potential,
                neuron.threshold,
                neuron.initial_potential,
                neuron.minimum_voltage,
                neuron.maximum_voltage,
                neuron.refractory_period,
            ] {
                hash.float(value);
            }
            hash.add(neuron.voltage_clamped as u64);
        });
        hash.components(world, |hash, leak: &LeakCurrent| {
            hash.float(leak.resistance)
        });
        hash.components(world, |hash, adaptation: &AdaptationCurrent| {
            hash.float(adaptation.adaptation);
            hash.float(adaptation.time_constant);
        });
        hash.components(world, |hash, burst: &BurstCurrent| {
            hash.float(burst.peak);
            hash.float(burst.drain);
        });
        hash.components(world, |hash, neuron_type: &NeuronType| {
            hash.add(*neuron_type as u64)
        });
        hash.components(world, |_, _: &Dendrite| {});
        hash.components(world, |_, _: &Compartment| {});
        hash.components(world, |hash, clamp: &CurrentClamp| {
            hash.float(clamp.current_output)
        });
        hash.components(world, |hash, alternating: &AlternatingCurrent| {
            hash.float(alternating.amplitude);
            hash.float(alternating.frequency);
        });
        hash.components(world, |hash, synapse: &CurrentSynapse| {
            hash.float(synapse.tau);
            hash.float(synapse.maximum_current);
            hash.float(synapse.delay);
            hash.add(synapse.alpha_function as u64);
        });
        hash.components(world, |_, _: &ImmediateFireSynapse| {});
        hash.components(world, |hash, synapse: &ConductanceSynapse| {
            hash.float(synapse.conductance);
            hash.float(synapse.delay);
        });
        hash.components(world, |hash, generator: &RegularSpikeGenerator| {
            hash.float(generator.frequency)
        });
        hash.components(world, |hash, generator: &PoissonGenerator| {
            hash.float(generator.rate)
        });
        hash.components(world, |_, _: &TouchSensor| {});
        hash.components(world, |_, _: &VisualInput| {});
        hash.components(world, |_, _: &VoltageSeries| {});
        hash.total
    }
}

impl Engine {
    pub fn starting_at(time: f64) -> Engine {
        let mut network = Network::new(DT, DEFAULT_TEMPERATURE);
        network.set_time(time * 1e3);
        Engine {
            network,
            built: false,
            fingerprint: 0,
            preparation: Preparation::default(),
            places: Vec::new(),
            nodes: Vec::new(),
            cables: Vec::new(),
            sources: Vec::new(),
            source_index: HashMap::new(),
            generators: Vec::new(),
            conductances: Vec::new(),
            currents: Vec::new(),
            adaptations: Vec::new(),
            bursts: Vec::new(),
            clamps: HashMap::new(),
            cable_conductances: HashMap::new(),
            probes: Vec::new(),
            touched: Vec::new(),
            spikes: Vec::new(),
        }
    }

    pub fn build(world: &World) -> Engine {
        let mut engine = Engine::default();
        engine.rebuild(world);
        engine.fingerprint = Fingerprint::of(world);
        engine.built = true;
        engine
    }

    pub fn time(&self) -> f64 {
        self.network.time()
    }

    pub fn points(&self) -> usize {
        self.network.points()
    }

    pub fn spikes(&self) -> &[Spike] {
        &self.spikes
    }

    pub fn fired(&self, entity: Entity) -> bool {
        self.spikes.iter().any(|spike| spike.entity == entity)
    }

    pub fn sync(&mut self, world: &mut World) -> bool {
        self.network.set_celsius(Temperature::of(world).celsius);
        let fingerprint = Fingerprint::of(world);
        if self.built && fingerprint == self.fingerprint {
            return false;
        }
        if self.built {
            self.store(world);
        }
        self.rebuild(world);
        self.fingerprint = Fingerprint::of(world);
        self.built = true;
        true
    }

    fn place(&self, entity: Entity) -> Option<Place> {
        match self.places.get(entity.id() as usize) {
            Some(&Some((stored, place))) if stored == entity => Some(place),
            _ => None,
        }
    }

    pub fn voltage(&self, entity: Entity) -> Option<f64> {
        self.place(entity)
            .map(|place| self.network.voltage(place.site()))
    }

    pub fn cable_voltages(&self, cable: Entity) -> Option<Vec<f64>> {
        let &(_, tree, index) = self.cables.iter().find(|(entity, _, _)| *entity == cable)?;
        let tree = self.network.tree(tree);
        Some(
            tree.cable_points(index)
                .iter()
                .map(|&point| tree.voltage(point))
                .collect(),
        )
    }

    pub fn inject(&mut self, entity: Entity, current: f64) {
        if let Some(place) = self.place(entity) {
            let base = self.clamps.get(&(place.tree, place.point)).copied();
            let tree = self.network.tree_mut(place.tree);
            tree.inject(place.point, base.unwrap_or(0.0) + current);
        }
    }

    pub fn fire(&mut self, world: &World, entity: Entity) {
        if let Some(&(_, generator)) = self.generators.iter().find(|(e, _)| *e == entity) {
            if world.satisfies::<&TouchSensor>(entity).unwrap_or(false) {
                self.network.generators[generator].fire();
            }
            return;
        }
        let Some(place) = self.place(entity) else {
            return;
        };
        let tree = self.network.tree_mut(place.tree);
        match place.soma {
            Some(soma) => tree.fire(soma),
            None => tree.kick(place.point, neuronify_engine::KICK_DURATION),
        }
    }

    pub fn touch(&mut self, world: &World, sites: Vec<(Entity, f64)>) {
        for site in std::mem::take(&mut self.touched) {
            let base = self.clamps.get(&(site.tree, site.point)).copied();
            self.network
                .tree_mut(site.tree)
                .inject(site.point, base.unwrap_or(0.0));
        }
        let mut pulses = Vec::new();
        let mut interval = None;
        for (entity, current) in sites {
            let Some(place) = self.place(entity) else {
                continue;
            };
            if place.soma.is_some() {
                self.inject(entity, current);
                self.touched.push(place.site());
                continue;
            }
            let Some(rhythm) = pulse_interval(current) else {
                continue;
            };
            let rhythm = match dendritic(world, entity) {
                true => rhythm.max(Preparation::of(world).shortest_dendrite_pulse_interval()),
                false => (rhythm / AXON_PULSE_FACTOR).max(SHORTEST_PULSE_INTERVAL),
            };
            interval = Some(interval.map_or(rhythm, |interval: f64| interval.min(rhythm)));
            pulses.push((place.site(), PULSE_CURRENT * pulse_size(world, entity)));
        }
        match interval {
            Some(interval) => self.network.set_pulses(&pulses, interval, PULSE_DURATION),
            None => self.network.stop_pulses(),
        }
    }

    pub fn set_rate(&mut self, generator: Entity, rate: f64) {
        if let Some(&(_, index)) = self.generators.iter().find(|(e, _)| *e == generator) {
            self.network.generators[index].rhythm = Rhythm::Poisson { rate: rate * 1e-3 };
        }
    }

    pub fn run(&mut self, steps: u32) {
        self.spikes.clear();
        for _ in 0..steps {
            self.network.step();
        }
        let sources = &self.sources;
        self.spikes.extend(self.network.events().map(|event| Spike {
            entity: sources[event.source],
            time: event.time,
        }));
    }

    fn rebuild(&mut self, world: &World) {
        self.network.clear();
        self.preparation = Preparation::of(world);
        self.network.set_celsius(Temperature::of(world).celsius);
        self.network.set_random_state(Randomness::of(world).state);
        self.places.clear();
        self.nodes.clear();
        self.cables.clear();
        self.sources.clear();
        self.source_index.clear();
        self.generators.clear();
        self.conductances.clear();
        self.currents.clear();
        self.adaptations.clear();
        self.bursts.clear();
        self.clamps.clear();
        self.cable_conductances.clear();
        self.probes.clear();
        self.touched.clear();

        let mut links: Vec<CableLink> = world
            .query::<(&Connection, &Cable, &Membrane, Option<&Myelin>)>()
            .iter()
            .filter(|(_, (connection, ..))| {
                !connection.directional
                    && node(world, connection.from)
                    && node(world, connection.to)
            })
            .map(
                |(entity, (connection, cable, membrane, myelin))| CableLink {
                    entity,
                    from: connection.from,
                    to: connection.to,
                    cable: cable.clone(),
                    membrane: membrane.clone(),
                    myelinated: myelin.is_some(),
                },
            )
            .collect();
        links.sort_by_key(|link| link.entity.id());

        let mut entities: Vec<Entity> = world
            .query::<()>()
            .with::<hecs::Or<&LeakyNeuron, &Compartment>>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        entities.sort_by_key(|entity| entity.id());
        let index: HashMap<Entity, usize> = entities
            .iter()
            .enumerate()
            .map(|(index, &entity)| (entity, index))
            .collect();
        let mut parents: Vec<usize> = (0..entities.len()).collect();
        for link in &links {
            let (a, b) = (
                root(&mut parents, index[&link.from]),
                root(&mut parents, index[&link.to]),
            );
            parents[a.max(b)] = a.min(b);
        }
        let mut groups: Vec<(Vec<Entity>, Vec<&CableLink>)> = Vec::new();
        let mut group_of = vec![usize::MAX; entities.len()];
        for (member, &entity) in entities.iter().enumerate() {
            let top = root(&mut parents, member);
            if group_of[top] == usize::MAX {
                group_of[top] = groups.len();
                groups.push((Vec::new(), Vec::new()));
            }
            groups[group_of[top]].0.push(entity);
        }
        for link in &links {
            let top = root(&mut parents, index[&link.from]);
            groups[group_of[top]].1.push(link);
        }

        let mut terminals: Vec<Entity> = world
            .query::<&Connection>()
            .with::<&ConductanceSynapse>()
            .iter()
            .map(|(_, connection)| connection.from)
            .filter(|&from| compartment(world, from))
            .collect();
        terminals.sort_by_key(|entity| entity.id());
        terminals.dedup();

        for (mut order, tree_links) in groups {
            if let Some(first_soma) = order.iter().position(|&e| soma(world, e)) {
                let soma = order.remove(first_soma);
                order.insert(0, soma);
            }
            self.add_tree(world, &order, tree_links, &terminals);
        }
        self.add_mechanisms(world);
    }

    fn add_tree(
        &mut self,
        world: &World,
        order: &[Entity],
        mut tree_links: Vec<&CableLink>,
        terminals: &[Entity],
    ) {
        let local: HashMap<Entity, usize> = order
            .iter()
            .enumerate()
            .map(|(index, &entity)| (entity, index))
            .collect();
        let mut tree = loop {
            let mut morphology = Morphology::default();
            for &entity in order {
                morphology.node(node_membrane(world, entity, &tree_links));
            }
            if morphology.cables.is_empty() && morphology.nodes == [NodeMembrane::Point] {
                return;
            }
            for link in &tree_links {
                morphology.cable(
                    local[&link.from],
                    local[&link.to],
                    link.cable.length,
                    link.cable.diameter,
                    link.membrane.clone(),
                );
            }
            match Tree::build(
                &morphology,
                D_LAMBDA,
                self.preparation.resting_potential(),
                self.network.celsius(),
            ) {
                Ok(tree) => break tree,
                Err(BuildError::Loop { cable }) => {
                    tree_links.remove(cable);
                }
                Err(error) => panic!("Could not build a tree: {error:?}"),
            }
        };
        let index = self.network.trees().len();
        let mut places = Vec::with_capacity(order.len());
        for (node, &entity) in order.iter().enumerate() {
            let point = tree.node_point(node);
            let soma = world.get::<&LeakyNeuron>(entity).ok().map(|neuron| {
                let spiking = Spiking {
                    threshold: neuron.threshold,
                    reset: neuron.initial_potential,
                    refractory: neuron.refractory_period,
                    bounds: neuron
                        .voltage_clamped
                        .then_some((neuron.minimum_voltage, neuron.maximum_voltage)),
                };
                let source = self.source(entity);
                tree.add_soma(node, spiking, source)
            });
            if soma.is_none()
                && terminals
                    .binary_search_by_key(&entity.id(), |e| e.id())
                    .is_ok()
            {
                let source = self.source(entity);
                tree.detect(node, TERMINAL_THRESHOLD, source);
            }
            places.push(Place {
                tree: index,
                point,
                soma,
            });
        }
        let mut hillocks = Vec::new();
        for link in &tree_links {
            for (soma_end, other) in [(link.from, link.to), (link.to, link.from)] {
                let (Some(fired), false) = (places[local[&soma_end]].soma, dendritic(world, other))
                else {
                    continue;
                };
                if soma(world, other) {
                    continue;
                }
                let source = self.network.source();
                self.sources.push(other);
                tree.detect(local[&other], TERMINAL_THRESHOLD, source);
                hillocks.push((source, fired));
            }
        }
        let index = self.network.add_tree(tree);
        for (source, fired) in hillocks {
            self.network.connect(
                source,
                Target::Fire {
                    tree: index,
                    soma: fired,
                },
                0.0,
            );
        }
        let first_cable = self.cables.len();
        for (&entity, place) in order.iter().zip(places) {
            self.nodes.push((entity, place));
            let slot = entity.id() as usize;
            if self.places.len() <= slot {
                self.places.resize(slot + 1, None);
            }
            self.places[slot] = Some((entity, place));
        }
        for (cable, link) in tree_links.iter().enumerate() {
            self.cables.push((link.entity, index, cable));
            for end in [link.from, link.to] {
                *self.cable_conductances.entry(end).or_default() +=
                    cable_conductance(link.cable.diameter);
            }
        }
        self.restore(world, index, first_cable);
    }

    fn source(&mut self, entity: Entity) -> usize {
        let source = self.network.source();
        self.sources.push(entity);
        self.source_index.insert(entity, source);
        source
    }

    fn source_of(&self, entity: Entity) -> Option<usize> {
        self.source_index.get(&entity).copied()
    }

    fn restore(&mut self, world: &World, index: usize, first_cable: usize) {
        let celsius = self.network.celsius();
        let tree = self.network.tree_mut(index);
        for &(entity, place) in self.nodes.iter().rev() {
            if place.tree != index {
                break;
            }
            let rest = world
                .get::<&LeakyNeuron>(entity)
                .map_or(self.preparation.resting_potential(), |neuron| {
                    neuron.resting_potential
                });
            match world.get::<&NodeState>(entity) {
                Ok(state) => {
                    tree.set_state(place.point, state.voltage, &state.gates, celsius);
                    if let Some(soma) = place.soma {
                        tree.set_since_spike(soma, state.since_spike);
                    }
                }
                Err(_) => tree.set_state(place.point, rest, &[], celsius),
            }
        }
        for &(entity, _, cable) in &self.cables[first_cable..] {
            let points = tree.cable_points(cable).to_vec();
            let inner = &points[1..points.len() - 1];
            let state = world.get::<&CableState>(entity);
            match state {
                Ok(state) if state.voltage.len() == inner.len() => {
                    let stride = state.gates.len() / inner.len().max(1);
                    for (k, &point) in inner.iter().enumerate() {
                        let gates = &state.gates[k * stride..(k + 1) * stride];
                        tree.set_state(point, state.voltage[k], gates, celsius);
                    }
                }
                state => {
                    let mut stored = vec![tree.voltage(points[0])];
                    if let Ok(state) = state {
                        stored.extend(&state.voltage);
                    }
                    stored.push(tree.voltage(*points.last().unwrap()));
                    for (k, &point) in inner.iter().enumerate() {
                        let x =
                            (k + 1) as f64 / (inner.len() + 1) as f64 * (stored.len() - 1) as f64;
                        let below = (x as usize).min(stored.len() - 2);
                        let f = x - below as f64;
                        let voltage = stored[below] + f * (stored[below + 1] - stored[below]);
                        tree.set_state(point, voltage, &[], celsius);
                    }
                }
            }
        }
    }

    fn add_mechanisms(&mut self, world: &World) {
        let generators: Vec<(Entity, Rhythm, f64)> = world
            .query::<(
                &GeneratorDynamics,
                Option<&RegularSpikeGenerator>,
                Option<&PoissonGenerator>,
            )>()
            .iter()
            .map(|(entity, (dynamics, regular, poisson))| {
                let rhythm = match (regular, poisson) {
                    (Some(regular), _) if regular.frequency > 0.0 => Rhythm::Regular {
                        interval: 1e3 / regular.frequency,
                    },
                    (_, Some(poisson)) => Rhythm::Poisson {
                        rate: poisson.rate * 1e-3,
                    },
                    _ => Rhythm::Manual,
                };
                (entity, rhythm, dynamics.time_since_fire)
            })
            .collect();
        for (entity, rhythm, since) in generators {
            let source = self.source(entity);
            let generator = self.network.add_generator(source, rhythm, since);
            self.generators.push((entity, generator));
        }

        let mut connections: Vec<(Entity, Connection)> = world
            .query::<&Connection>()
            .iter()
            .filter(|(_, connection)| connection.directional)
            .map(|(entity, connection)| (entity, connection.clone()))
            .collect();
        connections.sort_by_key(|(entity, _)| entity.id());
        for (entity, connection) in connections {
            let Some(target) = self.place(connection.to) else {
                continue;
            };
            let sign = neuron_type(world, connection.from).sign();
            if let Ok(clamp) = world.get::<&CurrentClamp>(connection.from) {
                match world.get::<&AlternatingCurrent>(connection.from) {
                    Ok(alternating) => {
                        self.network.add_alternating(Alternating {
                            site: target.site(),
                            amplitude: sign * alternating.amplitude,
                            frequency: alternating.frequency,
                        });
                    }
                    Err(_) => {
                        *self.clamps.entry((target.tree, target.point)).or_default() +=
                            sign * clamp.current_output;
                    }
                }
            } else if let Ok(ending) = world.get::<&ConductanceSynapse>(entity) {
                let Some(source) = self.source_of(connection.from) else {
                    continue;
                };
                let kinetics = ending_kinetics(world, connection.from);
                let state = world
                    .get::<&SynapseState>(entity)
                    .map(|state| (*state).clone())
                    .unwrap_or_default();
                let scale = match target.soma {
                    Some(_) => SOMA_ENDING_SCALE,
                    None => {
                        self.cable_conductances
                            .get(&connection.to)
                            .copied()
                            .unwrap_or(0.0)
                            * self.preparation.cable_ending_scale()
                    }
                };
                let synapse = self.network.add_conductance(Conductance {
                    site: target.site(),
                    weight: ending.conductance * connection.strength * scale * kinetics.scale,
                    tau: kinetics.tau,
                    reversal: kinetics.reversal,
                    open: state.open,
                });
                self.network
                    .connect(source, Target::Conductance(synapse), ending.delay.max(DT));
                for time in state.pending {
                    self.network.schedule(time, Target::Conductance(synapse));
                }
                self.conductances.push((entity, synapse));
            } else if let Ok(synapse) = world.get::<&CurrentSynapse>(entity) {
                let Some(source) = self.source_of(connection.from) else {
                    continue;
                };
                let index = self.network.add_current(Current {
                    site: target.site(),
                    maximum: sign * synapse.maximum_current,
                    tau: synapse.tau,
                    alpha: synapse.alpha_function,
                    exponential: synapse.exponential,
                    linear: synapse.linear,
                });
                self.network
                    .connect(source, Target::Current(index), synapse.delay);
                for &time in &synapse.triggers {
                    self.network.schedule(time, Target::Current(index));
                }
                self.currents.push((entity, index));
            } else if world
                .satisfies::<&ImmediateFireSynapse>(entity)
                .unwrap_or(false)
            {
                let (Some(source), Some(soma)) = (self.source_of(connection.from), target.soma)
                else {
                    continue;
                };
                self.network.connect(
                    source,
                    Target::Fire {
                        tree: target.tree,
                        soma,
                    },
                    0.0,
                );
            }
        }
        let clamps: Vec<((usize, usize), f64)> = self
            .clamps
            .iter()
            .map(|(&site, &current)| (site, current))
            .collect();
        for ((tree, point), current) in clamps {
            self.network.tree_mut(tree).inject(point, current);
        }
        let wired: Vec<(Entity, Entity)> = world
            .query::<&Connection>()
            .with::<&VoltageSeries>()
            .iter()
            .map(|(entity, connection)| (entity, connection.from))
            .collect();
        for (wire, from) in wired {
            if let Some(place) = self.place(from) {
                let probe = self.network.add_probe(place.site(), PROBE_EVERY);
                self.probes.push((wire, from, probe));
            }
        }

        let somas: Vec<(Entity, Place)> = self
            .nodes
            .iter()
            .filter(|(_, place)| place.soma.is_some())
            .copied()
            .collect();
        for (entity, place) in somas {
            let source = self.source_of(entity).unwrap();
            if let Ok(adaptation) = world.get::<&AdaptationCurrent>(entity) {
                let rest = world.get::<&LeakyNeuron>(entity).unwrap().resting_potential;
                let index = self.network.add_adaptation(Adaptation {
                    site: place.site(),
                    increment: adaptation.adaptation,
                    tau: adaptation.time_constant,
                    reversal: rest,
                    conductance: adaptation.conductance,
                });
                self.network.connect(source, Target::Adaptation(index), 0.0);
                self.adaptations.push((entity, index));
            }
            if let Ok(burst) = world.get::<&BurstCurrent>(entity) {
                let index = self.network.add_burst(Burst {
                    site: place.site(),
                    peak: burst.peak,
                    drain: burst.drain,
                    boost: burst.boost,
                });
                self.network.connect(source, Target::Burst(index), 0.0);
                self.bursts.push((entity, index));
            }
        }
    }

    pub fn reflect(&self, world: &mut World) {
        for (entity, state) in world.query_mut::<&mut NodeState>() {
            if let Some(place) = self.place(entity) {
                let tree = self.network.tree(place.tree);
                state.voltage = tree.voltage(place.point);
                state.gates.clear();
                state.gates.extend(tree.gates(place.point));
                if let Some(soma) = place.soma {
                    state.since_spike = tree.since_spike(soma);
                }
            }
        }
        let mut missing = Vec::new();
        for &(entity, index, cable) in &self.cables {
            let tree = self.network.tree(index);
            let points = tree.cable_points(cable);
            let inner = &points[1..points.len() - 1];
            if inner.is_empty() {
                continue;
            }
            let voltages = inner.iter().map(|&point| tree.voltage(point));
            match world.get::<&mut CableState>(entity) {
                Ok(mut state) => {
                    state.voltage.clear();
                    state.voltage.extend(voltages);
                }
                Err(_) => missing.push((
                    entity,
                    CableState {
                        voltage: voltages.collect(),
                        gates: inner.iter().flat_map(|&point| tree.gates(point)).collect(),
                    },
                )),
            }
        }
        for (entity, state) in missing {
            let _ = world.insert_one(entity, state);
        }
        for &(entity, generator) in &self.generators {
            if let Ok(mut dynamics) = world.get::<&mut GeneratorDynamics>(entity) {
                dynamics.time_since_fire = self.network.generators[generator].since;
            }
        }
        for &(entity, index) in &self.conductances {
            if let Ok(mut state) = world.get::<&mut SynapseState>(entity) {
                state.open = self.network.conductances[index].open;
            }
        }
        if self.currents.is_empty() {
            return;
        }
        let pending = self.network.pending_all();
        for &(entity, index) in &self.currents {
            if let Ok(mut synapse) = world.get::<&mut CurrentSynapse>(entity) {
                let state = &self.network.currents[index];
                synapse.exponential = state.exponential;
                synapse.linear = state.linear;
                synapse.triggers.clear();
                synapse.triggers.extend(
                    pending
                        .iter()
                        .filter(|(target, _)| *target == Target::Current(index))
                        .map(|&(_, time)| time),
                );
            }
        }
    }

    pub fn samples(&mut self) -> Vec<ProbeSamples> {
        self.probes
            .iter()
            .map(|&(wire, target, probe)| ProbeSamples {
                wire,
                target,
                samples: self.network.samples(probe).collect(),
            })
            .collect()
    }

    pub fn store(&self, world: &mut World) {
        let preparation = Preparation::of(world);
        if self.built && preparation != self.preparation {
            rest(world, preparation);
        } else {
            self.store_cells(world);
        }
        for &(entity, index) in &self.conductances {
            let state = SynapseState {
                open: self.network.conductances[index].open,
                pending: self.network.pending(Target::Conductance(index)),
            };
            if world.get::<&SynapseState>(entity).is_ok() {
                *world.get::<&mut SynapseState>(entity).unwrap() = state;
            } else {
                let _ = world.insert_one(entity, state);
            }
        }
        for &(entity, index) in &self.currents {
            if let Ok(mut synapse) = world.get::<&mut CurrentSynapse>(entity) {
                let state = &self.network.currents[index];
                synapse.exponential = state.exponential;
                synapse.linear = state.linear;
                synapse.triggers = self.network.pending(Target::Current(index));
            }
        }
        for &(entity, index) in &self.adaptations {
            if let Ok(mut adaptation) = world.get::<&mut AdaptationCurrent>(entity) {
                adaptation.conductance = self.network.adaptations[index].conductance;
            }
        }
        for &(entity, index) in &self.bursts {
            if let Ok(mut burst) = world.get::<&mut BurstCurrent>(entity) {
                burst.boost = self.network.bursts[index].boost;
            }
        }
        for &(entity, generator) in &self.generators {
            if let Ok(mut dynamics) = world.get::<&mut GeneratorDynamics>(entity) {
                dynamics.time_since_fire = self.network.generators[generator].since;
            }
        }
        Randomness {
            state: self.network.random_state(),
        }
        .set_in(world);
    }

    fn store_cells(&self, world: &mut World) {
        let mut missing_nodes = Vec::new();
        for &(entity, place) in &self.nodes {
            let tree = self.network.tree(place.tree);
            let state = NodeState {
                voltage: tree.voltage(place.point),
                gates: tree.gates(place.point).collect(),
                since_spike: place
                    .soma
                    .map_or(f64::INFINITY, |soma| tree.since_spike(soma)),
            };
            match world.get::<&mut NodeState>(entity) {
                Ok(mut stored) => *stored = state,
                Err(_) => missing_nodes.push((entity, state)),
            }
        }
        let mut missing_cables = Vec::new();
        for &(entity, index, cable) in &self.cables {
            let tree = self.network.tree(index);
            let points = tree.cable_points(cable);
            let inner = &points[1..points.len() - 1];
            let mut state = CableState::default();
            for &point in inner {
                state.voltage.push(tree.voltage(point));
                state.gates.extend(tree.gates(point));
            }
            match world.get::<&mut CableState>(entity) {
                Ok(mut stored) => *stored = state,
                Err(_) => missing_cables.push((entity, state)),
            }
        }
        for (entity, state) in missing_nodes {
            let _ = world.insert_one(entity, state);
        }
        for (entity, state) in missing_cables {
            let _ = world.insert_one(entity, state);
        }
    }
}

pub fn pulse_interval(current: f64) -> Option<f64> {
    let neuron = LeakyNeuron::default();
    let resistance = LeakCurrent::default().resistance;
    let settled = neuron.resting_potential + current * resistance;
    if settled <= neuron.threshold {
        return None;
    }
    let time_constant = resistance * neuron.capacitance;
    let rise = ((settled - neuron.initial_potential) / (settled - neuron.threshold)).ln();
    Some(neuron.refractory_period + time_constant * rise)
}

pub fn visual_inputs(world: &mut World, engine: &mut Engine, time: f64, duration: f64) {
    let rates: Vec<(Entity, f64)> = world
        .query_mut::<&mut VisualInput>()
        .into_iter()
        .map(|(entity, input)| {
            input.phase = (input.phase + input.speed * duration).rem_euclid(1.0);
            let stale = input
                .seen_at
                .is_none_or(|seen_at| !(seen_at..seen_at + FRAME_INTERVAL).contains(&time));
            if stale {
                input.response = input.respond();
                input.seen_at = Some(time);
            }
            (entity, (input.sensitivity * input.response).max(0.0))
        })
        .collect();
    for (entity, rate) in rates {
        engine.set_rate(entity, rate);
    }
}

pub fn run_headless(world: &mut World, duration: f64) -> Vec<Spike> {
    let mut engine = Engine::default();
    let mut time = 0.0;
    let mut spikes = Vec::new();
    let mut remaining = (duration / DT).round() as u32;
    while remaining > 0 {
        let chunk = remaining.min(400);
        super::step_simulation(world, &mut engine, &mut time, chunk, None);
        spikes.extend_from_slice(engine.spikes());
        remaining -= chunk;
    }
    spikes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_cable, spawn_compartment, spawn_internode, Element};
    use crate::examples::EXAMPLE_CATEGORIES;
    use crate::simulation::step_simulation;
    use glam::Vec3;

    const WEAK: f64 = 0.25;
    const EXCITATORY: NeuronType = NeuronType::Excitatory;
    const INHIBITORY: NeuronType = NeuronType::Inhibitory;
    const DENDRITE_LENGTH: usize = 8;

    fn chain(world: &mut World, neuron_type: NeuronType, at: Vec3, length: usize) -> Vec<Entity> {
        let nodes: Vec<Entity> = (0..length)
            .map(|step| {
                let position = at + Vec3::Z * SEGMENT_LENGTH * step as f32;
                spawn_compartment(world, position, neuron_type)
            })
            .collect();
        for pair in nodes.windows(2) {
            spawn_cable(world, pair[0], pair[1], false);
        }
        nodes
    }

    fn dendrite(world: &mut World) -> Vec<Entity> {
        let nodes: Vec<Entity> = (0..DENDRITE_LENGTH)
            .map(|step| {
                let position = Vec3::Z * SEGMENT_LENGTH * step as f32;
                let node = spawn_compartment(world, position, EXCITATORY);
                world.insert_one(node, Dendrite).unwrap();
                node
            })
            .collect();
        for pair in nodes.windows(2) {
            spawn_cable(world, pair[0], pair[1], false);
        }
        nodes
    }

    fn axon_onto(
        world: &mut World,
        neuron_type: NeuronType,
        target: Entity,
        strength: f64,
    ) -> Entity {
        let axon = chain(world, neuron_type, Vec3::new(20.0, 0.0, 0.0), 3);
        let ending = spawn_cable(world, axon[2], target, true);
        world.get::<&mut Connection>(ending).unwrap().strength = strength;
        axon[0]
    }

    struct Run {
        world: World,
        engine: Engine,
        time: f64,
    }

    impl Run {
        fn new(world: World) -> Run {
            Run {
                world,
                engine: Engine::default(),
                time: 0.0,
            }
        }

        fn now(&self) -> f64 {
            self.time * 1e3
        }

        fn step(&mut self) -> Vec<Spike> {
            step_simulation(&mut self.world, &mut self.engine, &mut self.time, 1, None);
            self.engine.spikes().to_vec()
        }

        fn fire(&mut self, entity: Entity) {
            self.engine.sync(&mut self.world);
            self.engine.fire(&self.world, entity);
        }

        fn voltage(&self, entity: Entity) -> f64 {
            self.world.get::<&NodeState>(entity).unwrap().voltage
        }
    }

    fn voltage_ranges(
        world: World,
        fired: &[(Entity, f64)],
        watched: &[Entity],
        duration: f64,
    ) -> Vec<(f64, f64)> {
        let mut run = Run::new(world);
        let mut ranges = vec![(f64::MAX, f64::MIN); watched.len()];
        while run.now() < duration {
            for &(entity, at) in fired {
                if (run.now()..run.now() + DT).contains(&at) {
                    run.fire(entity);
                }
            }
            run.step();
            for (range, &node) in ranges.iter_mut().zip(watched) {
                let voltage = run.voltage(node);
                *range = (range.0.min(voltage), range.1.max(voltage));
            }
        }
        ranges
    }

    fn dendrite_receiving(endings: &[(NeuronType, usize, f64, f64)]) -> Vec<(f64, f64)> {
        let mut world = World::new();
        let dendrite = dendrite(&mut world);
        let fired: Vec<(Entity, f64)> = endings
            .iter()
            .map(|&(axon_type, site, strength, at)| {
                (
                    axon_onto(&mut world, axon_type, dendrite[site], strength),
                    at,
                )
            })
            .collect();
        voltage_ranges(world, &fired, &dendrite, 80.0)
    }

    fn spiked_along(ranges: &[(f64, f64)]) -> bool {
        ranges.iter().all(|range| range.1 > TERMINAL_THRESHOLD)
    }

    fn stayed_below_threshold(ranges: &[(f64, f64)]) -> bool {
        ranges.iter().all(|range| range.1 < TERMINAL_THRESHOLD)
    }

    fn resting_dendrite() -> f64 {
        dendrite_receiving(&[])[0].1
    }

    #[test]
    fn one_axon_ending_starts_a_dendritic_spike() {
        let ranges = dendrite_receiving(&[(EXCITATORY, 0, 1.0, 1.0)]);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn a_weak_axon_ending_only_stirs_the_dendrite_where_it_ends() {
        let rest = resting_dendrite();
        let ranges = dendrite_receiving(&[(EXCITATORY, 0, WEAK, 1.0)]);
        assert!(
            ranges[0].1 > rest + 5.0,
            "the ending reached {}",
            ranges[0].1
        );
        assert!(stayed_below_threshold(&ranges), "{ranges:?}");
    }

    #[test]
    fn two_weak_axon_endings_arriving_together_start_a_dendritic_spike() {
        let ranges = dendrite_receiving(&[(EXCITATORY, 0, WEAK, 1.0), (EXCITATORY, 0, WEAK, 1.0)]);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn two_weak_axon_endings_a_few_compartments_apart_add_up() {
        let ranges = dendrite_receiving(&[(EXCITATORY, 1, WEAK, 1.0), (EXCITATORY, 3, WEAK, 1.0)]);
        assert!(spiked_along(&ranges), "{ranges:?}");
    }

    #[test]
    fn weak_axon_endings_far_apart_do_not_add_up() {
        let endings = [
            (EXCITATORY, 1, WEAK, 1.0),
            (EXCITATORY, DENDRITE_LENGTH - 2, WEAK, 1.0),
        ];
        assert!(stayed_below_threshold(&dendrite_receiving(&endings)));
    }

    #[test]
    fn weak_axon_endings_two_milliseconds_apart_add_up() {
        let endings = [(EXCITATORY, 0, WEAK, 1.0), (EXCITATORY, 0, WEAK, 3.0)];
        assert!(spiked_along(&dendrite_receiving(&endings)));
    }

    #[test]
    fn weak_axon_endings_ten_milliseconds_apart_do_not_add_up() {
        let endings = [(EXCITATORY, 0, WEAK, 1.0), (EXCITATORY, 0, WEAK, 11.0)];
        assert!(stayed_below_threshold(&dendrite_receiving(&endings)));
    }

    #[test]
    fn an_inhibitory_axon_ending_barely_moves_a_resting_dendrite() {
        let mut world = World::new();
        let dendrite = dendrite(&mut world);
        let start = axon_onto(&mut world, INHIBITORY, dendrite[0], 1.0);
        let mut run = Run::new(world);
        while run.now() < 50.0 {
            run.step();
        }
        let rest = run.voltage(dendrite[0]);
        run.fire(start);
        while run.now() < 80.0 {
            run.step();
            let moved = run.voltage(dendrite[0]) - rest;
            assert!(moved.abs() < 3.0, "moved {moved} mV from {rest}");
        }
    }

    #[test]
    fn an_inhibitory_axon_ending_blocks_spikes_passing_it_for_several_milliseconds() {
        let passing = |after: f64| {
            let endings = [
                (INHIBITORY, DENDRITE_LENGTH / 2, 1.0, 1.0),
                (EXCITATORY, 0, 1.0, 1.0 + after),
            ];
            dendrite_receiving(&endings)[DENDRITE_LENGTH - 1].1 > TERMINAL_THRESHOLD
        };
        assert!(!passing(5.0));
        assert!(passing(40.0));
    }

    #[test]
    fn an_inhibitory_axon_ending_vetoes_an_excitatory_one_arriving_with_it() {
        let endings = [(EXCITATORY, 0, 1.0, 1.0), (INHIBITORY, 0, 1.0, 1.0)];
        assert!(stayed_below_threshold(&dendrite_receiving(&endings)));
    }

    #[test]
    fn spikes_do_not_travel_back_into_an_axon_ending() {
        let mut world = World::new();
        let dendrite = dendrite(&mut world);
        let start = axon_onto(&mut world, EXCITATORY, dendrite[0], 1.0);
        let fired = [(dendrite[DENDRITE_LENGTH - 1], 1.0)];
        let ranges = voltage_ranges(world, &fired, &[start], 40.0);
        assert!(
            ranges[0].1 < TERMINAL_THRESHOLD,
            "the axon reached {}",
            ranges[0].1
        );
    }

    fn crossing_time(mut run: Run, nodes: &[Entity]) -> f64 {
        let start = run.now();
        run.fire(nodes[0]);
        let far = nodes[nodes.len() - 1];
        while run.voltage(far) < TERMINAL_THRESHOLD {
            run.step();
            assert!(run.now() < start + 100.0);
        }
        run.now() - start
    }

    #[test]
    fn spikes_cross_a_dendrite_more_slowly_than_an_axon() {
        let mut world = World::new();
        let axon = chain(&mut world, EXCITATORY, Vec3::ZERO, DENDRITE_LENGTH);
        let through_axon = crossing_time(Run::new(world), &axon);
        let mut world = World::new();
        let dendrite = dendrite(&mut world);
        let through_dendrite = crossing_time(Run::new(world), &dendrite);
        assert!(
            through_dendrite > 1.4 * through_axon,
            "{through_dendrite} against {through_axon} ms"
        );
    }

    #[test]
    fn warming_a_running_axon_speeds_its_spikes_without_a_rebuild() {
        let crossing = |celsius: f64| {
            let mut world = World::new();
            let axon = chain(&mut world, EXCITATORY, Vec3::ZERO, DENDRITE_LENGTH);
            let mut run = Run::new(world);
            run.step();
            Temperature { celsius }.set_in(&mut run.world);
            assert!(!run.engine.sync(&mut run.world));
            crossing_time(run, &axon)
        };
        let (cold, warm) = (crossing(DEFAULT_TEMPERATURE), crossing(26.3));
        assert!(warm < 0.85 * cold, "{warm} against {cold} ms");
    }

    fn soma(world: &mut World, at: Vec3) -> Entity {
        Element::ExcitatoryNeuron.spawn(world, at)
    }

    fn soma_fired_by_an_ending_of_strength(strength: f64) -> bool {
        let mut world = World::new();
        let neuron = soma(&mut world, Vec3::ZERO);
        let start = axon_onto(&mut world, EXCITATORY, neuron, strength);
        let mut run = Run::new(world);
        run.fire(start);
        (0..1000).any(|_| run.step().iter().any(|spike| spike.entity == neuron))
    }

    #[test]
    fn a_weak_axon_ending_on_a_soma_does_not_fire_it_alone() {
        assert!(soma_fired_by_an_ending_of_strength(1.0));
        assert!(!soma_fired_by_an_ending_of_strength(WEAK));
    }

    fn decay_once_open(run: &mut Run, value: impl Fn(&World) -> f64) -> f64 {
        while value(&run.world) <= 0.0 {
            run.step();
        }
        run.step();
        let before = value(&run.world);
        run.step();
        value(&run.world) / before
    }

    #[test]
    fn an_ending_decays_with_the_inhibitory_time_constant_once_its_neuron_turns_inhibitory() {
        let mut world = World::new();
        let neuron = soma(&mut world, Vec3::ZERO);
        let start = axon_onto(&mut world, EXCITATORY, neuron, WEAK);
        let mut run = Run::new(world);
        run.step();
        let axon: Vec<Entity> = run
            .world
            .query::<()>()
            .with::<&Compartment>()
            .iter()
            .map(|(compartment, _)| compartment)
            .collect();
        for compartment in axon {
            *run.world.get::<&mut NeuronType>(compartment).unwrap() = NeuronType::Inhibitory;
        }
        run.fire(start);

        let open = |world: &World| {
            world
                .query::<&SynapseState>()
                .iter()
                .map(|(_, state)| state.open)
                .sum::<f64>()
        };
        let decay = decay_once_open(&mut run, open);

        assert!(
            (decay - (-DT / INHIBITORY_TAU).exp()).abs() < 1e-9,
            "{decay}"
        );
    }

    #[test]
    fn a_synapse_decays_with_its_new_time_constant_after_an_edit() {
        let mut world = World::new();
        let from = soma(&mut world, Vec3::ZERO);
        let to = soma(&mut world, Vec3::Z * 10.0);
        let synapse = crate::elements::spawn_synapse(&mut world, from, to);
        let mut run = Run::new(world);
        run.step();
        run.world.get::<&mut CurrentSynapse>(synapse).unwrap().tau = 8.0;
        run.fire(from);

        let exponential =
            |world: &World| world.get::<&CurrentSynapse>(synapse).unwrap().exponential;
        let decay = decay_once_open(&mut run, exponential);

        assert!((decay - (-DT / 8.0).exp()).abs() < 1e-9, "{decay}");
    }

    fn myelinated(world: &mut World, xs: &[f32]) -> Vec<Entity> {
        let nodes: Vec<Entity> = xs
            .iter()
            .map(|&x| spawn_compartment(world, Vec3::new(x, 0.0, 0.0), EXCITATORY))
            .collect();
        for pair in nodes.windows(2) {
            spawn_internode(world, pair[0], pair[1]);
        }
        nodes
    }

    fn first_spikes(world: World, first: Entity, nodes: &[Entity]) -> Vec<Vec<f64>> {
        let mut run = Run::new(world);
        run.fire(first);
        let mut spikes = vec![Vec::new(); nodes.len()];
        let mut above = vec![false; nodes.len()];
        while run.now() < 20.0 {
            run.step();
            for (index, &node) in nodes.iter().enumerate() {
                let now_above = run.voltage(node) > TERMINAL_THRESHOLD;
                if now_above && !above[index] {
                    spikes[index].push(run.now());
                }
                above[index] = now_above;
            }
        }
        spikes
    }

    #[test]
    fn spikes_jump_along_internodes_far_faster_than_along_a_plain_axon() {
        let mut world = World::new();
        let nodes = myelinated(&mut world, &[0.0, 5.6, 11.2, 16.8, 22.4, 28.0, 33.6]);
        let spikes = first_spikes(world, nodes[0], &nodes);
        let arrivals: Vec<f64> = spikes.iter().map(|spikes| spikes[0]).collect();
        assert!(
            arrivals.windows(2).all(|pair| pair[1] >= pair[0]),
            "{arrivals:?}"
        );
        let speed = 5.0 * INTERNODE_LENGTH as f64 * MICROMETRES_PER_UNIT
            / (arrivals[6] - arrivals[1])
            * 1e-3;
        assert!((3.0..10.0).contains(&speed), "{speed} m/s");
    }

    #[test]
    fn spikes_fire_every_node_once_however_many_internodes_branch_from_one() {
        for branches in 2..6 {
            let mut world = World::new();
            let hub = spawn_compartment(&mut world, Vec3::ZERO, EXCITATORY);
            let mut nodes = vec![hub];
            for branch in 0..branches {
                let mut previous = hub;
                for step in 1..4 {
                    let angle = branch as f32 * std::f32::consts::TAU / branches as f32;
                    let at =
                        Vec3::new(angle.cos(), 0.0, angle.sin()) * INTERNODE_LENGTH * step as f32;
                    let next = spawn_compartment(&mut world, at, EXCITATORY);
                    spawn_internode(&mut world, previous, next);
                    nodes.push(next);
                    previous = next;
                }
            }
            let spikes = first_spikes(world, hub, &nodes);
            assert!(spikes.iter().all(|spikes| spikes.len() == 1), "{spikes:?}");
        }
    }

    fn neuron_with_axon(world: &mut World, compartments: usize) -> (Entity, Vec<Entity>, Entity) {
        let neuron = soma(world, Vec3::ZERO);
        let axon = chain(world, EXCITATORY, Vec3::Z * SEGMENT_LENGTH, compartments);
        spawn_cable(world, neuron, axon[0], false);
        let target = soma(world, Vec3::Z * SEGMENT_LENGTH * (compartments + 1) as f32);
        spawn_cable(world, axon[compartments - 1], target, true);
        (neuron, axon, target)
    }

    #[test]
    fn every_bundled_example_builds_and_rests() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let mut world = example.spawn().unwrap();
            let mut engine = Engine::default();
            step_simulation(&mut world, &mut engine, &mut 0.0, 40, None);
            for (_, state) in world.query::<&NodeState>().with::<&Compartment>().iter() {
                assert!(
                    (-90.0..-40.0).contains(&state.voltage),
                    "{}: {}",
                    example.file(),
                    state.voltage
                );
            }
        }
    }

    #[test]
    fn a_soma_spike_reaches_its_target_through_the_axon_and_ending() {
        let mut world = World::new();
        let (neuron, axon, target) = neuron_with_axon(&mut world, 12);
        let mut run = Run::new(world);
        run.fire(neuron);
        let mut spikes = Vec::new();
        while run.now() < 20.0 {
            spikes.extend(run.step());
        }
        let time_of = |entity| {
            spikes
                .iter()
                .find(|spike| spike.entity == entity)
                .map(|spike| spike.time)
        };
        let (sent, arrived, fired) = (
            time_of(neuron).unwrap(),
            time_of(*axon.last().unwrap()).unwrap(),
            time_of(target).unwrap(),
        );
        let travel = arrived - sent;
        assert!((3.0..6.0).contains(&travel), "{travel} ms down the axon");
        assert!(fired > arrived + ENDING_DELAY, "{arrived} {fired}");
    }

    fn busy_circuit() -> (World, Vec<Entity>) {
        let mut world = World::new();
        let (neuron, axon, target) = neuron_with_axon(&mut world, 12);
        let poisson = Element::PoissonGenerator.spawn(&mut world, Vec3::X * -5.0);
        world.get::<&mut PoissonGenerator>(poisson).unwrap().rate = 200.0;
        crate::elements::spawn_synapse(&mut world, poisson, neuron);
        let adaptive = Element::AdaptiveNeuron.spawn(&mut world, Vec3::X * 5.0);
        crate::elements::spawn_synapse(&mut world, target, adaptive);
        let mut watched = vec![neuron, target, adaptive];
        watched.extend(axon);
        (world, watched)
    }

    #[test]
    fn saving_and_loading_mid_spike_continues_exactly_as_an_unbroken_run() {
        let (world, watched) = busy_circuit();
        let mut unbroken = Run::new(world);
        unbroken.fire(watched[0]);
        while unbroken.now() < 3.0 {
            unbroken.step();
        }
        assert!(watched[3..]
            .iter()
            .any(|&node| unbroken.voltage(node) > 0.0));
        unbroken.engine.store(&mut unbroken.world);
        let bytes = crate::serialization::save(&unbroken.world, unbroken.time, 1.0).unwrap();
        let saved = crate::serialization::load(&bytes).unwrap();
        let mut loaded = Run {
            world: saved.world,
            engine: Engine::starting_at(saved.time),
            time: saved.time,
        };
        let entities: Vec<Entity> = {
            let mut loaded_entities: Vec<Entity> =
                loaded.world.iter().map(|e| e.entity()).collect();
            loaded_entities.sort_by_key(|e| e.id());
            let mut original: Vec<Entity> = unbroken.world.iter().map(|e| e.entity()).collect();
            original.sort_by_key(|e| e.id());
            watched
                .iter()
                .map(|w| loaded_entities[original.iter().position(|o| o == w).unwrap()])
                .collect()
        };
        let (mut traces, mut spikes) = ((Vec::new(), Vec::new()), (Vec::new(), Vec::new()));
        for _ in 0..4000 {
            spikes
                .0
                .extend(unbroken.step().iter().map(|spike| spike.time));
            spikes
                .1
                .extend(loaded.step().iter().map(|spike| spike.time));
            traces
                .0
                .extend(watched.iter().map(|&node| unbroken.voltage(node)));
            traces
                .1
                .extend(entities.iter().map(|&node| loaded.voltage(node)));
        }
        assert!(spikes.0.len() > 10, "{} spikes", spikes.0.len());
        assert_eq!(spikes.0, spikes.1);
        assert_eq!(traces.0, traces.1);
    }

    #[test]
    fn an_edit_elsewhere_keeps_a_spike_in_flight() {
        let mut world = World::new();
        let (neuron, axon, _) = neuron_with_axon(&mut world, 12);
        let mut run = Run::new(world);
        run.fire(neuron);
        while run.now() < 2.0 {
            run.step();
        }
        Element::ExcitatoryNeuron.spawn(&mut run.world, Vec3::X * 30.0);
        let mut reached = false;
        while run.now() < 10.0 {
            run.step();
            reached |= run.voltage(*axon.last().unwrap()) > TERMINAL_THRESHOLD;
        }
        assert!(reached);
    }

    #[test]
    fn a_lookup_misses_an_entity_that_reused_an_erased_slot() {
        let mut world = World::new();
        let (_, axon, _) = neuron_with_axon(&mut world, 3);
        let engine = Engine::build(&world);
        let last = *axon.last().unwrap();
        world.despawn(last).unwrap();
        let reused = world.spawn((Compartment,));
        assert_eq!(reused.id(), last.id());
        assert!(engine.voltage(reused).is_none());
        assert!(engine.voltage(axon[0]).is_some());
    }
}
