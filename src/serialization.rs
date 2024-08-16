use crate::{
    Compartment, CompartmentCurrent, Connection, CurrentSource, Deletable, LeakCurrent,
    LearningSynapse, Neuron, NeuronDynamics, NeuronType, Position, Selectable, SpatialDynamics,
    StaticConnectionSource, StimulateCurrent, SynapseCurrent, Trigger, VoltageSeries, Voltmeter,
};
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
    Neuron,
    CurrentSource,
    StaticConnectionSource,
    NeuronDynamics,
    LeakCurrent,
    Deletable,
    Selectable,
    LearningSynapse,
    SynapseCurrent,
    Voltmeter,
    VoltageSeries,
    Connection,
    Trigger,
    CompartmentCurrent,
    Compartment,
    SpatialDynamics,
    StimulateCurrent,
    NeuronType,
);
