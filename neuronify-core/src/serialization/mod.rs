//! Save files (`.neuronify`) and scenarios.
//!
//! A save file starts with `MAGIC` and its version as a little-endian `u32`,
//! followed by the world in the hecs column format.
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

#[macro_use]
mod schema;
mod v1;

/// The version that `save` writes.
pub const VERSION: u32 = 1;

const MAGIC: &[u8] = b"neuronify";

pub fn save(world: &World) -> Result<Vec<u8>, String> {
    let mut serializer = postcard::Serializer {
        output: postcard::ser_flavors::StdVec::new(),
    };
    serialize(world, &mut v1::SaveContext, &mut serializer)
        .map_err(|e| format!("Could not save: {e}"))?;
    let body = serializer
        .output
        .finalize()
        .map_err(|e| format!("Could not save: {e}"))?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

/// Reads a save file of any version and upgrades it to the current one.
pub fn load(bytes: &[u8]) -> Result<World, String> {
    let (version, body) = bytes
        .strip_prefix(MAGIC)
        .ok_or("This is not a Neuronify save file")?
        .split_first_chunk()
        .ok_or("The save file ends before its version")?;
    let version = u32::from_le_bytes(*version);
    match version {
        1 => v1::deserialize(body),
        _ => Err(format!(
            "The save file has version {version}, \
             but this Neuronify reads versions 1 to {VERSION}"
        )),
    }
}

/// Reads a legacy `.nfy` file (JSON) or a `.neuronify` save file.
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.starts_with(MAGIC) {
        load(bytes)
    } else {
        load_legacy(bytes)
    }
}

fn load_legacy(bytes: &[u8]) -> Result<World, String> {
    let contents = std::str::from_utf8(bytes).map_err(|e| format!("File is not UTF-8: {e}"))?;
    let simulation = crate::legacy::parse_legacy_nfy(contents)?;
    let mut world = World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};

    const LEAKY_NFY: &str = include_str!("../../examples/leaky.nfy");

    #[test]
    fn loads_legacy_file() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap();
        assert!(!world.is_empty());
    }

    #[test]
    fn loads_save_file() {
        let original = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap();
        let bytes = save(&original).unwrap();
        assert!(bytes.starts_with(MAGIC));

        let loaded = load_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.len(), original.len());
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
    fn loads_version_1() {
        assert_fixture(&load(include_bytes!("fixtures/v1.neuronify")).unwrap());
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
        std::fs::write(path, save(&fixture_world()).unwrap()).unwrap();
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
        let position = |x: f32| Position {
            position: glam::Vec3::new(x, 0.0, 30.0),
        };
        world.spawn((
            position(0.0),
            RegularSpikeGenerator::default(),
            GeneratorDynamics::default(),
            Deletable {},
        ));
        world.spawn((
            position(3.0),
            PoissonGenerator::default(),
            GeneratorDynamics::default(),
            Deletable {},
        ));
        let neuron = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
        let compartment = world.spawn((
            position(6.0),
            NeuronType::Excitatory,
            Compartment {
                voltage: -10.0,
                m: -0.625,
                h: 0.0,
                n: 0.0,
                influence: 0.0,
                capacitance: 1.0,
                injected_current: 0.0,
                fire_impulse: 0.0,
            },
            StaticConnectionSource {},
            Deletable {},
            Selectable { selected: false },
            SpatialDynamics {
                velocity: glam::Vec3::ZERO,
                acceleration: glam::Vec3::ZERO,
            },
        ));
        world.spawn((
            Connection {
                from: neuron,
                to: compartment,
                strength: 1.0,
                directional: false,
            },
            Deletable {},
            CompartmentCurrent {
                capacitance: crate::constants::COUPLING_CAPACITANCE,
            },
        ));
        world
    }

    fn count<T: hecs::Component>(world: &World) -> usize {
        world.query::<&T>().iter().count()
    }

    /// Checks a fixture after it has been upgraded to the current version.
    fn assert_fixture(world: &World) {
        assert_eq!(world.len(), 190);
        assert_eq!(count::<Position>(world), 125);
        assert_eq!(count::<LeakyNeuron>(world), 44);
        assert_eq!(count::<LeakyDynamics>(world), 44);
        assert_eq!(count::<LeakCurrent>(world), 44);
        assert_eq!(count::<AdaptationCurrent>(world), 1);
        assert_eq!(count::<CurrentClamp>(world), 10);
        assert_eq!(count::<CurrentSynapse>(world), 45);
        assert_eq!(count::<ImmediateFireSynapse>(world), 14);
        assert_eq!(count::<Inhibitory>(world), 7);
        assert_eq!(count::<TouchSensor>(world), 14);
        assert_eq!(count::<GeneratorDynamics>(world), 16);
        assert_eq!(count::<RegularSpikeGenerator>(world), 1);
        assert_eq!(count::<PoissonGenerator>(world), 1);
        assert_eq!(count::<VoltmeterSize>(world), 14);
        assert_eq!(count::<Annotation>(world), 30);
        assert_eq!(count::<StaticConnectionSource>(world), 1);
        assert_eq!(count::<Deletable>(world), 145);
        assert_eq!(count::<Selectable>(world), 1);
        assert_eq!(count::<Voltmeter>(world), 14);
        assert_eq!(count::<VoltageSeries>(world), 14);
        assert_eq!(count::<Connection>(world), 79);
        assert_eq!(count::<CompartmentCurrent>(world), 1);
        assert_eq!(count::<Compartment>(world), 1);
        assert_eq!(count::<SpatialDynamics>(world), 1);
        assert_eq!(count::<NeuronType>(world), 45);

        for (_, connection) in world.query::<&Connection>().iter() {
            assert!(world.contains(connection.from) && world.contains(connection.to));
        }
        assert!(world
            .query::<&Annotation>()
            .iter()
            .any(|(_, a)| a.text == "Touch the sensor to make the leaky neuron fire."));
    }
}
