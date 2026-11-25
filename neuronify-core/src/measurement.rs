pub mod drawing;
pub mod firing_rate;
pub mod frame;
pub mod gui;
pub mod loudspeaker;
pub mod spike_detector;
pub mod visual_input;
pub mod voltmeter;

use std::collections::VecDeque;

use glam::Vec3;
use hecs::{Entity, EntityBuilder, Or, World};
use neuronify_engine::units::Milliseconds;
use serde::{Deserialize, Serialize};

use crate::components::{Compartment, Connection, Deletable, LeakyNeuron, MeterSize, Position};
use crate::constants::DT;
use crate::simulation::engine::ProbeSamples;
use crate::simulation::Spike;
use firing_rate::{RatePlot, RateSeries};
use frame::RowOrder;
use loudspeaker::Loudspeaker;
use spike_detector::SpikeDetector;
use voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meter {
    Voltmeter,
    SpikeDetector,
    RatePlot,
    Loudspeaker,
}

pub struct About {
    pub name: &'static str,
    pub description: &'static str,
    pub size: Option<MeterSize>,
    pub offset: Vec3,
}

impl Meter {
    pub const ALL: [Meter; 4] = [
        Meter::Voltmeter,
        Meter::SpikeDetector,
        Meter::RatePlot,
        Meter::Loudspeaker,
    ];

    pub fn of(world: &World, entity: Entity) -> Option<Meter> {
        Meter::ALL.into_iter().find(|meter| match meter {
            Meter::Voltmeter => world.satisfies::<&Voltmeter>(entity),
            Meter::SpikeDetector => world.satisfies::<&SpikeDetector>(entity),
            Meter::RatePlot => world.satisfies::<&RatePlot>(entity),
            Meter::Loudspeaker => world.satisfies::<&Loudspeaker>(entity),
        })
    }

    pub fn about(self) -> &'static About {
        match self {
            Meter::Voltmeter => &voltmeter::ABOUT,
            Meter::SpikeDetector => &spike_detector::ABOUT,
            Meter::RatePlot => &firing_rate::ABOUT,
            Meter::Loudspeaker => &loudspeaker::ABOUT,
        }
    }

    pub fn measures(self, world: &World, target: Entity) -> bool {
        match self {
            Meter::Voltmeter => world.satisfies::<Or<&LeakyNeuron, &Compartment>>(target),
            Meter::SpikeDetector | Meter::RatePlot | Meter::Loudspeaker => {
                world.satisfies::<&LeakyNeuron>(target)
            }
        }
    }

    pub fn spawn(self, world: &mut World, position: Vec3) -> Entity {
        let mut meter = EntityBuilder::new();
        meter.add_bundle((Position { position }, Deletable {}));
        if let Some(size) = self.about().size {
            meter.add(size);
        }
        match self {
            Meter::Voltmeter => meter.add_bundle((Voltmeter {}, VoltmeterRange::default())),
            Meter::SpikeDetector => {
                meter.add_bundle((SpikeDetector::default(), RowOrder::default()))
            }
            Meter::RatePlot => meter.add_bundle((RatePlot::default(), RateSeries::default())),
            Meter::Loudspeaker => meter.add(Loudspeaker::default()),
        };
        world.spawn(meter.build())
    }

    pub fn wire(self, world: &mut World, target: Entity, meter: Entity) -> Entity {
        let mut wire = EntityBuilder::new();
        wire.add_bundle((
            Connection {
                from: target,
                to: meter,
                strength: 1.0,
                directional: true,
            },
            Deletable {},
        ));
        match self {
            Meter::Voltmeter => {
                wire.add(VoltageSeries::new());
            }
            Meter::SpikeDetector | Meter::RatePlot => {
                wire.add(SpikeTrain::default());
            }
            Meter::Loudspeaker => {}
        }
        world.spawn(wire.build())
    }

    pub fn spawn_measuring(self, world: &mut World, target: Entity, position: Vec3) -> Entity {
        let meter = self.spawn(world, position);
        self.wire(world, target, meter);
        meter
    }
}

pub fn is_meter(world: &World, entity: Entity) -> bool {
    Meter::of(world, entity).is_some()
}

pub fn record(
    world: &mut World,
    spikes: &[Spike],
    probes: &[ProbeSamples],
    time: Milliseconds,
    duration: Milliseconds,
) {
    record_trains(world, spikes, time);
    for meter in Meter::ALL {
        match meter {
            Meter::Voltmeter => voltmeter::record_voltages(world, spikes, probes),
            Meter::SpikeDetector => spike_detector::advance(world, time),
            Meter::RatePlot => firing_rate::record_rates(world, time),
            Meter::Loudspeaker => loudspeaker::listen(world, spikes, time, duration),
        }
    }
}

pub fn bound(world: &mut World) {
    for meter in Meter::ALL {
        match meter {
            Meter::Voltmeter => voltmeter::bound(world),
            Meter::SpikeDetector => spike_detector::bound(world),
            Meter::RatePlot => firing_rate::bound(world),
            Meter::Loudspeaker => loudspeaker::bound(world),
        }
    }
    let trains: Vec<(Entity, Option<Milliseconds>)> = world
        .query::<(Entity, Option<&Connection>)>()
        .with::<&SpikeTrain>()
        .iter()
        .map(|(wire, connection)| {
            let range = connection.and_then(|connection| shown_range(world, connection.to));
            (wire, range)
        })
        .collect();
    for (wire, range) in trains {
        let Ok(mut train) = world.get::<&mut SpikeTrain>(wire) else {
            continue;
        };
        train.times.retain(|time| time.is_finite());
        match range {
            Some(range) => keep_recent(
                &mut train.times,
                |&time| time,
                range,
                (range / DT) as usize + 1,
            ),
            None => train.times.clear(),
        }
    }
}

pub fn shown_range(world: &World, meter: Entity) -> Option<Milliseconds> {
    match Meter::of(world, meter)? {
        Meter::SpikeDetector => world
            .get::<&SpikeDetector>(meter)
            .ok()
            .map(|detector| detector.time_range),
        Meter::RatePlot => world.get::<&RatePlot>(meter).ok().map(|plot| plot.window),
        Meter::Voltmeter | Meter::Loudspeaker => None,
    }
}

fn keep_recent<T>(
    samples: &mut VecDeque<T>,
    time: impl Fn(&T) -> Milliseconds,
    span: Milliseconds,
    most: usize,
) {
    let Some(newest) = samples.iter().map(&time).reduce(Milliseconds::max) else {
        return;
    };
    samples.retain(|sample| time(sample) >= newest - span);
    let surplus = samples.len().saturating_sub(most);
    samples.drain(..surplus);
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpikeTrain {
    pub times: VecDeque<Milliseconds>,
}

fn record_trains(world: &mut World, spikes: &[Spike], time: Milliseconds) {
    let wires: Vec<(Entity, Entity, Milliseconds)> = world
        .query::<(Entity, &Connection)>()
        .with::<&SpikeTrain>()
        .iter()
        .filter_map(|(wire, connection)| {
            Some((wire, connection.from, shown_range(world, connection.to)?))
        })
        .collect();
    for (wire, from, range) in wires {
        let Ok(mut train) = world.get::<&mut SpikeTrain>(wire) else {
            continue;
        };
        for spike in spikes.iter().filter(|spike| spike.entity == from) {
            train.times.push_back(spike.time);
        }
        while train
            .times
            .front()
            .is_some_and(|&spike| spike < time - range)
        {
            train.times.pop_front();
        }
    }
}
