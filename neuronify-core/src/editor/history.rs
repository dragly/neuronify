use std::collections::{HashMap, HashSet};

use hecs::{Component, Entity, EntityBuilder, EntityRef, World};

use crate::components::*;

enum State {
    Absent,
    Whole(EntityBuilder),
    Edited(EntityBuilder),
}

type Change = Vec<(Entity, State)>;

struct Edited {
    copy: fn(EntityRef, &mut EntityBuilder),
    swap: fn(&World, Entity, &mut EntityBuilder),
}

const fn edited<T: Component + Clone>() -> Edited {
    Edited {
        copy: copy::<T>,
        swap: swap::<T>,
    }
}

const EDITED: [Edited; 4] = [
    edited::<Position>(),
    edited::<VoltmeterSize>(),
    edited::<NoteSize>(),
    edited::<Connection>(),
];

fn copy<T: Component + Clone>(entity: EntityRef, edited: &mut EntityBuilder) {
    if let Some(component) = entity.get::<&T>() {
        edited.add((*component).clone());
    }
}

fn swap<T: Component>(world: &World, entity: Entity, edited: &mut EntityBuilder) {
    if let (Some(recorded), Ok(mut current)) =
        (edited.get_mut::<&mut T>(), world.get::<&mut T>(entity))
    {
        std::mem::swap(recorded, &mut *current);
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
            let mut edited = EntityBuilder::new();
            for component in &EDITED {
                (component.copy)(current, &mut edited);
            }
            self.before.insert(entity, State::Edited(edited));
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
            components.add_bundle(edited.build());
        }
        self.before.insert(entity, State::Whole(components));
    }

    fn finish(mut self, world: &World) -> Change {
        for entity in world.iter().map(|entity| entity.entity()) {
            if !self.alive.contains(&entity) {
                self.before.insert(entity, State::Absent);
            }
        }
        self.before.into_iter().collect()
    }
}

#[derive(Default)]
pub struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
}

impl History {
    pub fn record(&mut self, recording: Recording, world: &World) {
        let change = recording.finish(world);
        if !change.is_empty() {
            self.undo.push(change);
            self.redo.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self, world: &mut World) {
        if let Some(change) = self.undo.pop() {
            let inverse = self.apply(world, change);
            self.redo.push(inverse);
        }
    }

    pub fn redo(&mut self, world: &mut World) {
        if let Some(change) = self.redo.pop() {
            let inverse = self.apply(world, change);
            self.undo.push(inverse);
        }
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
                    for component in &EDITED {
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
            for (_, connection) in world.query_mut::<&mut Connection>() {
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
