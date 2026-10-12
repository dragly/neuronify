use crate::components::*;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use hecs::{serialize::column::*, *};
use serde::{Deserialize, Serialize};
use std::any::TypeId;

macro_rules! component_id {
    ($($names:tt,)*) => {
        #[derive(Deserialize, Serialize)]
        enum ComponentId {
            $($names),*
        }
        pub struct SaveContext;
        impl SerializeContext for SaveContext {
            fn component_count(&self, archetype: &Archetype) -> usize {
                archetype.component_types()
                    .filter(|&t| $(t == TypeId::of::<$names>()) ||*)
                    .count()
            }

            fn serialize_component_ids<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(try_serialize_id::<$names, _, _>(archetype, &ComponentId::$names, &mut out)?;)*
                out.end()
            }

            fn serialize_components<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(try_serialize::<$names, _>(archetype, &mut out)?;)*
                out.end()
            }
        }

        pub struct LoadContext {
            components: Vec<ComponentId>,
        }

        impl Default for LoadContext {
            fn default() -> Self {
                Self::new()
            }
        }

        impl LoadContext {
            pub fn new() -> LoadContext {
                LoadContext { components: vec![] }
            }
        }

        impl DeserializeContext for LoadContext {
            fn deserialize_component_ids<'de, A>(
                &mut self,
                mut seq: A,
            ) -> Result<ColumnBatchType, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                self.components.clear(); // Discard data from the previous archetype
                let mut batch = ColumnBatchType::new();
                while let Some(id) = seq.next_element()? {
                    match id {
                        $(
                            ComponentId::$names => {
                                batch.add::<$names>();
                            }
                        )*
                    }
                    self.components.push(id);
                }
                Ok(batch)
            }

            fn deserialize_components<'de, A>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut ColumnBatchBuilder,
            ) -> Result<(), A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // Decode component data in the order that the component IDs appeared
                for component in &self.components {
                    match *component {
                        $(
                            ComponentId::$names => {
                                deserialize_column::<$names, _>(entity_count, &mut seq, batch)?;
                            }
                        )*
                    }
                }
                Ok(())
            }
        }

    }
}

component_id!(
    Position,
    LeakyNeuron,
    LeakyDynamics,
    LeakCurrent,
    AdaptationCurrent,
    CurrentClamp,
    CurrentSynapse,
    ImmediateFireSynapse,
    Inhibitory,
    TouchSensor,
    GeneratorDynamics,
    RegularSpikeGenerator,
    PoissonGenerator,
    VoltmeterSize,
    Annotation,
    StaticConnectionSource,
    Deletable,
    Selectable,
    Voltmeter,
    VoltageSeries,
    Connection,
    CompartmentCurrent,
    Compartment,
    SpatialDynamics,
    NeuronType,
);

/// Reads a legacy `.nfy` file (JSON) or a `.neuronify` save file (postcard).
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if !bytes.starts_with(b"{") {
        return load_save_file(bytes);
    }
    // A save file with 123 archetypes starts with `{` too,
    // so try it as a save file before reporting why it is not a legacy file
    load_legacy(bytes).or_else(|legacy_error| load_save_file(bytes).map_err(|_| legacy_error))
}

fn load_legacy(bytes: &[u8]) -> Result<World, String> {
    let contents = std::str::from_utf8(bytes).map_err(|e| format!("File is not UTF-8: {e}"))?;
    let simulation = crate::legacy::parse_legacy_nfy(contents)?;
    let mut world = World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
    Ok(world)
}

fn load_save_file(bytes: &[u8]) -> Result<World, String> {
    let mut context = LoadContext::new();
    let mut deserializer = postcard::Deserializer::from_bytes(bytes);
    deserialize(&mut context, &mut deserializer)
        .map_err(|e| format!("Could not read save file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::ser_flavors::Flavor;

    const LEAKY_NFY: &str = include_str!("../examples/leaky.nfy");

    fn save(world: &World) -> Vec<u8> {
        let mut serializer = postcard::Serializer {
            output: postcard::ser_flavors::StdVec::new(),
        };
        serialize(world, &mut SaveContext, &mut serializer).unwrap();
        serializer.output.finalize().unwrap()
    }

    #[test]
    fn loads_legacy_file() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap();
        assert!(!world.is_empty());
    }

    #[test]
    fn loads_save_file() {
        let original = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap();
        let loaded = load_from_bytes(&save(&original)).unwrap();
        assert_eq!(loaded.len(), original.len());
    }

    #[test]
    fn loads_save_file_starting_with_brace() {
        // A save file starts with its number of archetypes, and 123 is `{`
        let mut world = World::new();
        for mask in 1..=123u32 {
            let mut builder = EntityBuilder::new();
            if mask & 1 != 0 {
                builder.add(Position {
                    position: glam::Vec3::ZERO,
                });
            }
            if mask & 2 != 0 {
                builder.add(Inhibitory);
            }
            if mask & 4 != 0 {
                builder.add(TouchSensor);
            }
            if mask & 8 != 0 {
                builder.add(Deletable {});
            }
            if mask & 16 != 0 {
                builder.add(StaticConnectionSource {});
            }
            if mask & 32 != 0 {
                builder.add(Selectable { selected: false });
            }
            if mask & 64 != 0 {
                builder.add(Annotation {
                    text: String::new(),
                });
            }
            world.spawn(builder.build());
        }
        let bytes = save(&world);
        assert_eq!(bytes[0], b'{');

        let loaded = load_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.len(), world.len());
    }

    #[test]
    fn rejects_html() {
        assert!(load_from_bytes(b"<!DOCTYPE html><html></html>").is_err());
    }
}
