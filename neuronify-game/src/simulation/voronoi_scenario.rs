//! Voronoi terrain scenario setup.
//!
//! Spawns entities for the "Neural Infiltration" scenario on the Voronoi map:
//! player convergent network, enemy outpost with inhibitory suppressor,
//! driver-controlled macrophages, and victory condition.

use std::collections::HashMap;

use glam::Vec3;
use hecs::Entity;

use neuronify_core::{
    Deletable, GeneratorDynamics, Inhibitory, LeakCurrent, LeakyDynamics, LeakyNeuron,
    NeuronType, Position, RegularSpikeGenerator, Selectable, VisualRadius, NODE_RADIUS,
};

use neuronify_game_lib::voronoi_map::game_integration;
use neuronify_game_lib::voronoi_map::terrain_model::{EditorTerrain, MapModel};

use crate::components::*;
use crate::constants::*;
use crate::map;
use crate::simulation::setup;
use crate::simulation::victory;
use crate::spawning;

/// Spawn a neuron with standard components.
fn spawn_neuron(
    world: &mut hecs::World,
    pos: Vec3,
    neuron_type: NeuronType,
    player: Option<PlayerId>,
    is_origin: bool,
    auto_fire_hz: Option<f64>,
    scenario_id: &str,
) -> Entity {
    let mut b = hecs::EntityBuilder::new();
    b.add(Position { position: pos });
    b.add(LeakyNeuron::default());
    b.add(LeakyDynamics::default());
    b.add(LeakCurrent::default());
    b.add(neuron_type.clone());
    b.add(MetabolicState::default());
    b.add(Health::new(NEURON_HEALTH));
    b.add(Deletable {});
    b.add(Selectable { selected: false });

    if matches!(neuron_type, NeuronType::Inhibitory) {
        b.add(Inhibitory);
    }

    if let Some(player) = player {
        b.add(Ownership { player });
        if is_origin {
            b.add(OriginNeuron { player });
            b.add(ProductionQueue::default());
            b.add(Anchored);
            b.add(VisualRadius { radius: NODE_RADIUS * 1.5 });
        } else {
            b.add(VisualRadius { radius: NODE_RADIUS });
        }
    } else {
        b.add(VisualRadius { radius: NODE_RADIUS });
    }

    if let Some(hz) = auto_fire_hz {
        let freq = if is_origin { hz * 5.0 } else { hz };
        b.add(RegularSpikeGenerator { frequency: freq });
        b.add(GeneratorDynamics::default());
    }

    b.add(NeuronScenarioId(scenario_id.to_string()));
    world.spawn(b.build())
}

/// Set up the full Voronoi scenario: terrain hex lookup, entities, victory.
///
/// Returns (victory_condition, scenario_name, briefing, objective).
pub fn setup_voronoi_scenario(
    world: &mut hecs::World,
    model: &MapModel,
    scenario_terrain: &mut HashMap<(i32, i32), map::HexTerrain>,
) -> (Option<victory::VictoryCondition>, String, String, String) {
    // ── Terrain hex sampling ─────────────────────────────────────────────
    scenario_terrain.clear();
    let (min_x, min_z, max_x, max_z) = game_integration::world_bounds(model);

    for row in -15..35 {
        for col in -15..40 {
            let wp = map::hex::hex_to_world(col, row);
            if wp.x < min_x - 20.0
                || wp.x > max_x + 20.0
                || wp.z < min_z - 20.0
                || wp.z > max_z + 20.0
            {
                continue;
            }
            let et = game_integration::sample_terrain_at(model, wp.x, wp.z);
            let (passable, speed_mult, resource_glucose) = match et {
                EditorTerrain::Open => (true, 1.0, false),
                EditorTerrain::Vessel => (false, 1.0, true),
                EditorTerrain::GlialScar => (false, 1.0, false),
                EditorTerrain::Csf => (false, 1.0, false),
            };
            let terrain_type = match et {
                EditorTerrain::Open => map::TerrainType::Open,
                EditorTerrain::Vessel => map::TerrainType::Vessel,
                EditorTerrain::GlialScar => map::TerrainType::GlialScar,
                EditorTerrain::Csf => map::TerrainType::Csf,
            };
            scenario_terrain.insert(
                (col, row),
                map::HexTerrain {
                    col,
                    row,
                    terrain: terrain_type,
                    passable,
                    speed_mult,
                    resource_glucose,
                },
            );
        }
    }

    // ── Entity placement ─────────────────────────────────────────────────
    // Voronoi map: 700×500. Player base lower-left, enemy upper-right.

    // Player base (lower-left open area).
    // Lean start: origin + generator + 2 forward path neurons.
    // Player builds the rest of the network manually.
    let p_origin_pos = game_integration::voronoi_to_world(model, 100.0, 400.0);
    let p_gen_pos    = game_integration::voronoi_to_world(model, 130.0, 385.0);
    let p1_pos       = game_integration::voronoi_to_world(model, 155.0, 410.0);
    let p2_pos       = game_integration::voronoi_to_world(model, 155.0, 370.0);

    let p_origin = spawn_neuron(
        world, p_origin_pos, NeuronType::Excitatory,
        Some(PlayerId::Player1), true, Some(3.0), "p_origin",
    );
    let p_gen = spawn_neuron(
        world, p_gen_pos, NeuronType::Excitatory,
        Some(PlayerId::Player1), false, Some(5.0), "p_gen",
    );
    let p1 = spawn_neuron(world, p1_pos, NeuronType::Excitatory, Some(PlayerId::Player1), false, None, "p1");
    let p2 = spawn_neuron(world, p2_pos, NeuronType::Excitatory, Some(PlayerId::Player1), false, None, "p2");

    // Two parallel paths from origin.
    setup::connect_axon(world, p_origin, p_origin_pos, p_gen, p_gen_pos, NeuronType::Excitatory);
    setup::connect_axon(world, p_origin, p_origin_pos, p1, p1_pos, NeuronType::Excitatory);
    setup::connect_axon(world, p_origin, p_origin_pos, p2, p2_pos, NeuronType::Excitatory);

    // Tag all compartments with player ownership.
    {
        use neuronify_core::Compartment;
        let player_comps: Vec<hecs::Entity> = world
            .query::<&Compartment>()
            .iter()
            .filter(|(e, _)| world.get::<&Ownership>(*e).is_err())
            .map(|(e, _)| e)
            .collect();
        for e in player_comps {
            world.insert_one(e, Ownership { player: PlayerId::Player1 }).ok();
        }
    }

    // Dendrites on player neurons.
    for &entity in &[p_origin, p_gen, p1, p2] {
        let pos = world.get::<&Position>(entity).unwrap().position;
        spawning::spawn_neuron_dendrites(world, pos, &NeuronType::Excitatory, PlayerId::Player1, entity, 5);
    }

    // Player glial cells adjacent to the bottom-left vessel segment
    // (vessel runs from Voronoi (100,350) to (150,420)).
    // Place glials right at the vessel edge so their short processes touch it.
    let pg1_pos = game_integration::voronoi_to_world(model, 105.0, 355.0);
    let pg2_pos = game_integration::voronoi_to_world(model, 145.0, 415.0);
    spawning::spawn_glial(world, pg1_pos, PlayerId::Player1, 5);
    spawning::spawn_glial(world, pg2_pos, PlayerId::Player1, 5);

    // Player defensive units near the base.
    let def1_pos = game_integration::voronoi_to_world(model, 200.0, 400.0);
    let def2_pos = game_integration::voronoi_to_world(model, 220.0, 370.0);
    spawning::spawn_microglial_cell(world, def1_pos, Faction::Biological);
    spawning::spawn_microglial_cell(world, def2_pos, Faction::Biological);

    // Enemy outpost (upper-right open area).
    let e1_pos = game_integration::voronoi_to_world(model, 580.0, 100.0);
    let e2_pos = game_integration::voronoi_to_world(model, 600.0, 115.0);
    let e3_pos = game_integration::voronoi_to_world(model, 560.0, 115.0);
    let e_inh_pos = game_integration::voronoi_to_world(model, 580.0, 115.0);
    let e_drv1_pos = game_integration::voronoi_to_world(model, 520.0, 100.0);
    let e_drv2_pos = game_integration::voronoi_to_world(model, 610.0, 140.0);

    let e1 = spawn_neuron(world, e1_pos, NeuronType::Excitatory, None, false, None, "e1");
    let e2 = spawn_neuron(world, e2_pos, NeuronType::Excitatory, None, false, None, "e2");
    let e3 = spawn_neuron(world, e3_pos, NeuronType::Excitatory, None, false, None, "e3");
    let e_inh = spawn_neuron(world, e_inh_pos, NeuronType::Inhibitory, None, false, Some(8.0), "e_inh");
    let e_drv1 = spawn_neuron(world, e_drv1_pos, NeuronType::Excitatory, None, false, Some(4.0), "e_drv1");
    let e_drv2 = spawn_neuron(world, e_drv2_pos, NeuronType::Excitatory, None, false, Some(4.0), "e_drv2");

    // Inhibitory suppressor blocks single inputs to targets.
    setup::connect_axon(world, e_inh, e_inh_pos, e1, e1_pos, NeuronType::Inhibitory);
    setup::connect_axon(world, e_inh, e_inh_pos, e2, e2_pos, NeuronType::Inhibitory);
    setup::connect_axon(world, e_inh, e_inh_pos, e3, e3_pos, NeuronType::Inhibitory);

    // Macrophages in the open corridor between bases.
    let mac1_pos = game_integration::voronoi_to_world(model, 350.0, 250.0);
    let mac2_pos = game_integration::voronoi_to_world(model, 450.0, 200.0);

    let mac1 = spawning::spawn_macrophage(world, mac1_pos, Faction::Tumor);
    let mac2 = spawning::spawn_macrophage(world, mac2_pos, Faction::Tumor);

    // Mast cells 40% of the way from driver to macrophage.
    world.insert_one(mac1, MacrophageActivation { active: false }).ok();
    world.insert_one(mac2, MacrophageActivation { active: false }).ok();
    let mast1_pos = e_drv1_pos + (mac1_pos - e_drv1_pos) * 0.4;
    let mast2_pos = e_drv2_pos + (mac2_pos - e_drv2_pos) * 0.4;
    spawning::spawn_mast_cell(world, mast1_pos, e_drv1);
    spawning::spawn_mast_cell(world, mast2_pos, e_drv2);

    // ── Enemy spawn towers ───────────────────────────────────────────────
    // Towers are regular neurons (no auto-fire) that receive excitatory
    // input from the driver network. Energy comes from enemy glial cells
    // near vessels. When drivers fire, signal propagates to towers, which
    // fire and trigger NeuronSpawner to release combat units.

    // Tower 1: spawns microglia (axon cutters).
    let tower1_pos = game_integration::voronoi_to_world(model, 480.0, 170.0);
    let tower1 = spawn_neuron(world, tower1_pos, NeuronType::Excitatory, None, false, None, "tower1");
    world.insert_one(tower1, NeuronSpawner {
        faction: Faction::Tumor,
        spawn_type: NeuronSpawnType::MicroglialCell,
        cooldown: 8.0,
        timer: 0.0,
        spawn_offset: Vec3::new(2.0, 0.0, 0.0),
    }).ok();

    // Tower 2: spawns microglia (axon cutters).
    let tower2_pos = game_integration::voronoi_to_world(model, 520.0, 200.0);
    let tower2 = spawn_neuron(world, tower2_pos, NeuronType::Excitatory, None, false, None, "tower2");
    world.insert_one(tower2, NeuronSpawner {
        faction: Faction::Tumor,
        spawn_type: NeuronSpawnType::MicroglialCell,
        cooldown: 10.0,
        timer: 0.0,
        spawn_offset: Vec3::new(2.0, 0.0, 0.0),
    }).ok();

    // Wire drivers → towers so towers fire when drivers fire.
    setup::connect_axon(world, e_drv1, e_drv1_pos, tower1, tower1_pos, NeuronType::Excitatory);
    setup::connect_axon(world, e_drv2, e_drv2_pos, tower2, tower2_pos, NeuronType::Excitatory);

    // Enemy glial cells at the top-right vessel (runs from Voronoi (500,100) to (580,80)).
    let eg1_pos = game_integration::voronoi_to_world(model, 505.0, 98.0);
    let eg2_pos = game_integration::voronoi_to_world(model, 575.0, 82.0);
    for &gpos in &[eg1_pos, eg2_pos] {
        let glial = spawning::spawn_glial(world, gpos, PlayerId::Player1, 5);
        // Remove player ownership — enemy glial.
        let _ = world.remove_one::<Ownership>(glial);
        // Also strip ownership from the glial's process compartments.
        {
            use neuronify_core::Compartment;
            let process_comps: Vec<(hecs::Entity, Vec3)> = world
                .query::<(&GlialProcess, &Position)>()
                .iter()
                .filter(|(e, _)| {
                    world.get::<&Ownership>(*e)
                        .map(|o| o.player == PlayerId::Player1)
                        .unwrap_or(false)
                })
                .map(|(e, (_, p))| (e, p.position))
                .collect();
            for (e, pos) in process_comps {
                if pos.distance(gpos) < 20.0 {
                    let _ = world.remove_one::<Ownership>(e);
                }
            }
        }
        // Connect glial to nearest driver so energy flows into the network.
        let nearest_drv = if gpos.distance(e_drv1_pos) < gpos.distance(e_drv2_pos) {
            (e_drv1, e_drv1_pos)
        } else {
            (e_drv2, e_drv2_pos)
        };
        setup::connect_axon(world, glial, gpos, nearest_drv.0, nearest_drv.1, NeuronType::Excitatory);
    }

    // Victory condition.
    let victory_cond = victory::parse_victory("all_neurons_dead:e1,e2,e3");

    let name = "Neural Infiltration".to_string();
    let briefing = "The enemy has fortified an outpost in the upper cortex, \
        protected by dormant macrophages, spawn towers, and an inhibitory suppressor. \
        Enemy glial cells harvest glucose from nearby vessels to power the network. \
        Driver neurons trigger spawn towers that release microglia and T-cells \u{2014} \
        disable the drivers to shut them down. \
        Blood vessels cut through the center \u{2014} axons cannot cross them. \
        Use convergent summation to overwhelm the inhibitory gate and destroy all three targets."
        .to_string();
    let objective = "Destroy all three target neurons (e1, e2, e3)".to_string();

    (victory_cond, name, briefing, objective)
}
