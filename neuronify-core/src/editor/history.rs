use std::collections::{HashMap, HashSet};

use hecs::{Component, Entity, EntityBuilder, EntityRef, World};

use crate::components::*;
use crate::measurement::firing_rate::RatePlot;
use crate::measurement::frame::RowOrder;
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::SpikeDetector;
use crate::measurement::visual_input::{ReceptiveFieldCell, VisualInput};
use crate::measurement::voltmeter::VoltmeterRange;

enum State {
    Absent,
    Whole(EntityBuilder),
    Edited(EntityBuilder),
}

type Change = Vec<(Entity, State)>;

fn copy_edited(entity: EntityRef) -> EntityBuilder {
    let mut edited = EntityBuilder::new();
    copy::<Position>(entity, &mut edited);
    copy::<MeterSize>(entity, &mut edited);
    copy::<NoteSize>(entity, &mut edited);
    copy::<Connection>(entity, &mut edited);
    copy::<LeakyNeuron>(entity, &mut edited);
    copy::<CompartmentSoma>(entity, &mut edited);
    copy::<CurrentClamp>(entity, &mut edited);
    copy::<RegularSpikeGenerator>(entity, &mut edited);
    copy::<PoissonGenerator>(entity, &mut edited);
    copy::<Annotation>(entity, &mut edited);
    copy::<NeuronType>(entity, &mut edited);
    copy::<LeakCurrent>(entity, &mut edited);
    copy::<AdaptationCurrent>(entity, &mut edited);
    copy::<CurrentSynapse>(entity, &mut edited);
    copy::<Label>(entity, &mut edited);
    copy::<AlternatingCurrent>(entity, &mut edited);
    copy::<VisualInput>(entity, &mut edited);
    copy::<Cable>(entity, &mut edited);
    copy::<Membrane>(entity, &mut edited);
    copy::<Preparation>(entity, &mut edited);
    copy::<Temperature>(entity, &mut edited);
    copy::<ReceptiveFieldCell>(entity, &mut edited);
    copy::<VoltmeterRange>(entity, &mut edited);
    copy::<SpikeDetector>(entity, &mut edited);
    copy::<RowOrder>(entity, &mut edited);
    copy::<RatePlot>(entity, &mut edited);
    copy::<Loudspeaker>(entity, &mut edited);
    edited
}

fn copy<T: Component + Clone>(entity: EntityRef, edited: &mut EntityBuilder) {
    if let Some(component) = entity.get::<&T>() {
        edited.add((*component).clone());
    }
}

fn put_back(world: &mut World, entity: Entity, mut edited: EntityBuilder) {
    let Ok(now) = world.entity(entity) else {
        return;
    };
    keep_simulated(now, &mut edited);
    let _ = world.insert(entity, edited.build());
}

fn keep_simulated(now: EntityRef, edited: &mut EntityBuilder) {
    if let (Some(now), Some(set)) = (
        now.get::<&AdaptationCurrent>(),
        edited.get_mut::<&mut AdaptationCurrent>(),
    ) {
        *set = AdaptationCurrent {
            adaptation: set.adaptation,
            time_constant: set.time_constant,
            ..(*now).clone()
        };
    }
    if let (Some(now), Some(set)) = (
        now.get::<&CurrentSynapse>(),
        edited.get_mut::<&mut CurrentSynapse>(),
    ) {
        *set = CurrentSynapse {
            maximum_current: set.maximum_current,
            tau: set.tau,
            delay: set.delay,
            ..(*now).clone()
        };
    }
    if let (Some(now), Some(set)) = (
        now.get::<&SpikeDetector>(),
        edited.get_mut::<&mut SpikeDetector>(),
    ) {
        *set = SpikeDetector {
            time_range: set.time_range,
            ..(*now).clone()
        };
    }
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
        if !world.contains(entity) {
            return;
        }
        if let Some(State::Edited(edited)) = self.before.remove(&entity) {
            put_back(world, entity, edited);
        }
        let Some(components) = take(world, entity) else {
            return;
        };
        if self.alive.contains(&entity) {
            self.before.insert(entity, State::Whole(components));
        }
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
                State::Edited(edited) => {
                    if let Ok(now) = world.entity(entity) {
                        let undone = copy_edited(now);
                        put_back(world, entity, edited);
                        inverse.push((entity, State::Edited(undone)));
                    }
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
