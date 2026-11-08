use crate::channel::ChannelId;
use crate::morphology::{Membrane, Morphology, NodeMembrane};
use crate::spiking::{
    crossing, Crossing, Detector, Kick, Soma, Spiking, CLAMP_CONDUCTANCE, KICK_VOLTAGE,
};
use crate::table::{Lookup, Tables};
use crate::units::{
    Celsius, Microsiemens, Milliseconds, Millivolts, Nanoamperes, Nanofarads, SquareMicrometres,
};
use std::f64::consts::PI;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    Empty,
    Loop { cable: usize },
    Disconnected { node: usize },
}

struct ChannelPoints {
    channel: ChannelId,
    points: Vec<usize>,
    conductance: Vec<Microsiemens>,
    reversal: Vec<Millivolts>,
    gates: usize,
    states: Vec<f64>,
}

struct Accumulator {
    capacitance: Vec<Nanofarads>,
    leak: Vec<Microsiemens>,
    leak_current: Vec<Nanoamperes>,
    channels: [Vec<(Microsiemens, Nanoamperes)>; ChannelId::ALL.len()],
}

impl Default for Accumulator {
    fn default() -> Self {
        Accumulator {
            capacitance: Vec::new(),
            leak: Vec::new(),
            leak_current: Vec::new(),
            channels: std::array::from_fn(|_| Vec::new()),
        }
    }
}

impl Accumulator {
    fn point(&mut self) -> usize {
        self.capacitance.push(Nanofarads::ZERO);
        self.leak.push(Microsiemens::ZERO);
        self.leak_current.push(Nanoamperes::ZERO);
        for channel in &mut self.channels {
            channel.push((Microsiemens::ZERO, Nanoamperes::ZERO));
        }
        self.capacitance.len() - 1
    }

    fn membrane(&mut self, point: usize, area: SquareMicrometres, membrane: &Membrane) {
        self.capacitance[point] += membrane.capacitance * area;
        let leak = membrane.leak * area;
        self.leak[point] += leak;
        self.leak_current[point] += leak * membrane.leak_reversal;
        for density in &membrane.channels {
            let conductance = density.conductance * area;
            let entry = &mut self.channels[density.channel.index()][point];
            entry.0 += conductance;
            entry.1 += conductance * density.reversal;
        }
    }
}

pub struct Tree {
    parent: Vec<usize>,
    capacitance: Vec<Nanofarads>,
    axial: Vec<Microsiemens>,
    leak: Vec<Microsiemens>,
    leak_current: Vec<Nanoamperes>,
    voltage: Vec<Millivolts>,
    injection: Vec<Nanoamperes>,
    channels: Vec<ChannelPoints>,
    node_points: Vec<usize>,
    cable_points: Vec<Vec<usize>>,
    slot_start: Vec<usize>,
    slots: Vec<(usize, usize)>,
    somas: Vec<Soma>,
    detectors: Vec<Detector>,
    kicks: Vec<Kick>,
    crossings: Vec<Crossing>,
    diagonal: Vec<Microsiemens>,
    rhs: Vec<Nanoamperes>,
    lookups: Vec<Lookup>,
}

impl Tree {
    pub fn build(
        morphology: &Morphology,
        d_lambda: f64,
        voltage: Millivolts,
        celsius: Celsius,
    ) -> Result<Tree, BuildError> {
        if morphology.nodes.is_empty() {
            return Err(BuildError::Empty);
        }
        let mut neighbours = vec![Vec::new(); morphology.nodes.len()];
        for (index, cable) in morphology.cables.iter().enumerate() {
            neighbours[cable.from].push(index);
            neighbours[cable.to].push(index);
        }
        let mut accumulator = Accumulator::default();
        let mut parent = Vec::new();
        let mut axial = Vec::new();
        let mut node_points = vec![usize::MAX; morphology.nodes.len()];
        let mut cable_points = vec![Vec::new(); morphology.cables.len()];
        let mut used = vec![false; morphology.cables.len()];

        node_points[0] = accumulator.point();
        parent.push(0);
        axial.push(Microsiemens::ZERO);
        let mut queue = std::collections::VecDeque::from([0]);
        while let Some(node) = queue.pop_front() {
            for &index in &neighbours[node] {
                if used[index] {
                    continue;
                }
                used[index] = true;
                let cable = &morphology.cables[index];
                let other = if cable.from == node {
                    cable.to
                } else {
                    cable.from
                };
                if node_points[other] != usize::MAX {
                    return Err(BuildError::Loop { cable: index });
                }
                let segments = cable.segments(d_lambda);
                let piece = cable.length / segments as f64;
                let area = PI * cable.diameter * piece;
                let resistance = (4.0 * cable.membrane.axial_resistivity)
                    .along(piece, PI * cable.diameter.squared());
                let mut points = vec![node_points[node]];
                for step in 0..segments {
                    let point = if step + 1 == segments {
                        let point = accumulator.point();
                        node_points[other] = point;
                        queue.push_back(other);
                        point
                    } else {
                        accumulator.point()
                    };
                    parent.push(*points.last().unwrap());
                    axial.push(1.0 / resistance);
                    let previous = *points.last().unwrap();
                    accumulator.membrane(previous, area / 2.0, &cable.membrane);
                    accumulator.membrane(point, area / 2.0, &cable.membrane);
                    points.push(point);
                }
                if cable.from != node {
                    points.reverse();
                }
                cable_points[index] = points;
            }
        }
        if let Some(node) = node_points.iter().position(|&point| point == usize::MAX) {
            return Err(BuildError::Disconnected { node });
        }
        for (node, membrane) in morphology.nodes.iter().enumerate() {
            let point = node_points[node];
            match membrane {
                NodeMembrane::Point => {}
                NodeMembrane::Lumped {
                    capacitance,
                    conductance,
                    reversal,
                } => {
                    accumulator.capacitance[point] += *capacitance;
                    accumulator.leak[point] += *conductance;
                    accumulator.leak_current[point] += *conductance * *reversal;
                }
                NodeMembrane::Patch { area, membrane } => {
                    accumulator.membrane(point, *area, membrane)
                }
            }
        }

        let count = parent.len();
        let mut channels: Vec<ChannelPoints> = Vec::new();
        for (channel, sums) in ChannelId::ALL.into_iter().zip(&accumulator.channels) {
            let gates = channel.channel().gates;
            let steady: Vec<f64> = gates
                .iter()
                .map(|gate| gate.steady_state(voltage, celsius))
                .collect();
            let mut arrays = ChannelPoints {
                channel,
                points: Vec::new(),
                conductance: Vec::new(),
                reversal: Vec::new(),
                gates: gates.len(),
                states: Vec::new(),
            };
            for (point, &(conductance, driving)) in sums.iter().enumerate() {
                if conductance > Microsiemens::ZERO {
                    arrays.points.push(point);
                    arrays.conductance.push(conductance);
                    arrays.reversal.push(driving / conductance);
                    arrays.states.extend(&steady);
                }
            }
            if !arrays.points.is_empty() {
                channels.push(arrays);
            }
        }

        let mut slot_start = vec![0; count + 1];
        for arrays in &channels {
            for &point in &arrays.points {
                slot_start[point + 1] += 1;
            }
        }
        for point in 0..count {
            slot_start[point + 1] += slot_start[point];
        }
        let mut slots = vec![(0, 0); slot_start[count]];
        let mut next = slot_start.clone();
        for (index, arrays) in channels.iter().enumerate() {
            for (k, &point) in arrays.points.iter().enumerate() {
                slots[next[point]] = (index, k);
                next[point] += 1;
            }
        }

        Ok(Tree {
            parent,
            capacitance: accumulator.capacitance,
            axial,
            leak: accumulator.leak,
            leak_current: accumulator.leak_current,
            voltage: vec![voltage; count],
            injection: vec![Nanoamperes::ZERO; count],
            channels,
            node_points,
            cable_points,
            slot_start,
            slots,
            somas: Vec::new(),
            detectors: Vec::new(),
            kicks: Vec::new(),
            crossings: Vec::new(),
            diagonal: vec![Microsiemens::ZERO; count],
            rhs: vec![Nanoamperes::ZERO; count],
            lookups: vec![Lookup::default(); count],
        })
    }

    pub fn points(&self) -> usize {
        self.parent.len()
    }

    pub fn node_point(&self, node: usize) -> usize {
        self.node_points[node]
    }

    pub fn cable_points(&self, cable: usize) -> &[usize] {
        &self.cable_points[cable]
    }

    pub fn node_voltage(&self, node: usize) -> Millivolts {
        self.voltage[self.node_points[node]]
    }

    pub fn voltage(&self, point: usize) -> Millivolts {
        self.voltage[point]
    }

    fn point_slots(&self, point: usize) -> &[(usize, usize)] {
        &self.slots[self.slot_start[point]..self.slot_start[point + 1]]
    }

    pub fn gates(&self, point: usize) -> impl Iterator<Item = f64> + '_ {
        self.point_slots(point).iter().flat_map(|&(index, k)| {
            let arrays = &self.channels[index];
            arrays.states[k * arrays.gates..(k + 1) * arrays.gates]
                .iter()
                .copied()
        })
    }

    pub fn set_state(
        &mut self,
        point: usize,
        voltage: Millivolts,
        gates: &[f64],
        celsius: Celsius,
    ) {
        self.voltage[point] = voltage;
        let count: usize = self
            .point_slots(point)
            .iter()
            .map(|&(index, _)| self.channels[index].gates)
            .sum();
        let mut stored = gates.iter().copied();
        for slot in self.slot_start[point]..self.slot_start[point + 1] {
            let (index, k) = self.slots[slot];
            let arrays = &mut self.channels[index];
            let states = &mut arrays.states[k * arrays.gates..(k + 1) * arrays.gates];
            if gates.len() == count {
                states.fill_with(|| stored.next().unwrap());
            } else {
                for (state, gate) in states.iter_mut().zip(arrays.channel.channel().gates) {
                    *state = gate.steady_state(voltage, celsius);
                }
            }
        }
    }

    pub fn cable_voltage(&self, cable: usize, fraction: f64) -> Millivolts {
        let points = &self.cable_points[cable];
        let x = fraction.clamp(0.0, 1.0) * (points.len() - 1) as f64;
        let index = (x as usize).min(points.len() - 2);
        let f = x - index as f64;
        let (a, b) = (self.voltage[points[index]], self.voltage[points[index + 1]]);
        a + f * (b - a)
    }

    pub fn add_soma(&mut self, node: usize, spiking: Spiking, source: usize) -> usize {
        self.somas.push(Soma {
            source,
            point: self.node_points[node],
            spiking,
            since: Milliseconds::INFINITY,
            forced: false,
            clamped: false,
            before: Millivolts::ZERO,
        });
        self.somas.len() - 1
    }

    pub fn somas(&self) -> usize {
        self.somas.len()
    }

    pub fn since_spike(&self, soma: usize) -> Milliseconds {
        self.somas[soma].since
    }

    pub fn set_since_spike(&mut self, soma: usize, since: Milliseconds) {
        self.somas[soma].since = since;
    }

    pub fn fire(&mut self, soma: usize) {
        self.somas[soma].forced = true;
    }

    pub fn detect(&mut self, node: usize, threshold: Millivolts, source: usize) {
        self.detectors.push(Detector {
            source,
            point: self.node_points[node],
            threshold,
            before: Millivolts::ZERO,
        });
    }

    pub fn kick(&mut self, point: usize, duration: Milliseconds) {
        match self.kicks.iter_mut().find(|kick| kick.point == point) {
            Some(kick) => kick.remaining = kick.remaining.max(duration),
            None => self.kicks.push(Kick {
                point,
                remaining: duration,
            }),
        }
    }

    pub fn crossings(&mut self) -> std::vec::Drain<'_, Crossing> {
        self.crossings.drain(..)
    }

    pub fn inject(&mut self, point: usize, current: Nanoamperes) {
        self.injection[point] = current;
    }

    pub fn add(&mut self, point: usize, conductance: Microsiemens, current: Nanoamperes) {
        self.diagonal[point] += conductance;
        self.rhs[point] += current;
    }

    pub fn step(&mut self, tables: &Tables) {
        self.begin(tables);
        self.finish(tables);
    }

    pub fn begin(&mut self, tables: &Tables) {
        let dt = tables.dt();
        for i in 0..self.points() {
            let capacitive = self.capacitance[i] / dt;
            self.diagonal[i] = capacitive + self.leak[i];
            self.rhs[i] = capacitive * self.voltage[i] + self.leak_current[i] + self.injection[i];
        }
        if !self.channels.is_empty() {
            for (lookup, &voltage) in self.lookups.iter_mut().zip(&self.voltage) {
                *lookup = Lookup::new(voltage);
            }
        }
        for arrays in &mut self.channels {
            let table = tables.channel(arrays.channel);
            let states = arrays.states.chunks_exact_mut(arrays.gates);
            for (k, (&point, states)) in arrays.points.iter().zip(states).enumerate() {
                let open = arrays.conductance[k] * table.advance(states, &self.lookups[point]);
                self.diagonal[point] += open;
                self.rhs[point] += open * arrays.reversal[k];
            }
        }
        for soma in &mut self.somas {
            soma.before = self.voltage[soma.point];
            if std::mem::take(&mut soma.forced) && soma.spiking.target(soma.since + dt).is_none() {
                soma.since = Milliseconds::ZERO;
                self.crossings.push(Crossing {
                    source: soma.source,
                    fraction: 0.0,
                });
            }
            let target = soma.spiking.target(soma.since + dt);
            soma.clamped = target.is_some();
            if let Some(target) = target {
                self.diagonal[soma.point] += CLAMP_CONDUCTANCE;
                self.rhs[soma.point] += CLAMP_CONDUCTANCE * target;
            }
        }
        for kick in &mut self.kicks {
            self.diagonal[kick.point] += CLAMP_CONDUCTANCE;
            self.rhs[kick.point] += CLAMP_CONDUCTANCE * KICK_VOLTAGE;
            kick.remaining -= dt;
        }
        self.kicks
            .retain(|kick| kick.remaining > Milliseconds::ZERO);
        for detector in &mut self.detectors {
            detector.before = self.voltage[detector.point];
        }
    }

    pub fn finish(&mut self, tables: &Tables) {
        let dt = tables.dt();
        for i in 1..self.points() {
            let g = self.axial[i];
            self.diagonal[i] += g;
            self.diagonal[self.parent[i]] += g;
        }
        for i in (1..self.points()).rev() {
            let p = self.parent[i];
            let factor = self.axial[i] / self.diagonal[i];
            self.diagonal[p] -= factor * self.axial[i];
            let carried = factor * self.rhs[i];
            self.rhs[p] += carried;
        }
        self.voltage[0] = self.rhs[0] / self.diagonal[0];
        for i in 1..self.points() {
            self.voltage[i] =
                (self.rhs[i] + self.axial[i] * self.voltage[self.parent[i]]) / self.diagonal[i];
        }
        for soma in &mut self.somas {
            soma.since += dt;
            let after = self.voltage[soma.point];
            if soma.clamped {
                continue;
            }
            if let Some(fraction) = crossing(soma.spiking.threshold, soma.before, after) {
                soma.since = (1.0 - fraction) * dt;
                self.crossings.push(Crossing {
                    source: soma.source,
                    fraction,
                });
            }
        }
        for soma in &self.somas {
            if let Some((low, high)) = soma.spiking.bounds {
                let voltage = &mut self.voltage[soma.point];
                *voltage = voltage.max(low.min(high)).min(high.max(low));
            }
        }
        for detector in &self.detectors {
            let after = self.voltage[detector.point];
            if detector.before < detector.threshold {
                if let Some(fraction) = crossing(detector.threshold, detector.before, after) {
                    self.crossings.push(Crossing {
                        source: detector.source,
                        fraction,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::Density;
    use crate::units::{
        MicrofaradsPerSquareCentimetre, Micrometres, OhmCentimetres, OhmSquareCentimetres,
        SiemensPerSquareCentimetre,
    };

    const CELSIUS: Celsius = Celsius(16.3);
    const DT: Milliseconds = Milliseconds(0.025);

    fn passive(resistance: f64, reversal: f64, axial_resistivity: f64) -> Membrane {
        Membrane::passive(
            OhmSquareCentimetres(resistance),
            Millivolts(reversal),
            OhmCentimetres(axial_resistivity),
        )
    }

    fn cable(
        morphology: &mut Morphology,
        ends: (usize, usize),
        size: (f64, f64),
        membrane: Membrane,
    ) {
        morphology.cable(
            ends.0,
            ends.1,
            Micrometres(size.0),
            Micrometres(size.1),
            membrane,
        );
    }

    fn passive_cable(length: f64, diameter: f64) -> Morphology {
        let mut morphology = Morphology::default();
        let start = morphology.node(NodeMembrane::Point);
        let end = morphology.node(NodeMembrane::Point);
        cable(
            &mut morphology,
            (start, end),
            (length, diameter),
            passive(10_000.0, 0.0, 100.0),
        );
        morphology
    }

    fn build(morphology: &Morphology, d_lambda: f64, voltage: f64, celsius: Celsius) -> Tree {
        Tree::build(morphology, d_lambda, Millivolts(voltage), celsius).unwrap()
    }

    #[test]
    fn the_hines_solve_matches_a_dense_solve_on_a_branched_tree() {
        let mut morphology = Morphology::default();
        let soma = morphology.node(NodeMembrane::Lumped {
            capacitance: Nanofarads(0.02),
            conductance: Microsiemens(0.001),
            reversal: Millivolts(-70.0),
        });
        let mut previous = soma;
        let mut tips = Vec::new();
        for size in [(120.0, 2.0), (80.0, 1.2), (200.0, 0.6)] {
            let node = morphology.node(NodeMembrane::Point);
            cable(
                &mut morphology,
                (previous, node),
                size,
                passive(20_000.0, -70.0, 150.0),
            );
            tips.push(node);
            previous = node;
        }
        let side = morphology.node(NodeMembrane::Point);
        cable(
            &mut morphology,
            (side, tips[0]),
            (150.0, 0.8),
            Membrane::squid(),
        );
        let mut tree = build(&morphology, 0.1, -65.0, CELSIUS);
        tree.inject(tree.node_point(side), Nanoamperes(0.05));
        let tables = Tables::new(DT, Celsius(6.3));
        for _ in 0..40 {
            tree.step(&tables);
        }
        let n = tree.points();
        let before = tree.voltage.clone();
        tree.step(&tables);
        let mut matrix = vec![vec![0.0; n]; n];
        let mut rhs = vec![0.0; n];
        for i in 0..n {
            let capacitive = tree.capacitance[i] / tables.dt();
            matrix[i][i] += (capacitive + tree.leak[i]).0;
            rhs[i] = (capacitive * before[i] + tree.leak_current[i] + tree.injection[i]).0;
        }
        for arrays in &tree.channels {
            let powers: Vec<i32> = arrays
                .channel
                .channel()
                .gates
                .iter()
                .map(|gate| gate.power)
                .collect();
            for (k, &point) in arrays.points.iter().enumerate() {
                let mut open = arrays.conductance[k];
                for (gate, power) in powers.iter().enumerate() {
                    open *= arrays.states[k * arrays.gates + gate].powi(*power);
                }
                matrix[point][point] += open.0;
                rhs[point] += (open * arrays.reversal[k]).0;
            }
        }
        for i in 1..n {
            let (p, g) = (tree.parent[i], tree.axial[i].0);
            matrix[i][i] += g;
            matrix[p][p] += g;
            matrix[i][p] -= g;
            matrix[p][i] -= g;
        }
        for column in 0..n {
            let pivot = (column..n)
                .max_by(|&a, &b| matrix[a][column].abs().total_cmp(&matrix[b][column].abs()))
                .unwrap();
            matrix.swap(column, pivot);
            rhs.swap(column, pivot);
            let (upper, lower) = matrix.split_at_mut(column + 1);
            let pivot_row = &upper[column];
            for (offset, row) in lower.iter_mut().enumerate() {
                let factor = row[column] / pivot_row[column];
                for (value, pivot) in row[column..].iter_mut().zip(&pivot_row[column..]) {
                    *value -= factor * pivot;
                }
                rhs[column + 1 + offset] -= factor * rhs[column];
            }
        }
        let mut dense = vec![0.0; n];
        for row in (0..n).rev() {
            let known: f64 = (row + 1..n).map(|k| matrix[row][k] * dense[k]).sum();
            dense[row] = (rhs[row] - known) / matrix[row][row];
        }
        for (i, (expected, actual)) in dense.iter().zip(&tree.voltage).enumerate() {
            assert!(
                (expected - actual.0).abs() < 1e-9,
                "point {i}: {expected} vs {actual:?}"
            );
        }
    }

    #[test]
    fn a_passive_cable_attenuates_like_the_cable_equation() {
        let (length, diameter, current) = (1000.0, 1.0, 0.01);
        let mut tree = build(&passive_cable(length, diameter), 0.1, 0.0, CELSIUS);
        tree.inject(tree.node_point(0), Nanoamperes(current));
        let tables = Tables::new(DT, Celsius(6.3));
        for _ in 0..(Milliseconds(300.0) / tables.dt()) as usize {
            tree.step(&tables);
        }
        let lambda = (10_000.0 * diameter * 1e-4 / (4.0 * 100.0)).sqrt() * 1e4;
        let axial_per_um = 4.0 * 100.0 / (PI * (diameter * 1e-4).powi(2)) * 1e-4 * 1e-6;
        let ratio = length / lambda;
        let analytic = |x: f64| {
            current * axial_per_um * lambda * ((length - x) / lambda).cosh() / ratio.sinh()
        };
        for fraction in [0.0, 0.5, 1.0] {
            let expected = analytic(fraction * length);
            let actual = tree.cable_voltage(0, fraction).0;
            assert!(
                ((actual - expected) / expected).abs() < 0.01,
                "{fraction}: {actual} vs {expected}"
            );
        }
    }

    #[test]
    fn a_loop_is_refused() {
        let mut morphology = passive_cable(100.0, 1.0);
        let third = morphology.node(NodeMembrane::Point);
        cable(&mut morphology, (1, third), (100.0, 1.0), Membrane::squid());
        cable(&mut morphology, (third, 0), (100.0, 1.0), Membrane::squid());
        assert!(matches!(
            Tree::build(&morphology, 0.1, Millivolts(-65.0), CELSIUS),
            Err(BuildError::Loop { .. })
        ));
    }

    #[test]
    fn a_tree_rebuilt_from_its_point_states_continues_the_same_trace() {
        let mut morphology = Morphology::default();
        let start = morphology.node(NodeMembrane::Point);
        let middle = morphology.node(NodeMembrane::Point);
        let end = morphology.node(NodeMembrane::Point);
        cable(
            &mut morphology,
            (start, middle),
            (400.0, 1.0),
            Membrane::squid(),
        );
        cable(
            &mut morphology,
            (middle, end),
            (300.0, 0.5),
            passive(20_000.0, -65.0, 150.0),
        );
        let tables = Tables::new(DT, CELSIUS);
        let mut original = build(&morphology, 0.1, -65.0, CELSIUS);
        original.inject(original.node_point(start), Nanoamperes(0.5));
        for _ in 0..60 {
            original.step(&tables);
        }
        original.inject(original.node_point(start), Nanoamperes::ZERO);
        let mut copy = build(&morphology, 0.1, -40.0, CELSIUS);
        for point in 0..original.points() {
            let gates: Vec<f64> = original.gates(point).collect();
            copy.set_state(point, original.voltage(point), &gates, CELSIUS);
        }
        for _ in 0..200 {
            original.step(&tables);
            copy.step(&tables);
        }
        assert_eq!(original.voltage, copy.voltage);
    }

    #[test]
    fn a_soma_spike_follows_its_shape_and_starts_a_spike_down_the_axon() {
        let mut morphology = Morphology::default();
        let soma = morphology.node(NodeMembrane::Lumped {
            capacitance: Nanofarads(0.2),
            conductance: Microsiemens(0.01),
            reversal: Millivolts(-70.0),
        });
        let end = morphology.node(NodeMembrane::Point);
        cable(
            &mut morphology,
            (soma, end),
            (1000.0, 0.25),
            Membrane::squid(),
        );
        let tables = Tables::new(DT, CELSIUS);
        let mut tree = build(&morphology, 0.1, -65.0, CELSIUS);
        let spiking = Spiking {
            threshold: Millivolts(-55.0),
            reset: Millivolts(-80.0),
            refractory: Milliseconds(2.0),
            bounds: None,
        };
        tree.add_soma(soma, spiking, 0);
        tree.detect(end, Millivolts(-20.0), 1);
        let mut events = Vec::new();
        let mut trace = Vec::new();
        for step in 0..(Milliseconds(20.0) / tables.dt()) as usize {
            let t = step as f64 * tables.dt();
            let current = if t < Milliseconds(1.5) { 2.0 } else { 0.0 };
            tree.inject(tree.node_point(soma), Nanoamperes(current));
            tree.step(&tables);
            trace.push((t + tables.dt(), tree.node_voltage(soma).0));
            events.extend(
                tree.crossings()
                    .map(|c| (c.source, t + c.fraction * tables.dt())),
            );
        }
        let [(first, fired), (second, arrived)] = events[..] else {
            panic!("events {events:?}")
        };
        assert_eq!((first, second), (0, 1));
        let peak = trace.iter().map(|&(_, v)| v).fold(f64::MIN, f64::max);
        assert!((25.0..=30.0).contains(&peak), "peak {peak}");
        let held = trace
            .iter()
            .find(|&&(t, _)| t >= fired + Milliseconds(1.5))
            .map(|&(_, v)| v)
            .unwrap();
        assert!((held + 80.0).abs() < 0.5, "held at {held}");
        let speed = 1000.0 / (arrived - fired).0 / 1000.0;
        assert!((0.25..0.6).contains(&speed), "speed {speed} m/s");
    }

    fn spikes(trace: &[f64]) -> usize {
        trace
            .windows(2)
            .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
            .count()
    }

    #[test]
    fn a_squid_patch_rests_near_minus_65_mv_and_fires_repeatedly() {
        let area = SquareMicrometres(PI * 31.83 * 31.83);
        let mut morphology = Morphology::default();
        morphology.node(NodeMembrane::Patch {
            area,
            membrane: Membrane::squid(),
        });
        let tables = Tables::new(DT, Celsius(6.3));
        let mut tree = build(&morphology, 0.1, -65.0, CELSIUS);
        let mut trace = Vec::new();
        let current = Nanoamperes(10.0 * area.0 * 1e-5);
        for step in 0..(Milliseconds(50.0) / tables.dt()) as usize {
            let t = (step as f64 + 0.5) * tables.dt();
            let during = (Milliseconds(5.0)..Milliseconds(45.0)).contains(&t);
            tree.inject(
                tree.node_point(0),
                if during { current } else { Nanoamperes::ZERO },
            );
            tree.step(&tables);
            trace.push(tree.node_voltage(0).0);
        }
        assert!((trace[(Milliseconds(4.0) / tables.dt()) as usize] + 65.0).abs() < 0.5);
        assert!(spikes(&trace) >= 2);
        let peak = trace.iter().cloned().fold(f64::MIN, f64::max);
        assert!((30.0..50.0).contains(&peak), "peak {peak}");
    }

    #[test]
    fn a_one_micrometre_squid_axon_conducts_at_about_half_a_metre_per_second() {
        let mut morphology = Morphology::default();
        let start = morphology.node(NodeMembrane::Point);
        let end = morphology.node(NodeMembrane::Point);
        cable(
            &mut morphology,
            (start, end),
            (5000.0, 1.0),
            Membrane::squid(),
        );
        let tables = Tables::new(DT, Celsius(6.3));
        let mut tree = build(&morphology, 0.1, -65.0, CELSIUS);
        let mut arrivals = [None; 2];
        for step in 0..(Milliseconds(30.0) / tables.dt()) as usize {
            let t = (step as f64 + 0.5) * tables.dt();
            let during = (Milliseconds(1.0)..Milliseconds(2.0)).contains(&t);
            tree.inject(
                tree.node_point(start),
                Nanoamperes(if during { 0.5 } else { 0.0 }),
            );
            tree.step(&tables);
            for (arrival, fraction) in arrivals.iter_mut().zip([0.3, 0.7]) {
                if arrival.is_none() && tree.cable_voltage(0, fraction) > Millivolts(-20.0) {
                    *arrival = Some(t);
                }
            }
        }
        let [Some(a), Some(b)] = arrivals else {
            panic!("no spike: {arrivals:?}")
        };
        let speed = 0.4 * 5000.0 / (b - a).0 / 1000.0;
        assert!((0.4..0.7).contains(&speed), "speed {speed} m/s");
    }

    fn cortical(sodium: f64, potassium: f64) -> Membrane {
        Membrane {
            capacitance: MicrofaradsPerSquareCentimetre(1.0),
            axial_resistivity: OhmCentimetres(150.0),
            leak: SiemensPerSquareCentimetre(1.0 / 20_000.0),
            leak_reversal: Millivolts(-70.0),
            channels: vec![
                Density {
                    channel: ChannelId::CorticalSodium,
                    conductance: SiemensPerSquareCentimetre(sodium),
                    reversal: Millivolts(60.0),
                },
                Density {
                    channel: ChannelId::CorticalPotassium,
                    conductance: SiemensPerSquareCentimetre(potassium),
                    reversal: Millivolts(-90.0),
                },
            ],
        }
    }

    #[test]
    fn a_membrane_balanced_to_rest_stays_at_rest() {
        let mut a_type = cortical(0.05, 0.015);
        a_type.channels.push(Density {
            channel: ChannelId::ATypePotassium,
            conductance: SiemensPerSquareCentimetre(0.03),
            reversal: Millivolts(-90.0),
        });
        let body = Celsius(37.0);
        for membrane in [Membrane::squid(), cortical(0.1, 0.03), a_type] {
            let mut morphology = Morphology::default();
            let start = morphology.node(NodeMembrane::Point);
            let end = morphology.node(NodeMembrane::Point);
            let membrane = membrane.resting_at(Millivolts(-70.0), body);
            cable(&mut morphology, (start, end), (500.0, 0.6), membrane);
            let tables = Tables::new(DT, body);
            let mut tree = build(&morphology, 0.1, -70.0, body);
            for _ in 0..(Milliseconds(100.0) / tables.dt()) as usize {
                tree.step(&tables);
            }
            let drift = (0..tree.points()).map(|point| (tree.voltage(point).0 + 70.0).abs());
            assert!(drift.fold(0.0, f64::max) < 1e-6);
        }
    }

    #[test]
    fn a_thin_cortical_axon_at_body_temperature_conducts_brief_spikes() {
        let mut morphology = Morphology::default();
        let start = morphology.node(NodeMembrane::Point);
        let end = morphology.node(NodeMembrane::Point);
        let body = Celsius(37.0);
        let membrane = cortical(0.1, 0.03).resting_at(Millivolts(-70.0), body);
        cable(&mut morphology, (start, end), (2000.0, 0.6), membrane);
        let tables = Tables::new(DT, body);
        let mut tree = build(&morphology, 0.3, -70.0, body);
        let mut arrivals = [None; 2];
        let mut above = 0;
        for step in 0..(Milliseconds(30.0) / tables.dt()) as usize {
            let t = (step as f64 + 0.5) * tables.dt();
            let during = (Milliseconds(1.0)..Milliseconds(1.25)).contains(&t);
            tree.inject(
                tree.node_point(start),
                Nanoamperes(if during { 0.5 } else { 0.0 }),
            );
            tree.step(&tables);
            for (arrival, fraction) in arrivals.iter_mut().zip([0.3, 0.7]) {
                if arrival.is_none() && tree.cable_voltage(0, fraction) > Millivolts(-20.0) {
                    *arrival = Some(t);
                }
            }
            above += usize::from(tree.cable_voltage(0, 0.5) > Millivolts(-40.0));
        }
        let [Some(a), Some(b)] = arrivals else {
            panic!("no spike: {arrivals:?}")
        };
        let speed = 0.4 * 2000.0 / (b - a).0 / 1000.0;
        assert!((0.2..0.4).contains(&speed), "speed {speed} m/s");
        let width = (above as f64 * tables.dt()).0;
        assert!((0.5..1.2).contains(&width), "{width} ms above -40 mV");
    }
}
