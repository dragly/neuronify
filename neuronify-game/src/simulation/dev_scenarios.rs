//! Dev-only scenario stages for reviewing game progression.
//!
//! Accessible via `--dev` CLI flag. Each stage loads the Voronoi scenario
//! and then programmatically adds entities to represent a game phase.

use glam::Vec3;
use neuronify_core::{NeuronType, Position, Selectable};

use neuronify_game_lib::voronoi_map::game_integration;

use crate::components::*;
use crate::simulation::setup;
use crate::spawning;
use crate::ui::main_menu::ScenarioEntry;
use crate::map::ScenarioMeta;

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
                briefing: "The scenario as the player first sees it. Neural cluster in the lower-left, enemy outpost upper-right. No network has been built yet.".to_string(),
            },
            dev_stage: Some(DevStage::Early),
        },
        ScenarioEntry {
            meta: ScenarioMeta {
                id: "dev_late".to_string(),
                name: "Dev: Late Stage".to_string(),
                objective: "Network built to enemy side, ready to attack".to_string(),
                victory: String::new(),
                briefing: "The player has built a relay chain through the open corridor to the enemy outpost. Glial cells harvest glucose from nearby vessels. Combat units are staged near the front line.".to_string(),
            },
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
            dev_stage: Some(DevStage::Attack),
        },
    ]
}

/// Apply dev-stage modifications after the Voronoi scenario has been loaded.
pub fn apply_dev_stage(
    world: &mut hecs::World,
    economy: &mut PlayerEconomy,
    stage: DevStage,
) {
    match stage {
        DevStage::Early => {} // Base scenario is the early state.
        DevStage::Late => apply_late_stage(world, economy),
        DevStage::Attack => {
            apply_late_stage(world, economy);
            apply_attack_stage(world);
        }
    }
}

// ── Relay neuron positions through the open corridor ─────────────────────────
// Voronoi coords along the gap between the two vessel segments.
// Route: from player network (p6 area ~205,385) through the gap (~350,250)
// toward the enemy outpost (~520,100).

fn relay_world_positions() -> Vec<Vec3> {
    let model = game_integration::load_voronoi_model();
    vec![
        game_integration::voronoi_to_world(&model, 240.0, 330.0),
        game_integration::voronoi_to_world(&model, 280.0, 300.0),
        game_integration::voronoi_to_world(&model, 320.0, 270.0),
        game_integration::voronoi_to_world(&model, 360.0, 240.0),
        game_integration::voronoi_to_world(&model, 400.0, 210.0),
        game_integration::voronoi_to_world(&model, 440.0, 180.0),
        game_integration::voronoi_to_world(&model, 480.0, 150.0),
        game_integration::voronoi_to_world(&model, 510.0, 120.0),
    ]
}

// ── Late stage ───────────────────────────────────────────────────────────────

fn apply_late_stage(world: &mut hecs::World, economy: &mut PlayerEconomy) {
    // Find p6 (the furthest player neuron from origin in the convergent network).
    let p6 = find_neuron_by_id(world, "p6");

    // Spawn relay neurons along the corridor.
    let positions = relay_world_positions();
    let mut relay_entities: Vec<(hecs::Entity, Vec3)> = Vec::new();
    for pos in &positions {
        let entity = spawning::spawn_neuron_with_dendrites(
            world,
            *pos,
            NeuronType::Excitatory,
            PlayerId::Player1,
            5,
        );
        world.insert_one(entity, Selectable { selected: false }).ok();
        relay_entities.push((entity, *pos));
    }

    // Chain with axons: p6 → relay1 → relay2 → ... → relay8
    let mut prev = p6;
    for &(entity, pos) in &relay_entities {
        let prev_pos = world
            .get::<&Position>(prev.0)
            .map(|p| p.position)
            .unwrap_or(prev.1);
        setup::connect_axon(world, prev.0, prev_pos, entity, pos, NeuronType::Excitatory);
        prev = (entity, pos);
    }

    // Tag compartments created by the relay chain as player-owned.
    // We find compartments connected to player-owned neurons via BFS.
    {
        use neuronify_core::{Compartment, Connection, CompartmentCurrent};
        let player_entities: std::collections::HashSet<hecs::Entity> = world
            .query::<&Ownership>()
            .iter()
            .filter(|(_, o)| o.player == PlayerId::Player1)
            .map(|(e, _)| e)
            .collect();
        // Walk connections from player entities to find reachable unowned compartments.
        let mut to_tag = Vec::new();
        let connections: Vec<(hecs::Entity, hecs::Entity)> = world
            .query::<&Connection>()
            .with::<&CompartmentCurrent>()
            .iter()
            .map(|(_, c)| (c.from, c.to))
            .collect();
        let mut visited = player_entities.clone();
        let mut frontier: Vec<hecs::Entity> = player_entities.into_iter().collect();
        while let Some(entity) = frontier.pop() {
            for &(from, to) in &connections {
                let neighbor = if from == entity { to } else if to == entity { from } else { continue };
                if visited.insert(neighbor) {
                    if world.get::<&Compartment>(neighbor).is_ok() && world.get::<&Ownership>(neighbor).is_err() {
                        to_tag.push(neighbor);
                        frontier.push(neighbor);
                    }
                }
            }
        }
        for e in to_tag {
            world.insert_one(e, Ownership { player: PlayerId::Player1 }).ok();
        }
    }

    // Spawn glial cells near vessel terrain for economy.
    let model = game_integration::load_voronoi_model();
    let glial_positions = [
        game_integration::voronoi_to_world(&model, 260.0, 200.0),
        game_integration::voronoi_to_world(&model, 420.0, 280.0),
    ];
    for pos in &glial_positions {
        spawning::spawn_glial(world, *pos, PlayerId::Player1, 3);
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

fn find_neuron_by_id(world: &hecs::World, id: &str) -> (hecs::Entity, Vec3) {
    world
        .query::<(&NeuronScenarioId, &Position)>()
        .iter()
        .find(|(_, (sid, _))| sid.0 == id)
        .map(|(e, (_, p))| (e, p.position))
        .unwrap_or_else(|| panic!("dev scenario: neuron '{}' not found", id))
}
