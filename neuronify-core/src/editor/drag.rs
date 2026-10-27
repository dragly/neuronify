use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::{PI, TAU};

use glam::{DVec2, Vec3};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::SEGMENT_LENGTH;
use crate::elements::{spawn_cable, spawn_compartment};

use super::history::Recording;

pub struct Drag {
    entity: Entity,
    offset: Vec3,
    pose: Pose,
    stretching_chains: Vec<Vec<Entity>>,
}

impl Drag {
    pub fn new(world: &World, entity: Entity, offset: Vec3) -> Self {
        let mut drag = Drag {
            entity,
            offset,
            pose: Pose::default(),
            stretching_chains: Vec::new(),
        };
        drag.measure(world);
        drag
    }

    pub fn move_to(&mut self, world: &mut World, recording: &mut Recording, pointer: Vec3) {
        let target = (pointer + self.offset).with_y(0.0);
        recording.edit(world, self.entity);
        match world.get::<&mut Position>(self.entity) {
            Ok(mut dragged) if dragged.position != target => dragged.position = target,
            _ => return,
        }
        for (follower, moved) in self.pose.move_to(planar(target)) {
            recording.edit(world, follower);
            if let Ok(mut position) = world.get::<&mut Position>(follower) {
                position.position += Vec3::new(moved.x as f32, 0.0, moved.y as f32);
            }
        }
        let mut resampled = false;
        for chain in &self.stretching_chains {
            resampled |= resample(world, recording, chain);
        }
        if resampled {
            self.measure(world);
        }
    }

    fn measure(&mut self, world: &World) {
        let cables = Cables::of(world);
        let reached = Reached::from(world, self.entity, &cables);
        self.pose = Pose::of(world, &reached);
        let follows = &self.pose.follows;
        let free: HashSet<Entity> = reached
            .nodes
            .iter()
            .zip(&reached.free)
            .filter(|(_, &free)| free)
            .map(|(&node, _)| node)
            .collect();
        self.stretching_chains = reached
            .links
            .iter()
            .filter(|&&(_, a, b)| (follows[a] - follows[b]).abs() > 1e-4)
            .filter_map(|&(cable, _, _)| cables.chain_from(cable, &free))
            .collect();
    }
}

struct Cable {
    entity: Entity,
    from: Entity,
    to: Entity,
}

struct Cables {
    all: Vec<Cable>,
    at: HashMap<Entity, Vec<usize>>,
    connections_at: HashMap<Entity, usize>,
}

impl Cables {
    fn of(world: &World) -> Self {
        let mut cables = Cables {
            all: Vec::new(),
            at: HashMap::new(),
            connections_at: HashMap::new(),
        };
        for (entity, (connection, cable)) in world
            .query::<(&Connection, Option<&CompartmentCurrent>)>()
            .iter()
        {
            for end in [connection.from, connection.to] {
                *cables.connections_at.entry(end).or_default() += 1;
            }
            if cable.is_some() {
                for end in [connection.from, connection.to] {
                    cables.at.entry(end).or_default().push(cables.all.len());
                }
                cables.all.push(Cable {
                    entity,
                    from: connection.from,
                    to: connection.to,
                });
            }
        }
        cables
    }

    fn onwards_from(&self, node: Entity, free: &HashSet<Entity>) -> Option<&Cable> {
        if !free.contains(&node) || self.connections_at.get(&node) != Some(&2) {
            return None;
        }
        let [first, second] = self.at.get(&node)?[..] else {
            return None;
        };
        let (first, second) = (&self.all[first], &self.all[second]);
        if first.to == node && second.from == node {
            Some(second)
        } else if second.to == node && first.from == node {
            Some(first)
        } else {
            None
        }
    }

    fn chain_from(&self, start: usize, free: &HashSet<Entity>) -> Option<Vec<Entity>> {
        let first = &self.all[start];
        if self.onwards_from(first.from, free).is_some() {
            return None;
        }
        let mut chain = vec![first.entity];
        let mut end = first.to;
        while let Some(next) = self.onwards_from(end, free) {
            if chain.len() > self.all.len() {
                return None;
            }
            chain.push(next.entity);
            end = next.to;
        }
        Some(chain)
    }
}

struct Reached {
    nodes: Vec<Entity>,
    free: Vec<bool>,
    links: Vec<(usize, usize, usize)>,
}

impl Reached {
    fn from(world: &World, entity: Entity, cables: &Cables) -> Self {
        let mut reached = Reached {
            nodes: vec![entity],
            free: vec![false],
            links: Vec::new(),
        };
        let mut index = HashMap::from([(entity, 0)]);
        let mut queue = VecDeque::from([entity]);
        while let Some(node) = queue.pop_front() {
            for &cable in cables.at.get(&node).into_iter().flatten() {
                let Cable { from, to, .. } = cables.all[cable];
                let other = if from == node { to } else { from };
                if index.contains_key(&other) {
                    continue;
                }
                if !world.satisfies::<&Position>(other).unwrap_or(false) {
                    continue;
                }
                let moves = world.satisfies::<&SpatialDynamics>(other).unwrap_or(false);
                if moves {
                    queue.push_back(other);
                }
                index.insert(other, reached.nodes.len());
                reached.nodes.push(other);
                reached.free.push(moves);
            }
        }
        reached.links = cables
            .all
            .iter()
            .enumerate()
            .filter_map(|(cable, Cable { from, to, .. })| {
                Some((cable, *index.get(from)?, *index.get(to)?))
            })
            .collect();
        reached
    }
}

#[derive(Default)]
struct Pose {
    nodes: Vec<Entity>,
    free: Vec<bool>,
    pairs: Vec<(usize, usize)>,
    detached: Vec<(usize, usize)>,
    rest: Vec<DVec2>,
    follows: Vec<f64>,
    anchors: Vec<Anchor>,
    anchored: Vec<f64>,
    displacement: Vec<DVec2>,
}

struct Anchor {
    node: usize,
    coupling: f64,
    turn: f64,
}

impl Pose {
    fn of(world: &World, reached: &Reached) -> Self {
        let count = reached.nodes.len();
        let pairs: Vec<(usize, usize)> = reached.links.iter().map(|&(_, a, b)| (a, b)).collect();
        let rest = reached
            .nodes
            .iter()
            .map(|&node| {
                world
                    .get::<&Position>(node)
                    .map_or(DVec2::ZERO, |p| planar(p.position))
            })
            .collect();
        let mut fixed = vec![0.0; count];
        fixed[0] = 1.0;
        let follows = harmonic(&reached.free, &pairs, &fixed);
        let mut coupling = vec![0.0; count];
        for &(a, b) in &pairs {
            coupling[a] += follows[b] - follows[a];
            coupling[b] += follows[a] - follows[b];
        }
        let anchors: Vec<Anchor> = (1..count)
            .filter(|&node| !reached.free[node] && coupling[node] > 0.0)
            .map(|node| Anchor {
                node,
                coupling: coupling[node],
                turn: 0.0,
            })
            .collect();
        let detached: Vec<(usize, usize)> = pairs
            .iter()
            .copied()
            .filter(|&(a, b)| a != 0 && b != 0)
            .collect();
        let mut at_anchors = vec![0.0; count];
        for anchor in &anchors {
            at_anchors[anchor.node] = 1.0;
        }
        let anchored = harmonic(&reached.free, &detached, &at_anchors);
        Pose {
            nodes: reached.nodes.clone(),
            free: reached.free.clone(),
            pairs,
            detached,
            rest,
            follows,
            anchors,
            anchored,
            displacement: vec![DVec2::ZERO; count],
        }
    }

    fn move_to(&mut self, target: DVec2) -> Vec<(Entity, DVec2)> {
        let turns = self.turns_towards(target);
        let guide: Vec<DVec2> = self
            .pairs
            .iter()
            .map(|&(a, b)| {
                let edge = self.rest[a] - self.rest[b];
                let turn = match (a, b) {
                    (0, other) | (other, 0) => turns[other],
                    _ => 0.5 * (turns[a] + turns[b]),
                };
                DVec2::from_angle(turn).rotate(edge) - edge
            })
            .collect();
        let mut fixed = vec![DVec2::ZERO; self.nodes.len()];
        fixed[0] = target - self.rest[0];
        let displacement = fit(&self.free, &self.pairs, &fixed, &guide);
        let moved = (0..self.nodes.len())
            .filter(|&node| self.free[node])
            .map(|node| {
                (
                    self.nodes[node],
                    displacement[node] - self.displacement[node],
                )
            })
            .collect();
        self.displacement = displacement;
        moved
    }

    fn turns_towards(&mut self, target: DVec2) -> Vec<f64> {
        let start = self.rest[0];
        let mut fixed = vec![0.0; self.nodes.len()];
        let (mut weighted, mut total) = (0.0, 0.0);
        for anchor in &mut self.anchors {
            let before = self.rest[anchor.node] - start;
            let after = self.rest[anchor.node] - target;
            if before.length() > 1e-6 && after.length() > 1e-6 {
                anchor.turn += wrapped(before.angle_to(after) - anchor.turn);
            }
            fixed[anchor.node] = anchor.turn;
            weighted += anchor.coupling * anchor.turn;
            total += anchor.coupling;
        }
        let mean = if total > 0.0 { weighted / total } else { 0.0 };
        harmonic(&self.free, &self.detached, &fixed)
            .iter()
            .zip(&self.anchored)
            .map(|(turn, anchored)| turn + (1.0 - anchored) * mean)
            .collect()
    }
}

fn planar(position: Vec3) -> DVec2 {
    DVec2::new(position.x as f64, position.z as f64)
}

fn wrapped(angle: f64) -> f64 {
    (angle + PI).rem_euclid(TAU) - PI
}

fn harmonic(free: &[bool], pairs: &[(usize, usize)], fixed: &[f64]) -> Vec<f64> {
    let fixed: Vec<DVec2> = fixed.iter().map(|&value| DVec2::new(value, 0.0)).collect();
    let unguided = vec![DVec2::ZERO; pairs.len()];
    fit(free, pairs, &fixed, &unguided)
        .iter()
        .map(|value| value.x)
        .collect()
}

fn fit(free: &[bool], pairs: &[(usize, usize)], fixed: &[DVec2], guide: &[DVec2]) -> Vec<DVec2> {
    let gather = |difference: &dyn Fn(usize, usize, usize) -> DVec2| -> Vec<DVec2> {
        let mut result = vec![DVec2::ZERO; free.len()];
        for (pair, &(a, b)) in pairs.iter().enumerate() {
            let flow = difference(pair, a, b);
            result[a] += flow;
            result[b] -= flow;
        }
        for (result, &free) in result.iter_mut().zip(free) {
            if !free {
                *result = DVec2::ZERO;
            }
        }
        result
    };
    let laplacian = |values: &[DVec2]| gather(&|_, a, b| values[a] - values[b]);
    let dot = |a: &[DVec2], b: &[DVec2]| -> f64 { a.iter().zip(b).map(|(a, b)| a.dot(*b)).sum() };

    let mut solution = vec![DVec2::ZERO; fixed.len()];
    let mut residual = gather(&|pair, a, b| guide[pair] - (fixed[a] - fixed[b]));
    let mut direction = residual.clone();
    let mut residual_norm = dot(&residual, &residual);
    let enough = 1e-20 * residual_norm.max(1.0);
    for _ in 0..2 * fixed.len() {
        if residual_norm < enough {
            break;
        }
        let along = laplacian(&direction);
        let step = residual_norm / dot(&direction, &along);
        for i in 0..fixed.len() {
            solution[i] += step * direction[i];
            residual[i] -= step * along[i];
        }
        let next_norm = dot(&residual, &residual);
        for i in 0..fixed.len() {
            direction[i] = residual[i] + next_norm / residual_norm * direction[i];
        }
        residual_norm = next_norm;
    }
    solution
        .iter()
        .zip(fixed)
        .map(|(x, fixed)| *x + *fixed)
        .collect()
}

fn resample(world: &mut World, recording: &mut Recording, chain: &[Entity]) -> bool {
    let Some(nodes) = chain_nodes(world, chain) else {
        return false;
    };
    let Some(points) = nodes
        .iter()
        .map(|&node| world.get::<&Position>(node).ok().map(|p| p.position))
        .collect::<Option<Vec<Vec3>>>()
    else {
        return false;
    };
    let length: f32 = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum();
    if !length.is_finite() {
        return false;
    }
    let (start, end) = (nodes[0], nodes[nodes.len() - 1]);
    let is_compartment = |node| world.satisfies::<&Compartment>(node).unwrap_or(false);
    let fewest = if is_compartment(start) || is_compartment(end) {
        1
    } else {
        2
    };
    let segments = ((length / SEGMENT_LENGTH).round() as usize).max(fewest);
    if segments == chain.len() {
        return false;
    }

    let interior = &nodes[1..nodes.len() - 1];
    let kept = interior.len().min(segments - 1);
    for (&compartment, &cable) in interior[kept..].iter().zip(&chain[kept..]) {
        recording.take(world, compartment);
        recording.take(world, cable);
    }
    let mut compartments = interior[..kept].to_vec();
    let mut previous = nodes[kept];
    for _ in kept + 1..segments {
        let compartment = spawn_compartment_like(world, [previous, end]);
        spawn_cable(world, previous, compartment, false);
        compartments.push(compartment);
        previous = compartment;
    }
    let last = chain[chain.len() - 1];
    recording.edit(world, last);
    world.get::<&mut Connection>(last).unwrap().from = previous;
    for (step, &compartment) in compartments.iter().enumerate() {
        recording.edit(world, compartment);
        let distance = (step + 1) as f32 * length / segments as f32;
        world.get::<&mut Position>(compartment).unwrap().position = point_along(&points, distance);
    }
    true
}

fn chain_nodes(world: &World, chain: &[Entity]) -> Option<Vec<Entity>> {
    let ends = |cable| world.get::<&Connection>(cable).ok().map(|c| (c.from, c.to));
    let mut nodes = vec![ends(chain[0])?.0];
    for &cable in chain {
        let (from, to) = ends(cable)?;
        if from != nodes[nodes.len() - 1] {
            return None;
        }
        nodes.push(to);
    }
    Some(nodes)
}

fn point_along(points: &[Vec3], distance: f32) -> Vec3 {
    let mut remaining = distance;
    for pair in points.windows(2) {
        let length = pair[0].distance(pair[1]);
        if remaining <= length && length > 0.0 {
            return pair[0].lerp(pair[1], remaining / length);
        }
        remaining -= length;
    }
    points[points.len() - 1]
}

fn spawn_compartment_like(world: &mut World, neighbours: [Entity; 2]) -> Entity {
    let neuron_type = neighbours
        .into_iter()
        .find_map(|neighbour| world.get::<&NeuronType>(neighbour).ok().map(|t| *t))
        .unwrap_or_default();
    let state = neighbours
        .into_iter()
        .find_map(|neighbour| world.get::<&Compartment>(neighbour).ok().map(|c| *c));
    let dendrite = neighbours
        .into_iter()
        .any(|neighbour| world.satisfies::<&Dendrite>(neighbour).unwrap_or(false));
    let compartment = spawn_compartment(world, Vec3::ZERO, neuron_type);
    if let Some(state) = state {
        world.insert_one(compartment, state).unwrap();
    }
    if dendrite {
        world.insert_one(compartment, Dendrite).unwrap();
    }
    compartment
}
