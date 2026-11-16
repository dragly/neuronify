use neuronify_core::serialization::{is_too_new, load, meta, with_details};

const FROM_A_NEWER_NEURONIFY: &[u8] = include_bytes!("save_files/from_a_newer_neuronify.neuronify");
const WITH_A_PART_FROM_A_NEWER_NEURONIFY: &[u8] =
    include_bytes!("save_files/with_a_part_from_a_newer_neuronify.neuronify");

#[test]
fn a_file_from_a_newer_neuronify_shows_its_name_and_description_but_does_not_open() {
    let meta = meta(FROM_A_NEWER_NEURONIFY).unwrap();
    assert_eq!(meta.name, "Grid cells");
    assert_eq!(
        meta.description,
        "Cells that fire in a grid of places as the animal explores."
    );
    assert_eq!(meta.app, "9.0.0");
    assert!(is_too_new(FROM_A_NEWER_NEURONIFY));
    assert!(load(FROM_A_NEWER_NEURONIFY).is_err());
}

#[test]
fn a_file_from_a_newer_neuronify_keeps_its_simulation_when_renamed() {
    let renamed = with_details(FROM_A_NEWER_NEURONIFY, "Place cells", "Renamed here.").unwrap();
    let meta = meta(&renamed).unwrap();
    assert_eq!(
        (
            meta.name.as_str(),
            meta.description.as_str(),
            meta.app.as_str()
        ),
        ("Place cells", "Renamed here.", "9.0.0")
    );
    assert!(is_too_new(&renamed));
    let world = &FROM_A_NEWER_NEURONIFY[FROM_A_NEWER_NEURONIFY.len() - 24..];
    assert!(renamed.ends_with(world));
}

#[test]
fn a_part_from_a_newer_neuronify_does_not_keep_the_file_from_opening() {
    assert!(!is_too_new(WITH_A_PART_FROM_A_NEWER_NEURONIFY));
    let saved = load(WITH_A_PART_FROM_A_NEWER_NEURONIFY).unwrap();
    assert!(!saved.world.is_empty());
    assert_eq!(
        meta(WITH_A_PART_FROM_A_NEWER_NEURONIFY).unwrap().name,
        "Grid cells"
    );
}
