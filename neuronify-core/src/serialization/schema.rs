/// Defines the component ids of one save file version and a `deserialize` that reads it.
/// With `save` first, it also defines the `SaveContext` that writes it.
/// Only the current version has one.
macro_rules! schema {
    (save, $($names:ident,)*) => {
        schema!($($names,)*);

        pub struct SaveContext;

        impl ::hecs::serialize::column::SerializeContext for SaveContext {
            fn component_count(&self, archetype: &::hecs::Archetype) -> usize {
                archetype.component_types()
                    .filter(|&t| $(t == ::std::any::TypeId::of::<$names>()) ||*)
                    .count()
            }

            fn serialize_component_ids<S: ::serde::ser::SerializeTuple>(
                &mut self,
                archetype: &::hecs::Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(::hecs::serialize::column::try_serialize_id::<$names, _, _>(
                    archetype,
                    &ComponentId::$names,
                    &mut out,
                )?;)*
                out.end()
            }

            fn serialize_components<S: ::serde::ser::SerializeTuple>(
                &mut self,
                archetype: &::hecs::Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(::hecs::serialize::column::try_serialize::<$names, _>(archetype, &mut out)?;)*
                out.end()
            }
        }
    };
    ($($names:ident,)*) => {
        #[derive(::serde::Deserialize, ::serde::Serialize, PartialEq)]
        enum ComponentId {
            $($names),*
        }

        struct LoadContext {
            components: Vec<ComponentId>,
        }

        impl ::hecs::serialize::column::DeserializeContext for LoadContext {
            fn deserialize_component_ids<'de, A>(
                &mut self,
                mut seq: A,
            ) -> Result<::hecs::ColumnBatchType, A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                self.components.clear(); // Discard data from the previous archetype
                let mut batch = ::hecs::ColumnBatchType::new();
                while let Some(id) = seq.next_element()? {
                    if self.components.contains(&id) {
                        return Err($crate::serialization::column::reject(
                            "a component appears twice",
                        ));
                    }
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
                batch: &mut ::hecs::ColumnBatchBuilder,
            ) -> Result<(), A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                // Decode component data in the order that the component IDs appeared
                for component in &self.components {
                    match *component {
                        $(
                            ComponentId::$names => {
                                ::hecs::serialize::column::deserialize_column::<$names, _>(
                                    entity_count,
                                    &mut seq,
                                    batch,
                                )?;
                            }
                        )*
                    }
                }
                Ok(())
            }
        }

        pub fn deserialize(bytes: &[u8]) -> Result<::hecs::World, String> {
            let mut context = LoadContext { components: vec![] };
            $crate::serialization::column::deserialize(&mut context, bytes)
                .map_err(|e| format!("Could not read save file: {e}"))
        }
    };
}
