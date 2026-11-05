use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::TAU;

use crate::table::Tables;
use crate::tree::Tree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    pub tree: usize,
    pub point: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Conductance(usize),
    Current(usize),
    Adaptation(usize),
    Burst(usize),
    Fire { tree: usize, soma: usize },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub source: usize,
    pub time: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConductanceSynapse {
    pub site: Site,
    pub weight: f64,
    pub tau: f64,
    pub reversal: f64,
    pub open: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurrentSynapse {
    pub site: Site,
    pub maximum: f64,
    pub tau: f64,
    pub alpha: bool,
    pub exponential: f64,
    pub linear: f64,
}

impl CurrentSynapse {
    fn current(&self) -> f64 {
        match self.alpha {
            true => self.maximum * self.linear * self.exponential,
            false => self.maximum * self.exponential,
        }
    }

    fn trigger(&mut self) {
        match self.alpha {
            true => {
                self.linear = 0.0;
                self.exponential = std::f64::consts::E;
            }
            false => self.exponential = 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alternating {
    pub site: Site,
    pub amplitude: f64,
    pub frequency: f64,
}

impl Alternating {
    pub fn at(&self, time: f64) -> f64 {
        self.amplitude * (TAU * self.frequency * time * 1e-3).sin()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Adaptation {
    pub site: Site,
    pub increment: f64,
    pub tau: f64,
    pub reversal: f64,
    pub conductance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Burst {
    pub site: Site,
    pub peak: f64,
    pub drain: f64,
    pub boost: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rhythm {
    Regular { interval: f64 },
    Poisson { rate: f64 },
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generator {
    pub source: usize,
    pub rhythm: Rhythm,
    pub since: f64,
    forced: bool,
}

impl Generator {
    pub fn fire(&mut self) {
        self.forced = true;
    }
}

struct Pending {
    time: f64,
    order: u64,
    target: Target,
}

impl PartialEq for Pending {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Pending {}

impl PartialOrd for Pending {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pending {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .time
            .total_cmp(&self.time)
            .then(other.order.cmp(&self.order))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pulses {
    sites: Vec<(Site, f64)>,
    interval: f64,
    duration: f64,
    since: f64,
}

struct Probe {
    site: Site,
    every: u64,
    samples: Vec<(f64, f64)>,
}

pub struct Network {
    tables: Tables,
    trees: Vec<Tree>,
    steps: u64,
    sources: usize,
    outgoing: Vec<Vec<(Target, f64)>>,
    pub conductances: Vec<ConductanceSynapse>,
    pub currents: Vec<CurrentSynapse>,
    pub alternating: Vec<Alternating>,
    pub adaptations: Vec<Adaptation>,
    pub bursts: Vec<Burst>,
    pub generators: Vec<Generator>,
    decays: Decays,
    queue: BinaryHeap<Pending>,
    order: u64,
    random: u64,
    events: Vec<Event>,
    probes: Vec<Probe>,
    pulses: Option<Pulses>,
}

#[derive(Default)]
struct Decays {
    conductances: Vec<f64>,
    currents: Vec<f64>,
    adaptations: Vec<f64>,
}

impl Network {
    pub fn new(dt: f64, celsius: f64) -> Network {
        Network {
            tables: Tables::new(dt, celsius),
            trees: Vec::new(),
            steps: 0,
            sources: 0,
            outgoing: Vec::new(),
            conductances: Vec::new(),
            currents: Vec::new(),
            alternating: Vec::new(),
            adaptations: Vec::new(),
            bursts: Vec::new(),
            generators: Vec::new(),
            decays: Decays::default(),
            queue: BinaryHeap::new(),
            order: 0,
            random: 0x9e37_79b9_7f4a_7c15,
            events: Vec::new(),
            probes: Vec::new(),
            pulses: None,
        }
    }

    pub fn clear(&mut self) {
        self.trees.clear();
        self.sources = 0;
        self.outgoing.clear();
        self.conductances.clear();
        self.currents.clear();
        self.alternating.clear();
        self.adaptations.clear();
        self.bursts.clear();
        self.generators.clear();
        self.queue.clear();
        self.events.clear();
        self.probes.clear();
        if let Some(pulses) = &mut self.pulses {
            pulses.sites.clear();
        }
    }

    pub fn set_pulses(&mut self, sites: &[(Site, f64)], interval: f64, duration: f64) {
        let since = self.pulses.as_ref().map_or(0.0, |pulses| pulses.since);
        self.pulses = Some(Pulses {
            sites: sites.to_vec(),
            interval,
            duration,
            since,
        });
    }

    pub fn stop_pulses(&mut self) {
        self.pulses = None;
    }

    pub fn dt(&self) -> f64 {
        self.tables.dt()
    }

    pub fn celsius(&self) -> f64 {
        self.tables.celsius()
    }

    pub fn set_celsius(&mut self, celsius: f64) {
        if celsius != self.tables.celsius() {
            self.tables = Tables::new(self.tables.dt(), celsius);
        }
    }

    pub fn time(&self) -> f64 {
        self.steps as f64 * self.tables.dt()
    }

    pub fn set_time(&mut self, time: f64) {
        self.steps = (time / self.tables.dt()).round().max(0.0) as u64;
    }

    pub fn random_state(&self) -> u64 {
        self.random
    }

    pub fn set_random_state(&mut self, state: u64) {
        self.random = state;
    }

    fn uniform(&mut self) -> f64 {
        self.random = self.random.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.random;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn source(&mut self) -> usize {
        self.sources += 1;
        self.outgoing.push(Vec::new());
        self.sources - 1
    }

    pub fn add_tree(&mut self, tree: Tree) -> usize {
        self.trees.push(tree);
        self.trees.len() - 1
    }

    pub fn tree(&self, index: usize) -> &Tree {
        &self.trees[index]
    }

    pub fn tree_mut(&mut self, index: usize) -> &mut Tree {
        &mut self.trees[index]
    }

    pub fn trees(&self) -> &[Tree] {
        &self.trees
    }

    pub fn points(&self) -> usize {
        self.trees.iter().map(Tree::points).sum()
    }

    pub fn voltage(&self, site: Site) -> f64 {
        self.trees[site.tree].voltage(site.point)
    }

    pub fn connect(&mut self, source: usize, target: Target, delay: f64) {
        self.outgoing[source].push((target, delay));
    }

    pub fn add_conductance(&mut self, synapse: ConductanceSynapse) -> usize {
        self.decays
            .conductances
            .push((-self.tables.dt() / synapse.tau).exp());
        self.conductances.push(synapse);
        self.conductances.len() - 1
    }

    pub fn add_current(&mut self, synapse: CurrentSynapse) -> usize {
        self.decays
            .currents
            .push((-self.tables.dt() / synapse.tau).exp());
        self.currents.push(synapse);
        self.currents.len() - 1
    }

    pub fn add_alternating(&mut self, source: Alternating) -> usize {
        self.alternating.push(source);
        self.alternating.len() - 1
    }

    pub fn add_adaptation(&mut self, adaptation: Adaptation) -> usize {
        self.decays
            .adaptations
            .push((-self.tables.dt() / adaptation.tau).exp());
        self.adaptations.push(adaptation);
        self.adaptations.len() - 1
    }

    pub fn add_burst(&mut self, burst: Burst) -> usize {
        self.bursts.push(burst);
        self.bursts.len() - 1
    }

    pub fn add_generator(&mut self, source: usize, rhythm: Rhythm, since: f64) -> usize {
        self.generators.push(Generator {
            source,
            rhythm,
            since,
            forced: false,
        });
        self.generators.len() - 1
    }

    pub fn add_probe(&mut self, site: Site, every: u64) -> usize {
        self.probes.push(Probe {
            site,
            every: every.max(1),
            samples: Vec::new(),
        });
        self.probes.len() - 1
    }

    pub fn samples(&mut self, probe: usize) -> std::vec::Drain<'_, (f64, f64)> {
        self.probes[probe].samples.drain(..)
    }

    pub fn schedule(&mut self, time: f64, target: Target) {
        self.order += 1;
        self.queue.push(Pending {
            time,
            order: self.order,
            target,
        });
    }

    pub fn pending(&self, target: Target) -> Vec<f64> {
        let mut times: Vec<(f64, u64)> = self
            .queue
            .iter()
            .filter(|pending| pending.target == target)
            .map(|pending| (pending.time, pending.order))
            .collect();
        times.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        times.into_iter().map(|(time, _)| time).collect()
    }

    pub fn pending_all(&self) -> Vec<(Target, f64)> {
        let mut pending: Vec<(Target, f64, u64)> = self
            .queue
            .iter()
            .map(|pending| (pending.target, pending.time, pending.order))
            .collect();
        pending.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.2.cmp(&b.2)));
        pending
            .into_iter()
            .map(|(target, time, _)| (target, time))
            .collect()
    }

    pub fn events(&mut self) -> std::vec::Drain<'_, Event> {
        self.events.drain(..)
    }

    fn emit(&mut self, source: usize, time: f64) {
        self.events.push(Event { source, time });
        for index in 0..self.outgoing[source].len() {
            let (target, delay) = self.outgoing[source][index];
            self.schedule(time + delay, target);
        }
    }

    fn deliver(&mut self, target: Target) {
        match target {
            Target::Conductance(index) => {
                let synapse = &mut self.conductances[index];
                synapse.open += synapse.weight;
            }
            Target::Current(index) => self.currents[index].trigger(),
            Target::Adaptation(index) => {
                let adaptation = &mut self.adaptations[index];
                adaptation.conductance += adaptation.increment;
            }
            Target::Burst(index) => {
                let burst = &mut self.bursts[index];
                if burst.boost < 1e-12 {
                    burst.boost = burst.peak;
                }
            }
            Target::Fire { tree, soma } => self.trees[tree].fire(soma),
        }
    }

    fn decay(&mut self) {
        let dt = self.tables.dt();
        for (synapse, decay) in self.conductances.iter_mut().zip(&self.decays.conductances) {
            synapse.open *= decay;
        }
        for (synapse, decay) in self.currents.iter_mut().zip(&self.decays.currents) {
            if synapse.exponential > 0.0 {
                synapse.exponential *= decay;
                if synapse.alpha {
                    synapse.linear += dt / synapse.tau;
                }
            }
        }
        for (adaptation, decay) in self.adaptations.iter_mut().zip(&self.decays.adaptations) {
            adaptation.conductance *= decay;
        }
        for burst in &mut self.bursts {
            burst.boost = (burst.boost - burst.drain * dt).max(0.0);
        }
    }

    pub fn step(&mut self) {
        let dt = self.tables.dt();
        let time = self.time();
        self.decay();
        while self.queue.peek().is_some_and(|next| next.time <= time) {
            let pending = self.queue.pop().unwrap();
            self.deliver(pending.target);
        }
        for index in 0..self.generators.len() {
            let generator = &mut self.generators[index];
            generator.since += dt;
            let forced = std::mem::take(&mut generator.forced);
            let fired = forced
                || match generator.rhythm {
                    Rhythm::Regular { interval } => generator.since >= interval,
                    Rhythm::Poisson { rate } => rate > 0.0 && self.uniform() < rate * dt,
                    Rhythm::Manual => false,
                };
            if fired {
                let generator = &mut self.generators[index];
                generator.since = 0.0;
                let source = generator.source;
                self.emit(source, time);
            }
        }

        let pulsing = self.pulses.as_mut().is_some_and(|pulses| {
            pulses.since += dt;
            if pulses.since >= pulses.interval {
                pulses.since -= pulses.interval;
            }
            pulses.since < pulses.duration
        });
        for tree in &mut self.trees {
            tree.begin(&self.tables);
        }
        if pulsing {
            for &(site, current) in &self.pulses.as_ref().unwrap().sites {
                self.trees[site.tree].add(site.point, 0.0, current);
            }
        }
        for synapse in &self.conductances {
            if synapse.open > 0.0 {
                let tree = &mut self.trees[synapse.site.tree];
                tree.add(
                    synapse.site.point,
                    synapse.open,
                    synapse.open * synapse.reversal,
                );
            }
        }
        for synapse in &self.currents {
            if synapse.exponential > 0.0 {
                let tree = &mut self.trees[synapse.site.tree];
                tree.add(synapse.site.point, 0.0, synapse.current());
            }
        }
        for source in &self.alternating {
            let tree = &mut self.trees[source.site.tree];
            tree.add(source.site.point, 0.0, source.at(time + dt));
        }
        for adaptation in &self.adaptations {
            let tree = &mut self.trees[adaptation.site.tree];
            tree.add(
                adaptation.site.point,
                adaptation.conductance,
                adaptation.conductance * adaptation.reversal,
            );
        }
        for burst in &self.bursts {
            if burst.boost > 0.0 {
                let tree = &mut self.trees[burst.site.tree];
                tree.add(burst.site.point, 0.0, burst.boost);
            }
        }
        for index in 0..self.trees.len() {
            self.trees[index].finish(&self.tables);
            let crossings: Vec<_> = self.trees[index].crossings().collect();
            for crossing in crossings {
                self.emit(crossing.source, time + crossing.fraction * dt);
            }
        }
        self.steps += 1;
        let steps = self.steps;
        for probe in &mut self.probes {
            if steps.is_multiple_of(probe.every) {
                let voltage = self.trees[probe.site.tree].voltage(probe.site.point);
                probe.samples.push((steps as f64 * dt, voltage));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Morphology, NodeMembrane, Spiking};

    const DT: f64 = 0.025;

    fn point_neuron(network: &mut Network) -> (Site, usize) {
        let mut morphology = Morphology::default();
        morphology.node(NodeMembrane::Lumped {
            capacitance: 0.2,
            conductance: 0.01,
            reversal: -70.0,
        });
        let mut tree = Tree::build(&morphology, 0.1, -70.0, 16.3).unwrap();
        let source = network.source();
        tree.add_soma(
            0,
            Spiking {
                threshold: -55.0,
                reset: -80.0,
                refractory: 2.0,
                bounds: Some((-90.0, 60.0)),
            },
            source,
        );
        let tree = network.add_tree(tree);
        (Site { tree, point: 0 }, source)
    }

    fn run(network: &mut Network, duration: f64) -> Vec<Event> {
        let mut events = Vec::new();
        for _ in 0..(duration / DT).round() as usize {
            network.step();
            events.extend(network.events());
        }
        events
    }

    #[test]
    fn a_current_above_rheobase_fires_a_point_neuron_regularly() {
        let mut network = Network::new(DT, 16.3);
        let (site, _) = point_neuron(&mut network);
        network.tree_mut(site.tree).inject(site.point, 0.3);
        let events = run(&mut network, 1000.0);
        assert!((15..80).contains(&events.len()), "{} spikes", events.len());
        let intervals: Vec<f64> = events.windows(2).map(|w| w[1].time - w[0].time).collect();
        let spread = intervals.iter().cloned().fold(f64::MIN, f64::max)
            - intervals.iter().cloned().fold(f64::MAX, f64::min);
        assert!(spread < 2.0 * DT, "{intervals:?}");
    }

    #[test]
    fn a_regular_generator_fires_a_neuron_through_a_delayed_synapse() {
        let mut network = Network::new(DT, 16.3);
        let (site, neuron) = point_neuron(&mut network);
        let generator = network.source();
        network.add_generator(generator, Rhythm::Regular { interval: 50.0 }, 0.0);
        let synapse = network.add_current(CurrentSynapse {
            site,
            maximum: 6.0,
            tau: 2.0,
            alpha: false,
            exponential: 0.0,
            linear: 0.0,
        });
        network.connect(generator, Target::Current(synapse), 5.0);
        let events = run(&mut network, 160.0);
        let times = |source| {
            events
                .iter()
                .filter(|event| event.source == source)
                .map(|event| event.time)
                .collect::<Vec<_>>()
        };
        let (sent, fired) = (times(generator), times(neuron));
        assert_eq!(sent.len(), 3, "{sent:?}");
        assert_eq!(fired.len(), 3, "{fired:?}");
        for (sent, fired) in sent.iter().zip(&fired) {
            assert!((5.0..7.0).contains(&(fired - sent)), "{sent} {fired}");
        }
    }

    #[test]
    fn a_forced_spike_waits_out_the_refractory_period() {
        let mut network = Network::new(DT, 16.3);
        let (site, neuron) = point_neuron(&mut network);
        network.schedule(1.0, Target::Fire { tree: 0, soma: 0 });
        network.schedule(2.0, Target::Fire { tree: 0, soma: 0 });
        network.schedule(4.0, Target::Fire { tree: 0, soma: 0 });
        let events = run(&mut network, 10.0);
        let times: Vec<f64> = events
            .iter()
            .filter(|event| event.source == neuron)
            .map(|event| event.time)
            .collect();
        assert_eq!(times, vec![1.0, 4.0]);
        assert!(network.voltage(site) < -60.0);
    }

    #[test]
    fn an_inhibitory_conductance_shunts_towards_its_reversal() {
        let mut network = Network::new(DT, 16.3);
        let (site, _) = point_neuron(&mut network);
        network.tree_mut(site.tree).inject(site.point, 0.1);
        run(&mut network, 200.0);
        let depolarized = network.voltage(site);
        let synapse = network.add_conductance(ConductanceSynapse {
            site,
            weight: 0.05,
            tau: 6.0,
            reversal: -70.0,
            open: 0.0,
        });
        network.schedule(network.time(), Target::Conductance(synapse));
        run(&mut network, 2.0);
        let shunted = network.voltage(site);
        assert!(shunted < depolarized - 1.0, "{shunted} vs {depolarized}");
        assert!(shunted > -70.0, "{shunted}");
    }

    #[test]
    fn pending_events_survive_a_rebuild_bit_for_bit() {
        let build = || {
            let mut network = Network::new(DT, 16.3);
            let (site, _) = point_neuron(&mut network);
            let generator = network.source();
            network.add_generator(generator, Rhythm::Poisson { rate: 0.2 }, 0.0);
            let synapse = network.add_current(CurrentSynapse {
                site,
                maximum: 4.0,
                tau: 2.0,
                alpha: true,
                exponential: 0.0,
                linear: 0.0,
            });
            network.connect(generator, Target::Current(synapse), 3.0);
            (network, site)
        };
        let (mut original, site) = build();
        run(&mut original, 37.3);
        let (mut copy, _) = build();
        copy.set_time(original.time());
        copy.set_random_state(original.random_state());
        copy.currents[0] = original.currents[0].clone();
        copy.generators[0].since = original.generators[0].since;
        for time in original.pending(Target::Current(0)) {
            copy.schedule(time, Target::Current(0));
        }
        let tree = original.tree(site.tree);
        let gates: Vec<f64> = tree.gates(site.point).collect();
        let (voltage, since) = (tree.voltage(site.point), tree.since_spike(0));
        copy.tree_mut(site.tree)
            .set_state(site.point, voltage, &gates, 16.3);
        copy.tree_mut(site.tree).set_since_spike(0, since);
        let (a, b) = (run(&mut original, 300.0), run(&mut copy, 300.0));
        assert!(a.len() > 10);
        assert_eq!(a, b);
        assert_eq!(original.voltage(site), copy.voltage(site));
    }
}
