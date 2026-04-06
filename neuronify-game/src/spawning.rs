use glam::Vec3;
use hecs::Entity;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, ConnectionColor, Deletable, Inhibitory,
    LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronType, Position, Selectable, SpatialDynamics,
    StaticConnectionSource, VisualRadius, COUPLING_CAPACITANCE, NODE_RADIUS,
};
use crate::rendering::colors::glial_color;

use crate::components::*;
use crate::constants::*;

// ── Combat unit spawning ──────────────────────────────────────────────────────

/// Spawn a microglial cell — biological connection cutter.
/// Moves toward nearest enemy Compartment and drains AxonHealth.
pub fn spawn_microglial_cell(world: &mut hecs::World, position: Vec3, faction: Faction) -> Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { position });
    builder.add(MicroglialCell);
    builder.add(MobileUnit { speed: MICROGLIA_SPEED, target: None, faction, manual_target: None });
    builder.add(AxonCutter {
        shot_damage: MICROGLIA_SHOT_DAMAGE,
        shoot_cooldown: MICROGLIA_SHOOT_COOLDOWN,
        shoot_timer: 0.0,
    });
    builder.add(Health::new(MICROGLIA_HEALTH));
    builder.add(Deletable {});
    builder.add(VisualRadius { radius: 2.0 });
    if faction == Faction::Biological {
        builder.add(Ownership { player: PlayerId::Player1 });
    }
    world.spawn(builder.build())
}

/// Spawn a macrophage — biological neuron destroyer.
/// Locks onto a single neuron soma and rapidly drains its metabolic energy.
pub fn spawn_macrophage(world: &mut hecs::World, position: Vec3, faction: Faction) -> Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { position });
    builder.add(MacrophageUnit);
    builder.add(MobileUnit { speed: MACROPHAGE_SPEED, target: None, faction, manual_target: None });
    builder.add(NeuronEngulfment {
        shot_damage: MACROPHAGE_SHOT_DAMAGE,
        target: None,
        shoot_cooldown: MACROPHAGE_SHOOT_COOLDOWN,
        shoot_timer: 0.0,
        fire_range: MACROPHAGE_FIRE_RANGE,
    });
    builder.add(Health::new(MACROPHAGE_HEALTH));
    builder.add(Deletable {});
    builder.add(VisualRadius { radius: 3.5 });
    if faction == Faction::Biological {
        builder.add(Ownership { player: PlayerId::Player1 });
    }
    world.spawn(builder.build())
}

/// Spawn a T-cell — biological fast raider.
/// Moves at high speed toward the nearest enemy mobile unit; delivers a burst of
/// damage on contact, then enters a cooldown window of vulnerability.
pub fn spawn_t_cell(world: &mut hecs::World, position: Vec3, faction: Faction) -> Entity {
    world.spawn((
        Position { position },
        TCellUnit,
        MobileUnit {
            speed: TCELL_SPEED,
            target: None,
            faction,
            manual_target: None,
        },
        BurstAttack {
            damage: TCELL_BURST_DAMAGE,
            range: TCELL_BURST_RANGE,
            cooldown: TCELL_BURST_COOLDOWN,
            cooldown_timer: 0.0,
        },
        Health::new(TCELL_HEALTH),
        Deletable {},
        VisualRadius { radius: 1.2 },
    ))
}

// ─────────────────────────────────────────────────────────────────────────────

const DENDRITE_LENGTH: f32 = 2.5;
const DENDRITE_COMPARTMENTS: usize = 2;
/// How strongly each arm's direction wanders per segment (radians of random offset per unit length).
const DENDRITE_WANDER: f32 = 0.5;

/// Compute deterministic wander-based compartment positions for one dendrite arm.
/// Returns `DENDRITE_COMPARTMENTS` successive positions along the arm.
fn arm_positions(soma_pos: Vec3, arm_idx: usize, num_arms: usize, seed: f32) -> Vec<Vec3> {
    let base_angle = std::f32::consts::TAU * arm_idx as f32 / num_arms as f32 + seed * 0.5;
    let mut dir = Vec3::new(base_angle.cos(), 0.0, base_angle.sin());
    let mut pos = soma_pos;
    let mut out = Vec::with_capacity(DENDRITE_COMPARTMENTS);
    for seg in 0..DENDRITE_COMPARTMENTS {
        let wx = (seed * 7.3 + arm_idx as f32 * 13.7 + seg as f32 * 17.3).sin() * DENDRITE_WANDER;
        let wz = (seed * 11.1 + arm_idx as f32 * 7.7 + seg as f32 * 23.1).cos() * DENDRITE_WANDER;
        dir = (dir + Vec3::new(wx, 0.0, wz)).normalize_or_zero();
        if dir.length_squared() < 0.01 {
            dir = Vec3::new(base_angle.cos(), 0.0, base_angle.sin());
        }
        pos += dir * DENDRITE_LENGTH;
        out.push(pos);
    }
    out
}

/// Spawn a glial cell (astrocyte) with processes arranged radially.
pub fn spawn_glial(
    world: &mut hecs::World,
    position: Vec3,
    player: PlayerId,
    num_processes: usize,
) -> Entity {
    let soma = world.spawn((
        Position { position },
        GlialCell::default(),
        NeuronType::Excitatory,
        StaticConnectionSource {},
        Health::new(GLIAL_HEALTH),
        Ownership { player },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.3 },
        ConnectionColor(glial_color()),
    ));

    let seed = soma.id() as f32 * 2.399_963;
    for i in 0..num_processes {
        let positions = arm_positions(position, i, num_processes, seed);
        let mut prev_entity = soma;
        for comp_pos in positions {
            let compartment = world.spawn((
                Position { position: comp_pos },
                NeuronType::Excitatory,
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                GlialProcess,
                Ownership { player },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
                ConnectionColor(glial_color()),
            ));

            world.spawn((
                Connection {
                    from: prev_entity,
                    to: compartment,
                    strength: 1.0,
                    directional: false,
                },
                Deletable {},
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));

            prev_entity = compartment;
        }
    }

    soma
}

// ── Neuroblast / maturation ───────────────────────────────────────────────────

/// Spawn an immature neuroblast at `pos`.
/// It stays idle (no `MovePath`) until the player issues a move order.
pub fn spawn_neuroblast(
    world: &mut hecs::World,
    pos: Vec3,
    cell_type: crate::components::ProducibleCell,
) -> Entity {
    world.spawn((
        Position { position: pos },
        crate::components::Neuroblast {
            cell_type,
            speed: crate::constants::NEUROBLAST_SPEED,
        },
        Health::new(crate::constants::NEUROBLAST_HEALTH),
        Ownership { player: PlayerId::Player1 },
        Deletable {},
        Selectable { selected: false },
        VisualRadius { radius: NODE_RADIUS * 0.6 },
    ))
}

/// Convert an existing entity (a matured `MaturingNeuron`) into a full neuron
/// in-place.  Adds LeakyNeuron, LeakyDynamics, MetabolicState, and dendrites
/// without touching `Position` or `Ownership`.
pub fn mature_neuroblast(world: &mut hecs::World, entity: Entity, neuron_type: NeuronType) {
    let position = world
        .get::<&Position>(entity)
        .ok()
        .map(|p| p.position)
        .unwrap_or(Vec3::ZERO);

    let is_inhibitory = matches!(neuron_type, NeuronType::Inhibitory);

    let _ = world.insert(
        entity,
        (
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            neuron_type.clone(),
            MetabolicState::default(),
            VisualRadius { radius: NODE_RADIUS },
        ),
    );

    if is_inhibitory {
        let _ = world.insert_one(entity, Inhibitory);
    }

    let player = world
        .get::<&Ownership>(entity)
        .ok()
        .map(|o| o.player)
        .unwrap_or(PlayerId::Player1);

    const NUM_DENDRITES: usize = 5;
    let seed = entity.id() as f32 * 1.618_034;
    for i in 0..NUM_DENDRITES {
        let positions = arm_positions(position, i, NUM_DENDRITES, seed);
        let mut prev_entity = entity;
        for comp_pos in positions {
            let compartment = world.spawn((
                Position { position: comp_pos },
                neuron_type.clone(),
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                Dendrite,
                Ownership { player },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
            ));

            world.spawn((
                Connection {
                    from: prev_entity,
                    to: compartment,
                    strength: 1.0,
                    directional: false,
                },
                Deletable {},
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));

            prev_entity = compartment;
        }
    }
}
/// Spawn a growth cone that will travel from `source_pos` toward `target` (or
/// the live position of `target_entity`), laying axon compartments as it goes.
pub fn spawn_growth_cone(
    world: &mut hecs::World,
    source: hecs::Entity,
    source_pos: Vec3,
    target: Vec3,
    target_entity: Option<hecs::Entity>,
    neuron_type: NeuronType,
    waypoints: Vec<Vec3>,
) -> hecs::Entity {
    world.spawn((
        Position { position: source_pos },
        crate::components::GrowthCone {
            last_comp: source,
            last_comp_pos: source_pos,
            target,
            target_entity,
            neuron_type,
            speed: crate::constants::GROWTH_CONE_SPEED,
            waypoints: waypoints.into_iter().collect(),
        },
        Deletable {},
    ))
}


/// Spawn a neuron soma with `num_dendrites` dendrite branches.
pub fn spawn_neuron_with_dendrites(
    world: &mut hecs::World,
    position: Vec3,
    neuron_type: NeuronType,
    player: PlayerId,
    num_dendrites: usize,
) -> Entity {
    let soma = world.spawn((
        Position { position },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        neuron_type.clone(),
        MetabolicState::default(),
        Health::new(NEURON_HEALTH),
        Ownership { player },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
    ));

    if matches!(neuron_type, NeuronType::Inhibitory) {
        let _ = world.insert_one(soma, Inhibitory);
    }

    let seed = soma.id() as f32 * 1.618_034;
    for i in 0..num_dendrites {
        let positions = arm_positions(position, i, num_dendrites, seed);
        let mut prev_entity = soma;
        for comp_pos in positions {
            let compartment = world.spawn((
                Position { position: comp_pos },
                neuron_type.clone(),
                Compartment {
                    voltage: -10.0,
                    m: -0.625,
                    h: 0.0,
                    n: 0.0,
                    influence: 0.0,
                    capacitance: 1.0,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                Dendrite,
                Ownership { player },
                StaticConnectionSource {},
                Deletable {},
                Selectable { selected: false },
                SpatialDynamics {
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                },
            ));

            world.spawn((
                Connection {
                    from: prev_entity,
                    to: compartment,
                    strength: 1.0,
                    directional: false,
                },
                Deletable {},
                CompartmentCurrent {
                    capacitance: COUPLING_CAPACITANCE,
                },
            ));

            prev_entity = compartment;
        }
    }

    soma
}
