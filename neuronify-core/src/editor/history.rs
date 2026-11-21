use std::collections::{HashMap, HashSet};

use hecs::{Entity, EntityBuilder, EntityRef, World};
use neuronify_engine::units::{Microsiemens, Milliseconds, Nanoamperes};

use crate::components::*;
use crate::edited::{edited, edited_settings, Edited, Settings};
use crate::measurement::visual_input::{ReceptiveFieldCell, VisualInput};
use crate::measurement::Meter;

enum State {
    Absent,
    Whole(EntityBuilder),
    Edited(EntityBuilder),
}

type Change = Vec<(Entity, State)>;

struct Adaptation {
    adaptation: Microsiemens,
    time_constant: Milliseconds,
}

impl Settings<AdaptationCurrent> for Adaptation {
    fn of(current: &AdaptationCurrent) -> Self {
        Adaptation {
            adaptation: current.adaptation,
            time_constant: current.time_constant,
        }
    }

    fn swap(&mut self, current: &mut AdaptationCurrent) {
        std::mem::swap(&mut self.adaptation, &mut current.adaptation);
        std::mem::swap(&mut self.time_constant, &mut current.time_constant);
    }
}

struct Synapse {
    maximum_current: Nanoamperes,
    tau: Milliseconds,
    delay: Milliseconds,
}

impl Settings<CurrentSynapse> for Synapse {
    fn of(synapse: &CurrentSynapse) -> Self {
        Synapse {
            maximum_current: synapse.maximum_current,
            tau: synapse.tau,
            delay: synapse.delay,
        }
    }

    fn swap(&mut self, synapse: &mut CurrentSynapse) {
        std::mem::swap(&mut self.maximum_current, &mut synapse.maximum_current);
        std::mem::swap(&mut self.tau, &mut synapse.tau);
        std::mem::swap(&mut self.delay, &mut synapse.delay);
    }
}

const EDITED: [Edited; 22] = [
    edited::<Position>(),
    edited::<MeterSize>(),
    edited::<NoteSize>(),
    edited::<Connection>(),
    edited::<LeakyNeuron>(),
    edited::<CompartmentSoma>(),
    edited::<CurrentClamp>(),
    edited::<RegularSpikeGenerator>(),
    edited::<PoissonGenerator>(),
    edited::<Annotation>(),
    edited::<NeuronType>(),
    edited::<LeakCurrent>(),
    edited_settings::<AdaptationCurrent, Adaptation>(),
    edited_settings::<CurrentSynapse, Synapse>(),
    edited::<Label>(),
    edited::<AlternatingCurrent>(),
    edited::<VisualInput>(),
    edited::<Cable>(),
    edited::<Membrane>(),
    edited::<Preparation>(),
    edited::<Temperature>(),
    edited::<ReceptiveFieldCell>(),
];

fn edited_components() -> impl Iterator<Item = &'static Edited> {
    EDITED.iter().chain(
        Meter::ALL
            .into_iter()
            .flat_map(|meter| meter.about().edited),
    )
}

fn copy_edited(entity: EntityRef) -> EntityBuilder {
    let mut edited = EntityBuilder::new();
    for component in edited_components() {
        (component.copy)(entity, &mut edited);
    }
    edited
}

fn take(world: &mut World, entity: Entity) -> Option<EntityBuilder> {
    let mut components = EntityBuilder::new();
    components.add_bundle(world.take(entity).ok()?);
    Some(components)
}

pub struct Recording {
    alive: HashSet<Entity>,
    before: HashMap<Entity, State>,
}

impl Recording {
    pub fn start(world: &World) -> Self {
        Recording {
            alive: world.iter().map(|entity| entity.entity()).collect(),
            before: HashMap::new(),
        }
    }

    pub fn edit(&mut self, world: &World, entity: Entity) {
        if !self.alive.contains(&entity) || self.before.contains_key(&entity) {
            return;
        }
        if let Ok(current) = world.entity(entity) {
            self.before
                .insert(entity, State::Edited(copy_edited(current)));
        }
    }

    pub fn take(&mut self, world: &mut World, entity: Entity) {
        let Some(mut components) = take(world, entity) else {
            return;
        };
        if !self.alive.contains(&entity) {
            return;
        }
        if let Some(State::Edited(mut edited)) = self.before.remove(&entity) {
            for component in edited_components() {
                (component.restore)(&mut components, &mut edited);
            }
        }
        self.before.insert(entity, State::Whole(components));
    }

    pub fn is_new(&self, entity: Entity) -> bool {
        !self.alive.contains(&entity)
    }

    pub fn spawned<'a>(&'a self, world: &'a World) -> impl Iterator<Item = Entity> + 'a {
        world
            .iter()
            .map(|entity| entity.entity())
            .filter(|&entity| self.is_new(entity))
    }

    fn finish(mut self, world: &World) -> Change {
        let spawned: Vec<Entity> = self.spawned(world).collect();
        for entity in spawned {
            self.before.insert(entity, State::Absent);
        }
        self.before.into_iter().collect()
    }
}

pub struct Snapshot {
    pub entity: Entity,
    edited: Vec<(Entity, EntityBuilder)>,
}

impl Snapshot {
    pub fn of(world: &World, entity: Entity, parts: Vec<Entity>) -> Option<Self> {
        let edited = std::iter::once(entity)
            .chain(parts)
            .map(|part| Some((part, copy_edited(world.entity(part).ok()?))))
            .collect::<Option<_>>()?;
        Some(Snapshot { entity, edited })
    }
}

#[derive(Default)]
pub struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
}

impl History {
    pub fn record(&mut self, recording: Recording, world: &World) -> bool {
        self.push(recording.finish(world))
    }

    pub fn revert(&mut self, recording: Recording, world: &mut World) {
        let change = recording.finish(world);
        self.apply(world, change);
    }

    pub fn record_snapshot(&mut self, snapshot: Snapshot) {
        self.push(
            snapshot
                .edited
                .into_iter()
                .map(|(entity, edited)| (entity, State::Edited(edited)))
                .collect(),
        );
    }

    fn push(&mut self, change: Change) -> bool {
        if change.is_empty() {
            return false;
        }
        self.undo.push(change);
        self.redo.clear();
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self, world: &mut World) -> bool {
        let Some(change) = self.undo.pop() else {
            return false;
        };
        let inverse = self.apply(world, change);
        self.redo.push(inverse);
        true
    }

    pub fn redo(&mut self, world: &mut World) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        let inverse = self.apply(world, change);
        self.undo.push(inverse);
        true
    }

    fn apply(&mut self, world: &mut World, change: Change) -> Change {
        let mut inverse = Change::new();
        let mut respawning = Vec::new();
        for (entity, state) in change {
            match state {
                State::Absent => {
                    if let Some(components) = take(world, entity) {
                        inverse.push((entity, State::Whole(components)));
                    }
                }
                State::Edited(mut edited) => {
                    for component in edited_components() {
                        (component.swap)(world, entity, &mut edited);
                    }
                    inverse.push((entity, State::Edited(edited)));
                }
                State::Whole(components) => respawning.push((entity, components)),
            }
        }

        let occupied: HashSet<u32> = world.iter().map(|entity| entity.entity().id()).collect();
        let (reused, free): (Vec<_>, Vec<_>) = respawning
            .into_iter()
            .partition(|(entity, _)| occupied.contains(&entity.id()));
        for (entity, mut components) in free {
            world.spawn_at(entity, components.build());
            inverse.push((entity, State::Absent));
        }
        let mut respawned = HashMap::new();
        for (entity, mut components) in reused {
            let replacement = world.spawn(components.build());
            respawned.insert(entity, replacement);
            inverse.push((replacement, State::Absent));
        }
        if !respawned.is_empty() {
            for connection in world.query_mut::<&mut Connection>() {
                reconnect(connection, &respawned);
            }
            for change in self.undo.iter_mut().chain(&mut self.redo) {
                remap(change, &respawned);
            }
            remap(&mut inverse, &respawned);
        }
        inverse
    }
}

fn remap(change: &mut Change, respawned: &HashMap<Entity, Entity>) {
    for (entity, state) in change {
        if let Some(&replacement) = respawned.get(entity) {
            *entity = replacement;
        }
        if let State::Whole(components) | State::Edited(components) = state {
            if let Some(connection) = components.get_mut::<&mut Connection>() {
                reconnect(connection, respawned);
            }
        }
    }
}

fn reconnect(connection: &mut Connection, respawned: &HashMap<Entity, Entity>) {
    for end in [&mut connection.from, &mut connection.to] {
        if let Some(&replacement) = respawned.get(&*end) {
            *end = replacement;
        }
    }
}
