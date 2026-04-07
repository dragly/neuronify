//! Pre-built scenarios for testing and demonstrating combat mechanics.
//!
//! Each scenario builds a specific world state, runs headlessly in tests to verify
//! the designed outcome, and can be loaded in-game for interactive play.

use glam::Vec3;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, CurrentSynapse, Deletable,
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
#[allow(dead_code)]
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

/// Load a scenario from an SVG map file into the world.
/// Returns an error string if the file could not be loaded.
///
/// ## Hex tile scales
/// Two coordinate levels are in play:
/// - **Map hexes** (large): the SVG terrain cells, each ~24 SVG-px radius → `HEX_RADIUS_WORLD`
///   world units after scaling. These define terrain type, passability, and resource locations.
/// - **Pathfinding hexes** (fine): the A* grid (`HexGrid`, cell size `HEX_GRID_CELL_SIZE = 4.0`).
///   Several fine cells fit inside each large map hex, giving unit movement sub-hex granularity.
pub fn setup_scenario_from_svg(
    world: &mut hecs::World,
    dish: &mut PetriDish,
    content: &str,
) -> Result<(), String> {
    use crate::map::{parse_scenario_svg, NeuronDefType};
    use crate::map::hex::{SVG_CX, SVG_CY, MAP_SCALE};
    use std::collections::HashMap;

    let map = parse_scenario_svg(content).map_err(|e| e.to_string())?;
    world.clear();

    // ── Coordinate transform ──────────────────────────────────────────────────
    // SVG viewBox is 0 0 1116 812. Map centre → world origin.
    //
    // Scale: SVG hex radius = 24 px → HEX_RADIUS_WORLD world units.
    // With MAP_SCALE = 0.28 each large map hex is ~6.7 world units radius.
    // The fine A* pathfinding grid (cell size 4.0) gives ~1.7 cells per hex
    // radius — several pathfinding cells per map hex for sub-hex granularity.
    //
    // Rotation: the SVG is laid out landscape (wider in x). Mapping SVG-y → world-x
    // and SVG-x → world-(-z) rotates the map 90° so it lies flat when the camera
    // looks along +z.
    let svg_to_world = |svg_x: f32, svg_y: f32| -> Vec3 {
        Vec3::new(
            -(svg_x - SVG_CX) * MAP_SCALE,
            0.0,
            -(svg_y - SVG_CY) * MAP_SCALE,
        )
    };

    // Expand the petri dish to contain the full map (SVG extents after scaling:
    // x: ±406*0.28 ≈ ±114, z: ±558*0.28 ≈ ±156 → max radius ~194).
    dish.radius = 220.0;

    // ── Spawn neurons ─────────────────────────────────────────────────────────
    let mut neuron_entities: HashMap<String, (hecs::Entity, Vec3)> = HashMap::new();

    for n in &map.neurons {
        let pos = svg_to_world(n.x, n.y);
        let neuron_type = match n.neuron_type {
            NeuronDefType::Inhibitory => NeuronType::Inhibitory,
            NeuronDefType::Excitatory => NeuronType::Excitatory,
        };
        let is_player = n.team == 1;

        let mut builder = hecs::EntityBuilder::new();
        builder.add(Position { position: pos });
        builder.add(LeakyNeuron::default());
        builder.add(LeakyDynamics::default());
        builder.add(LeakCurrent::default());
        builder.add(neuron_type.clone());
        builder.add(MetabolicState::default());
        builder.add(Health::new(NEURON_HEALTH));
        builder.add(Deletable {});
        builder.add(neuronify_core::VisualRadius { radius: NODE_RADIUS });
        builder.add(Selectable { selected: false });

        if matches!(neuron_type, NeuronType::Inhibitory) {
            builder.add(Inhibitory);
        }

        if is_player {
            builder.add(Ownership { player: PlayerId::Player1 });
            if n.is_origin {
                builder.add(OriginNeuron { player: PlayerId::Player1 });
                builder.add(ProductionQueue::default());
                builder.add(Anchored);
                builder.add(neuronify_core::VisualRadius { radius: NODE_RADIUS * 1.5 });
            }
        }

        // Auto-firing neurons use a RegularSpikeGenerator.
        // The origin neuron fires 5× faster than its base SVG rate.
        if n.auto_fire && n.fire_hz > 0.0 {
            let freq = if n.is_origin {
                n.fire_hz as f64 * 5.0
            } else {
                n.fire_hz as f64
            };
            builder.add(RegularSpikeGenerator { frequency: freq });
            builder.add(GeneratorDynamics::default());
        }

        builder.add(NeuronScenarioId(n.id.clone()));
        let entity = world.spawn(builder.build());
        neuron_entities.insert(n.id.clone(), (entity, pos));
    }

    // ── Spawn dendrites on player neurons ─────────────────────────────────────
    // Collect player neurons first to avoid borrow issues.
    let player_neurons: Vec<(hecs::Entity, Vec3, NeuronType)> = map.neurons.iter()
        .filter(|n| n.team == 1)
        .filter_map(|n| {
            let (entity, pos) = neuron_entities.get(&n.id)?;
            let nt = match n.neuron_type {
                NeuronDefType::Inhibitory => NeuronType::Inhibitory,
                NeuronDefType::Excitatory => NeuronType::Excitatory,
            };
            Some((*entity, *pos, nt))
        })
        .collect();

    for (entity, pos, nt) in player_neurons {
        spawning::spawn_neuron_dendrites(world, pos, &nt, PlayerId::Player1, entity, 5);
    }

    // ── Spawn axon compartment chains ─────────────────────────────────────────
    for axon in &map.axons {
        let Some(&(from_entity, from_pos)) = neuron_entities.get(&axon.from) else { continue };
        let Some(&(to_entity, to_pos)) = neuron_entities.get(&axon.to) else { continue };
        let neuron_type = if axon.excitatory {
            NeuronType::Excitatory
        } else {
            NeuronType::Inhibitory
        };
        setup::connect_axon(world, from_entity, from_pos, to_entity, to_pos, neuron_type);
    }

    // ── Spawn enemy macrophages + mast cells ─────────────────────────────────
    for mac in &map.macrophages {
        let mac_pos = svg_to_world(mac.x, mac.y);

        // Resolve driver neuron entity (if specified).
        let driver_entity = neuron_entities.get(&mac.driver_id).map(|(e, _)| *e);

        let mac_entity = spawning::spawn_macrophage(world, mac_pos, Faction::Tumor);

        if let Some(driver) = driver_entity {
            // Add activation component: starts dormant; cytokines will activate it.
            world.insert_one(mac_entity, MacrophageActivation { active: false }).ok();

            // Spawn mast cell between driver neuron and macrophage.
            let driver_pos = neuron_entities[&mac.driver_id].1;
            let mast_pos = driver_pos + (mac_pos - driver_pos) * 0.4;
            spawning::spawn_mast_cell(world, mast_pos, driver);
        }
    }

    Ok(())
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

// ── Scenario 1: Microglial Swarm ─────────────────────────────────────────────
//
// Default neural layout + one enemy spawn point at (30, 0, 0) that produces a
// Tumor macrophage every 5 s. No defensive units. Designed outcome: macrophages
// trickle in, reach the player neurons, and destroy all non-origin neurons within
// ~40 s. Origin survives due to ORIGIN_MIN_ENERGY floor.

fn setup_microglial_swarm(world: &mut hecs::World, dish: &PetriDish) {
    setup::setup_game(world, dish);
    add_axon_health_to_all_compartments(world);

    // Enemy spawn point: emits one Tumor macrophage every 5 s without neural simulation.
    // Positioned at (20, 0, 0) — far enough to give the player a reaction window,
    // close enough that macrophages reach the neural cluster within ~10 s.
    world.spawn((
        Position { position: Vec3::new(20.0, 0.0, 0.0) },
        EnemySpawnPoint {
            faction: Faction::Tumor,
            spawn_type: NeuronSpawnType::Macrophage,
            cooldown: 5.0,
            timer: 0.0,
            spawn_offset: Vec3::ZERO,
        },
        Deletable {},
        VisualRadius { radius: NODE_RADIUS * 1.2 },
    ));
}

// ── Scenario 2: Microglia Guard ───────────────────────────────────────────────
//
// Same spawn point as Scenario 1 (macrophages every 5 s from (30, 0, 0)), but the
// player has three Biological microglia placed in a defensive line at x=12 between
// the spawn point and the neural cluster.  Player microglia intercept and destroy
// the incoming macrophages before they reach the neurons.
// Designed outcome: neurons survive 30 s; all macrophages destroyed en route.

fn setup_astrocyte_guard(world: &mut hecs::World, dish: &PetriDish) {
    setup_microglial_swarm(world, dish); // reuse spawn-point setup

    // Three player microglia forming a defensive line between the spawn point and neurons.
    let guard_positions = [
        Vec3::new(12.0, 0.0, -6.0),
        Vec3::new(12.0, 0.0,  0.0),
        Vec3::new(12.0, 0.0,  6.0),
    ];
    for pos in guard_positions {
        spawning::spawn_microglial_cell(world, pos, Faction::Biological);
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
        ProductionQueue::default(),
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

    // One glial cell supports the origin-side neurons.
    spawning::spawn_glial(world, Vec3::new(10.0, 0.0, 0.0), PlayerId::Player1, 4);
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
        ProductionQueue::default(),
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
        ProductionQueue::default(),
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
        let empty_terrain = std::collections::HashMap::new();
        combat::apply_enemy_spawn_points(world, fdt);
        combat::tick_dying_units(world, fdt);
        combat::move_mobile_units(world, &empty_terrain, fdt);
        combat::apply_axon_cutting(world, fdt);
        combat::apply_neuron_engulfment(world, fdt);
        combat::apply_burst_attacks(world, fdt);
        combat::advance_attack_projectiles(world, fdt);

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
        let empty_terrain = std::collections::HashMap::new();
        combat::tick_dying_units(world, fdt);
        combat::apply_neuron_spawning(world, fdt);
        combat::move_mobile_units(world, &empty_terrain, fdt);
        combat::apply_axon_cutting(world, fdt);
        combat::apply_neuron_engulfment(world, fdt);
        combat::apply_burst_attacks(world, fdt);
        combat::advance_attack_projectiles(world, fdt);

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

    /// Scenario 1: Enemy spawn point produces macrophages that should destroy all
    /// non-origin neurons once they reach the neural cluster (no defense).
    #[test]
    fn test_scenario_1_microglial_swarm_neurons_die() {
        let mut world = hecs::World::new();
        setup_microglial_swarm(&mut world, &dish());

        run_headless(&mut world, 90.0);

        // Macrophages drain Health; neurons are despawned when health reaches 0.
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

    /// Scenario 2: Player microglia defensive line intercepts incoming macrophages;
    /// neurons should still be alive after 30 s.
    #[test]
    fn test_scenario_2_astrocyte_guard_neurons_survive() {
        let mut world = hecs::World::new();
        setup_astrocyte_guard(&mut world, &dish());

        run_headless(&mut world, 30.0);

        let alive_neurons = world
            .query::<(&LeakyNeuron, &MetabolicState)>()
            .iter()
            .filter(|(_, (_, ms))| ms.energy > 1.0)
            .count();

        assert!(
            alive_neurons >= 2,
            "At least 2 neurons should survive when protected by microglia (got {alive_neurons})"
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

    // ── Excitotoxic Wave SVG integration tests ────────────────────────────────

    const EXCITOTOXIC_WAVE_SVG: &str =
        include_str!("../../maps/excitotoxic-wave.svg");

    /// Test 1 — Parse: verify entity counts and terrain grid dimensions.
    #[test]
    fn test_svg_parse_entity_counts() {
        use crate::map::parse_scenario_svg;

        let map = parse_scenario_svg(EXCITOTOXIC_WAVE_SVG)
            .expect("SVG should parse without error");

        assert_eq!(map.neurons.len(), 16,
            "expected 16 neurons (p_origin, p_gen, p1-p8, e1-e3, e_inh, e_drv1, e_drv2)");
        assert_eq!(map.macrophages.len(), 2,
            "expected 2 macrophages (mac1, mac2)");
        assert_eq!(map.axons.len(), 18,
            "expected 18 axon definitions (13 player + 3 inhibitory + 2 driver visual)");
        assert!(map.grid_cols() >= 22,
            "terrain grid should span at least 22 columns (got {})", map.grid_cols());
        assert!(map.grid_rows() >= 16,
            "terrain grid should span at least 16 rows (got {})", map.grid_rows());
    }

    /// Test 2 — Terrain placement: no entity placed on an impassable hex.
    #[test]
    fn test_svg_no_entity_on_impassable_terrain() {
        use crate::map::parse_scenario_svg;
        use crate::map::hex::{SVG_CX, SVG_CY, MAP_SCALE};

        let map = parse_scenario_svg(EXCITOTOXIC_WAVE_SVG).unwrap();

        for n in &map.neurons {
            if let Some(hex) = map.terrain.get(&(n.col, n.row)) {
                assert!(
                    hex.passable,
                    "neuron '{}' at ({},{}) sits on impassable terrain {:?}",
                    n.id, n.col, n.row, hex.terrain
                );
            }
        }

        for m in &map.macrophages {
            let world_x = -(m.x - SVG_CX) * MAP_SCALE;
            let world_z = -(m.y - SVG_CY) * MAP_SCALE;
            let hx = crate::map::hex::world_to_hex(world_x, world_z);
            if let Some(t) = map.terrain.get(&(hx.col, hx.row)) {
                assert!(
                    t.passable,
                    "macrophage '{}' maps to impassable hex ({},{}) {:?}",
                    m.id, hx.col, hx.row, t.terrain
                );
            }
        }
    }

    /// Test 3 — Pathfinding: path from player origin (3,15) to enemy cluster
    /// (18,2) is non-empty and every intermediate step is on passable terrain.
    #[test]
    fn test_svg_pathfinding_avoids_impassable() {
        use crate::map::parse_scenario_svg;
        use crate::map::hex::{hex_center, SVG_CX, SVG_CY, MAP_SCALE};
        use crate::simulation::pathfinding::HexGrid;
        use crate::constants::HEX_GRID_CELL_SIZE;

        let map = parse_scenario_svg(EXCITOTOXIC_WAVE_SVG).unwrap();
        let mut world = hecs::World::new();
        let mover = world.spawn((neuronify_core::Position { position: Vec3::ZERO },));
        let grid = HexGrid::new(HEX_GRID_CELL_SIZE);

        let svg_to_world = |col: i32, row: i32| {
            let (sx, sy) = hex_center(col, row);
            Vec3::new(-(sx - SVG_CX) * MAP_SCALE, 0.0, -(sy - SVG_CY) * MAP_SCALE)
        };

        // p_origin is at hex (3,15); enemy cluster near e1 at hex (18,2).
        let start = grid.world_to_hex(svg_to_world(3, 15));
        let goal  = grid.world_to_hex(svg_to_world(18, 2));

        let path = grid.a_star(mover, start, goal, &map.terrain);

        assert!(
            !path.is_empty(),
            "A* should find a path from (3,15) to (18,2) through the map"
        );

        // Every intermediate waypoint must lie on passable terrain (goal exempt
        // because the tile may be solid but the game allows arrival there).
        for &step in path.iter().take(path.len().saturating_sub(1)) {
            let wp = grid.hex_to_world(step);
            let mh = crate::map::hex::world_to_hex(wp.x, wp.z);
            if let Some(t) = map.terrain.get(&(mh.col, mh.row)) {
                assert!(
                    t.passable,
                    "path passes through impassable hex ({},{}) {:?}",
                    mh.col, mh.row, t.terrain
                );
            }
        }
    }

    /// Test 4 — Signal propagation: after forcing p_origin to fire and running
    /// 200 combat ticks, the directly connected neuron p1 should have fired.
    #[test]
    fn test_svg_signal_propagates_from_origin() {
        use neuronify_core::{GeneratorDynamics, LeakyDynamics};
        use crate::map::parse_scenario_svg;

        let _ = parse_scenario_svg(EXCITOTOXIC_WAVE_SVG).unwrap(); // pre-check

        let mut world = hecs::World::new();
        let mut dish = PetriDish { center: Vec3::ZERO, radius: 220.0 };
        setup_scenario_from_svg(&mut world, &mut dish, EXCITOTOXIC_WAVE_SVG)
            .expect("SVG scenario should set up without error");

        // Force-fire p_origin by resetting its generator clock so it fires
        // immediately at tick 1.
        for (_, (gen, _)) in world
            .query::<(&mut GeneratorDynamics, &NeuronScenarioId)>()
            .iter()
            .filter(|(_, (_, sid))| sid.0 == "p_origin")
        {
            gen.time_since_fire = f64::INFINITY; // ensure the generator re-arms
        }
        for (_, (dyn_, _)) in world
            .query::<(&mut LeakyDynamics, &NeuronScenarioId)>()
            .iter()
            .filter(|(_, (_, sid))| sid.0 == "p_origin")
        {
            dyn_.voltage = 0.0; // prime just below threshold — generator will push it over
        }

        // 200 combat ticks at 0.016 s = 3.2 s of simulation.
        // p_origin auto-fires at 3 Hz → ~9 spikes; p1 receives direct excitatory input.
        run_headless_neural(&mut world, 200.0 * 0.016);

        let p1_fired = world
            .query::<(&LeakyDynamics, &NeuronScenarioId)>()
            .iter()
            .any(|(_, (d, sid))| sid.0 == "p1" && d.time_since_fire < 3.5);

        assert!(
            p1_fired,
            "p1 should have fired within 3.2 s after p_origin auto-fires at 3 Hz"
        );
    }

    /// Test 5 — Macrophage activation: mac1 activates when its driver neuron
    /// (e_drv1) has fired recently, and goes dormant once cytokines dissipate.
    #[test]
    fn test_svg_macrophage_activates_and_deactivates() {
        use neuronify_core::GeneratorDynamics;

        let mut world = hecs::World::new();
        let mut dish = PetriDish { center: Vec3::ZERO, radius: 220.0 };
        setup_scenario_from_svg(&mut world, &mut dish, EXCITOTOXIC_WAVE_SVG)
            .expect("SVG scenario should set up without error");

        // Macrophages with a driver start dormant.
        let any_mac_entity = world
            .query::<&MacrophageActivation>()
            .iter()
            .map(|(e, _)| e)
            .next()
            .expect("at least one MacrophageActivation entity should exist");
        assert!(
            !world.get::<&MacrophageActivation>(any_mac_entity).unwrap().active,
            "macrophage should start dormant"
        );

        // Simulate the driver firing: set GeneratorDynamics.time_since_fire = 0 for
        // all auto-firing enemy neurons so their mast cells emit immediately.
        for (_, gen) in world.query::<&mut GeneratorDynamics>().iter() {
            gen.time_since_fire = 0.0;
        }

        // Tick enough for the mast cell to emit (need emit_timer to expire).
        // MAST_CELL_EMIT_INTERVAL = 0.25 s; use dt = 0.3 s > interval.
        crate::simulation::cytokines::emit_cytokines(&mut world, 0.3);
        crate::simulation::cytokines::tick_cytokines(&mut world, 0.01);
        crate::simulation::cytokines::activate_macrophages(&mut world);

        let mac_active = world
            .get::<&MacrophageActivation>(any_mac_entity)
            .map(|a| a.active)
            .unwrap_or(false);
        assert!(mac_active, "macrophage should be active when driver fires and cytokines are present");

        // Now block the driver: set time_since_fire >> RECENT_FIRE_WINDOW (0.5 s)
        // so no new cytokines are emitted.
        for (_, gen) in world.query::<&mut GeneratorDynamics>().iter() {
            gen.time_since_fire = 10.0;
        }

        // Age all existing cytokines past CYTOKINE_LIFETIME (3.5 s).
        // 500 ticks × 0.01 s = 5.0 s > 3.5 s lifetime.
        for _ in 0..500 {
            crate::simulation::cytokines::tick_cytokines(&mut world, 0.01);
        }
        crate::simulation::cytokines::activate_macrophages(&mut world);

        let mac_dormant = world
            .get::<&MacrophageActivation>(any_mac_entity)
            .map(|a| !a.active)
            .unwrap_or(false);
        assert!(mac_dormant, "macrophage should be dormant after cytokines dissipate");
    }

    /// Test 6 — Victory: despawning all three target neurons (e1, e2, e3)
    /// triggers the parsed victory condition.
    #[test]
    fn test_svg_victory_triggers_when_targets_dead() {
        use crate::map::parse_scenario_svg;
        use crate::simulation::victory;

        let map = parse_scenario_svg(EXCITOTOXIC_WAVE_SVG).unwrap();
        let cond = victory::parse_victory(&map.meta.victory)
            .expect("scenario should have a valid victory condition");

        let mut world = hecs::World::new();
        let mut dish = PetriDish { center: Vec3::ZERO, radius: 220.0 };
        setup_scenario_from_svg(&mut world, &mut dish, EXCITOTOXIC_WAVE_SVG).unwrap();

        // Victory should NOT be satisfied while all neurons are alive.
        assert!(
            !victory::check_victory(&world, &cond),
            "victory should not be satisfied while e1/e2/e3 are alive"
        );

        // Despawn each target neuron in turn.
        let targets = ["e1", "e2", "e3"];
        for target_id in &targets {
            let entity = world
                .query::<&NeuronScenarioId>()
                .iter()
                .find(|(_, sid)| &sid.0 == target_id)
                .map(|(e, _)| e)
                .unwrap_or_else(|| panic!("target neuron '{}' not found in world", target_id));

            world.despawn(entity).unwrap();

            if *target_id != "e3" {
                assert!(
                    !victory::check_victory(&world, &cond),
                    "victory should not trigger until all three targets are dead (after killing {})", target_id
                );
            }
        }

        // All three dead → victory.
        assert!(
            victory::check_victory(&world, &cond),
            "victory should trigger after all three target neurons are despawned"
        );
    }
}
