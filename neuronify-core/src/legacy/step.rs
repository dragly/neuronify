use hecs::World;

use crate::measurement::voltmeter::{VoltageMeasurement, VoltageSeries};
use crate::{Connection, Position, Voltmeter};

use crate::components::*;

/// Records of spike events for testing/analysis.
#[derive(Clone, Debug)]
pub struct SpikeRecord {
    pub entity_index: usize,
    pub time: f64,
}

/// Run the classic C++ simulation step. Reproduces the exact step order from
/// graphengine.cpp lines 160-216.
///
/// Step order:
/// 1. Step all nodes (checkFire, compute currents, integrate voltage)
/// 2. Step all edges (synapse dynamics)
/// 3. Communicate fires through edges
/// 4. Propagate currents through edges
/// 5. Finalize (reset fired flags)
pub fn lif_step(world: &mut World, dt: f64, time: f64) {
    // =========================================================================
    // PHASE 1: Step all nodes
    // =========================================================================

    // 1a. checkFire() - BEFORE integration (critical: C++ checks at start of step)
    // Also handle refractory period enable/disable
    let neuron_entities: Vec<hecs::Entity> = world
        .query::<&LIFNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in &neuron_entities {
        let mut query = world
            .query_one::<(&LIFNeuron, &mut LIFDynamics)>(*entity)
            .unwrap();
        let (neuron, dynamics) = query.get().unwrap();

        // Update refractory state
        dynamics.time_since_fire += dt;
        dynamics.enabled = dynamics.time_since_fire >= dynamics.refractory_period;

        if dynamics.enabled && dynamics.voltage > neuron.threshold {
            // Fire!
            dynamics.fired = true;
            dynamics.voltage = neuron.initial_potential;
            dynamics.time_since_fire = 0.0;
            dynamics.enabled = false;
        }
        drop(query);
    }

    // 1b. Compute leak current for each neuron with a leak component
    for (_, (leak, neuron, dynamics)) in
        world.query_mut::<(&mut LeakCurrent, &LIFNeuron, &LIFDynamics)>()
    {
        if !dynamics.enabled {
            leak.current = 0.0;
            continue;
        }
        let v = dynamics.voltage;
        let em = neuron.resting_potential;
        leak.current = -(v - em) / leak.resistance;
    }

    // 1c. Compute adaptation current
    for (_, (adapt, neuron, dynamics)) in
        world.query_mut::<(&mut AdaptationCurrent, &LIFNeuron, &LIFDynamics)>()
    {
        if !dynamics.enabled {
            adapt.current = 0.0;
            continue;
        }
        // Decay conductance
        adapt.conductance -= adapt.conductance / adapt.time_constant * dt;
        // If the neuron just fired, increase conductance
        if dynamics.fired {
            adapt.conductance += adapt.adaptation;
        }
        let v = dynamics.voltage;
        let em = neuron.resting_potential;
        adapt.current = -adapt.conductance * (v - em);
    }

    // 1d. Integrate voltage: dV = (leak + adaptation + receivedCurrents) / capacitance * dt
    // received_currents already holds currents from previous step's phase 4.
    // We'll add leak and adaptation currents to it before integration.

    // Collect child currents (leak, adaptation) and add to integration
    {
        let leak_currents: Vec<(hecs::Entity, f64)> = world
            .query::<(&LeakCurrent, &LIFDynamics)>()
            .iter()
            .map(|(e, (leak, _))| (e, leak.current))
            .collect();

        for (entity, current) in leak_currents {
            if let Ok(mut dynamics) = world.get::<&mut LIFDynamics>(entity) {
                dynamics.received_currents += current;
            }
        }
    }

    {
        let adapt_currents: Vec<(hecs::Entity, f64)> = world
            .query::<(&AdaptationCurrent, &LIFDynamics)>()
            .iter()
            .map(|(e, (adapt, _))| (e, adapt.current))
            .collect();

        for (entity, current) in adapt_currents {
            if let Ok(mut dynamics) = world.get::<&mut LIFDynamics>(entity) {
                dynamics.received_currents += current;
            }
        }
    }

    // Now do the actual voltage integration
    for (_, (neuron, dynamics)) in world.query_mut::<(&LIFNeuron, &mut LIFDynamics)>()
    {
        if !dynamics.enabled {
            dynamics.received_currents = 0.0;
            continue;
        }

        let total_current = dynamics.received_currents;
        let dv = total_current / neuron.capacitance * dt;
        dynamics.voltage += dv;

        // Clamp voltage
        if neuron.voltage_clamped {
            dynamics.voltage = dynamics
                .voltage
                .clamp(neuron.minimum_voltage, neuron.maximum_voltage);
        }

        dynamics.received_currents = 0.0;
    }

    // =========================================================================
    // PHASE 2: Step all edges (synapse dynamics)
    // =========================================================================

    for (_, synapse) in world.query_mut::<&mut CurrentSynapse>() {
        // Compute current output
        if synapse.alpha_function {
            synapse.current_output = synapse.maximum_current * synapse.linear * synapse.exponential;
        } else {
            synapse.current_output = synapse.maximum_current * synapse.exponential;
        }

        // Decay exponential
        synapse.exponential -= synapse.exponential * dt / synapse.tau;

        // Linear ramp for alpha function
        if synapse.alpha_function {
            synapse.linear += dt / synapse.tau;
        }

        // Check trigger queue
        while !synapse.triggers.is_empty() && synapse.triggers[0] <= synapse.time {
            synapse.triggers.remove(0);
            // Trigger the synapse
            if synapse.alpha_function {
                synapse.linear = 0.0;
                synapse.exponential = std::f64::consts::E;
            } else {
                synapse.exponential = 1.0;
            }
        }

        synapse.time += dt;
    }

    // ImmediateFireSynapse: reset current to 0 each step
    for (_, synapse) in world.query_mut::<&mut ImmediateFireSynapse>() {
        synapse.current_output = 0.0;
    }

    // =========================================================================
    // PHASE 3: Communicate fires through edges
    // =========================================================================

    // Collect fire state and edge info
    let edges_with_fire: Vec<(hecs::Entity, hecs::Entity, hecs::Entity, bool)> = world
        .query::<&Connection>()
        .iter()
        .filter_map(|(edge_entity, conn)| {
            let source_fired = world
                .get::<&LIFDynamics>(conn.from)
                .map(|d| d.fired)
                .unwrap_or(false);
            Some((edge_entity, conn.from, conn.to, source_fired))
        })
        .collect();

    for (edge_entity, _source, _target, source_fired) in &edges_with_fire {
        if !source_fired {
            continue;
        }

        // CurrentSynapse receives fire
        if let Ok(mut synapse) = world.get::<&mut CurrentSynapse>(*edge_entity) {
            if synapse.delay > 0.0 {
                let trigger_time = synapse.time + synapse.delay;
                synapse.triggers.push(trigger_time);
            } else if synapse.alpha_function {
                synapse.linear = 0.0;
                synapse.exponential = std::f64::consts::E;
            } else {
                synapse.exponential = 1.0;
            }
        }

        // ImmediateFireSynapse receives fire
        if let Ok(mut synapse) = world.get::<&mut ImmediateFireSynapse>(*edge_entity) {
            synapse.current_output = 1e6;
        }
    }

    // =========================================================================
    // PHASE 4: Propagate currents through edges
    // =========================================================================

    let current_deliveries: Vec<(hecs::Entity, f64)> = edges_with_fire
        .iter()
        .filter_map(|(edge_entity, source, target, _)| {
            // Determine sign from source inhibitory marker
            let sign = if world.get::<&Inhibitory>(*source).is_ok() {
                -1.0
            } else {
                1.0
            };

            let mut total = 0.0;

            // Current from synapse (CurrentSynapse)
            if let Ok(synapse) = world.get::<&CurrentSynapse>(*edge_entity) {
                if synapse.current_output != 0.0 {
                    total += sign * synapse.current_output;
                }
            }

            // Current from ImmediateFireSynapse
            if let Ok(synapse) = world.get::<&ImmediateFireSynapse>(*edge_entity) {
                if synapse.current_output != 0.0 {
                    total += sign * synapse.current_output;
                }
            }

            // Current from source node (CurrentClamp via Edge.qml)
            if let Ok(clamp) = world.get::<&CurrentClamp>(*source) {
                if clamp.current_output != 0.0 {
                    total += sign * clamp.current_output;
                }
            }

            if total != 0.0 {
                Some((*target, total))
            } else {
                None
            }
        })
        .collect();

    for (target, current) in current_deliveries {
        if let Ok(mut dynamics) = world.get::<&mut LIFDynamics>(target) {
            dynamics.received_currents += current;
        }
    }

    // =========================================================================
    // PHASE 5: Update voltmeters
    // =========================================================================

    let voltmeter_updates: Vec<(hecs::Entity, f64, bool)> = world
        .query::<(&Voltmeter, &Connection)>()
        .iter()
        .filter_map(|(entity, (_, conn))| {
            let dynamics = world.get::<&LIFDynamics>(conn.from).ok()?;
            Some((entity, dynamics.voltage, dynamics.time_since_fire == 0.0))
        })
        .collect();

    for (entity, voltage, fired) in voltmeter_updates {
        if let Ok(mut series) = world.get::<&mut VoltageSeries>(entity) {
            series.measurements.push(VoltageMeasurement {
                voltage: voltage * 1000.0, // Convert V to mV for display
                time,
            });
            if fired {
                series.spike_times.push(time);
            }
        }
    }

    // =========================================================================
    // PHASE 6: Finalize - reset fired flags
    // =========================================================================

    for (_, dynamics) in world.query_mut::<&mut LIFDynamics>() {
        dynamics.fired = false;
    }
}

/// Backwards-compatible alias for `lif_step`.
pub fn classic_step(world: &mut World, dt: f64, time: f64) {
    lif_step(world, dt, time);
}

/// Run a headless simulation for testing.
pub fn run_headless(world: &mut World, steps: usize, dt: f64) -> Vec<SpikeRecord> {
    let mut spike_records = Vec::new();
    let mut time = 0.0;

    // Build entity-to-index map for neurons
    let neuron_entities: Vec<hecs::Entity> = world
        .query::<&LIFNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for _step in 0..steps {
        lif_step(world, dt, time);

        for (idx, entity) in neuron_entities.iter().enumerate() {
            if let Ok(dynamics) = world.get::<&LIFDynamics>(*entity) {
                if dynamics.time_since_fire == 0.0 {
                    spike_records.push(SpikeRecord {
                        entity_index: idx,
                        time,
                    });
                }
            }
        }

        time += dt;
    }

    spike_records
}
