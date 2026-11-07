use super::*;
use crate::elements::{spawn_synapse, spawn_voltmeter};
use crate::measurement::spike_detector::SpikeDetector;
use crate::measurement::voltmeter::{VoltageSeries, Voltmeter};
use crate::simulation::{fire_nearby, step_physics, step_simulation, Engine};
use std::collections::HashMap;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3::new(x, 0.0, z)
}

fn editor(tool: Tool) -> Editor {
    Editor {
        tool,
        ..Default::default()
    }
}

fn drag(editor: &mut Editor, world: &mut World, path: &[Vec3]) -> Vec<Option<Pan>> {
    let pans = path
        .iter()
        .enumerate()
        .map(|(index, &position)| {
            if index > 0 {
                editor.move_held();
            }
            editor.edit(world, position, false)
        })
        .collect();
    editor.lift(world);
    pans
}

fn count<T: Component>(world: &World) -> usize {
    world.query::<&T>().iter().count()
}

fn neuron(world: &mut World, position: Vec3) -> Entity {
    Element::ExcitatoryNeuron.spawn(world, position)
}

fn axon_endings(world: &World) -> Vec<Connection> {
    world
        .query::<&Connection>()
        .with::<&ConductanceSynapse>()
        .iter()
        .filter(|(_, connection)| connection.directional)
        .map(|(_, connection)| connection.clone())
        .collect()
}

#[test]
fn placing_keeps_a_distance_from_the_previous_element() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    let path = [
        at(0.0, 0.0),
        at(0.0, 1.0),
        at(0.0, 5.0),
        at(0.0, 7.0),
        at(0.0, 8.0),
    ];
    drag(&mut editor, &mut world, &path);
    assert_eq!(count::<LeakyNeuron>(&world), 2);

    drag(&mut editor, &mut world, &[at(0.0, 9.0)]);
    assert_eq!(count::<LeakyNeuron>(&world), 3);
}

#[test]
fn a_new_note_is_selected() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::Note));
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let note = editor.selection.unwrap();
    assert!(world.get::<&Annotation>(note).is_ok());
    assert_eq!(editor.tool, Tool::Edit);
}

#[test]
fn undoing_a_new_note_drops_the_selection() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::Note));
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);

    editor.undo(&mut world);
    editor.sync_selection(&mut world);

    assert_eq!(editor.selection, None);
}

#[test]
fn erasing_the_selected_element_drops_the_selection() {
    let mut world = World::new();
    let selected = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Erase);
    editor.selection = Some(selected);

    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    editor.sync_selection(&mut world);

    assert_eq!(editor.selection, None);
}

#[test]
fn the_selection_marks_the_selected_element() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Edit);

    editor.selection = Some(first);
    editor.sync_selection(&mut world);
    editor.selection = Some(second);
    editor.sync_selection(&mut world);
    editor.selection = None;
    editor.sync_selection(&mut world);

    assert_eq!(Selected::entity(&world), Some(second));
}

#[test]
fn dragging_a_new_note_moves_it() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::Note));
    drag(
        &mut editor,
        &mut world,
        &[at(0.0, 0.0), at(0.5, 0.0), at(3.0, 0.0)],
    );
    let notes = positions_with::<Annotation>(&world);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].1, at(2.5, 0.0));
}

#[test]
fn only_neurons_grow_dendrites() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::CurrentSource));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    assert_eq!(count::<Dendrite>(&world), 0);

    editor.tool = Tool::Place(Element::InhibitoryNeuron);
    drag(&mut editor, &mut world, &[at(20.0, 0.0)]);
    assert!(count::<Dendrite>(&world) > 0);
}

#[test]
fn with_compartments_an_inhibitory_generator_connects_with_a_synapse() {
    let mut world = World::new();
    let generator = Element::RegularSpikeGenerator.spawn(&mut world, at(0.0, 0.0));
    world.insert_one(generator, NeuronType::Inhibitory).unwrap();
    let target = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::StaticConnection);
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.5), at(0.0, 9.5)]);

    assert!(connected(&world, generator, target));
    assert_eq!(count::<CurrentSynapse>(&world), 1);
    assert_eq!(count::<Compartment>(&world), 0);
}

#[test]
fn static_connections_chain_from_neuron_to_neuron_unless_shift_is_held() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    let c = neuron(&mut world, at(0.0, 20.0));
    let mut editor = editor(Tool::StaticConnection);
    drag(
        &mut editor,
        &mut world,
        &[at(0.0, 0.5), at(0.0, 5.0), at(0.0, 10.5), at(0.0, 19.5)],
    );
    assert!(connected(&world, a, b) && connected(&world, b, c));
    assert_eq!(count::<CurrentSynapse>(&world), 2);

    for position in [at(0.0, 0.0), at(0.0, 10.0), at(0.0, 20.0)] {
        editor.edit(&mut world, position, true);
    }
    editor.release(&world);
    assert!(connected(&world, a, c));
    assert_eq!(count::<CurrentSynapse>(&world), 3);
}

fn straight(from: Vec3, to: Vec3) -> Vec<Vec3> {
    (0..=80)
        .map(|step| from.lerp(to, step as f32 / 80.0))
        .collect()
}

fn axon_released_on_a_dendrite(mut editor: Editor) -> (World, Entity, Entity) {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    spawn_dendrites(&mut world, target);
    let (_, nearest) = positions_with::<Dendrite>(&world)
        .into_iter()
        .min_by(|(_, a), (_, b)| a.length().total_cmp(&b.length()))
        .unwrap();
    drag(&mut editor, &mut world, &straight(Vec3::ZERO, nearest));
    (world, source, target)
}

fn single_ending(world: &World) -> Connection {
    let [ending] = &axon_endings(world)[..] else {
        panic!("expected one ending, got {:?}", axon_endings(world));
    };
    ending.clone()
}

#[test]
fn axons_end_on_the_dendrite_they_are_released_on() {
    let (world, _, target) = axon_released_on_a_dendrite(editor(Tool::Axon));
    let tree = compartment_tree(&world, target);
    let ending = single_ending(&world);
    assert!(tree.contains(&ending.to));
    assert!(!tree.contains(&ending.from));
}

#[test]
fn with_compartments_the_add_tools_end_axons_on_dendrites_too() {
    let mut adding = editor(Tool::Place(Element::ExcitatoryNeuron));
    adding.compartments = true;
    let (world, _, target) = axon_released_on_a_dendrite(adding);
    assert!(compartment_tree(&world, target).contains(&single_ending(&world).to));
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn erasing_a_neuron_leaves_the_dendrite_its_axon_ends_on() {
    let (mut world, source, target) = axon_released_on_a_dendrite(editor(Tool::Axon));
    let tree = compartment_tree(&world, target);
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(source);
    editor.erase_selection(&mut world);

    assert!(tree.iter().all(|&compartment| world.contains(compartment)));
    assert!(axon_endings(&world).is_empty());
}

#[test]
fn axons_pass_over_dendrites_on_their_way_to_a_neuron() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    spawn_dendrites(&mut world, target);
    let mut editor = editor(Tool::Axon);
    drag(
        &mut editor,
        &mut world,
        &straight(Vec3::ZERO, at(0.0, 30.0)),
    );

    let [ending] = &axon_endings(&world)[..] else {
        panic!("expected one ending, got {:?}", axon_endings(&world));
    };
    assert_eq!(ending.to, target);
}

#[test]
fn axons_do_not_end_on_their_own_dendrites() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    spawn_dendrites(&mut world, source);
    let dendrites = count::<Compartment>(&world);
    let (_, apical_tip) = positions_with::<Dendrite>(&world)
        .into_iter()
        .max_by(|(_, a), (_, b)| a.x.total_cmp(&b.x))
        .unwrap();
    let mut editor = editor(Tool::Axon);
    drag(&mut editor, &mut world, &straight(Vec3::ZERO, apical_tip));

    assert!(count::<Compartment>(&world) > dendrites);
    assert!(axon_endings(&world).is_empty());
}

const REACH: f32 = 1.5;

fn axon_beside_a_dendrite() -> (World, Editor, Vec<Entity>, Vec3) {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    spawn_dendrites(&mut world, target);
    let (dendrite, nearest) = positions_with::<Dendrite>(&world)
        .into_iter()
        .min_by(|(_, a), (_, b)| a.length().total_cmp(&b.length()))
        .unwrap();
    let radius = compartment_radii(&world)[&dendrite];
    let beside = nearest - nearest.normalize() * (radius + 0.9 * REACH);
    let mut editor = reaching(Tool::Axon, REACH);
    for position in straight(at(0.0, 0.5), beside) {
        editor.edit(&mut world, position, false);
    }
    let tree = compartment_tree(&world, target);
    (world, editor, tree, beside)
}

#[test]
fn an_axon_within_touch_reach_of_a_dendrite_shows_that_it_ends_there() {
    let (mut world, mut editor, tree, beside) = axon_beside_a_dendrite();
    assert!(beside.distance(at(0.0, 30.0)) > 2.0 * NODE_RADIUS + REACH);
    assert!(tree.contains(&editor.ending().unwrap()));
    editor.lift(&mut world);
    assert!(tree.contains(&single_ending(&world).to));
    assert_eq!(editor.ending(), None);
}

#[test]
fn the_snapped_tip_ends_where_the_released_axon_will() {
    let (mut world, mut editor, _, _) = axon_beside_a_dendrite();
    let palette = Theme::default().palette();
    let (from, ending) = editor.new_ending(&world).unwrap();
    assert_eq!(Some(ending), editor.ending());
    let preview = crate::rendering::paths::ending_path(&world, from, ending, &palette).unwrap();
    editor.lift(&mut world);
    assert_eq!(editor.new_ending(&world), None);
    let paths = connection_paths(&world, &compartment_radii(&world), &palette);
    let [released] = &paths
        .values()
        .filter(|path| path.directional)
        .collect::<Vec<_>>()[..]
    else {
        panic!("expected one ending");
    };
    let knob = |path: &crate::rendering::ConnectionPath| *path.points.last().unwrap();
    assert!(knob(&preview).distance(knob(released)) < 1e-4);
    assert_eq!(preview.terminal_radius, released.terminal_radius);
}

#[test]
fn an_axon_stops_growing_while_it_would_end_on_a_dendrite() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(20.0, 0.0));
    lay_dendrite(&mut world, target, at(20.0, 12.0)).unwrap();
    let mut editor = reaching(Tool::Axon, REACH);
    for position in straight(at(0.5, 0.0), at(19.0, 4.0)) {
        editor.edit(&mut world, position, false);
    }
    let compartments = count::<Compartment>(&world);
    let (from, _) = editor.new_ending(&world).unwrap();
    for position in straight(at(19.0, 4.0), at(19.0, 10.0)) {
        editor.edit(&mut world, position, false);
        assert!(editor.ending().is_some(), "{position}");
    }
    assert_eq!(count::<Compartment>(&world), compartments);
    assert_eq!(editor.new_ending(&world).unwrap().0, from);
}

#[test]
fn an_axon_stays_on_a_dendrite_until_half_a_reach_beyond_where_it_attached() {
    let (mut world, mut editor, tree, beside) = axon_beside_a_dendrite();
    let outward = -beside.normalize();
    let mut ending_at = |reaches: f32| {
        editor.edit(
            &mut world,
            beside + outward * (reaches - 0.9) * REACH,
            false,
        );
        editor.ending().filter(|ending| tree.contains(ending))
    };
    assert!(ending_at(1.3).is_some());
    assert!(ending_at(1.6).is_none());
    assert!(ending_at(1.3).is_none());
    assert!(ending_at(0.9).is_some());
}

#[test]
fn an_axon_moved_out_of_reach_again_ends_nowhere() {
    let (mut world, mut editor, _, beside) = axon_beside_a_dendrite();
    editor.edit(&mut world, beside - beside.normalize() * REACH, false);
    assert_eq!(editor.ending(), None);
    editor.lift(&mut world);
    assert!(axon_endings(&world).is_empty());
}

#[test]
fn an_axon_reaching_a_soma_forms_its_synapse_at_once_and_ends_there_until_it_leaves() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    let mut editor = reaching(Tool::Axon, REACH);
    for position in straight(at(0.0, 0.5), at(0.0, 18.0)) {
        editor.edit(&mut world, position, false);
    }
    assert_eq!(editor.ending(), Some(target));
    assert!(editor.take_formed());
    assert!(!editor.take_formed());
    assert_eq!(single_ending(&world).to, target);
    editor.edit(&mut world, at(0.0, 18.5), false);
    assert_eq!(editor.ending(), Some(target));
    editor.edit(&mut world, at(6.0, 20.0), false);
    assert_eq!(editor.ending(), None);
    assert!(!editor.take_formed());
}

#[test]
fn an_axon_brought_back_over_its_own_soma_does_not_end_on_it() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = reaching(Tool::Axon, REACH);
    let mut path = straight(at(0.0, 0.5), at(0.0, 8.0));
    path.extend(straight(at(0.0, 8.0), at(0.0, 0.0)));
    drag(&mut editor, &mut world, &path);
    assert!(axon_endings(&world).is_empty());
}

#[test]
fn what_is_being_drawn_stays_where_it_was_laid_until_released() {
    for tool in [Tool::Axon, Tool::Dendrite] {
        let mut world = World::new();
        neuron(&mut world, at(0.0, 0.0));
        let mut editor = editor(tool);
        let mut path = straight(at(0.0, 0.5), at(0.0, 10.0));
        path.extend(straight(at(0.0, 10.0), at(10.0, 10.0)));
        for position in path {
            editor.edit(&mut world, position, false);
        }
        let laid = positions_with::<Compartment>(&world);
        assert!(laid.len() > 5, "{tool:?}");

        step_physics(&mut world, 500, editor.drawn());
        assert_eq!(positions_with::<Compartment>(&world), laid, "{tool:?}");
        let growing = editor
            .axon_being_grown(&world)
            .or(editor.dendrite_being_grown());
        let (tip, start, _) = growing.unwrap();
        assert_eq!(position(&world, tip), start, "{tool:?}");

        editor.lift(&mut world);
        assert!(editor.drawn().is_empty(), "{tool:?}");
        step_physics(&mut world, 500, editor.drawn());
        assert_ne!(positions_with::<Compartment>(&world), laid, "{tool:?}");
    }
}

fn stroke(editor: &mut Editor, world: &mut World, corners: &[Vec3]) {
    let path: Vec<Vec3> = corners
        .windows(2)
        .flat_map(|pair| straight(pair[0], pair[1]))
        .collect();
    drag(editor, world, &path);
}

fn sharpest_bend(world: &World) -> f32 {
    let cables: Vec<(Entity, Entity)> = world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .filter(|(_, cable)| !cable.directional)
        .map(|(_, cable)| (cable.from, cable.to))
        .collect();
    let mut sharpest = std::f32::consts::PI;
    for &(before, joint) in &cables {
        for &(_, after) in cables.iter().filter(|&&(from, _)| from == joint) {
            let at = position(world, joint);
            let bend = (position(world, before) - at).angle_between(position(world, after) - at);
            sharpest = sharpest.min(bend);
        }
    }
    sharpest
}

#[test]
fn drawing_back_over_an_axon_or_dendrite_takes_it_back_instead_of_folding_it() {
    for tool in [Tool::Axon, Tool::Dendrite] {
        let mut world = World::new();
        neuron(&mut world, at(0.0, 0.0));
        let mut editor = editor(tool);
        stroke(
            &mut editor,
            &mut world,
            &[at(0.0, 0.5), at(0.0, 12.0), at(0.3, 5.0)],
        );
        let reach = positions_with::<Compartment>(&world)
            .into_iter()
            .map(|(_, compartment)| compartment.z)
            .fold(0.0, f32::max);
        assert!(reach < 7.5, "{tool:?} still reaches {reach}");
        assert!(sharpest_bend(&world) >= SHARPEST_BEND, "{tool:?}");
    }
}

#[test]
fn an_axon_taken_all_the_way_back_grows_out_again_on_the_other_side() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Axon);
    stroke(
        &mut editor,
        &mut world,
        &[at(0.0, 0.5), at(0.0, 10.0), at(0.3, -10.0)],
    );
    let tips: Vec<Vec3> = positions_with::<Compartment>(&world)
        .into_iter()
        .map(|(_, compartment)| compartment)
        .collect();
    assert!(tips.iter().all(|tip| tip.z < 0.0), "{tips:?}");
    assert!(tips.iter().any(|tip| tip.z < -6.0), "{tips:?}");
    assert!(chain(&world, soma, positions_with::<Compartment>(&world)[0].0).is_some());
}

#[test]
fn taking_a_dendrite_back_stops_where_the_stroke_began() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Dendrite);
    stroke(&mut editor, &mut world, &[at(0.0, 0.5), at(0.0, 8.0)]);
    let first = positions_with::<Dendrite>(&world);
    let (_, tip) = first
        .iter()
        .copied()
        .max_by(|(_, a), (_, b)| a.z.total_cmp(&b.z))
        .unwrap();
    stroke(&mut editor, &mut world, &[tip, at(0.0, 14.0), at(0.3, 1.0)]);
    assert_eq!(positions_with::<Dendrite>(&world), first);
    assert!(sharpest_bend(&world) >= SHARPEST_BEND);
}

fn grown_dendrites(world: &World, before: &[Entity]) -> Vec<(Entity, Vec3)> {
    positions_with::<Dendrite>(world)
        .into_iter()
        .filter(|(dendrite, _)| !before.contains(dendrite))
        .collect()
}

#[test]
fn dendrites_grow_out_from_a_neuron_one_segment_at_a_time() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    let path: Vec<Vec3> = (1..=20).map(|step| at(0.0, step as f32 * 0.5)).collect();
    drag(&mut editor(Tool::Dendrite), &mut world, &path);

    assert_eq!(count::<Dendrite>(&world), 5);
    assert_eq!(count::<Compartment>(&world), 5);
    assert!(axon_endings(&world).is_empty());
    let cables: Vec<Connection> = world
        .query::<&Connection>()
        .iter()
        .map(|(_, cable)| cable.clone())
        .collect();
    assert_eq!(cables.len(), 5);
    let z = |entity: Entity| world.get::<&Position>(entity).unwrap().position.z;
    for cable in cables {
        assert!(!cable.directional);
        assert!(world.get::<&Dendrite>(cable.to).is_ok());
        assert!(z(cable.from) < z(cable.to), "wired from the soma outwards");
    }
    assert_eq!(compartment_tree(&world, soma).len(), 5);
}

#[test]
fn dendrites_branch_off_the_dendrite_they_start_on() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    spawn_dendrites(&mut world, soma);
    let tree = compartment_tree(&world, soma);
    let (tip, tip_position) = positions_with::<Dendrite>(&world)
        .into_iter()
        .max_by(|(_, a), (_, b)| a.x.total_cmp(&b.x))
        .unwrap();
    let path = straight(tip_position, tip_position + at(0.0, 9.0));
    drag(&mut editor(Tool::Dendrite), &mut world, &path);

    let grown = grown_dendrites(&world, &tree);
    assert_eq!(grown.len(), 4);
    let (first, _) = grown
        .iter()
        .min_by(|(_, a), (_, b)| {
            a.distance(tip_position)
                .total_cmp(&b.distance(tip_position))
        })
        .unwrap();
    assert!(connected(&world, tip, *first));
    assert_eq!(compartment_tree(&world, soma).len(), tree.len() + 4);
}

#[test]
fn dendrites_end_where_they_are_let_go_even_on_another_neuron() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    let other = neuron(&mut world, at(0.0, 12.0));
    spawn_dendrites(&mut world, other);
    let others = compartment_tree(&world, other);
    let path = straight(at(0.0, 0.5), at(0.0, 12.0));
    drag(&mut editor(Tool::Dendrite), &mut world, &path);

    assert!(axon_endings(&world).is_empty());
    assert_eq!(compartment_tree(&world, other).len(), others.len());
    assert_eq!(compartment_tree(&world, soma).len(), 5);
}

#[test]
fn a_fast_dendrite_stroke_lays_evenly_spaced_compartments() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    drag(
        &mut editor(Tool::Dendrite),
        &mut world,
        &[at(0.0, 0.5), at(0.0, 12.0)],
    );

    let mut distances: Vec<f32> = positions_with::<Dendrite>(&world)
        .into_iter()
        .map(|(_, position)| position.z)
        .collect();
    distances.sort_by(f32::total_cmp);
    let expected: Vec<f32> = (1..=6).map(|step| step as f32 * SEGMENT_LENGTH).collect();
    assert_eq!(distances, expected);
}

#[test]
fn a_growing_dendrite_reaches_from_its_tip_to_the_pointer() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Dendrite);
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.edit(&mut world, at(0.0, 1.5), false);
    assert_eq!(
        editor.dendrite_being_grown(),
        Some((soma, at(0.0, 0.0), at(0.0, 1.5)))
    );
    editor.edit(&mut world, at(0.0, 2.5), false);
    editor.edit(&mut world, at(0.0, 3.5), false);
    let (from, start, end) = editor.dendrite_being_grown().unwrap();
    assert!(world.get::<&Dendrite>(from).is_ok());
    assert_eq!((start, end), (at(0.0, 2.5), at(0.0, 3.5)));
    assert_eq!(editor.axon_being_grown(&world), None);
    assert_eq!(editor.connection_being_drawn(&world), None);
}

#[test]
fn an_axon_ending_on_a_drawn_dendrite_makes_its_neuron_fire() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    let path = straight(at(0.0, 30.0), at(0.0, 20.0));
    drag(&mut editor(Tool::Dendrite), &mut world, &path);
    let (_, tip) = positions_with::<Dendrite>(&world)
        .into_iter()
        .min_by(|(_, a), (_, b)| a.z.total_cmp(&b.z))
        .unwrap();
    drag(
        &mut editor(Tool::Axon),
        &mut world,
        &straight(Vec3::ZERO, tip),
    );
    assert!(compartment_tree(&world, target).contains(&single_ending(&world).to));

    let (mut engine, mut time) = (Engine::default(), 0.0);
    engine.sync(&mut world);
    engine.fire(&world, source);
    let fired = (0..8000).position(|_| {
        step_simulation(&mut world, &mut engine, &mut time, 1, None);
        engine.fired(target)
    });
    assert!(fired.is_some());
}

#[test]
fn connections_start_only_near_a_source() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::StaticConnection);
    editor.edit(&mut world, at(0.0, 3.0), false);
    assert!(editor.connection_tool.is_none());
    editor.edit(&mut world, at(0.0, 1.0), false);
    assert!(editor.connection_tool.is_some());
}

#[test]
fn axons_grow_compartments_until_they_reach_a_neuron() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    let mut editor = editor(Tool::Axon);
    let path: Vec<Vec3> = (0..=40).map(|step| at(0.0, step as f32 * 0.5)).collect();
    drag(&mut editor, &mut world, &path);

    let compartments = count::<Compartment>(&world);
    assert!(compartments > 3, "{compartments}");
    let first = world
        .query::<&Connection>()
        .iter()
        .find(|(_, connection)| connection.from == source)
        .map(|(_, connection)| connection.to)
        .unwrap();
    assert!(world.get::<&Compartment>(first).is_ok());
    let ending = world
        .query::<&Connection>()
        .with::<&ConductanceSynapse>()
        .iter()
        .find(|(_, connection)| connection.to == target)
        .map(|(_, connection)| connection.clone())
        .unwrap();
    assert!(ending.directional);
    assert!(world.get::<&Compartment>(ending.from).is_ok());
}

#[test]
fn dragging_from_an_element_while_adding_connects_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    let c = neuron(&mut world, at(0.0, 20.0));
    let mut editor = editor(Tool::Place(Element::InhibitoryNeuron));
    drag(
        &mut editor,
        &mut world,
        &[at(0.0, 1.5), at(0.0, 5.0), at(0.0, 10.5), at(0.0, 19.5)],
    );
    assert!(connected(&world, a, b) && connected(&world, b, c));
    assert_eq!(count::<CurrentSynapse>(&world), 2);
    assert_eq!(count::<LeakyNeuron>(&world), 3);
}

#[test]
fn the_connection_being_added_is_drawn() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.edit(&mut world, at(0.0, 5.0), false);
    assert_eq!(
        editor.connection_being_drawn(&world),
        Some((at(0.0, 0.0), at(0.0, 5.0)))
    );
}

#[test]
fn the_connection_being_added_is_hidden_until_its_terminal_clears_the_source() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.edit(&mut world, at(0.3, 1.2), false);
    assert_eq!(editor.connection_being_drawn(&world), None);
    editor.edit(&mut world, at(0.0, 2.5), false);
    assert_eq!(
        editor.connection_being_drawn(&world),
        Some((at(0.0, 0.0), at(0.0, 2.5)))
    );
}

#[test]
fn painting_neurons_lasts_until_release() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::InhibitoryNeuron));
    assert_eq!(editor.painting(), None);
    editor.edit(&mut world, at(0.0, 0.0), false);
    assert_eq!(editor.painting(), Some(Element::InhibitoryNeuron));
    editor.edit(&mut world, at(0.0, 3.0), false);
    assert_eq!(editor.painting(), Some(Element::InhibitoryNeuron));
    editor.release(&world);
    assert_eq!(editor.painting(), None);
}

#[test]
fn holding_after_placing_stops_painting() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 0.0), false);
    assert!(editor.hold(&world, at(0.0, 0.0), false));
    assert_eq!(editor.painting(), None);
}

#[test]
fn pulling_a_connection_or_placing_an_input_is_not_painting() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut pulling = editor(Tool::Place(Element::ExcitatoryNeuron));
    pulling.edit(&mut world, at(0.0, 0.5), false);
    assert_eq!(pulling.painting(), None);
    let mut placing = editor(Tool::Place(Element::CurrentSource));
    placing.edit(&mut world, at(10.0, 10.0), false);
    assert_eq!(placing.painting(), None);
}

#[test]
fn a_growing_axon_reaches_from_its_last_compartment_to_the_pointer() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.edit(&mut world, at(0.0, 1.5), false);
    assert_eq!(
        editor.axon_being_grown(&world),
        Some((source, at(0.0, 0.0), at(0.0, 1.5)))
    );
    editor.edit(&mut world, at(0.0, 2.5), false);
    editor.edit(&mut world, at(0.0, 3.5), false);
    let (from, start, end) = editor.axon_being_grown(&world).unwrap();
    assert!(world.get::<&Compartment>(from).is_ok());
    assert_eq!((start, end), (at(0.0, 2.5), at(0.0, 3.5)));
    assert_eq!(editor.connection_being_drawn(&world), None);
    assert_eq!(editor.painting(), None);
}

#[test]
fn a_plain_connection_is_not_a_growing_axon() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.edit(&mut world, at(0.0, 5.0), false);
    assert_eq!(editor.axon_being_grown(&world), None);
}

#[test]
fn inputs_connect_to_the_neuron_they_are_pulled_to() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Place(Element::CurrentSource));
    let path: Vec<Vec3> = (0..=20).map(|step| at(0.0, step as f32 * 0.5)).collect();
    drag(&mut editor, &mut world, &path);
    let sources = positions_with::<CurrentClamp>(&world);
    assert_eq!(sources.len(), 1);
    assert!(connected(&world, sources[0].0, target));
}

#[test]
fn with_compartments_dragging_from_a_neuron_while_adding_grows_an_axon() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    let path: Vec<Vec3> = (0..=40).map(|step| at(0.0, step as f32 * 0.5)).collect();
    editor.edit(&mut world, path[0], false);
    editor.edit(&mut world, path[10], false);
    assert_eq!(editor.connection_being_drawn(&world), None);
    drag(&mut editor, &mut world, &path[10..]);

    assert!(chain(&world, source, target).unwrap().len() > 3);
    assert_eq!(count::<CurrentSynapse>(&world), 0);
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn with_compartments_dragging_from_near_a_dendrite_grows_an_axon_from_it() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let (_, tip) = positions_with::<Dendrite>(&world)
        .into_iter()
        .max_by(|(_, a), (_, b)| a.length().total_cmp(&b.length()))
        .unwrap();
    let outwards = tip.normalize();
    let path: Vec<Vec3> = (0..=20)
        .map(|step| tip + outwards * (1.5 + step as f32 * 0.5))
        .collect();
    drag(&mut editor, &mut world, &path);

    assert_eq!(count::<LeakyNeuron>(&world), 1);
    let grown_from_dendrite =
        world
            .query::<&Connection>()
            .with::<&Cable>()
            .iter()
            .any(|(_, cable)| {
                world.get::<&Dendrite>(cable.from).is_ok()
                    && world.get::<&Dendrite>(cable.to).is_err()
                    && world.get::<&Compartment>(cable.to).is_ok()
            });
    assert!(grown_from_dendrite);
}

#[test]
fn with_point_neurons_pressing_near_a_dendrite_adds_nothing() {
    let mut world = World::new();
    neuron_with_dendrites(&mut world, at(0.0, 0.0));
    let (_, tip_position) = positions_with::<Dendrite>(&world)
        .into_iter()
        .max_by(|(_, a), (_, b)| a.length().total_cmp(&b.length()))
        .unwrap();
    let beside_tip = tip_position * (1.0 + 1.5 / tip_position.length());
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, beside_tip, false);
    editor.release(&world);
    assert_eq!(count::<LeakyNeuron>(&world), 1);
    assert_eq!(editor.selection, None);
}

#[test]
fn with_compartments_inputs_still_connect_directly() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Place(Element::RegularSpikeGenerator));
    editor.compartments = true;
    let path: Vec<Vec3> = (0..=20).map(|step| at(0.0, step as f32 * 0.5)).collect();
    drag(&mut editor, &mut world, &path);
    let generators = positions_with::<RegularSpikeGenerator>(&world);
    assert!(connected(&world, generators[0].0, target));
    assert_eq!(count::<Compartment>(&world), 0);
}

fn closest_neighbouring_trees(world: &World) -> f32 {
    let trees: Vec<Vec<Vec3>> = positions_with::<LeakyNeuron>(world)
        .into_iter()
        .map(|(soma, soma_position)| {
            compartment_tree(world, soma)
                .into_iter()
                .map(|dendrite| position(world, dendrite))
                .chain([soma_position])
                .collect()
        })
        .collect();
    let mut closest = f32::INFINITY;
    for (index, tree) in trees.iter().enumerate() {
        for other in &trees[index + 1..] {
            for a in tree {
                for b in other {
                    closest = closest.min(a.distance(*b));
                }
            }
        }
    }
    closest
}

#[test]
fn painted_neurons_keep_their_dendrites_apart() {
    for (element, direction) in [
        (Element::ExcitatoryNeuron, at(0.0, 1.0)),
        (Element::ExcitatoryNeuron, at(1.0, 0.0)),
        (Element::ExcitatoryNeuron, at(-1.0, 0.0)),
        (Element::ExcitatoryNeuron, at(1.0, 1.0).normalize()),
        (Element::InhibitoryNeuron, at(0.0, 1.0)),
    ] {
        let mut world = World::new();
        let mut editor = editor(Tool::Place(element));
        editor.compartments = true;
        let path: Vec<Vec3> = (0..=200)
            .map(|step| direction * step as f32 * 0.5)
            .collect();
        drag(&mut editor, &mut world, &path);
        assert!(count::<LeakyNeuron>(&world) >= 3, "{element:?} {direction}");
        let closest = closest_neighbouring_trees(&world);
        assert!(
            closest > SEGMENT_LENGTH,
            "{element:?} {direction}: {closest}"
        );
    }
}

#[test]
fn tapping_an_element_while_adding_does_nothing() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.5, 0.0), false);
    assert!(!editor.pressed_background());
    editor.release(&world);
    assert_eq!(editor.selection, None);
    assert_eq!(editor.tool, Tool::Place(Element::ExcitatoryNeuron));
    assert_eq!(count::<LeakyNeuron>(&world), 1);
}

fn offer_from_neuron(world: &mut World) -> (Entity, Editor) {
    let a = neuron(world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(world, at(0.5, 0.0), false);
    editor.offer_connection(10.0);
    editor.release(world);
    (a, editor)
}

#[test]
fn tapping_a_neuron_while_adding_offers_a_connection_for_two_seconds() {
    let mut world = World::new();
    let (a, mut editor) = offer_from_neuron(&mut world);
    let hint = editor.connection_hint(&world, 11.0).unwrap();
    let bead = hint.string[HINT_SEGMENTS];
    assert_eq!(hint.string[0], position(&world, a));
    assert_eq!(bead.y, HINT_LENGTH);
    assert!((hint.string[HINT_SEGMENTS + 1].distance(bead) - TERMINAL_INSET).abs() < 1e-5);
    assert!(hint.label.x > bead.x);
    assert_eq!(hint.shown, 1.0);
    let later = editor.connection_hint(&world, 11.3).unwrap();
    assert!(later.string[HINT_SEGMENTS].distance(bead) > 0.1);
    assert_eq!(later.label, hint.label);
    assert_eq!(editor.connection_hint(&world, 12.0), None);
    assert_eq!(editor.connection_hint(&world, 11.0), None);
}

#[test]
fn placing_an_input_offers_no_connection() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::CurrentSource));
    editor.edit(&mut world, at(0.0, 0.0), false);
    editor.offer_connection(10.0);
    editor.release(&world);
    assert_eq!(editor.connection_hint(&world, 11.0), None);
}

#[test]
fn tapping_a_meter_with_a_meter_tool_offers_a_wire_to_pull_onto_a_neuron() {
    let mut world = World::new();
    let target = neuron(&mut world, at(10.0, 10.0));
    let speaker = Meter::Loudspeaker.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Loudspeaker);
    editor.edit(&mut world, at(0.3, 0.0), false);
    editor.offer_connection(10.0);
    editor.lift(&mut world);
    let hint = editor.connection_hint(&world, 11.0).unwrap();
    assert_eq!(hint.string[0], at(0.0, 0.0));
    assert!(measured(&world, speaker).is_empty());

    drag(&mut editor, &mut world, &[at(3.0, 0.0), at(10.0, 10.0)]);
    assert!(connected(&world, target, speaker));
    assert_eq!(editor_meters(&world).len(), 1);
}

#[test]
fn placing_a_meter_or_tapping_a_neuron_with_a_meter_tool_offers_no_wire() {
    for press in [at(20.0, 20.0), at(0.3, 0.0)] {
        let mut world = World::new();
        neuron(&mut world, at(0.0, 0.0));
        let mut editor = editor(Tool::Voltmeter);
        editor.edit(&mut world, press, false);
        editor.offer_connection(10.0);
        editor.lift(&mut world);
        assert_eq!(editor.connection_hint(&world, 11.0), None, "{press}");
        assert_eq!(editor_meters(&world).len(), 1, "{press}");
    }
}

#[test]
fn pulling_the_offered_connection_connects() {
    let mut world = World::new();
    let (a, mut editor) = offer_from_neuron(&mut world);
    let b = neuron(&mut world, at(10.0, 10.0));
    drag(&mut editor, &mut world, &[at(3.0, 0.0), at(10.0, 10.0)]);
    assert!(connected(&world, a, b));
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn pressing_where_the_hint_was_adds_once_it_is_gone() {
    let mut world = World::new();
    let (_, mut editor) = offer_from_neuron(&mut world);
    assert_eq!(editor.connection_hint(&world, 12.5), None);
    drag(&mut editor, &mut world, &[at(3.0, 0.0)]);
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn holding_an_element_while_adding_moves_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 1.5), false);
    assert!(editor.hold(&world, at(0.0, 1.5), false));
    assert_eq!(editor.connection_being_drawn(&world), None);
    drag(&mut editor, &mut world, &[at(0.0, 10.0), at(5.0, 3.5)]);
    assert_eq!(position(&world, a), at(5.0, 2.0));
    assert_eq!(editor.selection, None);
    assert_eq!(editor.tool, Tool::Place(Element::ExcitatoryNeuron));
    assert!(!connected(&world, a, b));
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn holding_after_placing_moves_the_new_element() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::CurrentSource));
    editor.edit(&mut world, at(0.0, 0.0), false);
    assert!(editor.hold(&world, at(0.0, 0.0), false));
    drag(&mut editor, &mut world, &[at(3.0, 4.0), at(6.0, 8.0)]);
    let placed = positions_with::<CurrentClamp>(&world);
    assert_eq!(placed[0].1, at(6.0, 8.0));
    assert_eq!(count::<CurrentClamp>(&world), 1);
}

#[test]
fn undo_puts_back_an_element_moved_while_adding() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 0.5), false);
    assert!(editor.hold(&world, at(0.0, 0.5), false));
    drag(&mut editor, &mut world, &[at(5.0, 3.5)]);
    assert_eq!(position(&world, a), at(5.0, 3.0));
    editor.undo(&mut world);
    assert_eq!(position(&world, a), at(0.0, 0.0));
    assert!(!editor.can_undo());
}

#[test]
fn erasing_removes_what_is_under_the_pointer() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let a = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);

    editor.tool = Tool::Erase;
    drag(&mut editor, &mut world, &[at(0.0, 5.0)]);
    assert!(!connected(&world, a, b));
    assert_eq!(count::<LeakyNeuron>(&world), 2);

    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    assert_eq!(count::<LeakyNeuron>(&world), 1);
    assert_eq!(count::<Dendrite>(&world), 0);
    assert_eq!(count::<Connection>(&world), 0);
}

#[test]
fn erasing_leaves_connections_that_are_hidden() {
    let mut world = World::new();
    let mut editor = editor(Tool::Erase);
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);

    ConnectionView::None.set_in(&mut world);
    drag(&mut editor, &mut world, &[at(0.0, 5.0)]);
    assert!(connected(&world, a, b));

    ConnectionView::All.set_in(&mut world);
    drag(&mut editor, &mut world, &[at(0.0, 5.0)]);
    assert!(!connected(&world, a, b));
}

#[test]
fn erasing_the_selection_takes_its_dendrites_and_connections_but_leaves_its_voltmeter() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let a = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);
    let voltmeter = spawn_voltmeter(&mut world, a, at(1.0, 0.0));

    editor.selection = Some(a);
    editor.erase_selection(&mut world);
    assert_eq!(editor.selection, None);
    assert!(world.contains(b));
    assert_eq!(count::<LeakyNeuron>(&world), 1);
    assert_eq!(count::<Dendrite>(&world), 0);
    assert_eq!(count::<Connection>(&world), 0);
    assert!(world.contains(voltmeter));
}

#[test]
fn tapping_a_neuron_with_a_meter_tool_places_a_meter_beside_it_measuring_it() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Voltmeter);
    drag(&mut editor, &mut world, &[at(0.5, 0.0)]);
    let (voltmeter, position) = world
        .query::<&Position>()
        .with::<&Voltmeter>()
        .iter()
        .map(|(entity, position)| (entity, position.position))
        .next()
        .unwrap();
    assert_eq!(position, at(1.0, 0.0));
    assert!(connected(&world, target, voltmeter));
}

#[test]
fn a_meter_tool_places_an_empty_meter_centered_where_it_is_pressed() {
    for tool in [
        Tool::Voltmeter,
        Tool::SpikeDetector,
        Tool::RatePlot,
        Tool::Loudspeaker,
    ] {
        let mut world = World::new();
        neuron(&mut world, at(0.0, 0.0));
        let mut editor = editor(tool);
        drag(&mut editor, &mut world, &[at(10.0, 10.0)]);
        let meter = editor_meters(&world);
        assert_eq!(meter.len(), 1, "{tool:?}");
        let (meter, _) = meter[0];
        assert!(measured(&world, meter).is_empty(), "{tool:?}");
        match Frame::of(&world, meter) {
            Some(frame) => assert!(frame.point(0.5, 0.5).distance(at(10.0, 10.0)) < 1e-4),
            None => assert_eq!(
                world.get::<&Position>(meter).unwrap().position,
                at(10.0, 10.0)
            ),
        }
    }
}

fn editor_meters(world: &World) -> Vec<(Entity, Meter)> {
    world
        .iter()
        .filter_map(|entity| Some((entity.entity(), Meter::of(world, entity.entity())?)))
        .collect()
}

#[test]
fn dragging_from_a_meter_to_a_neuron_measures_it_once_per_drag() {
    for tool in [Tool::Loudspeaker, Tool::StaticConnection] {
        let mut world = World::new();
        let first = neuron(&mut world, at(0.0, 0.0));
        let second = neuron(&mut world, at(0.0, -10.0));
        let speaker = Meter::Loudspeaker.spawn(&mut world, at(10.0, 0.0));
        let mut editor = editor(tool);
        drag(
            &mut editor,
            &mut world,
            &[at(10.0, 0.3), at(5.0, 0.0), at(0.3, 0.0), at(0.0, -9.8)],
        );
        let wired: Vec<Entity> = measured(&world, speaker)
            .into_iter()
            .map(|(_, target)| target)
            .collect();
        assert_eq!(wired, [first], "{tool:?}");

        drag(&mut editor, &mut world, &[at(10.0, 0.3), at(0.0, -9.8)]);
        let wired: Vec<Entity> = measured(&world, speaker)
            .into_iter()
            .map(|(_, target)| target)
            .collect();
        assert_eq!(wired, [first, second], "{tool:?}");
        assert_eq!(editor_meters(&world).len(), 1, "{tool:?}");
        assert_eq!(count::<CurrentSynapse>(&world), 0, "{tool:?}");
    }
}

#[test]
fn dragging_from_a_neuron_onto_a_meter_with_a_meter_tool_measures_it_without_a_new_meter() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let voltmeter = Meter::Voltmeter.spawn(&mut world, at(0.0, 10.0));
    let inside = Frame::of(&world, voltmeter).unwrap().point(0.5, 0.5);
    let mut editor = editor(Tool::SpikeDetector);
    drag(
        &mut editor,
        &mut world,
        &[at(0.3, 0.0), at(0.0, 5.0), inside],
    );
    assert!(connected(&world, target, voltmeter));
    assert_eq!(editor_meters(&world).len(), 1);
}

#[test]
fn dragging_from_a_neuron_to_an_empty_spot_places_the_meter_there() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::RatePlot);
    drag(
        &mut editor,
        &mut world,
        &[at(0.3, 0.0), at(0.0, 8.0), at(0.0, 15.0)],
    );
    let meters = editor_meters(&world);
    assert_eq!(meters.len(), 1);
    let (plot, _) = meters[0];
    assert!(connected(&world, target, plot));
    let center = Frame::of(&world, plot).unwrap().point(0.5, 0.5);
    assert!(center.distance(at(0.0, 15.0)) < 1e-4);
}

#[test]
fn placing_a_meter_and_dragging_on_to_a_neuron_measures_it() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Voltmeter);
    drag(
        &mut editor,
        &mut world,
        &[at(0.0, 15.0), at(0.0, 8.0), at(0.0, 0.5)],
    );
    let meters = editor_meters(&world);
    assert_eq!(meters.len(), 1);
    assert!(connected(&world, target, meters[0].0));
}

#[test]
fn dragging_moves_the_element_under_the_pointer_without_selecting_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    let pans = drag(&mut editor, &mut world, &[at(0.5, 0.0), at(5.5, 3.0)]);
    assert_eq!(pans, [None, None]);
    assert_eq!(editor.selection, None);
    assert_eq!(world.get::<&Position>(a).unwrap().position, at(5.0, 3.0));
}

#[test]
fn letting_go_of_an_element_without_moving_selects_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.edit(&mut world, at(0.5, 0.0), false);
    assert_eq!(editor.selection, None);
    editor.lift(&mut world);
    assert_eq!(editor.selection, Some(a));
}

#[test]
fn dragging_another_element_moves_the_selection_to_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(a);
    drag(&mut editor, &mut world, &[at(0.5, 10.0), at(3.5, 12.0)]);
    assert_eq!(editor.selection, Some(b));
    assert_eq!(position(&world, b), at(3.0, 12.0));
}

#[test]
fn dragging_empty_space_pans_and_keeps_the_selection() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(a);
    let pans = drag(&mut editor, &mut world, &[at(10.0, 10.0), at(12.0, 10.0)]);
    assert_eq!(pans, [None, Some(Pan(at(2.0, 0.0)))]);
    assert_eq!(editor.selection, Some(a));
}

#[test]
fn the_editor_knows_whether_a_press_landed_on_the_background() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.edit(&mut world, at(0.5, 0.0), false);
    assert!(!editor.pressed_background());
    editor.release(&world);
    editor.edit(&mut world, at(10.0, 10.0), false);
    assert!(editor.pressed_background());
    editor.release(&world);
    assert!(!editor.pressed_background());
}

#[test]
fn voltmeters_are_dragged_by_their_frame_and_resized_by_their_corners() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let voltmeter = spawn_voltmeter(&mut world, target, at(10.0, 5.0));
    let mut editor = editor(Tool::Edit);

    drag(&mut editor, &mut world, &[at(12.0, 13.0), at(13.0, 15.0)]);
    let size = world
        .get::<&MeterSize>(voltmeter)
        .map(|size| (size.width, size.height))
        .unwrap();
    assert_eq!(size, (10.0, 5.0));
    assert_eq!(
        world.get::<&Position>(voltmeter).unwrap().position,
        at(10.5, 5.0)
    );

    drag(&mut editor, &mut world, &[at(10.0, 9.0), at(11.0, 9.0)]);
    assert_eq!(
        world.get::<&Position>(voltmeter).unwrap().position,
        at(11.5, 5.0)
    );
}

#[test]
fn selected_notes_are_resized_by_their_corners() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    let size = |world: &World| {
        world
            .get::<&NoteSize>(note)
            .map(|size| (size.width, size.height))
            .unwrap()
    };
    let corners = |world: &World| {
        note_corners(
            world.get::<&Position>(note).unwrap().position,
            &world.get::<&NoteSize>(note).unwrap(),
        )
    };
    let [bottom_left, top_right] = corners(&world);

    drag(
        &mut editor,
        &mut world,
        &[top_right, top_right + at(1.0, 1.0)],
    );
    assert_eq!(size(&world), (7.2, 4.8));
    assert_eq!(editor.selection, None);

    let [bottom_left, top_right] = [bottom_left + at(1.0, 1.0), top_right + at(1.0, 1.0)];
    drag(&mut editor, &mut world, &[top_right]);
    assert_eq!(editor.selection, Some(note));
    let outside = top_right + at(0.3, 0.3);
    drag(&mut editor, &mut world, &[outside, outside + at(2.0, 0.8)]);
    let [resized_bottom_left, resized_top_right] = corners(&world);
    assert!(resized_bottom_left.abs_diff_eq(bottom_left, 1e-5));
    assert!(resized_top_right.abs_diff_eq(top_right + at(2.0, 0.8), 1e-5));
}

#[test]
fn notes_lie_under_the_circuit() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let inside = neuron(&mut world, at(1.0, 1.0));
    let mut editor = editor(Tool::Edit);
    drag(&mut editor, &mut world, &[at(1.2, 1.0)]);
    assert_eq!(editor.selection, Some(inside));
    drag(&mut editor, &mut world, &[at(-1.5, -2.0)]);
    assert_eq!(editor.selection, Some(note));
}

#[test]
fn stimulating_pushes_with_the_chosen_current() {
    let mut world = World::new();
    let mut editor = editor(Tool::Stimulate);
    editor.stimulation.current = 1.0;
    editor.edit(&mut world, at(0.0, 0.0), false);
    assert_eq!(editor.touch().map(|touch| touch.current), Some(1.0));
}

#[test]
fn viewers_touch_sensors_and_pan_everywhere_else() {
    let mut world = World::new();
    Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);

    assert_eq!(editor.view(&mut world, at(0.5, 0.0)), None);
    assert_eq!(editor.view(&mut world, at(5.0, 5.0)), None);
    assert_eq!(
        editor.touch().map(|touch| touch.position),
        Some(at(5.0, 5.0))
    );
    editor.release(&world);

    assert_eq!(editor.view(&mut world, at(10.0, 10.0)), None);
    assert_eq!(
        editor.view(&mut world, at(11.0, 10.0)),
        Some(Pan(at(1.0, 0.0)))
    );
    assert_eq!(editor.touch(), None);
}

#[test]
fn touch_sensors_fire_once_per_press() {
    let mut world = World::new();
    let sensor = Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Stimulate);
    let fired = |world: &mut World, editor: &mut Editor| {
        editor.edit(world, at(0.5, 0.0), false);
        let presses = editor.take_presses();
        presses.iter().any(|&press| {
            touch_sensor_near(world, press) && press.distance(position(world, sensor)) < 1.0
        })
    };

    assert!(fired(&mut world, &mut editor));
    assert!(!fired(&mut world, &mut editor));
    editor.release(&world);
    assert!(fired(&mut world, &mut editor));
}

#[test]
fn viewers_select_with_a_tap() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::View);

    editor.view(&mut world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    editor.release(&world);
    assert_eq!(editor.selection, Some(target));
    assert_eq!(editor.tool, Tool::View);

    editor.view(&mut world, at(10.0, 10.0));
    editor.tap(&world, at(10.0, 10.0));
    editor.release(&world);
    assert_eq!(editor.selection, None);
}

#[test]
fn touching_a_sensor_leaves_the_selection_alone() {
    let mut world = World::new();
    let target = neuron(&mut world, at(10.0, 0.0));
    Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(target);

    editor.view(&mut world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    assert_eq!(editor.selection, Some(target));
}

#[test]
fn holding_an_element_grabs_it() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::View);

    editor.view(&mut world, at(10.0, 10.0));
    assert!(!editor.hold(&world, at(10.0, 10.0), true));
    editor.release(&world);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0), true));
    assert_eq!(editor.tool, Tool::Edit);
    assert_eq!(editor.edit(&mut world, at(3.5, 2.0), false), None);
    assert_eq!(
        world.get::<&Position>(target).unwrap().position,
        at(3.0, 2.0)
    );
}

#[test]
fn a_hold_let_go_without_moving_selects() {
    let mut world = World::new();
    let earlier = neuron(&mut world, at(0.0, 10.0));
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::View);
    editor.selection = Some(earlier);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0), true));
    assert_eq!(editor.selection, Some(earlier));
    editor.lift(&mut world);
    assert_eq!(editor.selection, Some(target));
}

#[test]
fn a_hold_that_moves_selects_nothing() {
    let mut world = World::new();
    let earlier = neuron(&mut world, at(0.0, 10.0));
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::View);
    editor.selection = Some(earlier);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0), true));
    editor.move_held();
    assert_eq!(editor.selection, None);
    drag(&mut editor, &mut world, &[at(3.5, 2.0)]);
    assert_eq!(position(&world, target), at(3.0, 2.0));
    assert_eq!(editor.selection, None);
}

#[test]
fn a_drag_after_a_hold_that_moved_selects_nothing() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::View);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0), true));
    drag(&mut editor, &mut world, &[at(0.5, 0.0), at(3.5, 2.0)]);
    drag(&mut editor, &mut world, &[at(0.5, 10.0), at(3.5, 12.0)]);
    assert_eq!(position(&world, first), at(3.0, 2.0));
    assert_eq!(position(&world, second), at(3.0, 12.0));
    assert_eq!(editor.selection, None);
}

#[test]
fn a_cancelled_hold_selects_nothing() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::View);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0), true));
    editor.release(&world);
    editor.lift(&mut world);
    assert_eq!(editor.selection, None);
}

fn grow_axon(world: &mut World, from: Vec3, to: Vec3) {
    let steps = (from.distance(to) / 0.5) as usize;
    let path: Vec<Vec3> = (0..=steps)
        .map(|step| from.lerp(to, step as f32 / steps as f32))
        .collect();
    drag(&mut editor(Tool::Axon), world, &path);
}

fn drag_in_steps(world: &mut World, from: Vec3, to: Vec3) {
    let path: Vec<Vec3> = (0..=50)
        .map(|step| from.lerp(to, step as f32 / 50.0))
        .collect();
    drag(&mut editor(Tool::Edit), world, &path);
}

fn position(world: &World, entity: Entity) -> Vec3 {
    world.get::<&Position>(entity).unwrap().position
}

fn cables_out_of(world: &World, from: Entity) -> Vec<Entity> {
    world
        .query::<&Connection>()
        .with::<hecs::Or<&Cable, &ConductanceSynapse>>()
        .iter()
        .filter(|(_, cable)| cable.from == from)
        .map(|(_, cable)| cable.to)
        .collect()
}

fn chain(world: &World, from: Entity, to: Entity) -> Option<Vec<Entity>> {
    if from == to {
        return Some(vec![to]);
    }
    cables_out_of(world, from).into_iter().find_map(|next| {
        let mut rest = chain(world, next, to)?;
        rest.insert(0, from);
        Some(rest)
    })
}

fn evenly_spaced(world: &World, chain: &[Entity]) -> bool {
    spaced_by(world, chain, SEGMENT_LENGTH)
}

fn spaced_by(world: &World, chain: &[Entity], spacing: f32) -> bool {
    chain.windows(2).all(|pair| {
        let length = position(world, pair[0]).distance(position(world, pair[1]));
        (length - spacing).abs() < 0.25 * spacing
    })
}

fn grow_myelinated_axon(world: &mut World, from: Vec3, to: Vec3) {
    let steps = (from.distance(to) / 0.5) as usize;
    let path: Vec<Vec3> = (0..=steps)
        .map(|step| from.lerp(to, step as f32 / steps as f32))
        .collect();
    drag(&mut editor(Tool::MyelinatedAxon), world, &path);
}

fn myelinated(world: &World, from: Entity, to: Entity) -> bool {
    world
        .query::<&Connection>()
        .with::<&Myelin>()
        .iter()
        .any(|(_, cable)| cable.from == from && cable.to == to)
}

fn wrapped_and_bare(world: &World, nodes: &[Entity]) -> (Vec<Entity>, Vec<Entity>) {
    let internodes = nodes
        .windows(2)
        .take_while(|pair| myelinated(world, pair[0], pair[1]))
        .count();
    (nodes[..=internodes].to_vec(), nodes[internodes..].to_vec())
}

#[test]
fn myelinated_axons_lay_their_nodes_an_internode_apart() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 40.0));
    grow_myelinated_axon(&mut world, at(0.0, 0.0), at(0.0, 40.0));
    let nodes = chain(&world, source, target).unwrap();
    let (wrapped, _) = wrapped_and_bare(&world, &nodes);
    assert!(wrapped.len() > 3);
    assert!(spaced_by(&world, &wrapped, INTERNODE_LENGTH));
    assert_eq!(count::<Myelin>(&world), wrapped.len() - 1);
    assert_eq!(axon_endings(&world).len(), 1);
}

#[test]
fn myelinated_axons_end_in_a_bare_terminal() {
    for path in [
        vec![at(0.0, 0.0), at(0.0, 40.0)],
        vec![at(0.0, 0.0), at(0.0, 20.0), at(0.0, 40.0)],
    ] {
        let mut world = World::new();
        let source = neuron(&mut world, at(0.0, 0.0));
        let target = neuron(&mut world, at(0.0, 40.0));
        drag(&mut editor(Tool::MyelinatedAxon), &mut world, &path);
        let nodes = chain(&world, source, target).unwrap();
        let (_, bare) = wrapped_and_bare(&world, &nodes);
        let terminal = position(&world, bare[0]).distance(position(&world, target));
        assert!(bare.len() > 1);
        assert!(spaced_by(&world, &bare, SEGMENT_LENGTH));
        assert!(terminal < INTERNODE_LENGTH + SEGMENT_LENGTH, "{terminal}");
    }
}

#[test]
fn stretched_myelinated_axons_keep_their_internodes() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 40.0));
    grow_myelinated_axon(&mut world, at(0.0, 0.0), at(0.0, 40.0));
    let grown = chain(&world, source, target).unwrap().len();

    drag_in_steps(&mut world, at(0.0, 0.0), at(0.0, -40.0));

    let stretched = chain(&world, source, target).unwrap();
    assert!(
        stretched.len() > grown,
        "{grown} became {}",
        stretched.len()
    );
    let (wrapped, bare) = wrapped_and_bare(&world, &stretched);
    assert!(spaced_by(&world, &wrapped, INTERNODE_LENGTH));
    assert!(spaced_by(&world, &bare, SEGMENT_LENGTH));
    assert_eq!(count::<Myelin>(&world), wrapped.len() - 1);
}

fn steps_until_a_spike_crosses(tool: Tool, length: usize) -> usize {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, length as f32));
    let path: Vec<Vec3> = (0..=2 * length)
        .map(|step| at(0.0, step as f32 * 0.5))
        .collect();
    drag(&mut editor(tool), &mut world, &path);
    let (mut engine, mut time) = (Engine::default(), 0.0);
    engine.sync(&mut world);
    engine.fire(&world, source);
    (0..8000)
        .position(|_| {
            step_simulation(&mut world, &mut engine, &mut time, 1, None);
            engine.fired(target)
        })
        .unwrap()
}

#[test]
fn spikes_travel_ten_times_as_fast_along_myelinated_axons() {
    let extra =
        |tool| steps_until_a_spike_crosses(tool, 96) - steps_until_a_spike_crosses(tool, 48);
    let plain = extra(Tool::Axon);
    let myelinated = extra(Tool::MyelinatedAxon);
    assert!(
        10 * myelinated < plain,
        "{myelinated} against {plain} steps"
    );
}

fn axon_between_two_neurons() -> (World, Entity) {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    neuron(&mut world, at(0.0, 10.0));
    let path: Vec<Vec3> = (0..=20).map(|step| at(0.0, step as f32 * 0.5)).collect();
    drag(&mut editor(Tool::Axon), &mut world, &path);
    let compartment = world
        .query::<()>()
        .with::<&Compartment>()
        .iter()
        .next()
        .unwrap()
        .0;
    (world, compartment)
}

fn membranes(world: &World) -> Vec<Membrane> {
    world
        .query::<&Membrane>()
        .iter()
        .map(|(_, membrane)| membrane.clone())
        .collect()
}

#[test]
fn switching_the_channels_is_undone_in_one_step() {
    let (mut world, _) = axon_between_two_neurons();
    let squid = membranes(&world);
    let mut editor = editor(Tool::Edit);
    editor.prepare(&mut world, Preparation::Mammal);
    let mammal = membranes(&world);
    assert_ne!(mammal, squid);
    assert_eq!(
        Temperature::of(&world).celsius,
        Preparation::Mammal.celsius()
    );

    editor.undo(&mut world);
    assert_eq!(membranes(&world), squid);
    assert_eq!(Preparation::of(&world), Preparation::Squid);
    assert_eq!(
        Temperature::of(&world).celsius,
        Preparation::Squid.celsius()
    );

    editor.redo(&mut world);
    assert_eq!(membranes(&world), mammal);
    assert_eq!(Preparation::of(&world), Preparation::Mammal);
}

#[test]
fn a_running_axon_rests_at_the_new_resting_potential_after_a_switch() {
    let (mut world, compartment) = axon_between_two_neurons();
    let (mut engine, mut time) = (Engine::default(), 0.0);
    step_simulation(&mut world, &mut engine, &mut time, 40, None);
    let mut editor = editor(Tool::Edit);
    for preparation in [Preparation::Mammal, Preparation::Squid] {
        editor.prepare(&mut world, preparation);
        step_simulation(&mut world, &mut engine, &mut time, 1, None);
        let voltage = world.get::<&NodeState>(compartment).unwrap().voltage;
        let rest = preparation.resting_potential();
        assert!(
            (voltage - rest).abs() < 1.0,
            "{voltage} mV against {rest} mV"
        );
    }
}

#[test]
fn dendrites_move_along_with_a_dragged_neuron() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let before: Vec<(Entity, Vec3)> = positions_with::<Dendrite>(&world);

    drag_in_steps(&mut world, at(0.0, 0.5), at(3.0, 4.5));

    for (dendrite, previous) in before {
        let moved = position(&world, dendrite) - previous;
        assert!(moved.distance(at(3.0, 4.0)) < 1e-4, "moved {moved}");
    }
}

#[test]
fn stretched_axons_grow_compartments() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    let grown = chain(&world, source, target).unwrap().len();

    drag_in_steps(&mut world, at(0.0, 0.0), at(0.0, -30.0));

    let stretched = chain(&world, source, target).unwrap();
    assert!(
        stretched.len() > grown,
        "{grown} became {}",
        stretched.len()
    );
    assert!(evenly_spaced(&world, &stretched));
    let along: Vec<f32> = stretched.iter().map(|&c| position(&world, c).z).collect();
    assert!(along.windows(2).all(|pair| pair[0] < pair[1]), "{along:?}");
}

#[test]
fn axons_drawn_fast_fill_the_gaps_with_compartments() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    let path = [at(0.0, 0.5), at(0.0, 12.0), at(0.0, 29.5)];
    drag(&mut editor(Tool::Axon), &mut world, &path);

    let axon = chain(&world, source, target).unwrap();
    assert_eq!(axon.len(), 16);
    assert!(evenly_spaced(&world, &axon));
}

#[test]
fn compressed_axons_lose_compartments_but_keep_one() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));

    drag_in_steps(&mut world, at(0.0, 0.0), at(0.0, 18.5));

    let compressed = chain(&world, source, target).unwrap();
    assert_eq!(compressed.len(), 3);
    assert!(world.get::<&Compartment>(compressed[1]).is_ok());
    let into_target = world
        .query::<&Connection>()
        .iter()
        .find(|(_, cable)| cable.to == target)
        .map(|(_, cable)| cable.directional);
    assert_eq!(into_target, Some(true));
}

#[test]
fn compressing_keeps_branches_and_measured_compartments() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    let grown = chain(&world, source, target).unwrap();
    let (branching, measured) = (grown[3], grown[8]);
    let branching_at = position(&world, branching);
    grow_axon(&mut world, branching_at, branching_at + at(10.0, 0.0));
    spawn_voltmeter(&mut world, measured, at(10.0, 10.0));

    drag_in_steps(&mut world, at(0.0, 0.0), at(0.0, 8.0));

    let compressed = chain(&world, source, target).unwrap();
    assert!(compressed.len() < grown.len());
    assert!(compressed.contains(&branching) && compressed.contains(&measured));
    assert!(evenly_spaced(&world, &compressed));
}

#[test]
fn pulling_a_compartment_next_to_the_soma_lengthens_its_dendrite_by_the_pull() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    spawn_dendrites(&mut world, soma);
    let reach = |world: &World| {
        compartment_tree(world, soma)
            .iter()
            .map(|&dendrite| position(world, dendrite).length())
            .fold(0.0, f32::max)
    };
    let reached = reach(&world);
    let (first, first_at) = positions_with::<Dendrite>(&world)
        .into_iter()
        .min_by(|(_, a), (_, b)| a.length().total_cmp(&b.length()))
        .unwrap();
    let pulled = first_at + 6.0 * first_at.normalize();

    drag_in_steps(&mut world, first_at, pulled);

    assert!(position(&world, first).distance(pulled) < 1e-3);
    assert!(evenly_spaced(&world, &chain(&world, soma, first).unwrap()));
    assert!(reach(&world) < reached + 6.5, "reached {}", reach(&world));
}

#[test]
fn stretching_axons_leave_the_neurons_between_them_in_place() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let middle = neuron(&mut world, at(0.0, 20.0));
    neuron(&mut world, at(0.0, 40.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 20.0), at(0.0, 40.0));

    drag_in_steps(&mut world, at(0.0, 0.0), at(0.0, -10.0));

    assert_eq!(position(&world, middle), at(0.0, 20.0));
}

#[test]
fn a_dragged_compartment_pulls_its_axon_longer() {
    let mut world = World::new();
    let soma = neuron(&mut world, at(0.0, 0.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 6.0));
    let (tip, _) = positions_with::<Compartment>(&world)
        .into_iter()
        .find(|&(compartment, _)| cables_out_of(&world, compartment).is_empty())
        .unwrap();
    let tip_at = position(&world, tip);

    drag_in_steps(&mut world, tip_at, tip_at + at(0.0, 20.0));

    let pulled = chain(&world, soma, tip).unwrap();
    assert!(pulled.len() > 6, "{}", pulled.len());
    assert!(evenly_spaced(&world, &pulled));
}

fn neuron_with_dendrites(world: &mut World, position: Vec3) -> Entity {
    let soma = neuron(world, position);
    spawn_dendrites(world, soma);
    soma
}

fn bearing(world: &World, soma: Entity, towards: Entity, of: Entity) -> (f32, f32) {
    let flat = |offset: Vec3| glam::Vec2::new(offset.x, offset.z);
    let towards = flat(position(world, towards) - position(world, soma));
    let offset = flat(position(world, of) - position(world, soma));
    (towards.angle_to(offset), offset.length())
}

#[test]
fn dendrites_turn_to_keep_facing_the_neuron_they_connect_to() {
    let mut world = World::new();
    let source = neuron_with_dendrites(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 30.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 30.0));
    let dendrites: Vec<Entity> = positions_with::<Dendrite>(&world)
        .into_iter()
        .map(|(dendrite, _)| dendrite)
        .collect();
    let before: Vec<_> = dendrites
        .iter()
        .map(|&dendrite| bearing(&world, source, target, dendrite))
        .collect();

    drag_in_steps(&mut world, at(0.0, 0.0), at(15.0, 0.0));

    for (&dendrite, (angle, distance)) in dendrites.iter().zip(before) {
        let (turned_angle, turned_distance) = bearing(&world, source, target, dendrite);
        assert!(
            (turned_angle - angle).abs() < 1e-3,
            "{angle} became {turned_angle}"
        );
        assert!((turned_distance - distance).abs() < 1e-3);
    }
}

#[test]
fn swinging_around_a_neuron_moves_nothing_far_in_one_step() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 30.0));
    neuron_with_dendrites(&mut world, at(0.0, 0.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 30.0));
    let mut editor = editor(Tool::Edit);
    let centre = position(&world, target);

    let mut previous = positions_with::<Compartment>(&world);
    for step in 0..=300 {
        let swing = 1.5 * std::f32::consts::PI * step as f32 / 300.0;
        editor.edit(
            &mut world,
            centre + 30.0 * at(swing.sin(), -swing.cos()),
            false,
        );
        let current = positions_with::<Compartment>(&world);
        for &(compartment, now) in &current {
            if let Some(&(_, then)) = previous.iter().find(|(other, _)| *other == compartment) {
                assert!(
                    now.distance(then) < 3.0,
                    "jumped {} at step {step}",
                    now.distance(then)
                );
            }
        }
        previous = current;
    }
}

#[test]
fn axons_to_several_neurons_turn_without_bending() {
    let mut world = World::new();
    let source = neuron_with_dendrites(&mut world, at(0.0, 0.0));
    let up = neuron(&mut world, at(0.0, 20.0));
    let right = neuron(&mut world, at(20.0, 0.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(20.0, 0.0));

    drag_in_steps(&mut world, at(0.0, 0.0), at(-8.0, -4.0));

    for target in [up, right] {
        let axon = chain(&world, source, target).unwrap();
        let (start, end) = (position(&world, source), position(&world, target));
        let along = (end - start).normalize();
        for &compartment in &axon {
            let offset = position(&world, compartment) - start;
            let off_line = (offset - offset.dot(along) * along).length();
            assert!(off_line < 0.1, "{off_line} off the line");
        }
    }
}

#[test]
fn erasing_a_neuron_erases_its_axons() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    assert!(count::<Compartment>(&world) > 3);

    drag(&mut editor(Tool::Erase), &mut world, &[at(0.0, 0.0)]);

    assert!(!world.contains(source) && world.contains(target));
    assert_eq!(count::<Compartment>(&world), 0);
    assert_eq!(count::<Connection>(&world), 0);
}

#[test]
fn erasing_part_of_an_axon_erases_the_rest_of_it() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    let grown = chain(&world, source, target).unwrap();
    let cut = position(&world, grown[4]);

    drag(&mut editor(Tool::Erase), &mut world, &[cut]);

    assert_eq!(chain(&world, source, grown[3]).unwrap(), grown[..4]);
    assert_eq!(count::<Compartment>(&world), 3);
    assert!(world.contains(target));
}

#[test]
fn erasing_beside_an_axon_erases_the_rest_of_it() {
    let mut world = World::new();
    let source = neuron(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    let grown = chain(&world, source, target).unwrap();
    let beside = (position(&world, grown[3]) + position(&world, grown[4])) / 2.0 + at(1.8, 0.0);
    assert!(grown
        .iter()
        .all(|&element| position(&world, element).distance(beside) > ERASE_RADIUS));

    drag(&mut editor(Tool::Erase), &mut world, &[beside]);

    assert_eq!(chain(&world, source, grown[3]).unwrap(), grown[..4]);
    assert_eq!(count::<Compartment>(&world), 3);
    assert!(world.contains(target));
}

fn entities(world: &World) -> HashSet<Entity> {
    world.iter().map(|entity| entity.entity()).collect()
}

#[test]
fn undo_brings_back_what_was_erased() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let a = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);
    let voltmeter = spawn_voltmeter(&mut world, a, at(-10.0, 0.0));
    let before = entities(&world);

    editor.tool = Tool::Erase;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    assert_eq!(entities(&world), HashSet::from([b, voltmeter]));
    assert!(measured(&world, voltmeter).is_empty());
    assert!(editor.can_undo());

    editor.undo(&mut world);
    assert_eq!(entities(&world), before);
    assert!(connected(&world, a, b) && connected(&world, a, voltmeter));
    assert!(editor.can_redo());
}

#[test]
fn undo_reconnects_elements_whose_place_was_taken() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);
    let mut editor = editor(Tool::Erase);
    drag(&mut editor, &mut world, &[at(0.0, 5.0)]);
    drag(&mut editor, &mut world, &[at(0.0, 10.0)]);
    let c = neuron(&mut world, at(10.0, 10.0));
    assert_eq!(c.id(), b.id());

    editor.undo(&mut world);
    editor.undo(&mut world);

    let (restored, _) =
        nearest_within(positions_with::<LeakyNeuron>(&world), at(0.0, 10.0), 0.1).unwrap();
    assert!(world.contains(c) && restored != c);
    assert!(connected(&world, a, restored));
    assert_eq!(count::<LeakyNeuron>(&world), 3);
}

type Layout = HashMap<Entity, (Option<Vec3>, Option<(Entity, Entity)>)>;

fn layout(world: &World) -> Layout {
    world
        .query::<(Option<&Position>, Option<&Connection>)>()
        .iter()
        .map(|(entity, (position, connection))| {
            let ends = connection.map(|connection| (connection.from, connection.to));
            (entity, (position.map(|position| position.position), ends))
        })
        .collect()
}

fn step(editor: &mut Editor, world: &mut World, tool: Tool, path: &[Vec3]) -> Layout {
    editor.tool = tool;
    let steps: Vec<Vec3> = path
        .windows(2)
        .flat_map(|pair| (0..10).map(move |step| pair[0].lerp(pair[1], step as f32 / 10.0)))
        .chain(path.last().copied())
        .collect();
    drag(editor, world, &steps);
    layout(world)
}

#[test]
fn undo_and_redo_step_through_every_kind_of_edit() {
    let mut world = World::new();
    let mut editor = editor(Tool::Edit);
    editor.compartments = true;
    let excitatory = Tool::Place(Element::ExcitatoryNeuron);
    let mut layouts = vec![layout(&world)];
    let edits: [(Tool, &[Vec3]); 8] = [
        (excitatory, &[at(0.0, 0.0)]),
        (excitatory, &[at(0.0, 20.0)]),
        (Tool::StaticConnection, &[at(0.0, 0.5), at(0.0, 19.5)]),
        (Tool::Axon, &[at(0.0, 20.5), at(20.0, 20.0)]),
        (Tool::Dendrite, &[at(0.0, -0.5), at(-2.0, -9.0)]),
        (Tool::Edit, &[at(0.0, 0.5), at(-5.0, 3.5)]),
        (Tool::Voltmeter, &[at(-4.5, 3.0)]),
        (Tool::Erase, &[at(0.0, 20.0)]),
    ];
    for (tool, path) in edits {
        layouts.push(step(&mut editor, &mut world, tool, path));
    }
    assert!(layouts.windows(2).all(|pair| pair[0] != pair[1]));

    for expected in layouts.iter().rev().skip(1) {
        editor.undo(&mut world);
        assert_eq!(&layout(&world), expected);
    }
    assert!(!editor.can_undo());
    for expected in &layouts[1..] {
        editor.redo(&mut world);
        assert_eq!(&layout(&world), expected);
    }
    assert!(!editor.can_redo());
}

#[test]
fn undo_puts_back_axons_that_a_drag_stretched_or_compressed() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    neuron(&mut world, at(0.0, 20.0));
    grow_axon(&mut world, at(0.0, 0.0), at(0.0, 20.0));
    let mut editor = editor(Tool::Edit);
    let grown = layout(&world);
    let stretched = step(
        &mut editor,
        &mut world,
        Tool::Edit,
        &[at(0.0, 0.0), at(0.0, -30.0)],
    );
    let compressed = step(
        &mut editor,
        &mut world,
        Tool::Edit,
        &[at(0.0, -30.0), at(0.0, 15.0)],
    );
    assert!(stretched.len() > grown.len() && compressed.len() < grown.len());

    editor.undo(&mut world);
    assert_eq!(layout(&world), stretched);
    editor.undo(&mut world);
    assert_eq!(layout(&world), grown);
    editor.redo(&mut world);
    editor.redo(&mut world);
    assert_eq!(layout(&world), compressed);
}

#[test]
fn a_new_edit_clears_what_could_be_redone() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    editor.undo(&mut world);
    assert!(editor.can_redo());

    drag(&mut editor, &mut world, &[at(10.0, 0.0)]);
    assert!(!editor.can_redo());
    assert_eq!(count::<LeakyNeuron>(&world), 1);
}

#[test]
fn undoing_a_voltmeter_resize_and_move_keeps_its_trace() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let voltmeter = spawn_voltmeter(&mut world, target, at(10.0, 5.0));
    let mut editor = editor(Tool::Edit);
    drag(&mut editor, &mut world, &[at(12.0, 13.0), at(13.0, 15.0)]);
    drag(&mut editor, &mut world, &[at(10.0, 9.0), at(11.0, 9.0)]);
    let (wire, _) = measured(&world, voltmeter)[0];
    world
        .get::<&mut VoltageSeries>(wire)
        .unwrap()
        .spike_times
        .push(1.0);

    editor.undo(&mut world);
    editor.undo(&mut world);

    assert_eq!(position(&world, voltmeter), at(10.0, 5.0));
    let size = world.get::<&MeterSize>(voltmeter).unwrap();
    assert_eq!((size.width, size.height), (8.0, 4.0));
    let series = world.get::<&VoltageSeries>(wire).unwrap();
    assert_eq!(series.spike_times, [1.0]);
}

#[test]
fn undo_puts_back_a_resized_note() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(note);
    let size = |world: &World| {
        world
            .get::<&NoteSize>(note)
            .map(|size| (size.width, size.height))
            .unwrap()
    };
    let before = size(&world);
    let [_, top_right] = note_corners(at(0.0, 0.0), &world.get::<&NoteSize>(note).unwrap());
    drag(
        &mut editor,
        &mut world,
        &[top_right, top_right + at(2.0, 1.0)],
    );
    assert_ne!(size(&world), before);

    editor.undo(&mut world);
    assert_eq!(size(&world), before);
    assert_eq!(position(&world, note), at(0.0, 0.0));
}

fn threshold(world: &World, neuron: Entity) -> f64 {
    world.get::<&LeakyNeuron>(neuron).unwrap().threshold
}

fn set_threshold(world: &World, neuron: Entity, threshold: f64) -> bool {
    world.get::<&mut LeakyNeuron>(neuron).unwrap().threshold = threshold;
    true
}

#[test]
fn adjusting_a_parameter_until_it_settles_is_one_undo_step() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let original = threshold(&world, a);
    let mut editor = editor(Tool::Edit);
    editor.adjust(&world, a, true, || false);
    assert!(!editor.can_undo());

    for value in [-0.05, -0.045, -0.04] {
        editor.adjust(&world, a, false, || set_threshold(&world, a, value));
    }
    editor.adjust(&world, a, true, || false);

    editor.undo(&mut world);
    assert_eq!(threshold(&world, a), original);
    assert!(!editor.can_undo());
    editor.redo(&mut world);
    assert_eq!(threshold(&world, a), -0.04);
}

#[test]
fn an_adjustment_comes_before_the_stroke_that_follows_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let original = threshold(&world, a);
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.adjust(&world, a, false, || set_threshold(&world, a, -0.04));
    drag(&mut editor, &mut world, &[at(10.0, 0.0)]);

    editor.undo(&mut world);
    assert_eq!(count::<LeakyNeuron>(&world), 1);
    assert_eq!(threshold(&world, a), -0.04);
    editor.undo(&mut world);
    assert_eq!(threshold(&world, a), original);
}

fn two_connected_neurons(world: &mut World) -> (Entity, Entity, Entity) {
    let a = neuron(world, at(0.0, 0.0));
    let b = neuron(world, at(0.0, 20.0));
    let synapse = spawn_synapse(world, a, b);
    (a, b, synapse)
}

#[test]
fn tapping_a_connection_in_view_mode_selects_it() {
    let mut world = World::new();
    let (a, _, synapse) = two_connected_neurons(&mut world);
    let mut editor = editor(Tool::Edit);
    editor.tap(&world, at(0.3, 9.0));
    assert_eq!(editor.selection, Some(synapse));
    assert!(!editor.empty_at(&world, at(0.3, 9.0)));

    editor.tap(&world, at(0.0, 0.5));
    assert_eq!(editor.selection, Some(a));
    editor.tap(&world, at(5.0, 9.0));
    assert_eq!(editor.selection, None);
}

#[test]
fn only_connections_on_screen_can_be_tapped_and_a_tapped_one_stays() {
    let mut world = World::new();
    let (a, _, synapse) = two_connected_neurons(&mut world);
    let mut editor = editor(Tool::Edit);
    ConnectionView::LastSelected.set_in(&mut world);
    editor.tap(&world, at(0.3, 9.0));
    assert_eq!(editor.selection, None);
    assert!(editor.empty_at(&world, at(0.3, 9.0)));

    Selected::set(&mut world, Some(a));
    editor.tap(&world, at(0.3, 9.0));
    assert_eq!(editor.selection, Some(synapse));

    Selected::set(&mut world, editor.selection);
    assert!(hidden_connections(&world).is_empty());
}

#[test]
fn a_tap_on_a_connection_while_editing_selects_it_and_a_drag_pans() {
    let mut world = World::new();
    let (_, _, synapse) = two_connected_neurons(&mut world);
    let mut editor = editor(Tool::Edit);
    editor.edit(&mut world, at(0.3, 9.0), false);
    assert!(!editor.tap_while_editing());
    assert_eq!(editor.selection, Some(synapse));
    editor.release(&world);

    editor.selection = None;
    editor.edit(&mut world, at(0.3, 9.0), false);
    let pan = editor.edit(&mut world, at(0.3, 12.0), false);
    assert_eq!(pan, Some(Pan(at(0.0, 3.0))));
    editor.release(&world);
    assert_eq!(editor.selection, None);
}

fn reaching(tool: Tool, touch_reach: f32) -> Editor {
    Editor {
        touch_reach,
        ..editor(tool)
    }
}

#[test]
fn a_hold_reaches_a_lone_neuron_from_a_finger_width_away() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = reaching(Tool::View, 3.0);

    editor.view(&mut world, at(5.0, 0.0));
    assert!(!editor.hold(&world, at(5.0, 0.0), true));
    editor.release(&world);

    editor.view(&mut world, at(3.5, 0.0));
    assert!(editor.hold(&world, at(3.5, 0.0), true));
    editor.lift(&mut world);
    assert_eq!(editor.selection, Some(target));
}

#[test]
fn a_hold_reaches_from_the_edge_of_what_is_drawn() {
    let mut world = World::new();
    let pad = Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let editor = reaching(Tool::View, 0.5);
    let targets = Targets::new(&world);
    let held = |position| {
        editor
            .hold_target(&targets, position)
            .map(|(entity, _)| entity)
    };
    assert_eq!(held(at(1.65, 0.0)), Some(pad));
    assert_eq!(held(at(1.85, 0.0)), None);
}

#[test]
fn neighbouring_compartments_split_the_reach_between_them() {
    let mut world = World::new();
    let chain: Vec<Entity> = (0..3)
        .map(|i| spawn_compartment(&mut world, at(2.0 * i as f32, 0.0), NeuronType::Excitatory))
        .collect();
    let editor = reaching(Tool::View, 3.0);
    let targets = Targets::new(&world);
    let held = |position| {
        editor
            .hold_target(&targets, position)
            .map(|(entity, _)| entity)
    };
    assert_eq!(held(at(1.2, 0.0)), Some(chain[1]));
    assert_eq!(held(at(0.8, 0.0)), Some(chain[0]));
    assert_eq!(held(at(4.0, 2.9)), Some(chain[2]));
    assert_eq!(held(at(4.0, 3.5)), None);
}

#[test]
fn a_hold_anywhere_along_a_dendrite_picks_the_nearest_compartment() {
    let mut world = World::new();
    let chain: Vec<Entity> = (0..3)
        .map(|i| spawn_compartment(&mut world, at(4.0 * i as f32, 0.0), NeuronType::Excitatory))
        .collect();
    for pair in chain.windows(2) {
        spawn_cable(&mut world, pair[0], pair[1], false);
    }
    let editor = reaching(Tool::View, 0.1);
    let targets = Targets::new(&world);
    let held = |position| {
        editor
            .hold_target(&targets, position)
            .map(|(entity, _)| entity)
    };
    assert_eq!(held(at(1.9, 0.2)), Some(chain[0]));
    assert_eq!(held(at(2.1, 0.2)), Some(chain[1]));
    assert_eq!(held(at(6.0, -0.2)), Some(chain[1]));
    assert_eq!(held(at(2.0, 1.0)), None);
    assert_eq!(held(at(10.0, 0.0)), None);
}

#[test]
fn the_targets_around_a_point_pick_as_all_targets_do_nearby() {
    let mut world = World::new();
    let chain: Vec<Entity> = (0..5)
        .map(|i| spawn_compartment(&mut world, at(2.0 * i as f32, 0.0), NeuronType::Excitatory))
        .collect();
    for pair in chain.windows(2) {
        spawn_cable(&mut world, pair[0], pair[1], false);
    }
    neuron(&mut world, at(3.0, 3.0));
    let targets = Targets::new(&world);
    let center = at(4.0, 1.0);
    let around = targets.around(center, 2.0);
    for x in -10..=10 {
        for z in -10..=10 {
            let point = center + 0.14 * Vec3::new(x as f32, 0.0, z as f32);
            assert_eq!(around.near(point, 0.5), targets.near(point, 0.5), "{point}");
        }
    }
}

fn reach_along(shape: &Shape, gap: f32, direction: Vec3) -> f32 {
    shape.outline(gap, [direction])[0].distance(shape.center())
}

#[test]
fn the_outline_of_a_lone_neuron_is_a_circle_beyond_its_edge() {
    let mut world = World::new();
    let target = neuron(&mut world, at(1.0, 2.0));
    let shape = Targets::new(&world).shape(target).unwrap();
    for direction in [Vec3::X, Vec3::Z, Vec3::new(-0.6, 0.0, 0.8)] {
        let reach = reach_along(&shape, 0.5, direction);
        assert!((reach - shape.thickness() - 0.5).abs() < 1e-3, "{reach}");
    }
}

#[test]
fn the_outline_of_a_compartment_follows_its_cables_halfway() {
    let mut world = World::new();
    let chain: Vec<Entity> = (0..3)
        .map(|i| spawn_compartment(&mut world, at(4.0 * i as f32, 0.0), NeuronType::Excitatory))
        .collect();
    for pair in chain.windows(2) {
        spawn_cable(&mut world, pair[0], pair[1], false);
    }
    let shape = Targets::new(&world).shape(chain[1]).unwrap();
    let across = shape.thickness() + 0.5;
    assert!((reach_along(&shape, 0.5, Vec3::X) - 2.0 - across).abs() < 1e-3);
    assert!((reach_along(&shape, 0.5, -Vec3::X) - 2.0 - across).abs() < 1e-3);
    assert!((reach_along(&shape, 0.5, Vec3::Z) - across).abs() < 1e-3);
}

#[test]
fn the_outline_of_a_note_is_its_rectangle_with_rounded_corners() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let size = world.get::<&NoteSize>(note).unwrap().clone();
    let shape = Targets::new(&world).shape(note).unwrap();
    assert!((reach_along(&shape, 0.5, Vec3::X) - 0.5 * size.height - 0.5).abs() < 1e-3);
    assert!((reach_along(&shape, 0.5, Vec3::Z) - 0.5 * size.width - 0.5).abs() < 1e-3);
    let corner = Vec3::new(size.height, 0.0, size.width).normalize();
    let to_corner = 0.5 * Vec3::new(size.height, 0.0, size.width).length();
    assert!(reach_along(&shape, 0.5, corner) < to_corner + 0.5);
}

#[test]
fn a_stimulated_touch_sensor_offers_nothing_to_hold() {
    let mut world = World::new();
    Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = reaching(Tool::View, 1.0);
    editor.view(&mut world, at(0.0, 0.0));
    assert_eq!(editor.hold_candidate(&world, at(0.0, 0.0)), None);
}

#[test]
fn a_tap_on_a_cable_still_selects_the_connection_and_beside_it_the_neuron() {
    let mut world = World::new();
    let (a, _, synapse) = two_connected_neurons(&mut world);
    let mut editor = reaching(Tool::View, 3.5);
    editor.tap(&world, at(0.3, 3.0));
    assert_eq!(editor.selection, Some(synapse));
    editor.tap(&world, at(2.5, 0.0));
    assert_eq!(editor.selection, Some(a));
    assert!(editor.empty_at(&world, at(5.0, 0.0)));

    let mut editor = reaching(Tool::Edit, 3.5);
    let pan = [at(0.3, 3.0), at(0.3, 6.0)].map(|position| editor.edit(&mut world, position, false));
    assert_eq!(pan[1], Some(Pan(at(0.0, 3.0))));
    editor.release(&world);
    drag(&mut editor, &mut world, &[at(2.5, 0.0)]);
    assert_eq!(editor.selection, Some(a));
}

#[test]
fn adding_beside_a_neuron_still_adds_instead_of_reaching_it() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = reaching(Tool::Place(Element::ExcitatoryNeuron), 10.0);
    drag(&mut editor, &mut world, &[at(7.0, 0.0)]);
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn current_sources_connect_without_a_synapse() {
    let mut world = World::new();
    let source = Element::CurrentSource.spawn(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::StaticConnection);
    drag(&mut editor, &mut world, &[at(0.0, 0.5), at(0.0, 9.5)]);
    assert!(connected(&world, source, target));
    assert_eq!(count::<CurrentSynapse>(&world), 0);
}

#[test]
fn connecting_a_neuron_onto_a_voltmeter_adds_its_plot_once() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, -10.0));
    let voltmeter = spawn_voltmeter(&mut world, first, at(0.0, 5.0));
    let inside = Frame::of(&world, voltmeter).unwrap().point(0.5, 0.5);
    let mut editor = editor(Tool::StaticConnection);
    for _ in 0..2 {
        drag(&mut editor, &mut world, &[at(0.0, -9.5), inside]);
    }
    let measured: Vec<Entity> = measured(&world, voltmeter)
        .into_iter()
        .map(|(_, target)| target)
        .collect();
    assert_eq!(measured, [first, second]);
    assert_eq!(count::<CurrentSynapse>(&world), 0);
}

#[test]
fn erasing_measured_neurons_keeps_the_voltmeter() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, -10.0));
    let voltmeter = spawn_voltmeter(&mut world, first, at(0.0, 5.0));
    crate::elements::spawn_voltmeter_wire(&mut world, second, voltmeter);
    let mut editor = editor(Tool::Edit);
    editor.selection = Some(first);
    editor.erase_selection(&mut world);
    assert!(world.contains(voltmeter));
    assert_eq!(measured(&world, voltmeter).len(), 1);

    editor.selection = Some(second);
    editor.erase_selection(&mut world);
    assert!(world.contains(voltmeter));
    assert!(measured(&world, voltmeter).is_empty());
}

#[test]
fn a_spike_detector_gets_a_row_for_each_neuron_connected_onto_it() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, -10.0));
    let compartment = spawn_compartment(&mut world, at(0.0, -20.0), NeuronType::Excitatory);
    let mut editor = editor(Tool::SpikeDetector);
    drag(&mut editor, &mut world, &[at(0.3, 0.0)]);
    let (detector, _) = world.query::<&SpikeDetector>().iter().next().unwrap();
    let inside = Frame::of(&world, detector).unwrap().point(0.5, 0.5);

    editor.tool = Tool::StaticConnection;
    drag(&mut editor, &mut world, &[at(0.0, -9.5), inside]);
    drag(&mut editor, &mut world, &[at(0.0, -19.8), inside]);
    let rows: Vec<Entity> = measured(&world, detector)
        .into_iter()
        .map(|(_, target)| target)
        .collect();
    assert_eq!(rows, [first, second]);
    assert!(!rows.contains(&compartment));
}

#[test]
fn a_rate_plot_averages_the_neurons_connected_onto_it() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, -10.0));
    let mut editor = editor(Tool::RatePlot);
    drag(&mut editor, &mut world, &[at(0.3, 0.0)]);
    let (plot, _) = world
        .query::<&crate::measurement::firing_rate::RatePlot>()
        .iter()
        .next()
        .unwrap();
    let inside = Frame::of(&world, plot).unwrap().point(0.5, 0.5);
    editor.tool = Tool::StaticConnection;
    drag(&mut editor, &mut world, &[at(0.0, -9.5), inside]);
    let measured: Vec<Entity> = measured(&world, plot)
        .into_iter()
        .map(|(_, target)| target)
        .collect();
    assert_eq!(measured, [first, second]);
}

#[test]
fn a_loudspeaker_listens_to_each_neuron_connected_onto_it() {
    let mut world = World::new();
    let first = neuron(&mut world, at(0.0, 0.0));
    let second = neuron(&mut world, at(0.0, -10.0));
    let mut editor = editor(Tool::Loudspeaker);
    drag(&mut editor, &mut world, &[at(0.3, 0.0)]);
    let (speaker, position) = world
        .query::<&Position>()
        .with::<&crate::measurement::loudspeaker::Loudspeaker>()
        .iter()
        .map(|(speaker, position)| (speaker, position.position))
        .next()
        .unwrap();
    editor.tool = Tool::StaticConnection;
    drag(&mut editor, &mut world, &[at(0.0, -9.5), position]);
    let wired: Vec<Entity> = measured(&world, speaker)
        .into_iter()
        .map(|(_, target)| target)
        .collect();
    assert_eq!(wired, [first, second]);
    assert_eq!(count::<CurrentSynapse>(&world), 0);
}

#[test]
fn a_visual_input_is_picked_anywhere_on_its_picture() {
    let mut world = World::new();
    let input = Element::VisualInput.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    editor.tap(&world, at(3.0, 3.0));
    assert_eq!(editor.selection, Some(input));
    editor.tap(&world, at(5.0, 5.0));
    assert_eq!(editor.selection, None);
}

#[test]
fn a_touch_sensor_makes_its_neuron_fire_right_away() {
    let mut world = World::new();
    let sensor = Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let target = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::StaticConnection);
    drag(&mut editor, &mut world, &[at(0.0, 0.5), at(0.0, 9.5)]);
    assert!(connected(&world, sensor, target));
    assert_eq!(count::<ImmediateFireSynapse>(&world), 1);

    editor.tool = Tool::Stimulate;
    editor.edit(&mut world, at(0.0, 0.5), false);
    editor.release(&world);
    let (mut engine, mut time) = (Engine::default(), 0.0);
    for press in editor.take_presses() {
        fire_nearby(&mut world, &mut engine, press);
    }
    let fired_within = (0..40).find(|_| {
        step_simulation(&mut world, &mut engine, &mut time, 1, None);
        engine.fired(target)
    });
    assert!(
        fired_within.is_some_and(|steps| steps < 12),
        "{fired_within:?}"
    );
}

#[test]
fn changes_to_the_circuit_count_as_edits() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Edit);
    drag(&mut editor, &mut world, &[at(10.0, 10.0), at(12.0, 10.0)]);
    drag(&mut editor, &mut world, &[at(0.5, 0.0)]);
    editor.tool = Tool::View;
    drag(&mut editor, &mut world, &[at(0.0, 0.0), at(1.0, 0.0)]);
    editor.adjust(&world, a, true, || false);
    editor.undo(&mut world);
    assert!(
        !editor.take_edited(),
        "panning, selecting, stimulating and undoing nothing"
    );

    editor.tool = Tool::Place(Element::ExcitatoryNeuron);
    drag(&mut editor, &mut world, &[at(0.0, 20.0)]);
    assert!(editor.take_edited(), "placing");
    assert!(!editor.take_edited());

    editor.tool = Tool::Edit;
    drag(&mut editor, &mut world, &[at(0.5, 0.0), at(3.0, 0.0)]);
    assert!(editor.take_edited(), "dragging");

    editor.tool = Tool::Erase;
    drag(&mut editor, &mut world, &[at(30.0, 30.0)]);
    assert!(!editor.take_edited(), "erasing nothing");
    drag(&mut editor, &mut world, &[at(0.0, 20.0)]);
    assert!(editor.take_edited(), "erasing");

    editor.adjust(&world, a, false, || set_threshold(&world, a, -0.05));
    assert!(editor.take_edited(), "adjusting before it settles");

    editor.selection = Some(a);
    editor.erase_selection(&mut world);
    assert!(editor.take_edited(), "erasing the selection");
    editor.undo(&mut world);
    assert!(editor.take_edited(), "undoing");
    editor.redo(&mut world);
    assert!(editor.take_edited(), "redoing");
}

#[test]
fn moving_drawing_and_erasing_aim_but_panning_does_not() {
    let mut world = World::new();
    let placed = neuron(&mut world, at(0.0, 0.0));
    let mut panning = editor(Tool::Edit);
    panning.edit(&mut world, at(20.0, 20.0), false);
    assert!(!panning.aims());
    let mut moving = editor(Tool::Edit);
    moving.edit(&mut world, at(0.0, 0.0), false);
    assert!(moving.aims());
    assert_eq!(moving.picked(), Some(placed));
    let mut drawing = editor(Tool::Axon);
    drawing.edit(&mut world, at(0.0, 0.0), false);
    assert!(drawing.aims());
    let mut erasing = editor(Tool::Erase);
    erasing.edit(&mut world, at(20.0, 20.0), false);
    assert!(erasing.aims());
}
