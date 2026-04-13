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

/// Spawn a mast cell — stationary immune effector driven by a neuron.
/// Emits cytokine particles when its driver neuron fires, activating nearby macrophages.
pub fn spawn_mast_cell(world: &mut hecs::World, position: Vec3, driver: Entity) -> Entity {
    world.spawn((
        Position { position },
        MastCell { driver, emit_timer: 0.0 },
        Deletable {},
        VisualRadius { radius: 2.5 },
    ))
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
        Selectable { selected: false },
        VisualRadius { radius: 1.2 },
    ))
}

// ─────────────────────────────────────────────────────────────────────────────

/// Spacing between successive compartments. Must equal spring rest length (2×NODE_RADIUS)
/// so compartments sit at physics equilibrium after spawning.
const DENDRITE_LENGTH: f32 = 2.0;
/// Number of compartments along each primary arm.
const DENDRITE_COMPARTMENTS: usize = 4;
/// Number of compartments on each secondary branch.
const DENDRITE_BRANCH_COMPARTMENTS: usize = 2;
/// Per-segment wander strength (fraction of step length deflected laterally).
const DENDRITE_WANDER: f32 = 0.45;

/// Compute a single wander-perturbed step direction deterministically.
fn wander_dir(base_dir: Vec3, base_angle: f32, seed: f32, arm_idx: usize, seg: usize, sx: f32, sz: f32) -> Vec3 {
    let wx = (seed * sx + arm_idx as f32 * 13.7 + seg as f32 * 17.3).sin() * DENDRITE_WANDER;
    let wz = (seed * sz + arm_idx as f32 * 7.7  + seg as f32 * 23.1).cos() * DENDRITE_WANDER;
    let d = (base_dir + Vec3::new(wx, 0.0, wz)).normalize_or_zero();
    if d.length_squared() < 0.01 {
        Vec3::new(base_angle.cos(), 0.0, base_angle.sin())
    } else {
        d
    }
}

/// Spawn a sequence of dendrite compartments from `parent_entity`, following a wander path,
/// and connect them with `Connection + CompartmentCurrent`. Returns the spawned entities.
fn spawn_dendrite_chain<F>(
    world: &mut hecs::World,
    origin: Vec3,
    base_angle: f32,
    seed: f32,
    arm_idx: usize,
    num_steps: usize,
    start_depth: u32,
    parent_entity: hecs::Entity,
    sx: f32, sz: f32,
    mut spawn_compartment: F,
) -> Vec<(hecs::Entity, Vec3, u32)>
where
    F: FnMut(&mut hecs::World, Vec3, u32) -> hecs::Entity,
{
    let mut dir = Vec3::new(base_angle.cos(), 0.0, base_angle.sin());
    let mut pos = origin;
    let mut prev = parent_entity;
    let mut chain = Vec::with_capacity(num_steps);

    for seg in 0..num_steps {
        dir = wander_dir(dir, base_angle, seed, arm_idx, seg, sx, sz);
        pos += dir * DENDRITE_LENGTH;
        let depth = start_depth + seg as u32 + 1;
        let entity = spawn_compartment(world, pos, depth);
        world.spawn((
            Connection { from: prev, to: entity, strength: 1.0, directional: false },
            Deletable {},
            CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
        ));
        chain.push((entity, pos, depth));
        prev = entity;
    }
    chain
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
        let base_angle = std::f32::consts::TAU * i as f32 / num_processes as f32 + seed * 0.5;

        let main_chain = spawn_dendrite_chain(
            world, position, base_angle, seed, i,
            DENDRITE_COMPARTMENTS, 0, soma, 7.3, 11.1,
            |world, pos, depth| {
                world.spawn((
                    Position { position: pos },
                    NeuronType::Excitatory,
                    Compartment {
                        voltage: -10.0, m: -0.625, h: 0.0, n: 0.0,
                        influence: 0.0, capacitance: 1.0,
                        injected_current: 0.0, fire_impulse: 0.0,
                    },
                    GlialProcess,
                    DendriteDepth(depth),
                    Ownership { player },
                    StaticConnectionSource {},
                    Deletable {},
                    Selectable { selected: false },
                    SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
                    ConnectionColor(glial_color()),
                ))
            },
        );

        // One branch off the 3rd compartment of each process arm.
        if let Some(&(branch_parent, branch_origin, branch_depth)) = main_chain.get(2) {
            let branch_angle = base_angle + 1.1 + (seed * 3.1 + i as f32 * 2.7).sin() * 0.6;
            spawn_dendrite_chain(
                world, branch_origin, branch_angle, seed + i as f32 * 5.0, i,
                DENDRITE_BRANCH_COMPARTMENTS, branch_depth, branch_parent, 5.9, 8.7,
                |world, pos, depth| {
                    world.spawn((
                        Position { position: pos },
                        NeuronType::Excitatory,
                        Compartment {
                            voltage: -10.0, m: -0.625, h: 0.0, n: 0.0,
                            influence: 0.0, capacitance: 1.0,
                            injected_current: 0.0, fire_impulse: 0.0,
                        },
                        GlialProcess,
                        DendriteDepth(depth),
                        Ownership { player },
                        StaticConnectionSource {},
                        Deletable {},
                        Selectable { selected: false },
                        SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
                        ConnectionColor(glial_color()),
                    ))
                },
            );
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

    spawn_neuron_dendrites(world, position, &neuron_type, player, entity, 5);
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
    owner: crate::components::PlayerId,
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
            depth: 1,
            owner,
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

    spawn_neuron_dendrites(world, position, &neuron_type, player, soma, num_dendrites);
    soma
}

/// Shared helper: spawn dendrite arms (with branches) onto an already-spawned soma entity.
pub fn spawn_neuron_dendrites(
    world: &mut hecs::World,
    position: Vec3,
    neuron_type: &NeuronType,
    player: PlayerId,
    soma: hecs::Entity,
    num_dendrites: usize,
) {
    let seed = soma.id() as f32 * 1.618_034;
    for i in 0..num_dendrites {
        let base_angle = std::f32::consts::TAU * i as f32 / num_dendrites as f32 + seed * 0.5;

        let nt = neuron_type.clone();
        let main_chain = spawn_dendrite_chain(
            world, position, base_angle, seed, i,
            DENDRITE_COMPARTMENTS, 0, soma, 7.3, 11.1,
            |world, pos, depth| {
                world.spawn((
                    Position { position: pos },
                    nt.clone(),
                    Compartment {
                        voltage: -10.0, m: -0.625, h: 0.0, n: 0.0,
                        influence: 0.0, capacitance: 1.0,
                        injected_current: 0.0, fire_impulse: 0.0,
                    },
                    Dendrite,
                    DendriteDepth(depth),
                    Ownership { player },
                    StaticConnectionSource {},
                    Deletable {},
                    Selectable { selected: false },
                    SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
                ))
            },
        );

        // One branch off the 3rd compartment of each arm.
        if let Some(&(branch_parent, branch_origin, branch_depth)) = main_chain.get(2) {
            let branch_angle = base_angle + 1.1 + (seed * 3.1 + i as f32 * 2.7).sin() * 0.6;
            let nt = neuron_type.clone();
            spawn_dendrite_chain(
                world, branch_origin, branch_angle, seed + i as f32 * 5.0, i,
                DENDRITE_BRANCH_COMPARTMENTS, branch_depth, branch_parent, 5.9, 8.7,
                |world, pos, depth| {
                    world.spawn((
                        Position { position: pos },
                        nt.clone(),
                        Compartment {
                            voltage: -10.0, m: -0.625, h: 0.0, n: 0.0,
                            influence: 0.0, capacitance: 1.0,
                            injected_current: 0.0, fire_impulse: 0.0,
                        },
                        Dendrite,
                        DendriteDepth(depth),
                        Ownership { player },
                        StaticConnectionSource {},
                        Deletable {},
                        Selectable { selected: false },
                        SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
                    ))
                },
            );
        }
    }
}
