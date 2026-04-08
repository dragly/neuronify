//! Dev-only scenario stages for reviewing game progression.
//!
//! Accessible via `--dev` CLI flag. Each stage loads the base excitotoxic-wave
//! SVG and then programmatically adds entities to represent a game phase.

use glam::Vec3;
use neuronify_core::{NeuronType, Position, Selectable};

use crate::components::*;
use crate::map::hex::hex_to_world;
use crate::simulation::setup;
use crate::spawning;
use crate::ui::main_menu::ScenarioEntry;
use crate::map::ScenarioMeta;

const BASE_SVG: &str = include_str!("../../maps/excitotoxic-wave.svg");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevStage {
    Early,
    Late,
    Attack,
}

/// Build the three dev scenario menu entries.
pub fn load_dev_scenario_entries() -> Vec<ScenarioEntry> {
    vec![
        ScenarioEntry {
            meta: ScenarioMeta {
                id: "dev_early".to_string(),
                name: "Dev: Early Stage".to_string(),
                objective: "Game start — no player actions taken".to_string(),
                victory: String::new(),
                briefing: "The scenario as the player first sees it. Neural cluster at the south, enemy outpost at the north. No network has been built yet.".to_string(),
            },
            svg_content: BASE_SVG,
            dev_stage: Some(DevStage::Early),
        },
        ScenarioEntry {
            meta: ScenarioMeta {
                id: "dev_late".to_string(),
                name: "Dev: Late Stage".to_string(),
                objective: "Network built to enemy side, ready to attack".to_string(),
                victory: String::new(),
                briefing: "The player has built a relay chain from their cluster through the open corridor to the enemy outpost. Glial cells harvest glucose along the route. Combat units are staged near the front line.".to_string(),
            },
            svg_content: BASE_SVG,
            dev_stage: Some(DevStage::Late),
        },
        ScenarioEntry {
            meta: ScenarioMeta {
                id: "dev_attack".to_string(),
                name: "Dev: Attack Stage".to_string(),
                objective: "Combat underway, enemy defenses active".to_string(),
                victory: String::new(),
                briefing: "Player units are engaging the enemy. Macrophages are activated by driver neurons. Some enemy neurons show damage. The battle is in progress.".to_string(),
            },
            svg_content: BASE_SVG,
            dev_stage: Some(DevStage::Attack),
        },
    ]
}

/// Apply dev-stage modifications after the base SVG has been loaded.
pub fn apply_dev_stage(
    world: &mut hecs::World,
    economy: &mut PlayerEconomy,
    stage: DevStage,
) {
    match stage {
        DevStage::Early => {} // Base SVG is the early state — nothing to add.
        DevStage::Late => apply_late_stage(world, economy),
        DevStage::Attack => {
            apply_late_stage(world, economy);
            apply_attack_stage(world);
        }
    }
}

// ── Relay neuron positions along the verified passable corridor ──────────────

// Relay path verified to avoid all impassable terrain (vessels, scars, CSF).
// Each consecutive pair has a straight-line connection that stays in passable hexes.
// Route: go left to col 2 to cross the vessel band at rows 9-10, then east along row 7.
const RELAY_HEXES: [(i32, i32); 8] = [
    (2, 11),  // open — cross above vessel row via col 2
    (2, 9),   // open — past vessels (col 2 rows 9-11 all passable)
    (4, 7),   // open — diagonal into the clear row-7 corridor
    (7, 7),   // open — east along row 7
    (10, 7),  // open — continuing east
    (13, 6),  // open — angle toward enemy cluster
    (15, 6),  // open — detour around glial scar at (15,5)
    (16, 5),  // open — approaching enemy outpost
];

const GLIAL_HEXES: [(i32, i32); 2] = [
    (3, 9),   // open, adjacent to vessel hexes at (3,10) and (4,9)
    (6, 7),   // open, adjacent to vessel hex at (6,8)
];

// ── Late stage ───────────────────────────────────────────────────────────────

fn apply_late_stage(world: &mut hecs::World, economy: &mut PlayerEconomy) {
    // Find p8 (the furthest player neuron from origin in the SVG network).
    let p8 = find_neuron_by_id(world, "p8");

    // Spawn relay neurons along the corridor.
    let mut relay_entities: Vec<(hecs::Entity, Vec3)> = Vec::new();
    for &(col, row) in &RELAY_HEXES {
        let pos = hex_to_world(col, row);
        let entity = spawning::spawn_neuron_with_dendrites(
            world,
            pos,
            NeuronType::Excitatory,
            PlayerId::Player1,
            5,
        );
        world.insert_one(entity, Selectable { selected: false }).ok();
        relay_entities.push((entity, pos));
    }

    // Chain with axons: p8 → relay1 → relay2 → ... → relay7
    let mut prev = p8;
    for &(entity, pos) in &relay_entities {
        let prev_pos = world
            .get::<&Position>(prev.0)
            .map(|p| p.position)
            .unwrap_or(prev.1);
        setup::connect_axon(world, prev.0, prev_pos, entity, pos, NeuronType::Excitatory);
        prev = (entity, pos);
    }

    // Spawn glial cells near vessel terrain for economy.
    for &(col, row) in &GLIAL_HEXES {
        let pos = hex_to_world(col, row);
        spawning::spawn_glial(world, pos, PlayerId::Player1, 3);
    }

    // Spawn combat units near the front line (last relay neuron area).
    let front_pos = relay_entities.last().map(|r| r.1).unwrap_or(Vec3::ZERO);
    let offset_right = Vec3::new(4.0, 0.0, 2.0);
    let offset_left = Vec3::new(-4.0, 0.0, 2.0);
    let offset_back = Vec3::new(0.0, 0.0, 6.0);

    spawning::spawn_microglial_cell(world, front_pos + offset_right, Faction::Biological);
    spawning::spawn_microglial_cell(world, front_pos + offset_left, Faction::Biological);
    spawning::spawn_microglial_cell(world, front_pos + offset_back, Faction::Biological);
    spawning::spawn_t_cell(world, front_pos + offset_right * 1.5, Faction::Biological);
    spawning::spawn_t_cell(world, front_pos + offset_left * 1.5, Faction::Biological);

    // Represent accumulated resources.
    economy.building_blocks = 800.0;
}

// ── Attack stage (on top of late stage) ──────────────────────────────────────

fn apply_attack_stage(world: &mut hecs::World) {
    // Find enemy neuron entities for targeting.
    let e1 = find_neuron_by_id(world, "e1");
    let e2 = find_neuron_by_id(world, "e2");

    // Set manual attack targets on all player combat units.
    let player_combat: Vec<hecs::Entity> = world
        .query::<(&MobileUnit, &Ownership)>()
        .iter()
        .filter(|(_, (mu, o))| o.player == PlayerId::Player1 && mu.faction == Faction::Biological)
        .map(|(e, _)| e)
        .collect();

    for entity in player_combat {
        if let Ok(mut mu) = world.get::<&mut MobileUnit>(entity) {
            mu.manual_target = Some(e1.0);
        }
    }

    // Spawn extra T-cells in attack position near enemy neurons.
    let attack_pos = e1.1 + Vec3::new(8.0, 0.0, 4.0);
    spawning::spawn_t_cell(world, attack_pos, Faction::Biological);
    spawning::spawn_t_cell(world, attack_pos + Vec3::new(-4.0, 0.0, 3.0), Faction::Biological);

    // Activate enemy macrophages.
    let enemy_macs: Vec<hecs::Entity> = world
        .query::<&MacrophageActivation>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in enemy_macs {
        if let Ok(mut activation) = world.get::<&mut MacrophageActivation>(entity) {
            activation.active = true;
        }
    }

    // Damage enemy neurons e1 and e2.
    for (entity, _pos) in [&e1, &e2] {
        if let Ok(mut health) = world.get::<&mut Health>(*entity) {
            health.current = health.max * 0.6;
        }
        if let Ok(mut metab) = world.get::<&mut MetabolicState>(*entity) {
            metab.energy = metab.max_energy * 0.5;
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Find a neuron entity by its `NeuronScenarioId`, returning entity + position.
fn find_neuron_by_id(world: &hecs::World, id: &str) -> (hecs::Entity, Vec3) {
    world
        .query::<(&NeuronScenarioId, &Position)>()
        .iter()
        .find(|(_, (sid, _))| sid.0 == id)
        .map(|(e, (_, p))| (e, p.position))
        .unwrap_or_else(|| panic!("dev scenario: neuron '{}' not found", id))
}
