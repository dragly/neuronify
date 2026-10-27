use std::collections::{HashMap, VecDeque};

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;

pub struct Drag {
    entity: Entity,
    offset: Vec3,
    followers: Vec<(Entity, f32)>,
}

impl Drag {
    pub fn new(world: &World, entity: Entity, offset: Vec3) -> Self {
        let mut drag = Drag {
            entity,
            offset,
            followers: Vec::new(),
        };
        drag.measure(world);
        drag
    }

    pub fn move_to(&self, world: &mut World, pointer: Vec3) {
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
    }
}

struct Cable {
    from: Entity,
    to: Entity,
}

struct Cables {
    all: Vec<Cable>,
    at: HashMap<Entity, Vec<usize>>,
}

impl Cables {
    fn of(world: &World) -> Self {
        let mut cables = Cables {
            all: Vec::new(),
            at: HashMap::new(),
        };
        for (_, connection) in world
            .query::<&Connection>()
            .with::<&CompartmentCurrent>()
            .iter()
        {
            for end in [connection.from, connection.to] {
                cables.at.entry(end).or_default().push(cables.all.len());
            }
            cables.all.push(Cable {
                from: connection.from,
                to: connection.to,
            });
        }
        cables
    }
}

struct Reached {
    nodes: Vec<Entity>,
    free: Vec<bool>,
    links: Vec<(usize, usize)>,
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
                let Cable { from, to } = cables.all[cable];
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
            .filter_map(|Cable { from, to }| Some((*index.get(from)?, *index.get(to)?)))
            .collect();
        reached
    }

    fn follows(&self) -> Vec<f64> {
        let mut fixed = vec![0.0; self.nodes.len()];
        fixed[0] = 1.0;
        harmonic(&self.free, &fixed, &self.links)
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
