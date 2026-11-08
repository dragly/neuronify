//! Save files (`.neuronify`) and scenarios.
//!
//! A save file starts with `MAGIC` and its version as a little-endian `u32`,
//! followed by the simulation time as a little-endian `f64` (from version 5,
//! in seconds until version 14 and in milliseconds from version 15),
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

use crate::components::{Milliseconds, Seconds};
use crate::constants::{DEFAULT_SPEED, SPEEDS};

#[macro_use]
mod schema;
mod column;
mod v1;
mod v10;
mod v11;
mod v12;
mod v13;
mod v14;
mod v15;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;

/// The version that `save` writes.
pub const VERSION: u32 = 15;

const MAGIC: &[u8] = b"neuronify";

pub struct SavedWorld {
    pub world: World,
    pub time: Milliseconds,
    pub speed: f64,
}

pub fn save(world: &World, time: Milliseconds, speed: f64) -> Result<Vec<u8>, String> {
    let mut serializer = postcard::Serializer {
        output: postcard::ser_flavors::StdVec::new(),
    };
    serialize(world, &mut v15::SaveContext, &mut serializer)
        .map_err(|e| format!("Could not save: {e}"))?;
    let body = serializer
        .output
        .finalize()
        .map_err(|e| format!("Could not save: {e}"))?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&time.0.to_le_bytes());
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
            9 => v10::upgrade_from_v9(&mut world),
            10 => v11::upgrade_from_v10(&mut world),
            11 => v12::upgrade_from_v11(&mut world),
            12 => v13::upgrade_from_v12(&mut world),
            13 => v14::upgrade_from_v13(&mut world),
            14 => v15::upgrade_from_v14(&mut world),
            _ => unreachable!("no upgrade from version {from}"),
        }
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
    let mut world = World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
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
    use crate::measurement::spike_detector::SpikeDetector;
    use crate::measurement::visual_input::{VisualInput, DEFAULT_PICTURE_SPEED};
    use crate::measurement::voltmeter::{measured, VoltageSeries, Voltmeter, VoltmeterRange};
    use hecs::Entity;
    use std::any::TypeId;
    use std::collections::BTreeMap;

    const LEAKY_NFY: &str = include_str!("../legacy/fixtures/leaky.nfy");
    const FIXTURE_TIME: Milliseconds = Milliseconds(2500.0);
    const FIXTURE_SPEED: f64 = 0.5;

    #[test]
    fn loads_legacy_file() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        assert!(!world.is_empty());
    }

    #[test]
    fn loads_save_file() {
        let original = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let bytes = save(&original, Milliseconds::ZERO, DEFAULT_SPEED).unwrap();
        assert!(bytes.starts_with(MAGIC));

        let loaded = load_from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded.len(), original.len());
    }

    #[test]
    fn keeps_the_simulation_time() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let time = Milliseconds(12.5);
        let loaded = load(&save(&world, time, DEFAULT_SPEED).unwrap()).unwrap();
        assert_eq!(loaded.time, time);
    }

    #[test]
    fn keeps_the_simulation_speed() {
        let world = load_from_bytes(LEAKY_NFY.as_bytes()).unwrap().world;
        let loaded = load(&save(&world, Milliseconds::ZERO, 0.5).unwrap()).unwrap();
        assert_eq!(loaded.speed, 0.5);
    }

    #[test]
    fn snaps_the_speed_to_one_the_playback_bar_offers() {
        let loaded_speed = |speed| {
            load(&save(&World::new(), Milliseconds::ZERO, speed).unwrap())
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
            let bytes = save(&World::new(), Milliseconds(time), 2.0).unwrap();
            load(&bytes).unwrap().time.0
        };
        assert_eq!(loaded_time(f64::NAN), 0.0);
        assert_eq!(loaded_time(f64::INFINITY), 0.0);
        assert_eq!(loaded_time(-5.0), 0.0);
        assert_eq!(loaded_time(7.5), 7.5);
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
        let mut bytes = save(&World::new(), Milliseconds::ZERO, DEFAULT_SPEED).unwrap();
        bytes.pop();
        bytes.extend(varint(archetypes.len() as u64));
        for archetype in archetypes {
            bytes.extend(archetype);
        }
        bytes
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
            true_proportions: true,
        };
        display.set_in(&mut world);
        let loaded = load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap())
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
        let saved = load(include_bytes!("fixtures/v10.neuronify")).unwrap();
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
        let saved = load(include_bytes!("fixtures/v11.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_12() {
        let saved = load(include_bytes!("fixtures/v12.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_13() {
        let saved = load(include_bytes!("fixtures/v13.neuronify")).unwrap();
        assert_eq!(
            composition(&saved.world),
            composition(&fixture_world_of_version_13())
        );
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn loads_version_14() {
        let saved = load(include_bytes!("fixtures/v14.neuronify")).unwrap();
        assert_eq!(composition(&saved.world), composition(&fixture_world()));
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
        let saved = load(include_bytes!("fixtures/v15.neuronify")).unwrap();
        assert_eq!(composition(&saved.world), composition(&fixture_world()));
        assert_eq!(saved.time, FIXTURE_TIME);
        assert_eq!(saved.speed, FIXTURE_SPEED);
    }

    #[test]
    fn version_14_counts_meter_times_in_milliseconds() {
        let bytes = include_bytes!("fixtures/v14.neuronify");
        let old = v14::deserialize(&bytes[MAGIC.len() + 4 + 16..]).unwrap();
        let new = load(bytes).unwrap().world;
        let in_milliseconds = |seconds: Milliseconds| Seconds(seconds.0).to_milliseconds();
        let mut checked = 0;
        for (entity, detector) in old.query::<(Entity, &SpikeDetector)>().iter() {
            let upgraded = new.get::<&SpikeDetector>(entity).unwrap();
            assert_eq!(upgraded.time_range, in_milliseconds(detector.time_range));
            checked += 1;
        }
        for (entity, plot) in old.query::<(Entity, &RatePlot)>().iter() {
            let upgraded = new.get::<&RatePlot>(entity).unwrap();
            assert_eq!(upgraded.window, in_milliseconds(plot.window));
            assert_eq!(upgraded.kernel_width, in_milliseconds(plot.kernel_width));
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    fn keeps_the_channels() {
        let mut world = World::new();
        crate::simulation::model::prepare(&mut world, Preparation::Mammal);
        let loaded = load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap())
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
        ] {
            view.set_in(&mut world);
            let loaded = load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap())
                .unwrap()
                .world;
            assert_eq!(count::<ConnectionView>(&loaded), 1);
            assert_eq!(ConnectionView::of(&loaded), view);
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
        let loaded = load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap())
            .unwrap()
            .world;
        assert_eq!(count::<Selected>(&loaded), 1);
        assert_eq!(Selected::entity(&loaded), Some(second));
    }

    #[test]
    fn shows_all_connections_in_files_without_a_connection_view() {
        let world = load(include_bytes!("fixtures/v13.neuronify"))
            .unwrap()
            .world;
        assert_eq!(ConnectionView::of(&world), ConnectionView::All);
    }

    #[test]
    fn version_11_upgrades_to_engine_units_and_cables_with_membranes() {
        let bytes = include_bytes!("fixtures/v11.neuronify");
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

        let loaded = load(&save(&world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap())
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

    fn fixture_world() -> World {
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
            include_str!("../legacy/fixtures/adaptation.nfy"),
            include_str!("../legacy/fixtures/burst.nfy"),
            include_str!("../legacy/fixtures/disinhibition.nfy"),
            include_str!("../legacy/fixtures/frPlot.nfy"),
            include_str!("../legacy/fixtures/generators.nfy"),
            include_str!("../legacy/fixtures/if_response.nfy"),
            include_str!("../legacy/fixtures/inhibitory.nfy"),
            include_str!("../legacy/fixtures/input_summation.nfy"),
            include_str!("../legacy/fixtures/lateral_inhibition.nfy"),
            include_str!("../legacy/fixtures/leaky.nfy"),
            include_str!("../legacy/fixtures/tutorial_1_intro.nfy"),
            include_str!("../legacy/fixtures/two_neuron_oscillator.nfy"),
            include_str!("../legacy/fixtures/visualInput.nfy"),
        ];
        let mut world = World::new();
        for example in examples {
            let simulation = crate::legacy::parse_legacy_nfy(example).unwrap();
            crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
        }
        let position = |x: f32| glam::Vec3::new(x, 0.0, 30.0);
        Element::RegularSpikeGenerator.spawn(&mut world, position(0.0));
        Element::PoissonGenerator.spawn(&mut world, position(3.0));
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
