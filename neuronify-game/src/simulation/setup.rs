use glam::Vec3;
use hecs::Entity;

use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, Deletable, GeneratorDynamics, LeakCurrent,
    LeakyDynamics, LeakyNeuron, NeuronType, Position, RegularSpikeGenerator, Selectable,
    SpatialDynamics, StaticConnectionSource, VisualRadius, COUPLING_CAPACITANCE, NODE_RADIUS,
};

use crate::components::*;
use crate::constants::*;
use crate::spawning;

#[derive(Clone, Debug)]
pub struct PetriDish {
    pub center: Vec3,
    pub radius: f32,
}

// ── Layout constants ──────────────────────────────────────────────────────────

const ORIGIN_POS: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const VESSEL_POS: Vec3 = Vec3::new(-14.0, 0.0, 26.0);
const GLIAL_POS: Vec3 = Vec3::new(-28.0, 0.0, 12.0);
const NEURON_A_POS: Vec3 = Vec3::new(-20.0, 0.0, 0.0);
const NEURON_B_POS: Vec3 = Vec3::new(-38.0, 0.0, -14.0);
const NEURON_C_POS: Vec3 = Vec3::new(-38.0, 0.0, 14.0);

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generate evenly-spaced waypoint positions between `from` and `to` (exclusive).
/// Spacing is as close to `2 * NODE_RADIUS` (the spring equilibrium length) as
/// possible given the total distance.
fn equilibrium_waypoints(from: Vec3, to: Vec3) -> Vec<Vec3> {
    let total = from.distance(to);
    if total < NODE_RADIUS * 2.1 {
        return vec![];
    }
    // n waypoints divide the segment into n+1 gaps; we want each gap ≈ 2 units.
    let n = ((total / (NODE_RADIUS * 2.0)).round() as usize).saturating_sub(1);
    if n == 0 {
        return vec![];
    }
    let dir = (to - from).normalize_or_zero();
    let step = total / (n + 1) as f32;
    (1..=n).map(|i| from + dir * step * i as f32).collect()
}

fn spawn_axon_comp(world: &mut hecs::World, pos: Vec3, neuron_type: NeuronType) -> Entity {
    world.spawn((
        Position { position: pos },
        neuron_type,
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
        Deletable {},
        Selectable { selected: false },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
    ))
}

fn spawn_bridge_comp(world: &mut hecs::World, pos: Vec3, neuron_type: NeuronType) -> Entity {
    // No SpatialDynamics — bridge sits fixed just outside the target soma surface.
    world.spawn((
        Position { position: pos },
        neuron_type,
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
        Deletable {},
        Selectable { selected: false },
    ))
}

fn spawn_glial_process_comp(world: &mut hecs::World, pos: Vec3) -> Entity {
    world.spawn((
        Position { position: pos },
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
        StaticConnectionSource {},
        Deletable {},
        Selectable { selected: false },
        SpatialDynamics {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },
    ))
}

fn spawn_glial_bridge_comp(world: &mut hecs::World, pos: Vec3) -> Entity {
    world.spawn((
        Position { position: pos },
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
        StaticConnectionSource {},
        Deletable {},
        Selectable { selected: false },
    ))
}

fn link(world: &mut hecs::World, from: Entity, to: Entity, directional: bool) {
    world.spawn((
        Connection {
            from,
            to,
            strength: 1.0,
            directional,
        },
        Deletable {},
        CompartmentCurrent {
            capacitance: COUPLING_CAPACITANCE,
        },
    ));
}

/// Connect `from` to a neuron soma via a straight axon chain.
/// Compartments are auto-spaced at equilibrium distance; a fixed bridge
/// compartment is placed just outside the target surface.
fn connect_axon(
    world: &mut hecs::World,
    from: Entity,
    from_pos: Vec3,
    target: Entity,
    target_pos: Vec3,
    neuron_type: NeuronType,
) {
    let bridge_dist = NODE_RADIUS + NODE_RADIUS * 0.5; // 1.5 units from soma
    let dir_out = (from_pos - target_pos).normalize_or_zero(); // points from target toward source
    let bridge_pos = target_pos + dir_out * bridge_dist;

    let waypoints = equilibrium_waypoints(from_pos, bridge_pos);

    let mut prev = from;
    for wp in waypoints {
        let comp = spawn_axon_comp(world, wp, neuron_type.clone());
        // Intermediate links use directional: false, matching the manual axon tool.
        link(world, prev, comp, false);
        prev = comp;
    }

    let bridge = spawn_bridge_comp(world, bridge_pos, neuron_type);
    link(world, prev, bridge, false);
    // Only the bridge → target link is directional (shows arrowhead at the synapse).
    link(world, bridge, target, true);
}

/// Connect a glial cell to a blood vessel via glial process compartments.
fn connect_glial_to_vessel(
    world: &mut hecs::World,
    from: Entity,
    from_pos: Vec3,
    vessel: Entity,
    vessel_pos: Vec3,
) {
    let bridge_dist = BLOOD_VESSEL_VISUAL_RADIUS + NODE_RADIUS * 0.5;
    let dir_out = (from_pos - vessel_pos).normalize_or_zero();
    let bridge_pos = vessel_pos + dir_out * bridge_dist;

    let waypoints = equilibrium_waypoints(from_pos, bridge_pos);

    let mut prev = from;
    for wp in waypoints {
        let comp = spawn_glial_process_comp(world, wp);
        link(world, prev, comp, false);
        prev = comp;
    }

    let bridge = spawn_glial_bridge_comp(world, bridge_pos);
    link(world, prev, bridge, false);
    link(world, bridge, vessel, false);
}

/// Connect a glial cell to a neuron soma via glial process compartments.
/// The bridge near the neuron carries the GlialProcess marker so that
/// lactate-distribution BFS can follow this path to reach the neuron.
fn connect_glial_to_neuron(
    world: &mut hecs::World,
    from: Entity,
    from_pos: Vec3,
    target: Entity,
    target_pos: Vec3,
) {
    let bridge_dist = NODE_RADIUS + NODE_RADIUS * 0.5;
    let dir_out = (from_pos - target_pos).normalize_or_zero();
    let bridge_pos = target_pos + dir_out * bridge_dist;

    let waypoints = equilibrium_waypoints(from_pos, bridge_pos);

    let mut prev = from;
    for wp in waypoints {
        let comp = spawn_glial_process_comp(world, wp);
        link(world, prev, comp, false);
        prev = comp;
    }

    // The bridge near the neuron is a GlialProcess compartment so the BFS
    // in spawn_lactate_packets can traverse from the glial cell to this neuron.
    let bridge = spawn_glial_bridge_comp(world, bridge_pos);
    link(world, prev, bridge, false);
    link(world, bridge, target, false);
}

// ── setup_game ────────────────────────────────────────────────────────────────

pub fn setup_game(world: &mut hecs::World, _dish: &PetriDish) {
    world.clear();

    // Origin neuron — immovable pacemaker at center
    let origin = world.spawn((
        Position {
            position: ORIGIN_POS,
        },
        LeakyNeuron::default(),
        LeakyDynamics::default(),
        LeakCurrent::default(),
        NeuronType::Excitatory,
        RegularSpikeGenerator { frequency: 20.0 },
        GeneratorDynamics::default(),
        Anchored,
        MetabolicState {
            energy: MAX_NEURON_ENERGY,
            max_energy: MAX_NEURON_ENERGY,
        },
        OriginNeuron {
            player: PlayerId::Player1,
        },
        Ownership {
            player: PlayerId::Player1,
        },
        VisualRadius {
            radius: NODE_RADIUS * 1.5,
        },
    ));

    // Blood vessel — resource supply point
    let vessel = world.spawn((
        Position {
            position: VESSEL_POS,
        },
        BloodVessel::default(),
        VesselAnchor,
        Anchored,
        Selectable { selected: false },
        VisualRadius {
            radius: BLOOD_VESSEL_VISUAL_RADIUS,
        },
    ));

    // Glial cell — metabolic support, connected to vessel and to all three neurons
    let glial = spawning::spawn_glial(world, GLIAL_POS, PlayerId::Player1, 0);

    // Spawn neurons without initial dendrites so the pre-built axon bridges don't
    // collide with radial dendrite compartments. (A dendrite in the direction of the
    // target would land within 0.1 units of the first axon compartment, causing
    // enormous repulsion forces that scatter the chain visually.)
    // The player can still draw new axons from these somas manually.

    // Neuron A — excitatory relay from origin
    let neuron_a = spawning::spawn_neuron_with_dendrites(world, NEURON_A_POS, NeuronType::Excitatory, PlayerId::Player1, 0);

    // Neuron B — excitatory output (receives input from A, inhibition from C)
    let neuron_b = spawning::spawn_neuron_with_dendrites(world, NEURON_B_POS, NeuronType::Excitatory, PlayerId::Player1, 0);

    // Neuron C — inhibitory interneuron (excited by A, inhibits B: lateral inhibition)
    let neuron_c = spawning::spawn_neuron_with_dendrites(world, NEURON_C_POS, NeuronType::Inhibitory, PlayerId::Player1, 0);

    // ── Axon connections ──────────────────────────────────────────────────────

    // Origin → Neuron A
    connect_axon(world, origin, ORIGIN_POS, neuron_a, NEURON_A_POS, NeuronType::Excitatory);

    // Neuron A → Neuron B
    connect_axon(world, neuron_a, NEURON_A_POS, neuron_b, NEURON_B_POS, NeuronType::Excitatory);

    // Neuron A → Neuron C
    connect_axon(world, neuron_a, NEURON_A_POS, neuron_c, NEURON_C_POS, NeuronType::Excitatory);

    // Neuron C → Neuron B  (lateral inhibition — inhibitory bridge)
    connect_axon(world, neuron_c, NEURON_C_POS, neuron_b, NEURON_B_POS, NeuronType::Inhibitory);

    // ── Glial process connections ─────────────────────────────────────────────

    // Glial → blood vessel (glucose/blocks intake)
    connect_glial_to_vessel(world, glial, GLIAL_POS, vessel, VESSEL_POS);

    // Glial → Neuron A, B, C (lactate delivery)
    connect_glial_to_neuron(world, glial, GLIAL_POS, neuron_a, NEURON_A_POS);
    connect_glial_to_neuron(world, glial, GLIAL_POS, neuron_b, NEURON_B_POS);
    connect_glial_to_neuron(world, glial, GLIAL_POS, neuron_c, NEURON_C_POS);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_core::{
        constants::{FHN_CDT, LIF_DT},
        fhn_step, lif_step, GeneratorDynamics, LeakyDynamics,
    };
    use std::collections::HashSet;

    fn run_game_loop(world: &mut hecs::World, seconds: f64) -> HashSet<hecs::Entity> {
        let iterations = 4u32;
        let fire_window = iterations as f64 * LIF_DT;
        let total_frames = (seconds / (iterations as f64 * LIF_DT)) as u32;
        let mut ever_fired: HashSet<hecs::Entity> = HashSet::new();
        let mut time = 0.0f64;

        for _ in 0..total_frames {
            for _ in 0..iterations {
                lif_step(world, LIF_DT, time);
                time += LIF_DT;
            }

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

            ever_fired.extend(recently_fired.iter());

            for _ in 0..iterations {
                fhn_step(world, FHN_CDT, &recently_fired);
            }
        }

        ever_fired
    }

    #[test]
    fn test_neuron_a_fires_from_origin() {
        let dish = PetriDish {
            center: Vec3::ZERO,
            radius: 120.0,
        };
        let mut world = hecs::World::new();
        setup_game(&mut world, &dish);

        let neuron_a = world
            .query::<(&LeakyDynamics, &Position)>()
            .iter()
            .find(|(e, (_, p))| {
                world.get::<&OriginNeuron>(*e).is_err()
                    && (p.position - NEURON_A_POS).length() < 0.5
            })
            .map(|(e, _)| e)
            .expect("Neuron A not found");

        let fired = run_game_loop(&mut world, 1.5);

        assert!(
            fired.contains(&neuron_a),
            "Neuron A should fire within 1.5 s: origin → axon chain → A"
        );
    }

    #[test]
    fn test_neuron_b_fires_from_a() {
        let dish = PetriDish {
            center: Vec3::ZERO,
            radius: 120.0,
        };
        let mut world = hecs::World::new();
        setup_game(&mut world, &dish);

        let neuron_b = world
            .query::<(&LeakyDynamics, &Position)>()
            .iter()
            .find(|(_, (_, p))| (p.position - NEURON_B_POS).length() < 0.5)
            .map(|(e, _)| e)
            .expect("Neuron B not found");

        let fired = run_game_loop(&mut world, 2.0);

        assert!(
            fired.contains(&neuron_b),
            "Neuron B should fire within 2 s: origin → A → B"
        );
    }
}
