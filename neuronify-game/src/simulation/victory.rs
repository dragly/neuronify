//! Victory condition parsing and checking.
//!
//! Victory strings use the format `condition_type:param1,param2,...`.
//! New condition types can be added to `VictoryCondition` and `parse_victory`
//! without changing the checking interface.

use crate::components::NeuronScenarioId;

/// A parsed victory condition for a scenario.
#[derive(Clone, Debug)]
pub enum VictoryCondition {
    /// Win when all listed neuron IDs have been removed from the world (killed).
    AllNeuronsDead { neuron_ids: Vec<String> },
    // Future variants (not yet implemented):
    // NeuronCaptured { id: String },
    // SurviveTime { seconds: f64 },
    // ControlVessels { count: usize },
}

/// Parse a `data-victory` attribute string into a `VictoryCondition`.
/// Returns `None` for empty, malformed, or unknown condition types.
pub fn parse_victory(s: &str) -> Option<VictoryCondition> {
    let (kind, params) = s.split_once(':')?;
    match kind.trim() {
        "all_neurons_dead" => {
            let neuron_ids: Vec<String> = params
                .split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect();
            if neuron_ids.is_empty() {
                None
            } else {
                Some(VictoryCondition::AllNeuronsDead { neuron_ids })
            }
        }
        _ => None, // unknown condition type — ignore gracefully
    }
}

/// Check whether the given victory condition is currently satisfied.
pub fn check_victory(world: &hecs::World, condition: &VictoryCondition) -> bool {
    match condition {
        VictoryCondition::AllNeuronsDead { neuron_ids } => {
            neuron_ids.iter().all(|id| {
                // The neuron is "dead" if no entity with this NeuronScenarioId exists.
                // (Neurons are despawned immediately on health-zero by despawn_dead.)
                !world
                    .query::<&NeuronScenarioId>()
                    .iter()
                    .any(|(_, sid)| &sid.0 == id)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_core::Position;
    use crate::components::{Health, NeuronScenarioId};

    fn spawn_neuron_with_id(world: &mut hecs::World, id: &str) -> hecs::Entity {
        world.spawn((
            Position { position: glam::Vec3::ZERO },
            NeuronScenarioId(id.to_string()),
            Health::new(100.0),
        ))
    }

    // ── parse_victory ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_all_neurons_dead() {
        let cond = parse_victory("all_neurons_dead:e1,e2,e3").unwrap();
        let VictoryCondition::AllNeuronsDead { neuron_ids } = cond;
        assert_eq!(neuron_ids, vec!["e1", "e2", "e3"]);
    }

    #[test]
    fn test_parse_empty_returns_none() {
        assert!(parse_victory("").is_none());
    }

    #[test]
    fn test_parse_unknown_type_returns_none() {
        assert!(parse_victory("neuron_captured:e1").is_none());
    }

    #[test]
    fn test_parse_empty_id_list_returns_none() {
        assert!(parse_victory("all_neurons_dead:").is_none());
    }

    // ── check_victory ──────────────────────────────────────────────────────────

    #[test]
    fn test_no_victory_while_neurons_alive() {
        let mut world = hecs::World::new();
        spawn_neuron_with_id(&mut world, "e1");
        spawn_neuron_with_id(&mut world, "e2");
        spawn_neuron_with_id(&mut world, "e3");

        let cond = parse_victory("all_neurons_dead:e1,e2,e3").unwrap();
        assert!(!check_victory(&world, &cond), "should not win while all neurons alive");
    }

    #[test]
    fn test_no_victory_while_some_alive() {
        let mut world = hecs::World::new();
        let e1 = spawn_neuron_with_id(&mut world, "e1");
        spawn_neuron_with_id(&mut world, "e2");
        spawn_neuron_with_id(&mut world, "e3");

        world.despawn(e1).unwrap();

        let cond = parse_victory("all_neurons_dead:e1,e2,e3").unwrap();
        assert!(!check_victory(&world, &cond), "should not win while e2/e3 still alive");
    }

    #[test]
    fn test_victory_when_all_dead() {
        let mut world = hecs::World::new();
        let e1 = spawn_neuron_with_id(&mut world, "e1");
        let e2 = spawn_neuron_with_id(&mut world, "e2");
        let e3 = spawn_neuron_with_id(&mut world, "e3");

        world.despawn(e1).unwrap();
        world.despawn(e2).unwrap();
        world.despawn(e3).unwrap();

        let cond = parse_victory("all_neurons_dead:e1,e2,e3").unwrap();
        assert!(check_victory(&world, &cond), "should win when all listed neurons are dead");
    }

    #[test]
    fn test_victory_ignores_unlisted_neurons() {
        let mut world = hecs::World::new();
        let e1 = spawn_neuron_with_id(&mut world, "e1");
        spawn_neuron_with_id(&mut world, "other_neuron");
        world.despawn(e1).unwrap();

        let cond = parse_victory("all_neurons_dead:e1").unwrap();
        assert!(check_victory(&world, &cond), "victory should ignore neurons not in the list");
    }

    #[test]
    fn test_sequential_kill_triggers_victory_only_at_end() {
        let mut world = hecs::World::new();
        let e1 = spawn_neuron_with_id(&mut world, "e1");
        let e2 = spawn_neuron_with_id(&mut world, "e2");
        let e3 = spawn_neuron_with_id(&mut world, "e3");
        let cond = parse_victory("all_neurons_dead:e1,e2,e3").unwrap();

        assert!(!check_victory(&world, &cond));
        world.despawn(e1).unwrap();
        assert!(!check_victory(&world, &cond));
        world.despawn(e2).unwrap();
        assert!(!check_victory(&world, &cond));
        world.despawn(e3).unwrap();
        assert!(check_victory(&world, &cond), "victory triggers only after all three are dead");
    }
}
