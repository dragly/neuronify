use glam::Vec3;
use hecs::Entity;
use std::f32::consts::{PI, TAU};

use crate::components::*;
use crate::constants::*;
use crate::elements::{spawn_cable, spawn_compartment};

/// Apical dendrites grow this way, which is up on the screen with the default camera.
const APICAL_DIRECTION: Vec3 = Vec3::X;
/// Arms of the star that an inhibitory neuron grows.
const STAR_ARMS: usize = 5;
/// How far each step may turn sideways, as a fraction of the step length.
const WANDER: f32 = 0.45;
/// The apical dendrite runs nearly straight.
const APICAL_WANDER: f32 = 0.12;

fn wander(
    direction: Vec3,
    angle: f32,
    seed: f32,
    dendrite: usize,
    step: usize,
    scale: Vec3,
    amount: f32,
) -> Vec3 {
    let dx = (seed * scale.x + dendrite as f32 * 13.7 + step as f32 * 17.3).sin() * amount;
    let dz = (seed * scale.z + dendrite as f32 * 7.7 + step as f32 * 23.1).cos() * amount;
    let wandered = (direction + Vec3::new(dx, 0.0, dz)).normalize_or_zero();
    if wandered.length_squared() < 0.01 {
        Vec3::new(angle.cos(), 0.0, angle.sin())
    } else {
        wandered
    }
}

/// A dendrite or branch of `steps` compartments, starting at `parent`.
struct Chain {
    angle: f32,
    seed: f32,
    dendrite: usize,
    steps: usize,
    scale: Vec3,
    wander: f32,
}

fn spawn_chain(
    world: &mut hecs::World,
    parent: Entity,
    origin: Vec3,
    chain: Chain,
    neuron_type: &NeuronType,
) -> Vec<(Entity, Vec3)> {
    let mut direction = Vec3::new(chain.angle.cos(), 0.0, chain.angle.sin());
    let mut position = origin;
    let mut previous = parent;
    let mut spawned = Vec::with_capacity(chain.steps);
    for step in 0..chain.steps {
        direction = wander(
            direction,
            chain.angle,
            chain.seed,
            chain.dendrite,
            step,
            chain.scale,
            chain.wander,
        );
        position += direction * SEGMENT_LENGTH;
        let compartment = spawn_compartment(world, position, *neuron_type);
        world.insert_one(compartment, Dendrite).unwrap();
        spawn_cable(world, previous, compartment, false);
        spawned.push((compartment, position));
        previous = compartment;
    }
    spawned
}

const TRUNK_SCALE: Vec3 = Vec3::new(7.3, 0.0, 11.1);
const BRANCH_SCALE: Vec3 = Vec3::new(5.9, 0.0, 8.7);

/// Grows dendrites out of `soma` in the shape of its type.
///
/// An excitatory neuron is pyramidal, with a long apical dendrite along
/// `APICAL_DIRECTION` that ends in a tuft, and short basal dendrites on the
/// other side. An inhibitory neuron gets a compact star.
///
/// The dendrites are wired from the soma outwards, so a spike in the soma travels
/// out along them, while nothing flows back into the soma.
pub fn spawn_dendrites(world: &mut hecs::World, soma: Entity) {
    let (Some(origin), Some(neuron_type)) = (
        world.get::<&Position>(soma).ok().map(|p| p.position),
        world.get::<&NeuronType>(soma).ok().map(|t| *t),
    ) else {
        return;
    };
    // The shape follows from the entity id, so that each neuron looks different
    let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let seed = soma.id() as f32 * golden_ratio;
    match neuron_type {
        NeuronType::Excitatory => spawn_pyramidal(world, soma, origin, seed, &neuron_type),
        NeuronType::Inhibitory => spawn_star(world, soma, origin, seed, &neuron_type),
    }
}

fn spawn_pyramidal(
    world: &mut hecs::World,
    soma: Entity,
    origin: Vec3,
    seed: f32,
    neuron_type: &NeuronType,
) {
    let apical = APICAL_DIRECTION.z.atan2(APICAL_DIRECTION.x);
    let branch = |angle: f32, dendrite: usize| Chain {
        angle,
        seed: seed + dendrite as f32 * 5.0,
        dendrite,
        steps: 2,
        scale: BRANCH_SCALE,
        wander: WANDER,
    };
    let trunk = spawn_chain(
        world,
        soma,
        origin,
        Chain {
            angle: apical,
            seed,
            dendrite: 0,
            steps: 6,
            scale: TRUNK_SCALE,
            wander: APICAL_WANDER,
        },
        neuron_type,
    );
    // An oblique branch partway up, and a tuft of two at the top
    if let Some(&(fork, position)) = trunk.get(2) {
        spawn_chain(world, fork, position, branch(apical + 1.0, 1), neuron_type);
    }
    if let Some(&(top, position)) = trunk.last() {
        spawn_chain(world, top, position, branch(apical - 0.6, 2), neuron_type);
        spawn_chain(world, top, position, branch(apical + 0.6, 3), neuron_type);
    }
    // Basal dendrites spread out on the other side of the soma
    for (i, offset) in [-1.3_f32, -0.45, 0.45, 1.3].into_iter().enumerate() {
        let dendrite = 4 + i;
        let angle = apical + PI + offset + (seed * 2.3 + i as f32).sin() * 0.15;
        let arm = spawn_chain(
            world,
            soma,
            origin,
            Chain {
                angle,
                seed,
                dendrite,
                steps: 3,
                scale: TRUNK_SCALE,
                wander: WANDER,
            },
            neuron_type,
        );
        if let Some(&(fork, position)) = arm.get(1) {
            let outwards = angle + 0.9 * offset.signum();
            spawn_chain(
                world,
                fork,
                position,
                branch(outwards, dendrite),
                neuron_type,
            );
        }
    }
}

fn spawn_star(
    world: &mut hecs::World,
    soma: Entity,
    origin: Vec3,
    seed: f32,
    neuron_type: &NeuronType,
) {
    for dendrite in 0..STAR_ARMS {
        let angle = TAU * dendrite as f32 / STAR_ARMS as f32 + seed * 0.5;
        let arm = spawn_chain(
            world,
            soma,
            origin,
            Chain {
                angle,
                seed,
                dendrite,
                steps: 3,
                scale: TRUNK_SCALE,
                wander: WANDER,
            },
            neuron_type,
        );
        if let Some(&(fork, position)) = arm.get(1) {
            let angle = angle + 1.1 + (seed * 3.1 + dendrite as f32 * 2.7).sin() * 0.6;
            spawn_chain(
                world,
                fork,
                position,
                Chain {
                    angle,
                    seed: seed + dendrite as f32 * 5.0,
                    dendrite,
                    steps: 2,
                    scale: BRANCH_SCALE,
                    wander: WANDER,
                },
                neuron_type,
            );
        }
    }
}

/// The compartments of the dendrites and axons that grow out of `root`.
pub fn compartment_tree(world: &hecs::World, root: Entity) -> Vec<Entity> {
    let mut tree = Vec::new();
    let mut frontier = vec![root];
    while let Some(parent) = frontier.pop() {
        for (_, connection) in world.query::<&Connection>().iter() {
            if connection.from == parent
                && world.get::<&Compartment>(connection.to).is_ok()
                && !tree.contains(&connection.to)
            {
                tree.push(connection.to);
                frontier.push(connection.to);
            }
        }
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::step_simulation;

    fn driven_neuron(world: &mut hecs::World) -> Entity {
        let neuron = world.spawn((
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            Position {
                position: Vec3::ZERO,
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
                to: neuron,
                strength: 1.0,
                directional: true,
            },
            ImmediateFireSynapse::default(),
        ));
        neuron
    }

    /// Steps in which `neuron` fired, and the highest voltage seen in `tree`.
    fn run(world: &mut hecs::World, neuron: Entity, tree: &[Entity]) -> (Vec<usize>, f64) {
        let mut spikes = Vec::new();
        let mut peak = f64::MIN;
        let mut time = 0.0;
        for step in 0..4000 {
            step_simulation(world, &mut time, 1, None);
            if world.get::<&LeakyDynamics>(neuron).unwrap().time_since_fire == 0.0 {
                spikes.push(step);
            }
            for &compartment in tree {
                peak = peak.max(world.get::<&Compartment>(compartment).unwrap().voltage);
            }
        }
        (spikes, peak)
    }

    #[test]
    fn spikes_travel_out_but_not_back() {
        let mut plain = hecs::World::new();
        let plain_neuron = driven_neuron(&mut plain);
        let (plain_spikes, _) = run(&mut plain, plain_neuron, &[]);

        let mut branched = hecs::World::new();
        let branched_neuron = driven_neuron(&mut branched);
        spawn_dendrites(&mut branched, branched_neuron);
        let tree = compartment_tree(&branched, branched_neuron);
        let (branched_spikes, peak) = run(&mut branched, branched_neuron, &tree);

        assert!(!plain_spikes.is_empty());
        assert_eq!(plain_spikes, branched_spikes);
        assert!(peak > BRIDGE_VOLTAGE_THRESHOLD, "peak was {peak}");
    }

    #[test]
    fn the_tree_holds_every_compartment() {
        let mut world = hecs::World::new();
        let neuron = driven_neuron(&mut world);
        let other = driven_neuron(&mut world);
        spawn_dendrites(&mut world, neuron);
        let spawned = world.query::<&Dendrite>().iter().count();
        spawn_dendrites(&mut world, other);
        let tree = compartment_tree(&world, neuron);
        assert_eq!(tree.len(), spawned);
        assert!(compartment_tree(&world, other)
            .iter()
            .all(|compartment| !tree.contains(compartment)));
    }

    /// How far the dendrites of a fresh neuron reach along and against `APICAL_DIRECTION`.
    fn reach(neuron_type: NeuronType) -> (f32, f32) {
        let mut world = hecs::World::new();
        let soma = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            neuron_type,
        ));
        spawn_dendrites(&mut world, soma);
        compartment_tree(&world, soma)
            .iter()
            .map(|&compartment| {
                let position = world.get::<&Position>(compartment).unwrap().position;
                position.dot(APICAL_DIRECTION)
            })
            .fold((0.0, 0.0), |(up, down), along| {
                (up.max(along), down.min(along))
            })
    }

    #[test]
    fn pyramidal_neurons_reach_up_and_interneurons_stay_compact() {
        let (up, down) = reach(NeuronType::Excitatory);
        assert!(up > 12.0, "the apical dendrite reached {up}");
        assert!(down < -3.0, "the basal dendrites reached {down}");
        let (up, down) = reach(NeuronType::Inhibitory);
        assert!(up < 9.0 && down > -9.0, "the star reached {up} and {down}");
    }
}
