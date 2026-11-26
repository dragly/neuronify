//! Save files (`.neuronify`) and scenarios.
//!
//! A save file starts with `MAGIC` and a little-endian `u32`.
//! Files from before the container hold a single world, and the `u32` is its version.
//! From `CONTAINER_VERSION` on, the `u32` is the version of the container,
//! followed by the length of its index as a little-endian `u32`,
//! the index (postcard: the kind, version and length of each part)
//! and the parts in the order of the index.
//! The meta part comes first, so a Neuronify that cannot read the world
//! can still show the name and description from the first few bytes.
//!
//! Everything the simulation needs belongs in the world part.
//! Readers skip parts they don't know, or know only in an older version,
//! and leave them out when they save,
//! so add a part only for information an older Neuronify can do without.
//! If the simulation needs a new part, add a world version as well,
//! so that an older Neuronify refuses to open the file.
//! Meta keeps its fields: put new information in a new part.
//! Change the container version only if the index itself changes,
//! since a Neuronify that does not know it can show nothing from the file.
//!
//! A world of version N holds what a single-world file of version N holds after its version:
//! the simulation time as a little-endian `f64` (from version 5,
//! in seconds until version 14 and in milliseconds from version 15),
//! the simulation speed as a little-endian `f64` (from version 6)
//! and the world in the hecs column format.
//!
//! Each world version has a module with the components it can hold.
//! postcard writes a component id as its position in that list,
//! so a list must not change once files have been saved with it.
//! `current` reads and writes the newest version with the app's own components.
//! Older versions read into their own copies of the components,
//! or another version's copy where the fields are the same,
//! and never use the app's components, functions or defaults,
//! so nothing outside these modules changes how an old file loads.
//! Their upgrades end in the newest version's copies,
//! which `current::from_copies` turns into the app's components.
//! A test saves every component and reads it back through those copies,
//! so it fails when an app component stops saving in the newest format.
//! To change the list, or the fields of a component, add a version:
//! write `v{N+1}` with copies of the components that change,
//! `pub use` of the previous version's copies for the rest and its list (for tests only),
//! write `upgrade_from_v{N}` to turn a version N world into a version N+1 world
//! and add it to `UPGRADES`,
//! read version N with its own list (no longer for tests only),
//! change the list in `current` and point its `copies` at the new module,
//! and save a fixture with the `write_fixture` test.
//! Loading clamps every value to the ranges in `bounds`, which belong to the newest version.
//! To change a range, add a version too,
//! and let its upgrade move values from the old range into the new one where it can.

use hecs::{serialize::column::serialize, ComponentError, World};
use postcard::ser_flavors::Flavor;
use serde::{Deserialize, Serialize};

use crate::components::{Milliseconds, Seconds};
use crate::constants::{DEFAULT_SPEED, SPEEDS};
use container::Part;

#[macro_use]
mod schema;
pub mod bounds;
mod column;
mod container;
mod current;
mod v1;
mod v10;
mod v11;
mod v12;
mod v13;
mod v14;
mod v15;
mod v16;
mod v17;
mod v18;
mod v19;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;

/// The world version that `save` writes.
pub const WORLD_VERSION: u32 = 19;

pub const CONTAINER_VERSION: u32 = 19;

pub const LONGEST_DESCRIPTION: usize = 500;

const MAGIC: &[u8] = b"neuronify";
const META: &str = "meta";
const META_VERSION: u32 = 1;
const LONGEST_META: usize = 64 * 1024;
const LONGEST_APP: usize = 40;
const WORLD: &str = "world";
const TOO_NEW: &str = "The save file was made with a newer Neuronify";

type Upgrade = fn(&mut World) -> Result<(), ComponentError>;

const UPGRADES: [Upgrade; WORLD_VERSION as usize - 1] = [
    v2::upgrade_from_v1,
    v3::upgrade_from_v2,
    v4::upgrade_from_v3,
    v5::upgrade_from_v4,
    v6::upgrade_from_v5,
    v7::upgrade_from_v6,
    v8::upgrade_from_v7,
    v9::upgrade_from_v8,
    v10::upgrade_from_v9,
    v11::upgrade_from_v10,
    v12::upgrade_from_v11,
    v13::upgrade_from_v12,
    v14::upgrade_from_v13,
    v15::upgrade_from_v14,
    v16::upgrade_from_v15,
    v17::upgrade_from_v16,
    v18::upgrade_from_v17,
    v19::upgrade_from_v18,
];

pub struct SavedWorld {
    pub world: World,
    pub time: Milliseconds,
    pub speed: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub description: String,
    pub app: String,
}

enum Contents<'a> {
    World { version: u32, body: &'a [u8] },
    Parts(Vec<Part<'a>>),
}

enum Unreadable {
    TooNew,
    Damaged(String),
}

pub fn save(world: &World, time: Milliseconds, speed: f64, meta: &Meta) -> Result<Vec<u8>, String> {
    let mut serializer = postcard::Serializer {
        output: postcard::ser_flavors::StdVec::new(),
    };
    serialize(world, &mut current::SaveContext, &mut serializer)
        .map_err(|e| format!("Could not save: {e}"))?;
    let columns = serializer
        .output
        .finalize()
        .map_err(|e| format!("Could not save: {e}"))?;
    let mut body = time.0.to_le_bytes().to_vec();
    body.extend_from_slice(&speed.to_le_bytes());
    body.extend_from_slice(&columns);
    let meta = postcard::to_stdvec(meta).map_err(|e| format!("Could not save: {e}"))?;
    write_parts(&[
        Part {
            kind: META,
            version: META_VERSION,
            bytes: &meta,
        },
        Part {
            kind: WORLD,
            version: WORLD_VERSION,
            bytes: &body,
        },
    ])
}

fn write_parts(parts: &[Part]) -> Result<Vec<u8>, String> {
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    bytes.extend(container::write(parts)?);
    Ok(bytes)
}

/// Reads a save file of any version and upgrades it to the current one.
pub fn load(bytes: &[u8]) -> Result<SavedWorld, String> {
    let mut saved = read(bytes)?;
    bounds::bound(&mut saved.world);
    Ok(saved)
}

pub fn meta(bytes: &[u8]) -> Option<Meta> {
    let Ok(Contents::Parts(parts)) = contents(bytes) else {
        return None;
    };
    let part = parts
        .iter()
        .find(|part| part.kind == META && part.version == META_VERSION)
        .filter(|part| part.bytes.len() <= LONGEST_META)?;
    let meta: Meta = postcard::from_bytes(part.bytes).ok()?;
    Some(Meta {
        description: meta.description.chars().take(LONGEST_DESCRIPTION).collect(),
        app: meta.app.chars().take(LONGEST_APP).collect(),
        ..meta
    })
}

pub fn is_too_new(bytes: &[u8]) -> bool {
    match contents(bytes) {
        Ok(Contents::World { .. }) | Err(Unreadable::Damaged(_)) => false,
        Ok(Contents::Parts(parts)) => {
            world_of(&parts).is_some_and(|world| world.version > WORLD_VERSION)
        }
        Err(Unreadable::TooNew) => true,
    }
}

pub fn with_details(bytes: &[u8], name: &str, description: &str) -> Option<Vec<u8>> {
    let meta = Meta {
        name: name.to_owned(),
        description: description.to_owned(),
        app: meta(bytes).map(|meta| meta.app).unwrap_or_default(),
    };
    let meta = postcard::to_stdvec(&meta).ok()?;
    let meta = Part {
        kind: META,
        version: META_VERSION,
        bytes: &meta,
    };
    let others = match contents(bytes).ok()? {
        Contents::World { version, body } => vec![Part {
            kind: WORLD,
            version,
            bytes: body,
        }],
        Contents::Parts(parts) => parts.into_iter().filter(|part| part.kind != META).collect(),
    };
    write_parts(&[vec![meta], others].concat()).ok()
}

fn contents(bytes: &[u8]) -> Result<Contents<'_>, Unreadable> {
    let damaged = |problem: &str| Unreadable::Damaged(problem.to_owned());
    let (version, body) = bytes
        .strip_prefix(MAGIC)
        .ok_or_else(|| damaged("This is not a Neuronify save file"))?
        .split_first_chunk()
        .ok_or_else(|| damaged("The save file ends before its version"))?;
    match u32::from_le_bytes(*version) {
        0 => Err(damaged("The save file has version 0")),
        version @ 1..CONTAINER_VERSION => Ok(Contents::World { version, body }),
        CONTAINER_VERSION => container::read(body)
            .map(Contents::Parts)
            .map_err(Unreadable::Damaged),
        _ => Err(Unreadable::TooNew),
    }
}

fn world_of<'a>(parts: &[Part<'a>]) -> Option<Part<'a>> {
    parts.iter().find(|part| part.kind == WORLD).copied()
}

fn read(bytes: &[u8]) -> Result<SavedWorld, String> {
    let (version, body) = match contents(bytes) {
        Ok(Contents::World { version, body }) => (version, body),
        Ok(Contents::Parts(parts)) => {
            let world = world_of(&parts).ok_or("The save file holds no simulation")?;
            (world.version, world.bytes)
        }
        Err(Unreadable::TooNew) => return Err(TOO_NEW.to_owned()),
        Err(Unreadable::Damaged(problem)) => return Err(problem),
    };
    read_world(version, body)
}

fn read_world(version: u32, body: &[u8]) -> Result<SavedWorld, String> {
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
        10 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v10::deserialize(body)?, time, speed)
        }
        11 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v11::deserialize(body)?, time, speed)
        }
        12 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v12::deserialize(body)?, time, speed)
        }
        13 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v13::deserialize(body)?, time, speed)
        }
        14 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v14::deserialize(body)?, time, speed)
        }
        15 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v15::deserialize(body)?, time, speed)
        }
        16 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v16::deserialize(body)?, time, speed)
        }
        17 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v17::deserialize(body)?, time, speed)
        }
        18 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (v18::deserialize(body)?, time, speed)
        }
        19 => {
            let (time, body) = split_f64(body, "simulation time")?;
            let (speed, body) = split_f64(body, "simulation speed")?;
            (current::deserialize(body)?, time, speed)
        }
        0 => return Err("The simulation in the save file has version 0".to_owned()),
        _ => return Err(TOO_NEW.to_owned()),
    };
    for (from, upgrade) in (1..WORLD_VERSION).zip(UPGRADES) {
        if from >= version {
            upgrade(&mut world).map_err(|error| {
                format!("Could not upgrade the save file from version {from}: {error}")
            })?;
        }
    }
    if version < WORLD_VERSION {
        world = current::from_copies(&world)
            .map_err(|error| format!("Could not upgrade the save file: {error}"))?;
    }
    let time = match version {
        ..15 => Seconds(time).to_milliseconds(),
        _ => Milliseconds(time),
    };
    Ok(SavedWorld {
        world,
        time: bounded_time(time),
        speed: snapped_speed(speed),
    })
}

fn bounded_time(time: Milliseconds) -> Milliseconds {
    if time.is_finite() && time >= Milliseconds::ZERO {
        time
    } else {
        Milliseconds::ZERO
    }
}

fn snapped_speed(speed: f64) -> f64 {
    if speed.is_nan() {
        return DEFAULT_SPEED;
    }
    let speed = speed.clamp(SPEEDS[0], SPEEDS[SPEEDS.len() - 1]);
    let distance = |shown: &f64| (shown.ln() - speed.ln()).abs();
    SPEEDS
        .into_iter()
        .min_by(|a, b| distance(a).total_cmp(&distance(b)))
        .unwrap_or(DEFAULT_SPEED)
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
    if simulation.nodes.len() + simulation.edges.len() > column::MAXIMUM_ENTITY_ID as usize {
        return Err("The file holds more elements than a simulation can have".into());
    }
    let mut world = World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
    bounds::bound(&mut world);
    Ok(SavedWorld {
        world,
        time: Milliseconds::ZERO,
        speed: DEFAULT_SPEED,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::elements::{spawn_cable, spawn_compartment, spawn_internode, Element};
    use crate::measurement::firing_rate::RatePlot;
    use crate::measurement::frame::measured;
    use crate::measurement::loudspeaker::{Loudspeaker, Sound};
    use crate::measurement::spike_detector::SpikeDetector;
    use crate::measurement::voltmeter::{VoltageSeries, Voltmeter, VoltmeterRange};
    use crate::stimuli::poisson_generator::PoissonGenerator;
    use crate::stimuli::regular_spike_generator::RegularSpikeGenerator;
    use crate::stimuli::touch_sensor::TouchSensor;
    use crate::stimuli::visual_input::{VisualInput, DEFAULT_PICTURE_SPEED};
    use crate::stimuli::{CurrentClamp, GeneratorDynamics, Stimulus};
    use crate::tools::ConnectionMode;
    use hecs::Entity;
    use std::any::TypeId;
    use std::collections::{BTreeMap, HashSet};

    const LEAKY_NFY: &str = include_str!("legacy/fixtures/leaky.nfy");
    const FIXTURE_TIME: Milliseconds = Milliseconds(2500.0);
    const FIXTURE_SPEED: f64 = 0.5;
    const FIXTURE_SOMA: CompartmentSoma = CompartmentSoma {
        threshold: Millivolts(-25.0),
    };
    const FIXTURE_STROKE: ConnectionMode = ConnectionMode::One;

    #[test]
    fn loads_legacy_file() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        assert!(!world.is_empty());
    }

    #[test]
    fn loads_save_file() {
        let original = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let bytes = save(
            &original,
            Milliseconds::ZERO,
            DEFAULT_SPEED,
            &Meta::default(),
        )
        .unwrap();
        assert!(bytes.starts_with(MAGIC));

        let loaded = load_from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded.len(), original.len());
    }

    #[test]
    fn keeps_the_simulation_time() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let time = Milliseconds(12.5);
        let loaded = load(&save(&world, time, DEFAULT_SPEED, &Meta::default()).unwrap()).unwrap();
        assert_eq!(loaded.time, time);
    }

    #[test]
    fn keeps_the_simulation_speed() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let loaded =
            load(&save(&world, Milliseconds::ZERO, 0.5, &Meta::default()).unwrap()).unwrap();
        assert_eq!(loaded.speed, 0.5);
    }

    #[test]
    fn snaps_the_speed_to_one_the_playback_bar_offers() {
        let loaded_speed = |speed| {
            load(&save(&World::new(), Milliseconds::ZERO, speed, &Meta::default()).unwrap())
                .unwrap()
                .speed
        };
        assert_eq!(loaded_speed(3.0), 4.0);
        assert_eq!(loaded_speed(2.5), 2.0);
        assert_eq!(loaded_speed(1e9), 8.0);
        assert_eq!(loaded_speed(f64::INFINITY), 8.0);
        assert_eq!(loaded_speed(0.0), 0.5);
        assert_eq!(loaded_speed(-4.0), 0.5);
        assert_eq!(loaded_speed(f64::NAN), DEFAULT_SPEED);
    }

    #[test]
    fn starts_at_zero_when_the_saved_time_is_not_a_time() {
        let loaded_time = |time| {
            let bytes = save(&World::new(), Milliseconds(time), 2.0, &Meta::default()).unwrap();
            load(&bytes).unwrap().time.0
        };
        assert_eq!(loaded_time(f64::NAN), 0.0);
        assert_eq!(loaded_time(f64::INFINITY), 0.0);
        assert_eq!(loaded_time(-5.0), 0.0);
        assert_eq!(loaded_time(7.5), 7.5);
    }

    #[test]
    fn rejects_neuronify_1_files_with_more_elements_than_a_simulation_can_hold() {
        let elements = column::MAXIMUM_ENTITY_ID as usize + 1;
        let json = format!(
            r#"{{"fileFormatVersion":4,"nodes":[{}],"edges":[]}}"#,
            vec!["{}"; elements].join(",")
        );
        assert!(load_from_bytes(json.as_bytes()).is_err_and(|e| e.contains("more elements")));
    }

    #[test]
    fn rejects_html() {
        assert!(load_from_bytes(b"<!DOCTYPE html><html></html>").is_err());
    }

    fn meta_part(meta: &Meta) -> Vec<u8> {
        postcard::to_stdvec(meta).unwrap()
    }

    fn parts_of(bytes: &[u8]) -> Vec<Part<'_>> {
        match contents(bytes) {
            Ok(Contents::Parts(parts)) => parts,
            _ => panic!("not a container"),
        }
    }

    fn details() -> Meta {
        Meta {
            name: "Leaky".to_owned(),
            description: "A neuron that leaks.".to_owned(),
            app: "2.3.0".to_owned(),
        }
    }

    #[test]
    fn keeps_the_name_and_description() {
        let bytes = save(&World::new(), Milliseconds::ZERO, DEFAULT_SPEED, &details()).unwrap();
        assert_eq!(meta(&bytes), Some(details()));
        assert!(!is_too_new(&bytes));
    }

    #[test]
    fn writes_meta_before_the_world() {
        let bytes = save(&World::new(), Milliseconds::ZERO, DEFAULT_SPEED, &details()).unwrap();
        let kinds: Vec<&str> = parts_of(&bytes).iter().map(|part| part.kind).collect();
        assert_eq!(kinds, [META, WORLD]);
    }

    #[test]
    fn files_from_before_the_container_have_no_meta() {
        assert_eq!(
            meta(include_bytes!("serialization/fixtures/v18.neuronify")),
            None
        );
    }

    #[test]
    fn a_newer_world_is_too_new_to_load_but_shows_its_meta() {
        let meta_bytes = meta_part(&details());
        let bytes = write_parts(&[
            Part {
                kind: META,
                version: META_VERSION,
                bytes: &meta_bytes,
            },
            Part {
                kind: WORLD,
                version: WORLD_VERSION + 1,
                bytes: &[1, 2, 3],
            },
        ])
        .unwrap();
        assert!(load(&bytes).is_err_and(|e| e == TOO_NEW));
        assert!(is_too_new(&bytes));
        assert_eq!(meta(&bytes), Some(details()));
    }

    #[test]
    fn a_newer_container_is_too_new_and_shows_nothing() {
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(CONTAINER_VERSION + 1).to_le_bytes());
        bytes.extend_from_slice(&[0; 32]);
        assert!(load(&bytes).is_err_and(|e| e == TOO_NEW));
        assert!(is_too_new(&bytes));
        assert_eq!(meta(&bytes), None);
        assert_eq!(with_details(&bytes, "Renamed", ""), None);
    }

    #[test]
    fn damaged_files_are_not_too_new() {
        let mut bytes = save(&World::new(), Milliseconds::ZERO, DEFAULT_SPEED, &details()).unwrap();
        bytes.pop();
        assert!(load(&bytes).is_err());
        assert!(!is_too_new(&bytes));
        assert!(!is_too_new(b"neuronify\0\0\0\0"));
        assert!(!is_too_new(b"<!DOCTYPE html>"));
    }

    #[test]
    fn skips_parts_it_does_not_know() {
        let saved = save(&World::new(), Milliseconds(5.0), DEFAULT_SPEED, &details()).unwrap();
        let parts = parts_of(&saved);
        let thumbnail = Part {
            kind: "thumbnail",
            version: 1,
            bytes: &[7; 64],
        };
        let newer_meta = Part {
            kind: META,
            version: META_VERSION + 1,
            bytes: &[9; 8],
        };
        let world = world_of(&parts).unwrap();
        let bytes = write_parts(&[newer_meta, thumbnail, world]).unwrap();
        assert_eq!(load(&bytes).unwrap().time, Milliseconds(5.0));
        assert_eq!(meta(&bytes), None);
    }

    #[test]
    fn rejects_a_container_without_a_world() {
        let meta_bytes = meta_part(&details());
        let bytes = write_parts(&[Part {
            kind: META,
            version: META_VERSION,
            bytes: &meta_bytes,
        }])
        .unwrap();
        assert!(load(&bytes).is_err_and(|e| e.contains("no simulation")));
    }

    #[test]
    fn rejects_version_0() {
        assert!(load(b"neuronify\0\0\0\0").is_err_and(|e| e.contains("version 0")));
        let bytes = write_parts(&[Part {
            kind: WORLD,
            version: 0,
            bytes: &[],
        }])
        .unwrap();
        assert!(load(&bytes).is_err_and(|e| e.contains("version 0")));
    }

    #[test]
    fn bounds_the_description_and_the_app() {
        let long = Meta {
            name: "Leaky".to_owned(),
            description: "a".repeat(10 * LONGEST_DESCRIPTION),
            app: "9".repeat(10 * LONGEST_APP),
        };
        let bytes = save(&World::new(), Milliseconds::ZERO, DEFAULT_SPEED, &long).unwrap();
        let meta = meta(&bytes).unwrap();
        assert_eq!(meta.description.len(), LONGEST_DESCRIPTION);
        assert_eq!(meta.app.len(), LONGEST_APP);
    }

    #[test]
    fn new_details_keep_the_world_the_app_and_unknown_parts() {
        let meta_bytes = meta_part(&details());
        let thumbnail = Part {
            kind: "thumbnail",
            version: 3,
            bytes: &[7; 16],
        };
        let world = Part {
            kind: WORLD,
            version: WORLD_VERSION + 1,
            bytes: &[1, 2, 3],
        };
        let meta_part = Part {
            kind: META,
            version: META_VERSION,
            bytes: &meta_bytes,
        };
        let bytes = write_parts(&[meta_part, thumbnail, world]).unwrap();
        let renamed = with_details(&bytes, "Renamed", "Described.").unwrap();
        assert_eq!(
            meta(&renamed),
            Some(Meta {
                name: "Renamed".to_owned(),
                description: "Described.".to_owned(),
                app: "2.3.0".to_owned(),
            })
        );
        assert_eq!(parts_of(&renamed)[1..], [thumbnail, world]);
        assert!(is_too_new(&renamed));
    }

    #[test]
    fn new_details_put_files_from_before_the_container_in_one() {
        for original in [
            &include_bytes!("serialization/fixtures/v1.neuronify")[..],
            include_bytes!("serialization/fixtures/v14.neuronify"),
            include_bytes!("serialization/fixtures/v18.neuronify"),
        ] {
            let renamed = with_details(original, "Renamed", "Described.").unwrap();
            let meta = meta(&renamed).unwrap();
            assert_eq!((meta.name.as_str(), meta.app.as_str()), ("Renamed", ""));
            let (before, after) = (load(original).unwrap(), load(&renamed).unwrap());
            assert_eq!(composition(&after.world), composition(&before.world));
            assert_eq!((after.time, after.speed), (before.time, before.speed));
        }
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

    fn varint(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        while value >= 0x80 {
            bytes.push(value as u8 | 0x80);
            value >>= 7;
        }
        bytes.push(value as u8);
        bytes
    }

    fn entity(id: u32) -> Vec<u8> {
        varint(1 << 32 | id as u64)
    }

    fn with_body(archetypes: &[Vec<u8>]) -> Vec<u8> {
        let mut body = Milliseconds::ZERO.0.to_le_bytes().to_vec();
        body.extend_from_slice(&DEFAULT_SPEED.to_le_bytes());
        body.extend(varint(archetypes.len() as u64));
        for archetype in archetypes {
            body.extend(archetype);
        }
        write_parts(&[Part {
            kind: WORLD,
            version: WORLD_VERSION,
            bytes: &body,
        }])
        .unwrap()
    }

    fn positions(entities: &[u32]) -> Vec<u8> {
        let mut archetype = varint(entities.len() as u64);
        archetype.extend([1, 0]);
        for &id in entities {
            archetype.extend(entity(id));
        }
        archetype.extend(vec![0; 12 * entities.len()]);
        archetype
    }

    #[test]
    fn reads_a_hand_written_body() {
        let saved = load(&with_body(&[positions(&[0, 1]), positions(&[2])])).unwrap();
        assert_eq!(count::<Position>(&saved.world), 3);
    }

    #[test]
    fn loads_entities_listed_from_the_highest_id_down_quickly() {
        let descending: Vec<u32> = (1..=5000).map(|n| column::MAXIMUM_ENTITY_ID - n).collect();
        let bytes = with_body(&[
            positions(&descending[..2500]),
            positions(&descending[2500..]),
        ]);
        let started = std::time::Instant::now();
        let saved = load(&bytes).unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        let mut ids: Vec<u32> = saved
            .world
            .query::<(Entity, &Position)>()
            .iter()
            .map(|(entity, _)| entity.id())
            .collect();
        ids.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(ids, descending);
    }

    #[test]
    fn rejects_an_entity_that_appears_twice_in_an_archetype() {
        let repeated = with_body(&[positions(&[0, 0])]);
        assert!(load(&repeated).is_err_and(|e| e.contains("appears twice")));
    }

    #[test]
    fn rejects_an_entity_that_appears_in_two_archetypes() {
        let repeated = with_body(&[positions(&[0]), positions(&[0])]);
        assert!(load(&repeated).is_err_and(|e| e.contains("appears twice")));
    }

    #[test]
    fn rejects_more_entities_than_the_file_holds() {
        let mut archetype = varint(u32::MAX as u64);
        archetype.extend([1, 0]);
        let crafted = with_body(&[archetype]);
        assert!(load(&crafted).is_err_and(|e| e.contains("more entities than it holds")));
    }

    #[test]
    fn rejects_entities_whose_components_the_file_cannot_hold_before_making_room_for_them() {
        let listed = 1000;
        let mut archetype = varint(listed as u64);
        archetype.extend([1, 0]);
        for id in 0..listed {
            archetype.extend(entity(id));
        }
        let crafted = with_body(&[archetype]);
        assert!(load(&crafted).is_err_and(|e| e.contains("more entities than it holds")));
    }

    #[test]
    fn rejects_entity_ids_beyond_the_largest_world() {
        let far = with_body(&[positions(&[column::MAXIMUM_ENTITY_ID])]);
        assert!(load(&far).is_err_and(|e| e.contains("too large")));
    }

    #[test]
    fn rejects_a_component_that_appears_twice_in_an_archetype() {
        let mut archetype = varint(1);
        archetype.extend([2, 0, 0]);
        archetype.extend(entity(0));
        archetype.extend(vec![0; 24]);
        let repeated = with_body(&[archetype]);
        assert!(load(&repeated).is_err_and(|e| e.contains("appears twice")));
    }

    #[test]
    fn loads_version_1() {
        // Version 1 turned the link to the next tutorial into an empty note
        assert_fixture(
            &load(include_bytes!("serialization/fixtures/v1.neuronify")).unwrap(),
            30,
            0,
        );
    }

    #[test]
    fn loads_version_2() {
        assert_fixture(
            &load(include_bytes!("serialization/fixtures/v2.neuronify")).unwrap(),
            29,
            0,
        );
    }

    #[test]
    fn loads_version_3() {
        assert_fixture(
            &load(include_bytes!("serialization/fixtures/v3.neuronify")).unwrap(),
            29,
            0,
        );
    }

    #[test]
    fn loads_version_4() {
        assert_fixture(
            &load(include_bytes!("serialization/fixtures/v4.neuronify")).unwrap(),
            29,
            1,
        );
    }

    #[test]
    fn loads_version_5() {
        let saved = load(include_bytes!("serialization/fixtures/v5.neuronify")).unwrap();
        assert_fixture(&saved, 29, 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, DEFAULT_SPEED);
    }

    #[test]
    fn loads_version_6() {
        let saved = load(include_bytes!("serialization/fixtures/v6.neuronify")).unwrap();
        assert_fixture(&saved, 29, 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_7() {
        let saved = load(include_bytes!("serialization/fixtures/v7.neuronify")).unwrap();
        assert_fixture(&saved, 30, 1);
        assert_eq!(count::<Link>(&saved.world), 1);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_8() {
        let saved = load(include_bytes!("serialization/fixtures/v8.neuronify")).unwrap();
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
            true_proportions: true,
        };
        display.set_in(&mut world);
        let loaded =
            load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                .unwrap()
                .world;
        assert_eq!(NeuronDisplay::of(&loaded), display);
    }

    #[test]
    fn shows_voltage_but_not_firing_rates_in_files_without_a_neuron_display() {
        let world = load(include_bytes!("serialization/fixtures/v7.neuronify"))
            .unwrap()
            .world;
        let display = NeuronDisplay::of(&world);
        assert!(display.voltage);
        assert!(!display.firing_rate);
    }

    #[test]
    fn loads_version_9() {
        let saved = load(include_bytes!("serialization/fixtures/v9.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_9())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
        for input in saved.world.query::<&VisualInput>().iter() {
            assert_eq!(input.speed, DEFAULT_PICTURE_SPEED);
        }
    }

    #[test]
    fn loads_version_10() {
        let saved = load(include_bytes!("serialization/fixtures/v10.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
        for input in saved.world.query::<&VisualInput>().iter() {
            assert_eq!(input.speed, DEFAULT_PICTURE_SPEED);
        }
    }

    #[test]
    fn loads_version_11() {
        let saved = load(include_bytes!("serialization/fixtures/v11.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_12() {
        let saved = load(include_bytes!("serialization/fixtures/v12.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_13() {
        let saved = load(include_bytes!("serialization/fixtures/v13.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_14() {
        let saved = load(include_bytes!("serialization/fixtures/v14.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_15())
        );
        assert_eq!(
            ConnectionView::of(&saved.world),
            ConnectionView::LastSelected
        );
        assert!(Selected::entity(&saved.world).is_some());
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_15() {
        let saved = load(include_bytes!("serialization/fixtures/v15.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_15())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_16() {
        let saved = load(include_bytes!("serialization/fixtures/v16.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_16())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_17() {
        let saved = load(include_bytes!("serialization/fixtures/v17.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_17())
        );
        let somas: Vec<CompartmentSoma> = saved
            .world
            .query::<&CompartmentSoma>()
            .iter()
            .copied()
            .collect();
        assert_eq!(somas, [FIXTURE_SOMA]);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_18() {
        let saved = load(include_bytes!("serialization/fixtures/v18.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_18())
        );
        assert_eq!(ConnectionMode::of(&saved.world), FIXTURE_STROKE);
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn version_16_neurons_with_dendrites_get_a_compartment_soma() {
        let mut world = World::new();
        let branched = Element::ExcitatoryNeuron.spawn(&mut world, glam::Vec3::ZERO);
        crate::dendrites::spawn_dendrites(&mut world, branched);
        world.remove_one::<CompartmentSoma>(branched).unwrap();
        let point = Element::ExcitatoryNeuron.spawn(&mut world, glam::Vec3::new(20.0, 0.0, 0.0));
        let mut copied = copies_of(&world);
        v17::upgrade_from_v16(&mut copied).unwrap();
        let world = current::from_copies(&copied).unwrap();
        assert_eq!(
            world.get::<&CompartmentSoma>(branched).as_deref(),
            Ok(&CompartmentSoma::default())
        );
        assert!(!world.satisfies::<&CompartmentSoma>(point));
    }

    #[test]
    fn version_15_neurons_keep_their_values_and_the_soma_they_were_simulated_with() {
        let bytes = include_bytes!("serialization/fixtures/v15.neuronify");
        let old = v15::deserialize(&bytes[MAGIC.len() + 4 + 16..]).unwrap();
        let new = load(bytes).unwrap().world;
        let mut checked = 0;
        for (entity, neuron) in old.query::<(Entity, &v15::LeakyNeuron)>().iter() {
            let upgraded = new.get::<&LeakyNeuron>(entity).unwrap();
            assert_eq!(upgraded.capacitance, neuron.capacitance);
            assert_eq!(upgraded.threshold, neuron.threshold);
            assert_eq!(upgraded.refractory_period, neuron.refractory_period);
            assert_eq!(upgraded.soma_diameter, Micrometres(20.0));
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    fn version_15_edge_fields_keep_the_side_they_face() {
        use crate::stimuli::receptive_field::{watched_picture, ReceptiveFieldCell};
        let bytes = include_bytes!("serialization/fixtures/v15.neuronify");
        let old = v15::deserialize(&bytes[MAGIC.len() + 4 + 16..]).unwrap();
        let new = load(bytes).unwrap().world;
        let mut checked = 0;
        for (entity, input) in old.query::<(Entity, &v16::VisualInput)>().iter() {
            let v19::ReceptiveField::Edge { angle } = input.field else {
                continue;
            };
            let upgraded = new
                .query::<(Entity, &ReceptiveFieldCell)>()
                .iter()
                .find(|(cell, _)| watched_picture(&new, *cell) == Some(entity))
                .map(|(_, cell)| cell.field)
                .unwrap();
            for (x, y) in [(0.3, 0.1), (-0.2, 0.4), (0.1, -0.3), (-0.4, -0.1)] {
                let faced = if x * angle.sin() + y * angle.cos() < 0.0 {
                    -1.0
                } else {
                    1.0
                };
                assert_eq!(upgraded.weight(x, y), faced);
            }
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    fn version_14_counts_meter_times_in_milliseconds() {
        let bytes = include_bytes!("serialization/fixtures/v14.neuronify");
        let old = v14::deserialize(&bytes[MAGIC.len() + 4 + 16..]).unwrap();
        let new = load(bytes).unwrap().world;
        let in_milliseconds = |seconds: Milliseconds| Seconds(seconds.0).to_milliseconds();
        let mut checked = 0;
        for (entity, detector) in old.query::<(Entity, &v19::SpikeDetector)>().iter() {
            let upgraded = new.get::<&SpikeDetector>(entity).unwrap();
            assert_eq!(upgraded.time_range, in_milliseconds(detector.time_range));
            checked += 1;
        }
        for (entity, plot) in old.query::<(Entity, &v19::RatePlot)>().iter() {
            let upgraded = new.get::<&RatePlot>(entity).unwrap();
            assert_eq!(upgraded.window, in_milliseconds(plot.window));
            assert_eq!(upgraded.kernel_width, in_milliseconds(plot.kernel_width));
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    fn loads_version_19() {
        let saved = load(include_bytes!("serialization/fixtures/v19.neuronify")).unwrap();
        assert_eq!(composition(&saved.world), composition(&fixture_world()));
        assert_eq!(ConnectionView::of(&saved.world), ConnectionView::Rings);
        assert_eq!(
            ConnectionDirection::of(&saved.world),
            ConnectionDirection::Outputs
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn upgrading_version_18_splits_each_visual_input_into_a_picture_and_a_wired_field() {
        use crate::stimuli::receptive_field::{watched_picture, ReceptiveFieldCell};
        let world = load(include_bytes!("serialization/fixtures/v18.neuronify"))
            .unwrap()
            .world;
        assert_eq!(count::<VisualInput>(&world), 2);
        let pictures_firing = world
            .query::<&VisualInput>()
            .with::<&GeneratorDynamics>()
            .iter()
            .count();
        assert_eq!(pictures_firing, 0);
        let mut sensitivities = Vec::new();
        for (cell, field) in world.query::<(Entity, &ReceptiveFieldCell)>().iter() {
            let picture = watched_picture(&world, cell).unwrap();
            assert!(world.satisfies::<&VisualInput>(picture));
            let synapses = world
                .query::<&Connection>()
                .with::<&CurrentSynapse>()
                .iter()
                .filter(|connection| connection.from == cell)
                .count();
            assert_eq!(synapses, 1);
            sensitivities.push(field.sensitivity.0);
        }
        sensitivities.sort_by(f64::total_cmp);
        assert_eq!(sensitivities, [1000.0, 1600.0]);
    }

    #[test]
    fn keeps_the_channels() {
        let mut world = World::new();
        crate::simulation::model::prepare(&mut world, Preparation::Mammal);
        let loaded =
            load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                .unwrap()
                .world;
        assert_eq!(Preparation::of(&loaded), Preparation::Mammal);
    }

    #[test]
    fn keeps_connection_view() {
        let mut world = World::new();
        for view in [
            ConnectionView::All,
            ConnectionView::LastSelected,
            ConnectionView::None,
            ConnectionView::Rings,
        ] {
            view.set_in(&mut world);
            let loaded =
                load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                    .unwrap()
                    .world;
            assert_eq!(count::<ConnectionView>(&loaded), 1);
            assert_eq!(ConnectionView::of(&loaded), view);
        }
    }

    #[test]
    fn keeps_the_stroke() {
        let mut world = World::new();
        for stroke in [
            ConnectionMode::One,
            ConnectionMode::Chain,
            ConnectionMode::FanOut,
        ] {
            stroke.set_in(&mut world);
            let loaded =
                load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                    .unwrap()
                    .world;
            assert_eq!(count::<ConnectionMode>(&loaded), 1);
            assert_eq!(ConnectionMode::of(&loaded), stroke);
        }
    }

    #[test]
    fn connects_in_a_chain_in_files_without_a_stroke() {
        let world = load(include_bytes!("serialization/fixtures/v17.neuronify"))
            .unwrap()
            .world;
        assert_eq!(ConnectionMode::of(&world), ConnectionMode::Chain);
    }

    #[test]
    fn keeps_connection_direction() {
        let mut world = World::new();
        for direction in [
            ConnectionDirection::Both,
            ConnectionDirection::Inputs,
            ConnectionDirection::Outputs,
        ] {
            direction.set_in(&mut world);
            let loaded =
                load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                    .unwrap()
                    .world;
            assert_eq!(count::<ConnectionDirection>(&loaded), 1);
            assert_eq!(ConnectionDirection::of(&loaded), direction);
        }
    }

    #[test]
    fn keeps_the_selection() {
        let mut world = World::new();
        let [first, second] = [0.0, 3.0].map(|x| {
            world.spawn((Position {
                position: glam::Vec3::new(x, 0.0, 0.0),
            },))
        });
        Selected::set(&mut world, Some(first));
        Selected::set(&mut world, Some(second));
        let loaded =
            load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                .unwrap()
                .world;
        assert_eq!(count::<Selected>(&loaded), 1);
        assert_eq!(Selected::entity(&loaded), Some(second));
    }

    #[test]
    fn shows_all_connections_in_files_without_a_connection_view() {
        let world = load(include_bytes!("serialization/fixtures/v13.neuronify"))
            .unwrap()
            .world;
        assert_eq!(ConnectionView::of(&world), ConnectionView::All);
    }

    #[test]
    fn version_11_upgrades_to_engine_units_and_cables_with_membranes() {
        let bytes = include_bytes!("serialization/fixtures/v11.neuronify");
        let old = v11::deserialize(&bytes[MAGIC.len() + 4 + 16..]).unwrap();
        let new = load(bytes).unwrap().world;
        for (entity, neuron) in old.query::<(Entity, &v11::LeakyNeuron)>().iter() {
            let upgraded = new.get::<&LeakyNeuron>(entity).unwrap();
            assert_eq!(upgraded.threshold, Millivolts(neuron.threshold * 1e3));
            assert_eq!(upgraded.capacitance, Nanofarads(neuron.capacitance * 1e9));
        }
        for (entity, leak) in old.query::<(Entity, &v11::LeakCurrent)>().iter() {
            let upgraded = new.get::<&LeakCurrent>(entity).unwrap();
            assert_eq!(upgraded.resistance, Megaohms(leak.resistance / 1e6));
        }
        for (entity, synapse) in old.query::<(Entity, &v11::CurrentSynapse)>().iter() {
            let upgraded = new.get::<&CurrentSynapse>(entity).unwrap();
            assert_eq!(
                upgraded.maximum_current,
                Nanoamperes(synapse.maximum_current * 1e9)
            );
            assert_eq!(upgraded.tau, Milliseconds(synapse.tau * 1e3));
        }
        for (entity, clamp) in old.query::<(Entity, &CurrentClamp)>().iter() {
            let upgraded = new.get::<&CurrentClamp>(entity).unwrap();
            assert_eq!(upgraded.current_output, clamp.current_output * 1e9);
        }
        assert_eq!(count::<v11::Compartment>(&new), 0);
        assert_eq!(count::<v11::CompartmentCurrent>(&new), 0);
        for (_, (connection, cable)) in new
            .query::<(Entity, (&Connection, Option<&Cable>))>()
            .iter()
        {
            let cable_between_nodes =
                !connection.directional && new.satisfies::<&Compartment>(connection.to);
            assert_eq!(cable.is_some(), cable_between_nodes);
        }
        assert_eq!(count::<Cable>(&new), count::<Membrane>(&new));
    }

    #[test]
    fn version_11_dendrites_taper_away_from_their_soma() {
        let mut world = World::new();
        let soma = world.spawn((
            v19::Position {
                position: glam::Vec3::ZERO,
            },
            v11::LeakyNeuron {
                capacitance: 1e-9,
                resting_potential: -0.065,
                threshold: -0.05,
                initial_potential: -0.065,
                voltage_clamped: true,
                minimum_voltage: -0.09,
                maximum_voltage: 0.06,
            },
        ));
        let mut from = soma;
        for step in 1..=6 {
            let to = world.spawn((
                v19::Position {
                    position: glam::Vec3::new(2.0 * step as f32, 0.0, 0.0),
                },
                v11::Compartment {
                    voltage: -0.065,
                    m: 0.05,
                    h: 0.6,
                    n: 0.32,
                    inhibition: 0.0,
                    capacitance: 1e-12,
                    injected_current: 0.0,
                    fire_impulse: 0.0,
                },
                v19::Dendrite,
            ));
            world.spawn((
                v19::Connection {
                    from,
                    to,
                    strength: 1.0,
                    directional: false,
                },
                v11::CompartmentCurrent { capacitance: 1e-12 },
            ));
            from = to;
        }
        for upgrade in &UPGRADES[10..] {
            upgrade(&mut world).unwrap();
        }
        let world = current::from_copies(&world).unwrap();
        let mut diameters = Vec::new();
        for (connection, cable) in world.query::<(&Connection, &Cable)>().iter() {
            let ends = [connection.from, connection.to]
                .map(|end| world.get::<&Position>(end).ok().map(|end| end.position));
            let tapered = crate::simulation::model::dendrite_diameter(&world, ends);
            assert_eq!(cable.diameter, tapered);
            diameters.push(cable.diameter);
        }
        assert_eq!(diameters.len(), 6);
        assert!(diameters.iter().any(|&diameter| diameter != diameters[0]));
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

        let loaded =
            load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap())
                .unwrap()
                .world;
        assert_eq!(count::<Dendrite>(&loaded), dendrites);
        assert!(loaded.satisfies::<&CompartmentSoma>(soma));
    }

    /// Writes `fixture_world` as `fixtures/v{WORLD_VERSION}.neuronify`.
    /// Run it when adding a version, so that version keeps loading:
    fn copies_of(world: &World) -> World {
        let bytes = save(world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()).unwrap();
        let Ok(Contents::Parts(parts)) = contents(&bytes) else {
            panic!("a saved world reads back as parts");
        };
        let body = world_of(&parts).unwrap().bytes;
        let (_, body) = split_f64(body, "simulation time").unwrap();
        let (_, body) = split_f64(body, "simulation speed").unwrap();
        v19::deserialize(body).unwrap()
    }

    #[test]
    fn the_app_components_still_save_in_the_newest_format() {
        let mut world = fixture_world();
        world.spawn((
            Position {
                position: glam::Vec3::new(4.0, 0.0, -3.0),
            },
            Dendrite,
            Loudspeaker {
                sound: Sound::Tone,
                volume: 0.4,
                muted: true,
                ..Loudspeaker::default()
            },
            CableState {
                voltage: vec![Millivolts(-61.0), Millivolts(-58.5)],
                gates: vec![0.1, 0.7, 0.3],
            },
            ConductanceSynapse {
                conductance: Microsiemens(0.008),
                delay: Milliseconds(2.5),
            },
            SynapseState {
                open: Microsiemens(0.002),
                pending: vec![Milliseconds(0.4)],
            },
        ));
        NeuronDisplay {
            voltage: true,
            firing_rate: false,
            true_proportions: true,
        }
        .set_in(&mut world);
        Temperature {
            celsius: Celsius(30.0),
        }
        .set_in(&mut world);
        Randomness { state: 42 }.set_in(&mut world);
        Preparation::Mammal.set_in(&mut world);
        let copied = copies_of(&world);
        let saved: HashSet<TypeId> = copied
            .archetypes()
            .flat_map(|archetype| archetype.component_types())
            .collect();
        assert_eq!(saved.len(), current::ComponentId::ALL.len());
        let again = current::from_copies(&copied).unwrap();
        assert_eq!(
            save(&again, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default()),
            save(&world, Milliseconds::ZERO, DEFAULT_SPEED, &Meta::default())
        );
    }

    /// `cargo test -p neuronify-core write_fixture -- --ignored`
    #[test]
    #[ignore]
    fn write_fixture() {
        let path = format!(
            "{}/src/serialization/fixtures/v{WORLD_VERSION}.neuronify",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = save(
            &fixture_world(),
            FIXTURE_TIME,
            FIXTURE_SPEED,
            &Meta::default(),
        )
        .unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    #[ignore]
    fn write_files_from_a_newer_neuronify() {
        let folder = format!("{}/tests/save_files", env!("CARGO_MANIFEST_DIR"));
        std::fs::create_dir_all(&folder).unwrap();
        let newer = Meta {
            name: "Grid cells".to_owned(),
            description: "Cells that fire in a grid of places as the animal explores.".to_owned(),
            app: "9.0.0".to_owned(),
        };
        let meta = meta_part(&newer);
        let meta = Part {
            kind: META,
            version: META_VERSION,
            bytes: &meta,
        };
        let thumbnail = Part {
            kind: "thumbnail",
            version: 1,
            bytes: &[0x80; 48],
        };
        let newer_world = Part {
            kind: WORLD,
            version: u32::MAX,
            bytes: &[0x55; 24],
        };
        let leaky = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let saved = save(&leaky, FIXTURE_TIME, FIXTURE_SPEED, &newer).unwrap();
        let world = world_of(&parts_of(&saved)).unwrap();
        for (file, parts) in [
            ("from_a_newer_neuronify", [meta, thumbnail, newer_world]),
            (
                "with_a_part_from_a_newer_neuronify",
                [meta, thumbnail, world],
            ),
        ] {
            let bytes = write_parts(&parts).unwrap();
            std::fs::write(format!("{folder}/{file}.neuronify"), bytes).unwrap();
        }
    }

    fn fixture_world() -> World {
        let mut world = fixture_world_of_version_18();
        ConnectionView::Rings.set_in(&mut world);
        ConnectionDirection::Outputs.set_in(&mut world);
        world
    }

    fn fixture_world_of_version_18() -> World {
        let mut world = fixture_world_of_version_17();
        FIXTURE_STROKE.set_in(&mut world);
        world
    }

    fn fixture_world_of_version_17() -> World {
        let mut world = fixture_world_of_version_16();
        let neuron = world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .next()
            .unwrap()
            .0;
        world.insert_one(neuron, FIXTURE_SOMA).unwrap();
        world
    }

    fn fixture_world_of_version_16() -> World {
        let mut world = fixture_world_of_version_15();
        let neuron = world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .next()
            .unwrap()
            .0;
        world.get::<&mut LeakyNeuron>(neuron).unwrap().soma_diameter = Micrometres(30.0);
        world
            .insert_one(
                neuron,
                PendingEvents {
                    adaptation: vec![FIXTURE_TIME],
                    burst: vec![FIXTURE_TIME],
                    fire: vec![FIXTURE_TIME],
                },
            )
            .unwrap();
        world
    }

    fn fixture_world_of_version_15() -> World {
        let mut world = fixture_world_of_version_13();
        let neuron = world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .next()
            .unwrap()
            .0;
        ConnectionView::LastSelected.set_in(&mut world);
        Selected::set(&mut world, Some(neuron));
        world
    }

    fn fixture_world_of_version_13() -> World {
        let mut world = fixture_world_of_version_9();
        let position = |x: f32| glam::Vec3::new(x, 0.0, 40.0);
        let neuron = world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .next()
            .unwrap()
            .0;
        let node = spawn_compartment(&mut world, position(8.0), NeuronType::Excitatory);
        spawn_internode(&mut world, neuron, node);
        world
    }

    /// Bundled examples, plus the components that none of them have,
    /// spawned the way the editor tools spawn them.
    fn fixture_world_of_version_9() -> World {
        let examples = [
            include_str!("legacy/fixtures/adaptation.nfy"),
            include_str!("legacy/fixtures/burst.nfy"),
            include_str!("legacy/fixtures/disinhibition.nfy"),
            include_str!("legacy/fixtures/frPlot.nfy"),
            include_str!("legacy/fixtures/generators.nfy"),
            include_str!("legacy/fixtures/if_response.nfy"),
            include_str!("legacy/fixtures/inhibitory.nfy"),
            include_str!("legacy/fixtures/input_summation.nfy"),
            include_str!("legacy/fixtures/lateral_inhibition.nfy"),
            include_str!("legacy/fixtures/leaky.nfy"),
            include_str!("legacy/fixtures/tutorial_1_intro.nfy"),
            include_str!("legacy/fixtures/two_neuron_oscillator.nfy"),
            include_str!("legacy/fixtures/visualInput.nfy"),
        ];
        let mut world = World::new();
        for example in examples {
            let simulation = crate::legacy::parse_legacy_nfy(example).unwrap();
            crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
        }
        let position = |x: f32| glam::Vec3::new(x, 0.0, 30.0);
        Stimulus::RegularSpikeGenerator.spawn(&mut world, position(0.0));
        Stimulus::PoissonGenerator.spawn(&mut world, position(3.0));
        let neuron = world
            .query::<(Entity, &LeakyNeuron)>()
            .iter()
            .next()
            .unwrap()
            .0;
        let compartment = spawn_compartment(&mut world, position(6.0), NeuronType::Excitatory);
        spawn_cable(&mut world, neuron, compartment, false);
        world
    }

    fn count<T: hecs::Component>(world: &World) -> usize {
        world.query::<(Entity, &T)>().iter().count()
    }

    /// Checks a fixture after it has been upgraded to the current version.
    fn assert_fixture(saved: &SavedWorld, notes: usize, burst_neurons: usize) {
        let world = &saved.world;
        let neurons = 44 + burst_neurons;
        assert_eq!(world.len(), 204);
        assert_eq!(count::<Position>(world), 125);
        assert_eq!(count::<LeakyNeuron>(world), neurons);
        assert_eq!(count::<NodeState>(world), neurons + 1);
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
                .filter(|neuron_type| **neuron_type == NeuronType::Inhibitory)
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
        for (voltmeter, _) in world.query::<(Entity, &Voltmeter)>().iter() {
            assert_eq!(measured(world, voltmeter).len(), 1);
        }
        assert_eq!(count::<Connection>(world), 79);
        assert_eq!(count::<Cable>(world), 1);
        assert_eq!(count::<Membrane>(world), 1);
        assert_eq!(count::<Compartment>(world), 1);
        assert_eq!(count::<SpatialDynamics>(world), 1);
        assert_eq!(count::<NeuronType>(world), neurons + 1);

        for connection in world.query::<&Connection>().iter() {
            assert!(world.contains(connection.from) && world.contains(connection.to));
        }
        assert!(world
            .query::<&Annotation>()
            .iter()
            .any(|a| a.text == "Touch the sensor to make the leaky neuron fire."));
    }
}
