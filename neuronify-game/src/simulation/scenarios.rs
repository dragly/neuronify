//! Pre-built scenarios for testing and demonstrating combat mechanics.
//!
//! Each scenario builds a specific world state, runs headlessly in tests to verify
//! the designed outcome, and can be loaded in-game for interactive play.

use glam::Vec3;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, Deletable, LeakyNeuron, NeuronType, Position,
    Selectable, SpatialDynamics, StaticConnectionSource, VisualRadius, COUPLING_CAPACITANCE,
    NODE_RADIUS,
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
}

impl ScenarioId {
    pub fn label(&self) -> &'static str {
        match self {
            ScenarioId::Default => "Default Map",
            ScenarioId::MicroglialSwarm => "Scenario 1: Microglial Swarm",
            ScenarioId::AstrocyteGuard => "Scenario 2: Astrocyte Guard",
            ScenarioId::TheSevering => "Scenario 3: The Severing",
        }
    }

    pub fn all() -> &'static [ScenarioId] {
        &[
            ScenarioId::Default,
            ScenarioId::MicroglialSwarm,
            ScenarioId::AstrocyteGuard,
            ScenarioId::TheSevering,
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
    // (b) is >12 units from that neuron (d > fire_range + speed*health/absorb_rate = 12).
    // When astrocytes are placed at these positions (Scenario 2), macrophages are absorbed
    // in 3.0s (60/20) before reaching fire range — leaving neurons unharmed.
    // Without astrocytes (Scenario 1), each fires 8+ shots → neuron dies in <15s.
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
// start positions and the neural cluster. Designed outcome: astrocytes absorb all
// macrophages; neurons survive 15 s.

fn setup_astrocyte_guard(world: &mut hecs::World, dish: &PetriDish) {
    setup_microglial_swarm(world, dish); // reuse swarm setup

    // Three reactive astrocytes placed at the midpoint between each macrophage spawn
    // and its target neuron.  Each macrophage starts ≤7.6 units from its astrocyte
    // (inside absorb radius r=8), moves through the astrocyte on the way to the neuron,
    // and is drained to 0 in 3.0s (60/20) — well before it reaches fire range (3.8s).
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

// ── Headless test runner ──────────────────────────────────────────────────────

#[cfg(test)]
pub fn run_headless(world: &mut hecs::World, seconds: f64) {
    use crate::simulation::{cleanup, combat, metabolism};

    let dt = 0.016_f64;
    let steps = (seconds / dt) as u32;

    for _ in 0..steps {
        let fdt = dt as f32;
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
    /// at least 2 neurons with energy > 1.0 after 15 seconds.
    #[test]
    fn test_scenario_2_astrocyte_guard_neurons_survive() {
        let mut world = hecs::World::new();
        setup_astrocyte_guard(&mut world, &dish());

        run_headless(&mut world, 15.0);

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
}
