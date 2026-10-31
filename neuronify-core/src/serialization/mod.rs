//! Save files (`.neuronify`) and scenarios.
//!
//! A save file starts with `MAGIC` and its version as a little-endian `u32`,
//! followed by the simulation time as a little-endian `f64` (from version 5),
//! the simulation speed as a little-endian `f64` (from version 6)
//! and the world in the hecs column format.
//!
//! Each version has a module with the components it can hold.
//! postcard writes a component id as its position in that list,
//! so a list must not change once files have been saved with it.
//! To change the list, or the fields of a component, add a version:
//! copy the newest module to `v{N+1}`, make the change there,
//! write `upgrade_from_v{N}` to turn a version N world into a version N+1 world,
//! and save a fixture with the `write_fixture` test.
//! If the fields of a component change,
//! the older modules need a copy of the old struct for the upgrade to convert.

use hecs::{serialize::column::serialize, World};
use postcard::ser_flavors::Flavor;

use crate::constants::DEFAULT_SPEED;

#[macro_use]
mod schema;
mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;

/// The version that `save` writes.
pub const VERSION: u32 = 9;

const MAGIC: &[u8] = b"neuronify";

pub struct SavedWorld {
    pub world: World,
    pub time: f64,
    pub speed: f64,
}

pub fn save(world: &World, time: f64, speed: f64) -> Result<Vec<u8>, String> {
    let mut serializer = postcard::Serializer {
        output: postcard::ser_flavors::StdVec::new(),
    };
    serialize(world, &mut v9::SaveContext, &mut serializer)
        .map_err(|e| format!("Could not save: {e}"))?;
    let body = serializer
        .output
        .finalize()
        .map_err(|e| format!("Could not save: {e}"))?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&time.to_le_bytes());
    bytes.extend_from_slice(&speed.to_le_bytes());
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

/// Reads a save file of any version and upgrades it to the current one.
pub fn load(bytes: &[u8]) -> Result<SavedWorld, String> {
    let (version, body) = bytes
        .strip_prefix(MAGIC)
        .ok_or("This is not a Neuronify save file")?
        .split_first_chunk()
        .ok_or("The save file ends before its version")?;
    let version = u32::from_le_bytes(*version);
    let (mut world, time, speed) = match version {
        1 => (v1::deserialize(body)?, 0.0, DEFAULT_SPEED),
        2 => (v2::deserialize(body)?, 0.0, DEFAULT_SPEED),
        3 => (v3::deserialize(body)?, 0.0, DEFAULT_SPEED),
        4 => (v4::deserialize(body)?, 0.0, DEFAULT_SPEED),
        5 => {
            let (time, body) = split_f64(body, "simulation time")?;
            (v5::deserialize(body)?, time, DEFAULT_SPEED)
        }
        6 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v6::deserialize(body)?, time, speed)
        }
        7 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v7::deserialize(body)?, time, speed)
        }
        8 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v8::deserialize(body)?, time, speed)
        }
        9 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v9::deserialize(body)?, time, speed)
        }
        _ => {
            return Err(format!(
                "The save file has version {version}, \
                 but this Neuronify reads versions 1 to {VERSION}"
            ))
        }
    };
    for from in version..VERSION {
        match from {
            1 => v2::upgrade_from_v1(&mut world),
            2 => v3::upgrade_from_v2(&mut world),
            3 => v4::upgrade_from_v3(&mut world),
            4 => v5::upgrade_from_v4(&mut world),
            5 => v6::upgrade_from_v5(&mut world),
            6 => v7::upgrade_from_v6(&mut world),
            7 => v8::upgrade_from_v7(&mut world),
            8 => v9::upgrade_from_v8(&mut world),
            _ => unreachable!("no upgrade from version {from}"),
        }
    }
    Ok(SavedWorld { world, time, speed })
}

fn split_f64<'a>(bytes: &'a [u8], name: &str) -> Result<(f64, &'a [u8]), String> {
    let (value, rest) = bytes
        .split_first_chunk()
        .ok_or_else(|| format!("The save file ends before its {name}"))?;
    Ok((f64::from_le_bytes(*value), rest))
}

/// Reads a legacy `.nfy` file (JSON) or a `.neuronify` save file.
pub fn load_from_bytes(bytes: &[u8]) -> Result<SavedWorld, String> {
    if bytes.starts_with(MAGIC) {
        load(bytes)
    } else {
        load_legacy(bytes)
    }
}

fn load_legacy(bytes: &[u8]) -> Result<SavedWorld, String> {
    let contents = std::str::from_utf8(bytes).map_err(|e| format!("File is not UTF-8: {e}"))?;
    let simulation = crate::legacy::parse_legacy_nfy(contents)?;
    let mut world = World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
    Ok(SavedWorld {
        world,
        time: 0.0,
        speed: DEFAULT_SPEED,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::elements::{spawn_cable, spawn_compartment, Element};
    use crate::measurement::voltmeter::{measured, VoltageSeries, Voltmeter, VoltmeterRange};
    use std::any::TypeId;
    use std::collections::BTreeMap;

    const LEAKY_NFY: &str = include_str!("../../examples/leaky.nfy");
    const FIXTURE_TIME: f64 = 2.5;
    const FIXTURE_SPEED: f64 = 0.5;

    #[test]
    fn loads_legacy_file() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        assert!(!world.is_empty());
    }

    #[test]
    fn loads_save_file() {
        let original = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let bytes = save(&original, 0.0, DEFAULT_SPEED).unwrap();
        assert!(bytes.starts_with(MAGIC));

        let loaded = load_from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded.len(), original.len());
    }

    #[test]
    fn keeps_the_simulation_time() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let loaded = load(&save(&world, 12.5, DEFAULT_SPEED).unwrap()).unwrap();
        assert_eq!(loaded.time, 12.5);
    }

    #[test]
    fn keeps_the_simulation_speed() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let loaded = load(&save(&world, 0.0, 0.5).unwrap()).unwrap();
        assert_eq!(loaded.speed, 0.5);
    }

    #[test]
    fn rejects_html() {
        assert!(load_from_bytes(b"<!DOCTYPE html><html></html>").is_err());
    }

    #[test]
    fn rejects_newer_versions() {
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(VERSION + 1).to_le_bytes());
        assert!(load(&bytes).is_err_and(|e| e.contains("reads versions 1 to")));
    }

    #[test]
    fn rejects_truncated_header() {
        assert!(load(b"neuronify\x01").is_err());
    }

    #[test]
    fn rejects_truncated_time() {
        let truncated = b"neuronify\x05\0\0\0\0\0\0";
        assert!(load(truncated).is_err_and(|e| e.contains("simulation time")));
    }

    #[test]
    fn rejects_truncated_speed() {
        let truncated = b"neuronify\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert!(load(truncated).is_err_and(|e| e.contains("simulation speed")));
    }

    #[test]
    fn loads_version_1() {
        // Version 1 turned the link to the next tutorial into an empty note
        assert_fixture(
            &load(include_bytes!("fixtures/v1.neuronify")).unwrap(),
            30,
            0,
        );
    }

    #[test]
    fn loads_version_2() {
        assert_fixture(
            &load(include_bytes!("fixtures/v2.neuronify")).unwrap(),
            29,
            0,
        );
    }

    #[test]
    fn loads_version_3() {
        assert_fixture(
            &load(include_bytes!("fixtures/v3.neuronify")).unwrap(),
            29,
            0,
        );
    }

    #[test]
    fn loads_version_4() {
        assert_fixture(
            &load(include_bytes!("fixtures/v4.neuronify")).unwrap(),
            29,
            1,
        );
    }

    #[test]
    fn loads_version_5() {
        let saved = load(include_bytes!("fixtures/v5.neuronify")).unwrap();
        assert_fixture(&saved, 29, 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, DEFAULT_SPEED);
    }

    #[test]
    fn loads_version_6() {
        let saved = load(include_bytes!("fixtures/v6.neuronify")).unwrap();
        assert_fixture(&saved, 29, 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_7() {
        let saved = load(include_bytes!("fixtures/v7.neuronify")).unwrap();
        assert_fixture(&saved, 30, 1);
        assert_eq!(count::<Link>(&saved.world), 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_8() {
        let saved = load(include_bytes!("fixtures/v8.neuronify")).unwrap();
        assert_fixture(&saved, 30, 1);
        assert_eq!(count::<Link>(&saved.world), 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn keeps_the_neuron_display() {
        let mut world = World::new();
        let display = NeuronDisplay {
            voltage: false,
            firing_rate: true,
        };
        display.set_in(&mut world);
        let loaded = load(&save(&world, 0.0, DEFAULT_SPEED).unwrap())
            .unwrap()
            .world;
        assert_eq!(NeuronDisplay::of(&loaded), display);
    }

    #[test]
    fn shows_voltage_but_not_firing_rates_in_files_without_a_neuron_display() {
        let world = load(include_bytes!("fixtures/v7.neuronify")).unwrap().world;
        let display = NeuronDisplay::of(&world);
        assert!(display.voltage);
        assert!(!display.firing_rate);
    }

    #[test]
    fn loads_version_9() {
        let saved = load(include_bytes!("fixtures/v9.neuronify")).unwrap();
        assert_eq!(composition(&saved.world), composition(&fixture_world()));
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    fn composition(world: &World) -> BTreeMap<Vec<TypeId>, u32> {
        world
            .archetypes()
            .filter(|archetype| !archetype.is_empty())
            .map(|archetype| {
                let mut types: Vec<TypeId> = archetype.component_types().collect();
                types.sort();
                (types, archetype.len())
            })
            .fold(BTreeMap::new(), |mut composition, (types, len)| {
                *composition.entry(types).or_default() += len;
                composition
            })
    }

    #[test]
    fn keeps_dendrites() {
        let mut world = World::new();
        let soma = Element::ExcitatoryNeuron.spawn(&mut world, glam::Vec3::ZERO);
        crate::dendrites::spawn_dendrites(&mut world, soma);
        let dendrites = count::<Dendrite>(&world);
        assert!(dendrites > 0);

        let loaded = load(&save(&world, 0.0, DEFAULT_SPEED).unwrap())
            .unwrap()
            .world;
        assert_eq!(count::<Dendrite>(&loaded), dendrites);
    }

    /// Writes `fixture_world` as `fixtures/v{VERSION}.neuronify`.
    /// Run it when adding a version, so that version keeps loading:
    /// `cargo test -p neuronify-core write_fixture -- --ignored`
    #[test]
    #[ignore]
    fn write_fixture() {
        let path = format!(
            "{}/src/serialization/fixtures/v{VERSION}.neuronify",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = save(&fixture_world(), FIXTURE_TIME, FIXTURE_SPEED).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    /// Bundled examples, plus the components that none of them have,
    /// spawned the way the editor tools spawn them.
    fn fixture_world() -> World {
        let examples = [
            include_str!("../../examples/adaptation.nfy"),
            include_str!("../../examples/burst.nfy"),
            include_str!("../../examples/disinhibition.nfy"),
            include_str!("../../examples/frPlot.nfy"),
            include_str!("../../examples/generators.nfy"),
            include_str!("../../examples/if_response.nfy"),
            include_str!("../../examples/inhibitory.nfy"),
            include_str!("../../examples/input_summation.nfy"),
            include_str!("../../examples/lateral_inhibition.nfy"),
            include_str!("../../examples/leaky.nfy"),
            include_str!("../../examples/tutorial_1_intro.nfy"),
            include_str!("../../examples/two_neuron_oscillator.nfy"),
            include_str!("../../examples/visualInput.nfy"),
        ];
        let mut world = World::new();
        for example in examples {
            let simulation = crate::legacy::parse_legacy_nfy(example).unwrap();
            crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
        }
        let position = |x: f32| glam::Vec3::new(x, 0.0, 30.0);
        Element::RegularSpikeGenerator.spawn(&mut world, position(0.0));
        Element::PoissonGenerator.spawn(&mut world, position(3.0));
        let neuron = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
        let compartment = spawn_compartment(&mut world, position(6.0), NeuronType::Excitatory);
        spawn_cable(&mut world, neuron, compartment, false);
        world
    }

    fn count<T: hecs::Component>(world: &World) -> usize {
        world.query::<&T>().iter().count()
    }

    /// Checks a fixture after it has been upgraded to the current version.
    fn assert_fixture(saved: &SavedWorld, notes: usize, burst_neurons: usize) {
        let world = &saved.world;
        let neurons = 44 + burst_neurons;
        assert_eq!(world.len(), 204);
        assert_eq!(count::<Position>(world), 125);
        assert_eq!(count::<LeakyNeuron>(world), neurons);
        assert_eq!(count::<LeakyDynamics>(world), neurons);
        assert_eq!(count::<LeakCurrent>(world), neurons);
        assert_eq!(count::<AdaptationCurrent>(world), 1);
        assert_eq!(count::<BurstCurrent>(world), burst_neurons);
        assert_eq!(count::<CurrentClamp>(world), 10);
        assert_eq!(count::<CurrentSynapse>(world), 45);
        assert_eq!(count::<ImmediateFireSynapse>(world), 14);
        assert_eq!(count::<v1::Inhibitory>(world), 0);
        assert_eq!(
            world
                .query::<&NeuronType>()
                .iter()
                .filter(|(_, neuron_type)| **neuron_type == NeuronType::Inhibitory)
                .count(),
            7
        );
        assert_eq!(count::<TouchSensor>(world), 14);
        assert_eq!(count::<GeneratorDynamics>(world), 16);
        assert_eq!(count::<RegularSpikeGenerator>(world), 1);
        assert_eq!(count::<PoissonGenerator>(world), 1);
        assert_eq!(count::<MeterSize>(world), 14);
        assert_eq!(count::<Annotation>(world), notes);
        assert_eq!(count::<NoteSize>(world), notes);
        assert_eq!(count::<StaticConnectionSource>(world), 1);
        assert_eq!(count::<Deletable>(world), 159 + notes + burst_neurons);
        assert_eq!(count::<Selectable>(world), 1);
        assert_eq!(count::<Voltmeter>(world), 14);
        assert_eq!(count::<VoltageSeries>(world), 14);
        assert_eq!(count::<VoltmeterRange>(world), 14);
        assert_eq!(count::<Label>(world), neurons);
        for (voltmeter, _) in world.query::<&Voltmeter>().iter() {
            assert_eq!(measured(world, voltmeter).len(), 1);
        }
        assert_eq!(count::<Connection>(world), 79);
        assert_eq!(count::<CompartmentCurrent>(world), 1);
        assert_eq!(count::<Compartment>(world), 1);
        assert_eq!(count::<SpatialDynamics>(world), 1);
        assert_eq!(count::<NeuronType>(world), neurons + 1);

        for (_, connection) in world.query::<&Connection>().iter() {
            assert!(world.contains(connection.from) && world.contains(connection.to));
        }
        assert!(world
            .query::<&Annotation>()
            .iter()
            .any(|(_, a)| a.text == "Touch the sensor to make the leaky neuron fire."));
    }
}
