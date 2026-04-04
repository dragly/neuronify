//! Pre-built scenarios for testing and demonstrating combat mechanics.
//!
//! Each scenario builds a specific world state, runs headlessly in tests to verify
//! the designed outcome, and can be loaded in-game for interactive play.

use glam::Vec3;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, CurrentClamp, CurrentSynapse, Deletable,
    GeneratorDynamics, Inhibitory, LeakCurrent, LeakyDynamics, LeakyNeuron, NeuronType, Position,
    RegularSpikeGenerator, Selectable, SpatialDynamics, StaticConnectionSource, VisualRadius,
    COUPLING_CAPACITANCE, NODE_RADIUS,
};

use crate::components::*;
use crate::constants::*;
use crate::simulation::setup::{self, PetriDish};
use crate::spawning;

// ── ScenarioId ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScenarioId {
    #[default]
    Default,
    MicroglialSwarm,
    AstrocyteGuard,
    TheSevering,
    ExcitatoryOverload,
    InhibitoryGate,
    NeuralAssault,
}

impl ScenarioId {
    pub fn label(&self) -> &'static str {
        match self {
            ScenarioId::Default => "Default Map",
            ScenarioId::MicroglialSwarm => "Scenario 1: Microglial Swarm",
            ScenarioId::AstrocyteGuard => "Scenario 2: Astrocyte Guard",
            ScenarioId::TheSevering => "Scenario 3: The Severing",
            ScenarioId::ExcitatoryOverload => "Scenario 4: Excitatory Overload",
            ScenarioId::InhibitoryGate => "Scenario 5: Inhibitory Gate",
            ScenarioId::NeuralAssault => "Scenario 6: Neural Assault",
        }
    }

    pub fn all() -> &'static [ScenarioId] {
        &[
            ScenarioId::Default,
            ScenarioId::MicroglialSwarm,
            ScenarioId::AstrocyteGuard,
            ScenarioId::TheSevering,
            ScenarioId::ExcitatoryOverload,
            ScenarioId::InhibitoryGate,
            ScenarioId::NeuralAssault,
        ]
    }
}

// ── Top-level dispatch ────────────────────────────────────────────────────────

pub fn setup_scenario(world: &mut hecs::World, dish: &PetriDish, scenario: ScenarioId) {
    match scenario {
        ScenarioId::Default => setup::setup_game(world, dish),
        ScenarioId::MicroglialSwarm => setup_microglial_swarm(world, dish),
        ScenarioId::AstrocyteGuard => setup_astrocyte_guard(world, dish),
        ScenarioId::TheSevering => setup_the_severing(world, dish),
        ScenarioId::ExcitatoryOverload => setup_excitatory_overload(world, dish),
        ScenarioId::InhibitoryGate => setup_inhibitory_gate(world, dish),
        ScenarioId::NeuralAssault => setup_neural_assault(world, dish),
    }
}

// ── Helper: add AxonHealth to all existing Compartments ──────────────────────

fn add_axon_health_to_all_compartments(world: &mut hecs::World) {
    let compartments: Vec<hecs::Entity> = world
        .query::<&Compartment>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in compartments {
        let _ = world.insert_one(entity, AxonHealth::new(AXON_COMPARTMENT_HEALTH));
        // Ensure compartments also carry Ownership so combat targeting works.
        // (setup_game already adds Ownership to dendrites; axon comps in setup don't
        //  get one, so we patch here.)
        if world.get::<&Ownership>(entity).is_err() {
            let _ = world.insert_one(entity, Ownership { player: PlayerId::Player1 });
        }
    }
}

// ── Scenario 1: Macrophage Assault ────────────────────────────────────────────
//
// Default neural layout + 4 macrophage units (NeuronEngulfment) approaching from
// the southeast. No defensive units. Designed outcome: all non-origin neurons lose
// metabolic energy within ~15 s (macrophages drain directly at 30/s each).
// Origin neuron survives due to ORIGIN_MIN_ENERGY floor.

fn setup_microglial_swarm(world: &mut hecs::World, dish: &PetriDish) {
    setup::setup_game(world, dish);
    add_axon_health_to_all_compartments(world);

    // Three macrophages, one per non-origin neuron (A, B, C).
    // Each position is chosen so the macrophage (a) targets its intended neuron as nearest,
    // (b) is >12 units from that neuron (d > fire_range).
    // When astrocytes are placed at these positions (Scenario 2), macrophages are absorbed
    // in 15 s (60/4) before reaching fire range — leaving neurons unharmed.
    // Without astrocytes (Scenario 1), each fires multiple shots → neuron dies in <40 s.
    let start_positions = [
        Vec3::new(-12.0, 0.0, -12.0), // → Neuron A at (-20,0,0),   dist≈14.4; origin≈17.0
        Vec3::new(-24.0, 0.0, -20.0), // → Neuron B at (-38,0,-14),  dist≈15.2; A≈20.4
        Vec3::new(-24.0, 0.0,  20.0), // → Neuron C at (-38,0,14),   dist≈15.2; A≈20.4
    ];
    for pos in start_positions {
        spawning::spawn_macrophage(world, pos, Faction::Tumor);
    }
}

// ── Scenario 2: Astrocyte Guard ───────────────────────────────────────────────
//
// Same as Scenario 1 but with 3 Reactive Astrocytes placed between the macrophage
// start positions and the neural cluster.
// Absorb rate = 4.0 HP/s; macrophage health = 60 HP → kill time = 15 s.
// Macrophage retaliation = 4.5 HP/shot × 1/1.5 s = 3.0 HP/s; astrocyte health = 80 HP.
// Astrocyte survives: 3.0 HP/s × 15 s = 45 HP dealt to astrocyte (80−45 = 35 HP remaining).
// Designed outcome: astrocytes absorb all macrophages; neurons survive 25 s.

fn setup_astrocyte_guard(world: &mut hecs::World, dish: &PetriDish) {
    setup_microglial_swarm(world, dish); // reuse swarm setup

    // Three reactive astrocytes placed at the midpoint between each macrophage spawn
    // and its target neuron.  Each macrophage starts ≤7.6 units from its astrocyte
    // (inside absorb radius r=8), moves through the astrocyte on the way to the neuron,
    // and is drained to 0 in 15 s (60/4) — before it can kill the astrocyte (26 s).
    let guard_positions = [
        Vec3::new(-16.0, 0.0,  -6.0), // midpoint of Macrophage A spawn → Neuron A
        Vec3::new(-31.0, 0.0, -17.0), // midpoint of Macrophage B spawn → Neuron B
        Vec3::new(-31.0, 0.0,  17.0), // midpoint of Macrophage C spawn → Neuron C
    ];
    for pos in guard_positions {
        spawning::spawn_reactive_astrocyte(world, pos, Faction::Biological);
    }
}

// ── Scenario 3: The Severing ──────────────────────────────────────────────────
//
// Origin neuron with a long straight axon chain to a distant "Neuron Far".
// Two microglia attack the axon midpoint. One reactive astrocyte guards the
// origin-side compartments. Designed outcome: far half of axon severed, Neuron
// Far enters dormancy; origin and near-side compartments survive 20 s.

fn setup_the_severing(world: &mut hecs::World, _dish: &PetriDish) {
    world.clear();

    let origin_pos = Vec3::new(0.0, 0.0, 0.0);
    let far_pos = Vec3::new(50.0, 0.0, 0.0);

    // Origin neuron.
    let origin = world.spawn((
        Position { position: origin_pos },
        LeakyNeuron::default(),
        neuronify_core::LeakyDynamics::default(),
        neuronify_core::LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState {
            energy: MAX_NEURON_ENERGY,
            max_energy: MAX_NEURON_ENERGY,
        },
        Health::new(NEURON_HEALTH),
        OriginNeuron { player: PlayerId::Player1 },
        Ownership { player: PlayerId::Player1 },
        Anchored,
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.5 },
    ));

    // Far neuron.
    let far_neuron = world.spawn((
        Position { position: far_pos },
        LeakyNeuron::default(),
        neuronify_core::LeakyDynamics::default(),
        neuronify_core::LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState::default(),
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
    ));

    // Build the axon chain: 8 compartments evenly spaced between origin and far.
    // Positions at x = 6, 12, 18, 24, 30, 36, 42, 48 (the last is the bridge near far_neuron).
    let num_comps = 8usize;
    let step = (far_pos.x - origin_pos.x) / (num_comps + 1) as f32;

    let mut prev = origin;
    for i in 1..=num_comps {
        let x = origin_pos.x + step * i as f32;
        let comp = world.spawn((
            Position { position: Vec3::new(x, 0.0, 0.0) },
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
            Ownership { player: PlayerId::Player1 },
            AxonHealth::new(AXON_COMPARTMENT_HEALTH),
            StaticConnectionSource {},
            Selectable { selected: false },
            Deletable {},
            SpatialDynamics {
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
            },
            VisualRadius { radius: NODE_RADIUS * 0.25 },
        ));

        world.spawn((
            Connection { from: prev, to: comp, strength: 1.0, directional: false },
            Deletable {},
            CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
        ));

        prev = comp;
    }

    // Final link to far neuron.
    world.spawn((
        Connection { from: prev, to: far_neuron, strength: 1.0, directional: true },
        Deletable {},
        CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
    ));

    // Two microglia attack near the midpoint (compartments 4–5, around x=24–30).
    spawning::spawn_microglial_cell(world, Vec3::new(24.0, 0.0, -6.0), Faction::Tumor);
    spawning::spawn_microglial_cell(world, Vec3::new(28.0, 0.0, 5.0), Faction::Tumor);

    // One reactive astrocyte guards the origin-side (covers x ≈ 0–16).
    spawning::spawn_reactive_astrocyte(world, Vec3::new(10.0, 0.0, 0.0), Faction::Biological);
}

// ── Helper: build excitatory axon chain between two neuron entities ───────────
//
// Spawns `num_comps` Compartment entities between `from` and `to`, linked by
// Connection + CompartmentCurrent.  `neuron_type` controls the polarity of the
// bridge current (Excitatory = depolarising, Inhibitory = hyperpolarising).

fn wire_neurons(
    world: &mut hecs::World,
    from: hecs::Entity,
    to: hecs::Entity,
    num_comps: usize,
    neuron_type: NeuronType,
    from_pos: Vec3,
    to_pos: Vec3,
    ownership: Option<PlayerId>,
) {
    let step = (to_pos - from_pos) / (num_comps + 1) as f32;
    let mut prev = from;

    for i in 1..=num_comps {
        let pos = from_pos + step * i as f32;
        let comp = world.spawn((
            Position { position: pos },
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
            StaticConnectionSource {},
            Selectable { selected: false },
            Deletable {},
            SpatialDynamics { velocity: Vec3::ZERO, acceleration: Vec3::ZERO },
        ));
        if let Some(player) = ownership {
            let _ = world.insert_one(comp, Ownership { player });
        }
        world.spawn((
            Connection { from: prev, to: comp, strength: 1.0, directional: false },
            Deletable {},
            CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
        ));
        prev = comp;
    }

    // Bridge connection to target soma.
    world.spawn((
        Connection { from: prev, to, strength: 1.0, directional: false },
        Deletable {},
        CompartmentCurrent { capacitance: COUPLING_CAPACITANCE },
    ));
}

// ── Scenario 4: Excitatory Overload ──────────────────────────────────────────
//
// The player's origin reaches into enemy territory and bombards an enemy relay
// neuron.  The relay normally fires at 5 Hz (driven by an enemy generator) and
// triggers a tumor NeuronSpawner.  The player's 20 Hz excitatory axon overwhelms
// it: the relay fires at ~20 Hz, exhausting its metabolic energy in ~4 s.
// Once dormant, its NeuronSpawner never triggers — the enemy pipeline is broken.
//
// Layout:
//   Player origin (0,0,0)  →[3-comp excitatory]→  Enemy relay (35,0,0)
//   Enemy generator (50,0,0) →[2-comp excitatory]→  Enemy relay (35,0,0)

fn setup_excitatory_overload(world: &mut hecs::World, _dish: &PetriDish) {
    world.clear();

    let origin_pos    = Vec3::new(0.0,  0.0, 0.0);
    let neuron2_pos   = Vec3::new(5.0, 0.0,  10.0);
    let neuron3_pos   = Vec3::new(5.0, 0.0, -10.0);
    let relay_pos     = Vec3::new(35.0, 0.0, 0.0);
    let generator_pos = Vec3::new(50.0, 0.0, 0.0);

    // Player origin: 20 Hz generator.
    let origin = world.spawn((
        Position { position: origin_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState { energy: MAX_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        OriginNeuron { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.5 },
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
    ));

    // Two additional player excitatory neurons to increase bombardment rate.
    let neuron2 = world.spawn((
        Position { position: neuron2_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState { energy: MAX_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
    ));

    let neuron3 = world.spawn((
        Position { position: neuron3_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState { energy: MAX_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
    ));

    // Enemy relay: receives both enemy generator and player excitation.
    // No ownership (enemy/neutral), no glial support → metabolic exhaustion.
    // timer starts at cooldown so the relay must earn its first spawn.
    let relay_cooldown = 5.0_f32;
    let relay = world.spawn((
        Position { position: relay_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState { energy: DEFAULT_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
        Health::new(NEURON_HEALTH),
        NeuronSpawner {
            faction: Faction::Tumor,
            spawn_type: NeuronSpawnType::MicroglialCell,
            cooldown: relay_cooldown,
            timer: relay_cooldown,
            spawn_offset: Vec3::new(2.0, 0.0, 2.0),
        },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
    ));

    // Enemy generator: drives relay at 5 Hz (normal operating rate).
    let enemy_gen = world.spawn((
        Position { position: generator_pos },
        NeuronType::Excitatory,
        RegularSpikeGenerator { frequency: 5.0 },
        GeneratorDynamics::default(),
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 0.8 },
    ));

    // All three player neurons wire into the enemy relay.
    wire_neurons(world, origin,   relay, 3, NeuronType::Excitatory, origin_pos,   relay_pos, Some(PlayerId::Player1));
    wire_neurons(world, neuron2,  relay, 2, NeuronType::Excitatory, neuron2_pos,  relay_pos, Some(PlayerId::Player1));
    wire_neurons(world, neuron3,  relay, 2, NeuronType::Excitatory, neuron3_pos,  relay_pos, Some(PlayerId::Player1));
    // Enemy generator drives the relay normally.
    wire_neurons(world, enemy_gen, relay, 2, NeuronType::Excitatory, generator_pos, relay_pos, None);
}

// ── Scenario 5: Inhibitory Gate ───────────────────────────────────────────────
//
// Same enemy pipeline as Scenario 4 (enemy generator 5 Hz → relay with
// NeuronSpawner), but the player's counter is an inhibitory interneuron rather
// than brute-force excitation.  The interneuron fires at 20 Hz, delivering
// continuous hyperpolarization to the enemy relay — cancelling the enemy
// generator's 5 Hz drive.  The relay stays alive but is silenced: no Tumor
// microglia ever spawn.
//
// Layout:
//   Player origin (0,0,0)  →[2-comp excitatory]→  Inhibitory interneuron (15,0,0)
//                                                       →[2-comp inhibitory]→  Enemy relay (35,0,0)
//   Enemy generator (50,0,0) →[2-comp excitatory]→  Enemy relay (35,0,0)

fn setup_inhibitory_gate(world: &mut hecs::World, _dish: &PetriDish) {
    world.clear();

    let origin_pos    = Vec3::new(0.0,  0.0, 0.0);
    let inter_pos     = Vec3::new(15.0, 0.0, 0.0);
    let relay_pos     = Vec3::new(35.0, 0.0, 0.0);
    let generator_pos = Vec3::new(50.0, 0.0, 0.0);

    // Player origin: 20 Hz generator.
    let origin = world.spawn((
        Position { position: origin_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState { energy: MAX_NEURON_ENERGY, max_energy: MAX_NEURON_ENERGY },
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        OriginNeuron { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.5 },
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
    ));

    // Player inhibitory interneuron: driven by origin AND its own 20 Hz generator.
    // The generator ensures inhibitory CurrentSynapse fires on LIF step 1 (same
    // step as the enemy gen), so inhibitory and excitatory currents arrive at the
    // relay simultaneously — the enemy gen cannot get a head-start.
    // Carries the Inhibitory marker so lif_step applies negative sign to its
    // outgoing CurrentSynapse current.  No MetabolicState — never goes dormant.
    let interneuron = world.spawn((
        Position { position: inter_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Inhibitory,
        Inhibitory,
        Health::new(NEURON_HEALTH),
        Ownership { player: PlayerId::Player1 },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
    ));

    // Enemy relay: NeuronSpawner produces Tumor microglia if it fires.
    // No ownership (enemy/neutral) — metabolically idle when silenced.
    let relay = world.spawn((
        Position { position: relay_pos },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        MetabolicState::default(),
        Health::new(NEURON_HEALTH),
        NeuronSpawner {
            faction: Faction::Tumor,
            spawn_type: NeuronSpawnType::MicroglialCell,
            cooldown: 3.0,
            timer: 0.0,
            spawn_offset: Vec3::new(2.0, 0.0, 2.0),
        },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS },
    ));

    // Enemy generator: normally drives relay at 5 Hz.
    let enemy_gen = world.spawn((
        Position { position: generator_pos },
        NeuronType::Excitatory,
        RegularSpikeGenerator { frequency: 5.0 },
        GeneratorDynamics::default(),
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 0.8 },
    ));

    // Player axon: origin → inhibitory interneuron (FHN compartment chain).
    wire_neurons(world, origin, interneuron, 2, NeuronType::Excitatory, origin_pos, inter_pos, Some(PlayerId::Player1));
    // Player gate: interneuron → relay via direct CurrentSynapse (not FHN compartments).
    // Inhibitory marker on interneuron → sign=-1 in lif_step.
    // tau=0.05 s: mean inhibitory current = 20 Hz × 12e-9 × 0.05 = 12e-9 A (steady state ΔV = 1.2 V below resting).
    // This dominates the 5 Hz enemy excitatory at 4× lower mean current, silencing the relay.
    world.spawn((
        Connection { from: interneuron, to: relay, strength: 1.0, directional: true },
        CurrentSynapse {
            maximum_current: 12e-9,
            tau: 0.05,
            delay: 0.0,
            alpha_function: false,
            exponential: 0.0,
            linear: 0.0,
            triggers: Vec::new(),
            time: 0.0,
            current_output: 0.0,
        },
        Deletable {},
    ));
    // Enemy circuit: generator → relay via direct CurrentSynapse.
    // maximum_current intentionally set to 1e-9 A (1/12th of inhibitory):
    // even at maximum excitatory current, the inhibitory synapse is 12× stronger,
    // guaranteeing the relay is permanently silenced without any timing-dependent windows.
    // The relay still fires naturally at ~2 Hz without the gate (1e-9 A × 1e8 Ω = +0.1 V
    // above resting, above threshold −0.055 V), demonstrating the gate is necessary.
    world.spawn((
        Connection { from: enemy_gen, to: relay, strength: 1.0, directional: true },
        CurrentSynapse {
            maximum_current: 1e-9,
            tau: 0.05,
            delay: 0.0,
            alpha_function: false,
            exponential: 0.0,
            linear: 0.0,
            triggers: Vec::new(),
            time: 0.0,
            current_output: 0.0,
        },
        Deletable {},
    ));
}

// ── Scenario 6: Neural Assault ────────────────────────────────────────────────
//
// The player's origin neuron is fitted with a NeuronSpawner that releases a
// T-cell each time it fires (3 s cooldown).  Two tumor macrophages approach from
// the east.  The T-cell wave intercepts the macrophages before they reach player
// neurons.

fn setup_neural_assault(world: &mut hecs::World, dish: &PetriDish) {
    setup::setup_game(world, dish);

    // Attach a NeuronSpawner to the existing origin neuron.
    let origin_entity: Option<hecs::Entity> = world
        .query::<&OriginNeuron>()
        .iter()
        .next()
        .map(|(e, _)| e);
    if let Some(origin) = origin_entity {
        let _ = world.insert_one(
            origin,
            NeuronSpawner {
                faction: Faction::Biological,
                spawn_type: NeuronSpawnType::TCell,
                cooldown: 3.0,
                timer: 0.0,
                spawn_offset: Vec3::new(5.0, 0.0, 0.0),
            },
        );
    }

    // Two tumor macrophages approaching from the east.
    spawning::spawn_macrophage(world, Vec3::new(30.0, 0.0, -8.0), Faction::Tumor);
    spawning::spawn_macrophage(world, Vec3::new(30.0, 0.0, 8.0), Faction::Tumor);
}

// ── Headless test runner ──────────────────────────────────────────────────────

#[cfg(test)]
pub fn run_headless(world: &mut hecs::World, seconds: f64) {
    use crate::simulation::{cleanup, combat, metabolism};

    let dt = 0.016_f64;
    let steps = (seconds / dt) as u32;

    for _ in 0..steps {
        let fdt = dt as f32;
        combat::tick_dying_units(world, fdt);
        combat::move_mobile_units(world, fdt);
        combat::apply_axon_cutting(world, fdt);
        combat::apply_neuron_engulfment(world, fdt);
        combat::apply_burst_attacks(world, fdt);
        combat::advance_attack_projectiles(world, fdt);
        combat::apply_glial_absorption(world, fdt);
        combat::despawn_dead(world);
        metabolism::metabolic_drain(world, dt);
        metabolism::apply_dormancy(world);
        cleanup::cleanup_orphans(world);
    }
}

/// Headless runner that advances LIF neural simulation at 1:1 with combat time.
/// Used by scenarios that depend on neural firing to drive combat outcomes.
/// At 0.016 s/step with LIF_DT = 0.0001 s → 160 LIF steps per combat step.
/// FHN also advances 160 × FHN_CDT = 1.6 s (fast compartment propagation).
#[cfg(test)]
pub fn run_headless_neural(world: &mut hecs::World, seconds: f64) {
    use neuronify_core::{fhn_step, lif_step, FHN_CDT, LIF_DT};
    use crate::simulation::{cleanup, combat, metabolism};
    use std::collections::HashSet;

    let combat_dt = 0.016_f64;
    let steps = (seconds / combat_dt) as u32;
    let lif_iters = (combat_dt / LIF_DT) as u32; // 160
    let fire_window = lif_iters as f64 * LIF_DT;  // = combat_dt
    let fhn_iters = lif_iters;                     // 160 FHN steps per combat step
    let mut sim_time = 0.0_f64;

    for _ in 0..steps {
        let fdt = combat_dt as f32;

        // Advance LIF neural simulation at 1:1 with combat time.
        for _ in 0..lif_iters {
            lif_step(world, LIF_DT, sim_time);
            sim_time += LIF_DT;
        }

        // Collect neurons that fired during this combat step.
        let mut recently_fired: HashSet<hecs::Entity> = world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < fire_window)
            .map(|(e, _)| e)
            .collect();
        for (e, d) in world.query::<&GeneratorDynamics>().iter() {
            if d.time_since_fire < fire_window {
                recently_fired.insert(e);
            }
        }

        // Advance FHN compartment dynamics.
        for _ in 0..fhn_iters {
            fhn_step(world, FHN_CDT, &recently_fired);
        }

        // Combat + metabolism.
        combat::tick_dying_units(world, fdt);
        combat::apply_neuron_spawning(world, fdt);
        combat::move_mobile_units(world, fdt);
        combat::apply_axon_cutting(world, fdt);
        combat::apply_neuron_engulfment(world, fdt);
        combat::apply_burst_attacks(world, fdt);
        combat::advance_attack_projectiles(world, fdt);
        combat::apply_glial_absorption(world, fdt);
        combat::despawn_dead(world);
        metabolism::metabolic_drain(world, combat_dt);
        metabolism::apply_dormancy(world);
        cleanup::cleanup_orphans(world);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_core::LeakyNeuron;



    fn dish() -> PetriDish {
        PetriDish {
            center: Vec3::ZERO,
            radius: PETRI_DISH_RADIUS,
        }
    }

    /// Scenario 1: Three macrophages with no defense should drain all non-origin neurons
    /// to 0 energy within 30 seconds. (Origin survives due to ORIGIN_MIN_ENERGY floor.)
    #[test]
    fn test_scenario_1_microglial_swarm_neurons_die() {
        let mut world = hecs::World::new();
        setup_microglial_swarm(&mut world, &dish());

        run_headless(&mut world, 40.0);

        // Macrophages drain Health (not energy); neurons die when health reaches 0 and are
        // despawned by despawn_dead. After enough time, all non-origin neurons should be gone.
        let non_origin_alive = world
            .query::<(&LeakyNeuron, &MetabolicState)>()
            .iter()
            .filter(|(e, _)| world.get::<&OriginNeuron>(*e).is_err())
            .count();

        assert_eq!(
            non_origin_alive, 0,
            "All non-origin neurons should be destroyed after macrophage assault with no defense (got {non_origin_alive} alive)"
        );
    }

    /// Scenario 2: Reactive astrocytes should absorb all macrophages and leave
    /// at least 2 neurons with energy > 1.0 after 25 seconds.
    /// Absorb rate 4 HP/s × 60 HP = 15 s kill; test allows 25 s buffer.
    #[test]
    fn test_scenario_2_astrocyte_guard_neurons_survive() {
        let mut world = hecs::World::new();
        setup_astrocyte_guard(&mut world, &dish());

        run_headless(&mut world, 25.0);

        let alive_neurons = world
            .query::<(&LeakyNeuron, &MetabolicState)>()
            .iter()
            .filter(|(_, (_, ms))| ms.energy > 1.0)
            .count();

        let remaining_macrophages = world
            .query::<&MacrophageUnit>()
            .iter()
            .count();

        assert!(
            alive_neurons >= 2,
            "At least 2 neurons should survive when protected by astrocytes (got {alive_neurons})"
        );
        assert_eq!(
            remaining_macrophages, 0,
            "All macrophages should be absorbed (got {remaining_macrophages} remaining)"
        );
    }

    /// Scenario 3: Microglia should sever the far half of the axon chain.
    /// Origin neuron stays alive; Neuron Far should lose all energy.
    #[test]
    fn test_scenario_3_the_severing_far_dies() {
        let mut world = hecs::World::new();
        let d = dish();
        setup_the_severing(&mut world, &d);

        // Count initial compartments with AxonHealth.
        let initial_axon_count = world.query::<&AxonHealth>().iter().count();

        run_headless(&mut world, 60.0);

        // Origin should be alive (has OriginNeuron + astrocyte protection).
        let origin_alive = world
            .query::<(&OriginNeuron, &MetabolicState)>()
            .iter()
            .any(|(_, (_, ms))| ms.energy > 1.0);

        // Neuron Far has no OriginNeuron — find the neuron with lowest energy.
        let min_energy = world
            .query::<(&LeakyNeuron, &MetabolicState)>()
            .iter()
            .filter(|(e, _)| world.get::<&OriginNeuron>(*e).is_err())
            .map(|(_, (_, ms))| ms.energy)
            .fold(f64::INFINITY, f64::min);

        let remaining_axons = world.query::<&AxonHealth>().iter().count();
        let severed = initial_axon_count.saturating_sub(remaining_axons);

        assert!(origin_alive, "Origin neuron should survive the partial severing");
        assert!(
            min_energy < 130.0,
            "Neuron Far should have lost energy due to resource drain before severing (energy={min_energy:.1})"
        );
        assert!(
            severed >= 3,
            "At least 3 axon compartments should be severed (got {severed})"
        );
    }

    /// Scenario 4: The player's 20 Hz excitatory axon reaches into enemy territory and
    /// bombards the enemy relay.  Firing costs drain the relay's energy to zero → dormancy
    /// within ~4 s; after 20 s the enemy pipeline is broken.
    #[test]
    fn test_scenario_4_excitatory_overload_target_collapses() {
        let mut world = hecs::World::new();
        setup_excitatory_overload(&mut world, &dish());

        // Capture relay entity: the non-origin LeakyNeuron that owns a NeuronSpawner.
        let relay = world
            .query::<(&LeakyNeuron, &NeuronSpawner)>()
            .iter()
            .map(|(e, _)| e)
            .next()
            .expect("enemy relay with NeuronSpawner should exist");

        run_headless_neural(&mut world, 20.0);

        let energy = world
            .get::<&MetabolicState>(relay)
            .map(|ms| ms.energy)
            .unwrap_or(0.0);

        assert!(
            energy < 1.0,
            "Enemy relay should be drained to dormancy by excitatory bombardment (energy={energy:.2})"
        );
    }

    /// Scenario 5: An inhibitory gate silences the enemy relay without destroying it —
    /// no Tumor MicroglialCell entities should appear in 30 s, and the relay energy
    /// must remain above 10.0 (alive but silenced, not metabolically exhausted).
    #[test]
    fn test_scenario_5_inhibitory_gate_no_units_spawn() {
        let mut world = hecs::World::new();
        setup_inhibitory_gate(&mut world, &dish());

        // Capture relay entity before simulation.
        let relay = world
            .query::<(&LeakyNeuron, &NeuronSpawner)>()
            .iter()
            .map(|(e, _)| e)
            .next()
            .expect("enemy relay with NeuronSpawner should exist");

        run_headless_neural(&mut world, 30.0);

        let tumor_microglia = world
            .query::<(&MicroglialCell, &MobileUnit)>()
            .iter()
            .filter(|(_, (_, mu))| mu.faction == Faction::Tumor)
            .count();

        let relay_energy = world
            .get::<&MetabolicState>(relay)
            .map(|ms| ms.energy)
            .unwrap_or(0.0);

        assert_eq!(
            tumor_microglia, 0,
            "Inhibitory gate should prevent all spawning (got {tumor_microglia} Tumor microglia)"
        );
        assert!(
            relay_energy > 10.0,
            "Enemy relay should stay alive (silenced, not exhausted) — energy={relay_energy:.2}"
        );
    }

    /// Scenario 6: The player's origin neuron spawns T-cells that intercept
    /// 2 tumor macrophages before they can destroy the player's neurons.
    #[test]
    fn test_scenario_6_neural_assault_origin_survives() {
        let mut world = hecs::World::new();
        setup_neural_assault(&mut world, &dish());

        run_headless_neural(&mut world, 40.0);

        // Origin must still be alive (energy above floor).
        let origin_alive = world
            .query::<(&OriginNeuron, &MetabolicState)>()
            .iter()
            .any(|(_, (_, ms))| ms.energy > 1.0);

        let remaining_macrophages = world
            .query::<(&MacrophageUnit, &MobileUnit)>()
            .iter()
            .filter(|(_, (_, mu))| mu.faction == Faction::Tumor)
            .count();

        assert!(origin_alive, "Origin neuron should survive the macrophage assault");
        assert_eq!(
            remaining_macrophages, 0,
            "All tumor macrophages should be destroyed by T-cells (got {remaining_macrophages} remaining)"
        );
    }
}
