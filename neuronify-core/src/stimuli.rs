pub mod alternating_current;
pub mod current_source;
pub mod drawing;
pub mod gui;
pub mod poisson_generator;
pub mod receptive_field;
pub mod regular_spike_generator;
pub mod touch_sensor;
pub mod visual_input;

use std::ops::RangeInclusive;

use glam::Vec3;
use hecs::{Entity, EntityBuilder, World};
use neuronify_engine::units::{Hertz, Milliseconds, Nanoamperes};
use serde::{Deserialize, Serialize};

use crate::components::{Deletable, NeuronType, Position};
use crate::measurement::frame::Frame;
use crate::serialization::bounds::{elapsed, within};
use crate::simulation::engine::Engine;
use alternating_current::AlternatingCurrent;
use poisson_generator::PoissonGenerator;
use receptive_field::ReceptiveFieldCell;
use regular_spike_generator::RegularSpikeGenerator;
use touch_sensor::TouchSensor;
use visual_input::VisualInput;

pub const CURRENTS: RangeInclusive<Nanoamperes> = Nanoamperes(-10_000.0)..=Nanoamperes(10_000.0);
pub const SPIKE_RATES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(100_000.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stimulus {
    CurrentSource,
    AlternatingCurrentSource,
    TouchSensor,
    RegularSpikeGenerator,
    PoissonGenerator,
    VisualInput,
    ReceptiveField,
}

pub struct About {
    pub name: &'static str,
    pub description: &'static str,
    pub size: Option<f32>,
}

impl Stimulus {
    pub const ALL: [Stimulus; 7] = [
        Stimulus::CurrentSource,
        Stimulus::AlternatingCurrentSource,
        Stimulus::TouchSensor,
        Stimulus::RegularSpikeGenerator,
        Stimulus::PoissonGenerator,
        Stimulus::VisualInput,
        Stimulus::ReceptiveField,
    ];

    pub fn of(world: &World, entity: Entity) -> Option<Stimulus> {
        let entity = world.entity(entity).ok()?;
        Stimulus::ALL.into_iter().find(|stimulus| match stimulus {
            Stimulus::CurrentSource => {
                entity.has::<CurrentClamp>() && !entity.has::<AlternatingCurrent>()
            }
            Stimulus::AlternatingCurrentSource => entity.has::<AlternatingCurrent>(),
            Stimulus::TouchSensor => entity.has::<TouchSensor>(),
            Stimulus::RegularSpikeGenerator => entity.has::<RegularSpikeGenerator>(),
            Stimulus::PoissonGenerator => entity.has::<PoissonGenerator>(),
            Stimulus::VisualInput => entity.has::<VisualInput>(),
            Stimulus::ReceptiveField => entity.has::<ReceptiveFieldCell>(),
        })
    }

    pub fn about(self) -> &'static About {
        match self {
            Stimulus::CurrentSource => &current_source::ABOUT,
            Stimulus::AlternatingCurrentSource => &alternating_current::ABOUT,
            Stimulus::TouchSensor => &touch_sensor::ABOUT,
            Stimulus::RegularSpikeGenerator => &regular_spike_generator::ABOUT,
            Stimulus::PoissonGenerator => &poisson_generator::ABOUT,
            Stimulus::VisualInput => &visual_input::ABOUT,
            Stimulus::ReceptiveField => &receptive_field::ABOUT,
        }
    }

    pub fn frame(self, center: Vec3) -> Option<Frame> {
        Some(Frame::square(center, self.about().size?))
    }

    pub fn builder(self, position: Vec3) -> EntityBuilder {
        let mut stimulus = EntityBuilder::new();
        stimulus.add_bundle((Position { position }, Deletable {}));
        match self {
            Stimulus::CurrentSource => stimulus.add(CurrentClamp::default()),
            Stimulus::AlternatingCurrentSource => stimulus.add_bundle((
                CurrentClamp {
                    current_output: Nanoamperes::ZERO,
                },
                AlternatingCurrent::default(),
            )),
            Stimulus::TouchSensor => {
                stimulus.add_bundle((TouchSensor, GeneratorDynamics::default()))
            }
            Stimulus::RegularSpikeGenerator => stimulus.add_bundle((
                RegularSpikeGenerator::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
            )),
            Stimulus::PoissonGenerator => stimulus.add_bundle((
                PoissonGenerator::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
            )),
            Stimulus::VisualInput => stimulus.add(VisualInput::default()),
            Stimulus::ReceptiveField => stimulus.add_bundle((
                ReceptiveFieldCell::default(),
                GeneratorDynamics::default(),
                NeuronType::Excitatory,
            )),
        };
        stimulus
    }

    pub fn spawn(self, world: &mut World, position: Vec3) -> Entity {
        world.spawn(self.builder(position).build())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentClamp {
    pub current_output: Nanoamperes,
}

impl Default for CurrentClamp {
    fn default() -> Self {
        Self {
            current_output: Nanoamperes(2.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratorDynamics {
    pub time_since_fire: Milliseconds,
}

impl Default for GeneratorDynamics {
    fn default() -> Self {
        Self {
            time_since_fire: Milliseconds::INFINITY,
        }
    }
}

pub fn framed_size(world: &World, entity: Entity) -> Option<f32> {
    Stimulus::of(world, entity)?.about().size
}

pub fn edge_toward(center: Vec3, toward: Vec3, size: f32) -> Option<Vec3> {
    let offset = toward - center;
    let reach = offset.x.abs().max(offset.z.abs());
    (reach > size * 0.5).then(|| center + offset * (size * 0.5 / reach))
}

pub fn advance(world: &mut World, engine: &mut Engine, time: Milliseconds, duration: Milliseconds) {
    for stimulus in Stimulus::ALL {
        match stimulus {
            Stimulus::VisualInput => visual_input::advance(world, duration),
            Stimulus::ReceptiveField => receptive_field::respond(world, engine, time),
            Stimulus::CurrentSource
            | Stimulus::AlternatingCurrentSource
            | Stimulus::TouchSensor
            | Stimulus::RegularSpikeGenerator
            | Stimulus::PoissonGenerator => {}
        }
    }
}

pub fn bound(world: &mut World) {
    for clamp in world.query_mut::<&mut CurrentClamp>() {
        let default = CurrentClamp::default();
        within(&mut clamp.current_output, CURRENTS, default.current_output);
    }
    for dynamics in world.query_mut::<&mut GeneratorDynamics>() {
        dynamics.time_since_fire = elapsed(dynamics.time_since_fire);
    }
    for stimulus in Stimulus::ALL {
        match stimulus {
            Stimulus::AlternatingCurrentSource => alternating_current::bound(world),
            Stimulus::RegularSpikeGenerator => regular_spike_generator::bound(world),
            Stimulus::PoissonGenerator => poisson_generator::bound(world),
            Stimulus::VisualInput => visual_input::bound(world),
            Stimulus::ReceptiveField => receptive_field::bound(world),
            Stimulus::CurrentSource | Stimulus::TouchSensor => {}
        }
    }
}
