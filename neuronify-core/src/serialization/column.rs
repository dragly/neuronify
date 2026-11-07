use std::cell::Cell;
use std::collections::HashSet;
use std::fmt;

use hecs::serialize::column::DeserializeContext;
use hecs::{ColumnBatch, ColumnBatchBuilder, ColumnBatchType, Entity, World};
use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};

pub const MAXIMUM_ENTITY_ID: u32 = 1 << 20;

thread_local! {
    static REJECTION: Cell<Option<&'static str>> = const { Cell::new(None) };
}

pub fn reject<E: de::Error>(reason: &'static str) -> E {
    REJECTION.set(Some(reason));
    E::custom(reason)
}

pub fn deserialize<C: DeserializeContext>(context: &mut C, bytes: &[u8]) -> Result<World, String> {
    REJECTION.set(None);
    let mut deserializer = postcard::Deserializer::from_bytes(bytes);
    let world = (&mut deserializer).deserialize_seq(WorldVisitor {
        context,
        room: bytes.len(),
        seen: HashSet::new(),
    });
    world.map_err(|error| {
        REJECTION
            .take()
            .map_or_else(|| error.to_string(), str::to_owned)
    })
}

struct WorldVisitor<'a, C> {
    context: &'a mut C,
    room: usize,
    seen: HashSet<u32>,
}

impl<'de, C: DeserializeContext> Visitor<'de> for WorldVisitor<'_, C> {
    type Value = World;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of archetypes")
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> Result<World, A::Error> {
        let mut world = World::new();
        let mut handles = Vec::new();
        while let Some(batch) = seq.next_element_seed(Archetype {
            context: &mut *self.context,
            room: &mut self.room,
            seen: &mut self.seen,
            handles: &mut handles,
        })? {
            world.spawn_column_batch_at(&handles, batch);
            handles.clear();
        }
        Ok(world)
    }
}

struct Archetype<'a, C> {
    context: &'a mut C,
    room: &'a mut usize,
    seen: &'a mut HashSet<u32>,
    handles: &'a mut Vec<Entity>,
}

impl<'de, C: DeserializeContext> DeserializeSeed<'de> for Archetype<'_, C> {
    type Value = ColumnBatch;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<ColumnBatch, D::Error> {
        deserializer.deserialize_tuple(4, self)
    }
}

impl<'de, C: DeserializeContext> Visitor<'de> for Archetype<'_, C> {
    type Value = ColumnBatch;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an entity count, a component count, component ids and components")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<ColumnBatch, A::Error> {
        let missing = |index| de::Error::invalid_length(index, &"an archetype of four parts");
        let entity_count: u32 = seq.next_element()?.ok_or_else(|| missing(0))?;
        let component_count: u32 = seq.next_element()?.ok_or_else(|| missing(1))?;
        *self.room = self
            .room
            .checked_sub(entity_count as usize)
            .ok_or_else(|| reject("it lists more entities than it holds"))?;
        let ty = seq
            .next_element_seed(ComponentIds {
                context: &mut *self.context,
                count: component_count,
            })?
            .ok_or_else(|| missing(2))?;
        let mut batch = ty.into_batch(entity_count);
        seq.next_element_seed(Components {
            context: self.context,
            entity_count,
            component_count,
            seen: self.seen,
            handles: self.handles,
            out: &mut batch,
        })?
        .ok_or_else(|| missing(3))?;
        batch
            .build()
            .map_err(|_| reject("an archetype is incomplete"))
    }
}

struct ComponentIds<'a, C> {
    context: &'a mut C,
    count: u32,
}

impl<'de, C: DeserializeContext> DeserializeSeed<'de> for ComponentIds<'_, C> {
    type Value = ColumnBatchType;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<ColumnBatchType, D::Error> {
        deserializer.deserialize_tuple(self.count as usize, self)
    }
}

impl<'de, C: DeserializeContext> Visitor<'de> for ComponentIds<'_, C> {
    type Value = ColumnBatchType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{} component ids", self.count)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<ColumnBatchType, A::Error> {
        self.context.deserialize_component_ids(seq)
    }
}

struct Components<'a, C> {
    context: &'a mut C,
    entity_count: u32,
    component_count: u32,
    seen: &'a mut HashSet<u32>,
    handles: &'a mut Vec<Entity>,
    out: &'a mut ColumnBatchBuilder,
}

impl<'de, C: DeserializeContext> DeserializeSeed<'de> for Components<'_, C> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        let parts = (self.component_count as usize)
            .checked_add(1)
            .ok_or_else(|| reject("an archetype has too many components"))?;
        deserializer.deserialize_tuple(parts, self)
    }
}

impl<'de, C: DeserializeContext> Visitor<'de> for Components<'_, C> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "the entities and components of {} entities",
            self.entity_count
        )
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        seq.next_element_seed(Entities {
            count: self.entity_count,
            seen: self.seen,
            out: self.handles,
        })?
        .ok_or_else(|| de::Error::invalid_length(0, &"the entities of an archetype"))?;
        self.context
            .deserialize_components(self.entity_count, seq, self.out)
    }
}

struct Entities<'a> {
    count: u32,
    seen: &'a mut HashSet<u32>,
    out: &'a mut Vec<Entity>,
}

impl<'de> DeserializeSeed<'de> for Entities<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_tuple(self.count as usize, self)
    }
}

impl<'de> Visitor<'de> for Entities<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{} entities", self.count)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        self.out.reserve(self.count as usize);
        while let Some(entity) = seq.next_element::<Entity>()? {
            if entity.id() >= MAXIMUM_ENTITY_ID {
                return Err(reject("an entity id is too large"));
            }
            if !self.seen.insert(entity.id()) {
                return Err(reject("an entity appears twice"));
            }
            self.out.push(entity);
        }
        if self.out.len() != self.count as usize {
            return Err(de::Error::invalid_length(self.out.len(), &self));
        }
        Ok(())
    }
}
