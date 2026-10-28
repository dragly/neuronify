use std::collections::{HashMap, HashSet, VecDeque};

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::SEGMENT_LENGTH;
use crate::elements::{spawn_cable, spawn_compartment};

pub struct Drag {
    entity: Entity,
    offset: Vec3,
    followers: Vec<(Entity, f32)>,
    stretching_chains: Vec<Vec<Entity>>,
}

impl Drag {
    pub fn new(world: &World, entity: Entity, offset: Vec3) -> Self {
        let mut drag = Drag {
            entity,
            offset,
            followers: Vec::new(),
            stretching_chains: Vec::new(),
        };
        drag.measure(world);
        drag
    }

    pub fn move_to(&mut self, world: &mut World, pointer: Vec3) {
        let target = (pointer + self.offset).with_y(0.0);
        let shift = match world.get::<&mut Position>(self.entity) {
            Ok(mut dragged) => {
                let shift = target - dragged.position;
                dragged.position = target;
                shift
            }
            Err(_) => return,
        };
        if shift == Vec3::ZERO {
            return;
        }
        for &(follower, follows) in &self.followers {
            if let Ok(mut position) = world.get::<&mut Position>(follower) {
                position.position += follows * shift;
            }
        }
        let mut resampled = false;
        for chain in &self.stretching_chains {
            resampled |= resample(world, chain);
        }
        if resampled {
            self.measure(world);
        }
    }

    fn measure(&mut self, world: &World) {
        let cables = Cables::of(world);
        let reached = Reached::from(world, self.entity, &cables);
        let follows = reached.follows();
        self.followers = reached
            .nodes
            .iter()
            .zip(&follows)
            .zip(&reached.free)
            .filter(|(_, &free)| free)
            .map(|((&node, &follows), _)| (node, follows as f32))
            .collect();
        let free: HashSet<Entity> = self.followers.iter().map(|&(node, _)| node).collect();
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

    fn follows(&self) -> Vec<f64> {
        let pairs: Vec<(usize, usize)> = self.links.iter().map(|&(_, a, b)| (a, b)).collect();
        let mut fixed = vec![0.0; self.nodes.len()];
        fixed[0] = 1.0;
        harmonic(&self.free, &fixed, &pairs)
    }
}

fn harmonic(free: &[bool], fixed: &[f64], pairs: &[(usize, usize)]) -> Vec<f64> {
    let laplacian = |values: &[f64]| -> Vec<f64> {
        let mut result = vec![0.0; values.len()];
        for &(a, b) in pairs {
            let flow = values[a] - values[b];
            result[a] += flow;
            result[b] -= flow;
        }
        for (result, &free) in result.iter_mut().zip(free) {
            if !free {
                *result = 0.0;
            }
        }
        result
    };
    let dot = |a: &[f64], b: &[f64]| -> f64 { a.iter().zip(b).map(|(a, b)| a * b).sum() };

    let mut solution = vec![0.0; fixed.len()];
    let mut residual: Vec<f64> = laplacian(fixed).iter().map(|value| -value).collect();
    let mut direction = residual.clone();
    let mut residual_norm = dot(&residual, &residual);
    for _ in 0..2 * fixed.len() {
        if residual_norm < 1e-20 {
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
        .map(|(x, fixed)| x + fixed)
        .collect()
}

fn resample(world: &mut World, chain: &[Entity]) -> bool {
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
        world.despawn(compartment).unwrap();
        world.despawn(cable).unwrap();
    }
    let mut compartments = interior[..kept].to_vec();
    let mut previous = nodes[kept];
    for _ in kept + 1..segments {
        let compartment = spawn_compartment_like(world, [previous, end]);
        spawn_cable(world, previous, compartment, false);
        compartments.push(compartment);
        previous = compartment;
    }
    world
        .get::<&mut Connection>(chain[chain.len() - 1])
        .unwrap()
        .from = previous;
    for (step, &compartment) in compartments.iter().enumerate() {
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
