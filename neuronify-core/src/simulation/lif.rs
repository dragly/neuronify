use hecs::World;
use rand::Rng;

use crate::components::*;
use crate::measurement::voltmeter::{VoltageMeasurement, VoltageSeries, Voltmeter};

#[derive(Clone, Debug)]
pub struct SpikeRecord {
    pub entity_index: usize,
    pub time: f64,
}

pub fn lif_step(world: &mut World, dt: f64, time: f64) {
    for (_, (generator, dynamics)) in
        world.query_mut::<(&RegularSpikeGenerator, &mut GeneratorDynamics)>()
    {
        dynamics.time_since_fire += dt;
        if generator.frequency > 0.0 && dynamics.time_since_fire >= 1.0 / generator.frequency {
            dynamics.fired = true;
            dynamics.time_since_fire = 0.0;
        }
    }

    {
        let mut rng = rand::thread_rng();
        let poisson_entities: Vec<hecs::Entity> = world
            .query::<&PoissonGenerator>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in poisson_entities {
            let mut query = world
                .query_one::<(&PoissonGenerator, &mut GeneratorDynamics)>(entity)
                .unwrap();
            let (gen, dynamics) = query.get().unwrap();
            dynamics.time_since_fire += dt;
            if gen.rate > 0.0 && rng.gen::<f64>() < gen.rate * dt {
                dynamics.fired = true;
                dynamics.time_since_fire = 0.0;
            }
            drop(query);
        }
    }

    let neuron_entities: Vec<hecs::Entity> = world
        .query::<&LeakyNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in &neuron_entities {
        let mut query = world
            .query_one::<(&LeakyNeuron, &mut LeakyDynamics)>(*entity)
            .unwrap();
        let (neuron, dynamics) = query.get().unwrap();

        dynamics.time_since_fire += dt;
        dynamics.enabled = dynamics.time_since_fire >= dynamics.refractory_period;

        if dynamics.enabled && dynamics.voltage > neuron.threshold {
            dynamics.fired = true;
            dynamics.voltage = neuron.initial_potential;
            dynamics.time_since_fire = 0.0;
            dynamics.enabled = false;
        }
        drop(query);
    }

    for (_, (leak, neuron, dynamics)) in
        world.query_mut::<(&mut LeakCurrent, &LeakyNeuron, &LeakyDynamics)>()
    {
        if !dynamics.enabled {
            leak.current = 0.0;
            continue;
        }
        let v = dynamics.voltage;
        let em = neuron.resting_potential;
        leak.current = -(v - em) / leak.resistance;
    }

    for (_, (adapt, neuron, dynamics)) in
        world.query_mut::<(&mut AdaptationCurrent, &LeakyNeuron, &LeakyDynamics)>()
    {
        if !dynamics.enabled {
            adapt.current = 0.0;
            continue;
        }
        adapt.conductance -= adapt.conductance / adapt.time_constant * dt;
        if dynamics.fired {
            adapt.conductance += adapt.adaptation;
        }
        let v = dynamics.voltage;
        let em = neuron.resting_potential;
        adapt.current = -adapt.conductance * (v - em);
    }

    {
        let leak_currents: Vec<(hecs::Entity, f64)> = world
            .query::<(&LeakCurrent, &LeakyDynamics)>()
            .iter()
            .map(|(e, (leak, _))| (e, leak.current))
            .collect();

        for (entity, current) in leak_currents {
            if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
                dynamics.received_currents += current;
            }
        }
    }

    {
        let adapt_currents: Vec<(hecs::Entity, f64)> = world
            .query::<(&AdaptationCurrent, &LeakyDynamics)>()
            .iter()
            .map(|(e, (adapt, _))| (e, adapt.current))
            .collect();

        for (entity, current) in adapt_currents {
            if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
                dynamics.received_currents += current;
            }
        }
    }

    for (_, (neuron, dynamics)) in world.query_mut::<(&LeakyNeuron, &mut LeakyDynamics)>() {
        if !dynamics.enabled {
            dynamics.received_currents = 0.0;
            continue;
        }

        let total_current = dynamics.received_currents;
        let dv = total_current / neuron.capacitance * dt;
        dynamics.voltage += dv;

        if neuron.voltage_clamped {
            dynamics.voltage = dynamics
                .voltage
                .clamp(neuron.minimum_voltage, neuron.maximum_voltage);
        }

        dynamics.received_currents = 0.0;
    }

    for (_, synapse) in world.query_mut::<&mut CurrentSynapse>() {
        if synapse.alpha_function {
            synapse.current_output = synapse.maximum_current * synapse.linear * synapse.exponential;
        } else {
            synapse.current_output = synapse.maximum_current * synapse.exponential;
        }

        synapse.exponential -= synapse.exponential * dt / synapse.tau;

        if synapse.alpha_function {
            synapse.linear += dt / synapse.tau;
        }

        while !synapse.triggers.is_empty() && synapse.triggers[0] <= synapse.time {
            synapse.triggers.remove(0);
            if synapse.alpha_function {
                synapse.linear = 0.0;
                synapse.exponential = std::f64::consts::E;
            } else {
                synapse.exponential = 1.0;
            }
        }

        synapse.time += dt;
    }

    for (_, synapse) in world.query_mut::<&mut ImmediateFireSynapse>() {
        synapse.current_output = 0.0;
    }

    let edges_with_fire: Vec<(hecs::Entity, hecs::Entity, hecs::Entity, bool)> = world
        .query::<&Connection>()
        .iter()
        .map(|(edge_entity, conn)| {
            let source_fired = world
                .get::<&LeakyDynamics>(conn.from)
                .map(|d| d.fired)
                .unwrap_or(false)
                || world
                    .get::<&GeneratorDynamics>(conn.from)
                    .map(|d| d.fired)
                    .unwrap_or(false);
            (edge_entity, conn.from, conn.to, source_fired)
        })
        .collect();

    for (edge_entity, _source, _target, source_fired) in &edges_with_fire {
        if !source_fired {
            continue;
        }

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

        if let Ok(mut synapse) = world.get::<&mut ImmediateFireSynapse>(*edge_entity) {
            synapse.current_output = 1e6;
        }
    }

    let current_deliveries: Vec<(hecs::Entity, f64)> = edges_with_fire
        .iter()
        .filter_map(|(edge_entity, source, _target, _)| {
            let sign = if world.get::<&Inhibitory>(*source).is_ok() {
                -1.0
            } else {
                1.0
            };

            let mut total = 0.0;

            if let Ok(synapse) = world.get::<&CurrentSynapse>(*edge_entity) {
                if synapse.current_output != 0.0 {
                    total += sign * synapse.current_output;
                }
            }

            if let Ok(synapse) = world.get::<&ImmediateFireSynapse>(*edge_entity) {
                if synapse.current_output != 0.0 {
                    total += sign * synapse.current_output;
                }
            }

            if let Ok(clamp) = world.get::<&CurrentClamp>(*source) {
                if clamp.current_output != 0.0 {
                    total += sign * clamp.current_output;
                }
            }

            if total != 0.0 {
                Some((*_target, total))
            } else {
                None
            }
        })
        .collect();

    for (target, current) in current_deliveries {
        if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(target) {
            dynamics.received_currents += current;
        }
    }

    let voltmeter_updates: Vec<(hecs::Entity, f64, bool)> = world
        .query::<(&Voltmeter, &Connection)>()
        .iter()
        .filter_map(|(entity, (_, conn))| {
            if let Ok(dynamics) = world.get::<&LeakyDynamics>(conn.from) {
                return Some((
                    entity,
                    dynamics.voltage * 1000.0,
                    dynamics.time_since_fire == 0.0,
                ));
            }
            if let Ok(compartment) = world.get::<&Compartment>(conn.from) {
                return Some((entity, compartment.voltage, false));
            }
            None
        })
        .collect();

    for (entity, voltage, fired) in voltmeter_updates {
        if let Ok(mut series) = world.get::<&mut VoltageSeries>(entity) {
            series
                .measurements
                .push(VoltageMeasurement { voltage, time });
            if fired {
                series.spike_times.push(time);
            }
        }
    }

    for (_, dynamics) in world.query_mut::<&mut LeakyDynamics>() {
        dynamics.fired = false;
    }

    for (_, dynamics) in world.query_mut::<&mut GeneratorDynamics>() {
        dynamics.fired = false;
    }
}

pub fn run_headless(world: &mut World, steps: usize, dt: f64) -> Vec<SpikeRecord> {
    let mut spike_records = Vec::new();
    let mut time = 0.0;

    let neuron_entities: Vec<hecs::Entity> = world
        .query::<&LeakyNeuron>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for _step in 0..steps {
        lif_step(world, dt, time);

        for (idx, entity) in neuron_entities.iter().enumerate() {
            if let Ok(dynamics) = world.get::<&LeakyDynamics>(*entity) {
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
