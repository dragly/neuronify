use std::cmp::Ordering;

use glam::Vec3;
use hecs::Entity;
use rand::Rng;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, Deletable, LeakyNeuron, NeuronType, Position,
    SpatialDynamics, COUPLING_CAPACITANCE, MIN_CREATION_DISTANCE_AXON,
};

use crate::components::*;
use crate::constants::*;
use crate::simulation::game;

pub struct AiState {
    pub tick_counter: u32,
    pub rng: rand::rngs::ThreadRng,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            tick_counter: 0,
            rng: rand::thread_rng(),
        }
    }
}

impl AiState {
    pub fn new() -> Self {
        Self::default()
    }
}

fn dist_2d(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}

fn cmp_f32(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

pub fn ai_tick(
    world: &mut hecs::World,
    ai: &mut AiState,
    dish: &game::PetriDish,
    economy: &mut PlayerEconomy,
) {
    ai_tick_for_player(world, ai, dish, PlayerId::Player2, economy);
}

pub fn ai_tick_for_player(
    world: &mut hecs::World,
    ai: &mut AiState,
    dish: &game::PetriDish,
    player: PlayerId,
    economy: &mut PlayerEconomy,
) {
    ai.tick_counter += 1;
    if !ai.tick_counter.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }

    let opponent = match player {
        PlayerId::Player1 => PlayerId::Player2,
        PlayerId::Player2 => PlayerId::Player1,
    };

    // Priority: ensure we have glial coverage near our vessels
    let has_glial = world
        .query::<(&GlialCell, &Ownership)>()
        .iter()
        .any(|(_, (_, o))| o.player == player);
    if !has_glial {
        ai_place_glial(world, player, economy);
        return;
    }

    let roll: f64 = ai.rng.gen();
    if roll < AI_EXPAND_CHANCE {
        ai_expand(world, ai, dish, player, economy);
    } else if roll < AI_EXPAND_CHANCE + AI_DEFEND_CHANCE {
        ai_defend(world, ai, player, opponent, economy);
    } else {
        ai_attack(world, ai, player, opponent, economy);
    }
}

/// Place a glial cell near the nearest owned blood vessel.
fn ai_place_glial(world: &mut hecs::World, player: PlayerId, economy: &mut PlayerEconomy) {
    if !game::try_spend_blocks(economy, GLIAL_COST) {
        return;
    }

    // Find nearest owned vessel
    let vessel_pos = world
        .query::<(&BloodVessel, &Position, &Ownership)>()
        .iter()
        .filter(|(_, (_, _, o))| o.player == player)
        .map(|(_, (_, p, _))| p.position)
        .next();

    // If no owned vessel, find nearest unowned vessel near origin
    let pos = vessel_pos.or_else(|| {
        world
            .query::<(&OriginNeuron, &Position)>()
            .iter()
            .find(|(_, (o, _))| o.player == player)
            .map(|(_, (_, p))| p.position)
    });

    if let Some(pos) = pos {
        world.spawn((
            Position {
                position: pos + Vec3::new(3.0, 0.0, 0.0),
            },
            GlialCell::default(),
            Ownership { player },
            SpatialDynamics {
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
            },
            Deletable {},
        ));
    }
}

/// Find all neurons owned by the given player.
fn ai_neurons(world: &hecs::World, player: PlayerId) -> Vec<(Entity, Vec3, f64)> {
    world
        .query::<(&Position, &Ownership, &MetabolicState)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o, _))| o.player == player)
        .map(|(e, (p, _, m))| (e, p.position, m.energy))
        .collect()
}

/// Find the nearest unowned blood vessel to expand toward.
fn find_target_vessel(
    world: &hecs::World,
    dish: &game::PetriDish,
    player: PlayerId,
) -> Option<Vec3> {
    let ai_neuron_positions: Vec<Vec3> = ai_neurons(world, player)
        .iter()
        .map(|(_, p, _)| *p)
        .collect();
    let ai_origin = match ai_neuron_positions.first() {
        Some(p) => *p,
        None => return None,
    };

    world
        .query::<(&BloodVessel, &Position)>()
        .without::<&Ownership>()
        .iter()
        .filter_map(|(_, (_, pos))| {
            let vpos = pos.position;
            // Skip if outside dish
            if dist_2d(vpos, dish.center) > dish.radius {
                return None;
            }
            Some((vpos, dist_2d(vpos, ai_origin)))
        })
        .min_by(|a, b| cmp_f32(a.1, b.1))
        .map(|(p, _)| p)
}

/// Expand: place a glial cell or neuron toward the nearest unclaimed vessel.
fn ai_expand(
    world: &mut hecs::World,
    ai: &mut AiState,
    dish: &game::PetriDish,
    player: PlayerId,
    economy: &mut PlayerEconomy,
) {
    let neurons = ai_neurons(world, player);
    if neurons.is_empty() {
        return;
    }

    let target = match find_target_vessel(world, dish, player) {
        Some(t) => t,
        None => return,
    };

    // Find the AI neuron closest to the target
    let (source_entity, source_pos, _source_energy) = match neurons
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(a.1, target), dist_2d(b.1, target)))
        .copied()
    {
        Some(n) => n,
        None => return,
    };

    let dir = Vec3::new(target.x - source_pos.x, 0.0, target.z - source_pos.z);
    let dist = dir.length();
    if dist < 2.0 {
        return;
    }
    let dir_norm = dir / dist;
    let step = dist.min(BUILD_RANGE * 0.7);
    let new_pos = source_pos + dir_norm * step;

    if dist_2d(new_pos, dish.center) > dish.radius * 0.95 {
        return;
    }

    // If target position lacks glial coverage, place a glial cell instead of a neuron
    if !game::is_within_glial_range(world, new_pos, player) {
        // Place glial cell if within vessel range
        if game::is_within_vessel_range(world, new_pos, player) {
            if !game::try_spend_blocks(economy, GLIAL_COST) {
                return;
            }
            world.spawn((
                Position { position: new_pos },
                GlialCell::default(),
                Ownership { player },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
                Deletable {},
            ));
        }
        return;
    }

    let total_cost = NEURON_SPAWN_COST + COMPARTMENT_SPAWN_COST * 3.0;
    if !game::try_spend_blocks(economy, total_cost) {
        return;
    }

    let neuron_type = if ai.rng.gen::<f64>() < 0.8 {
        NeuronType::Excitatory
    } else {
        NeuronType::Inhibitory
    };

    let new_entity = crate::spawning::spawn_neuron(world, new_pos, neuron_type.clone(), player);

    spawn_axon_between(world, source_entity, new_entity, &neuron_type, player);
}

/// Spawn compartment chain between two neurons and connect them.
fn spawn_axon_between(
    world: &mut hecs::World,
    from_neuron: Entity,
    to_neuron: Entity,
    neuron_type: &NeuronType,
    player: PlayerId,
) {
    let from_pos = world
        .get::<&Position>(from_neuron)
        .map(|p| p.position)
        .unwrap_or(Vec3::ZERO);
    let to_pos = world
        .get::<&Position>(to_neuron)
        .map(|p| p.position)
        .unwrap_or(Vec3::ZERO);

    let dir = to_pos - from_pos;
    let dist = dir.length();
    if dist < 0.1 {
        return;
    }
    let num_compartments = ((dist / (MIN_CREATION_DISTANCE_AXON * 1.5)) as usize).max(1);

    let mut prev_entity = from_neuron;
    for i in 1..=num_compartments {
        let t = i as f32 / (num_compartments + 1) as f32;
        let pos = from_pos + dir * t;

        let comp_entity = world.spawn((
            Position { position: pos },
            Compartment {
                voltage: -50.0,
                m: 0.0,
                h: 1.0,
                n: 0.0,
                influence: 0.0,
                capacitance: COUPLING_CAPACITANCE,
                injected_current: 0.0,
                fire_impulse: 0.0,
            },
            neuron_type.clone(),
            SpatialDynamics {
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
            },
            Ownership { player },
            Deletable {},
        ));

        world.spawn((
            Connection {
                from: prev_entity,
                to: comp_entity,
                strength: 1.0,
                directional: true,
            },
            CompartmentCurrent {
                capacitance: COUPLING_CAPACITANCE,
            },
        ));

        prev_entity = comp_entity;
    }

    world.spawn((
        Connection {
            from: prev_entity,
            to: to_neuron,
            strength: 1.0,
            directional: true,
        },
        CompartmentCurrent {
            capacitance: COUPLING_CAPACITANCE,
        },
    ));
}

/// Defend: place an inhibitory neuron near the border facing the opponent.
fn ai_defend(
    world: &mut hecs::World,
    ai: &mut AiState,
    player: PlayerId,
    opponent: PlayerId,
    economy: &mut PlayerEconomy,
) {
    let neurons = ai_neurons(world, player);
    if neurons.len() < 2 {
        return;
    }

    let enemy_positions: Vec<Vec3> = world
        .query::<(&Position, &Ownership)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o))| o.player == opponent)
        .map(|(_, (p, _))| p.position)
        .collect();

    if enemy_positions.is_empty() {
        return;
    }

    // Find AI neuron closest to any enemy neuron
    let (source_entity, source_pos, _source_energy) = match neurons.iter().min_by(|a, b| {
        let da = enemy_positions
            .iter()
            .map(|p| dist_2d(a.1, *p))
            .fold(f32::INFINITY, f32::min);
        let db = enemy_positions
            .iter()
            .map(|p| dist_2d(b.1, *p))
            .fold(f32::INFINITY, f32::min);
        cmp_f32(da, db)
    }) {
        Some(n) => (n.0, n.1, n.2),
        None => return,
    };

    let cost = NEURON_SPAWN_COST + COMPARTMENT_SPAWN_COST * 2.0;
    if !game::try_spend_blocks(economy, cost) {
        return;
    }

    let nearest_enemy = enemy_positions
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(source_pos, **a), dist_2d(source_pos, **b)))
        .copied()
        .unwrap_or(Vec3::ZERO);

    let dir = Vec3::new(
        nearest_enemy.x - source_pos.x,
        0.0,
        nearest_enemy.z - source_pos.z,
    );
    let dist = dir.length();
    if dist < 3.0 {
        return;
    }
    let offset_x: f32 = ai.rng.gen::<f32>() * 4.0 - 2.0;
    let offset_z: f32 = ai.rng.gen::<f32>() * 4.0 - 2.0;
    let new_pos = source_pos
        + (dir / dist) * (dist.min(BUILD_RANGE * 0.5))
        + Vec3::new(offset_x, 0.0, offset_z);

    let new_entity = crate::spawning::spawn_neuron(world, new_pos, NeuronType::Inhibitory, player);

    spawn_axon_between(
        world,
        source_entity,
        new_entity,
        &NeuronType::Inhibitory,
        player,
    );
}

/// Attack: project an excitatory axon toward nearby enemy neurons.
fn ai_attack(
    world: &mut hecs::World,
    ai: &mut AiState,
    player: PlayerId,
    opponent: PlayerId,
    economy: &mut PlayerEconomy,
) {
    let neurons = ai_neurons(world, player);
    if neurons.len() < 4 {
        return; // Need critical mass before attacking
    }

    let enemy_neurons: Vec<(Entity, Vec3)> = world
        .query::<(&Position, &Ownership)>()
        .with::<&LeakyNeuron>()
        .iter()
        .filter(|(_, (_, o))| o.player == opponent)
        .map(|(e, (p, _))| (e, p.position))
        .collect();

    if enemy_neurons.is_empty() {
        return;
    }

    let idx = ai.rng.gen_range(0..neurons.len());
    let (source_entity, source_pos, _) = neurons[idx];

    // Find nearest enemy neuron
    let (target_entity, _target_pos) = match enemy_neurons
        .iter()
        .min_by(|a, b| cmp_f32(dist_2d(source_pos, a.1), dist_2d(source_pos, b.1)))
        .copied()
    {
        Some(n) => n,
        None => return,
    };

    let target_dist = dist_2d(source_pos, _target_pos);
    if target_dist > BUILD_RANGE * 3.0 {
        return;
    }

    let cost = COMPARTMENT_SPAWN_COST * 5.0;
    if !game::try_spend_blocks(economy, cost) {
        return;
    }

    spawn_axon_between(
        world,
        source_entity,
        target_entity,
        &NeuronType::Excitatory,
        player,
    );
}
