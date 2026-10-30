use super::*;
use crate::measurement::voltmeter::VoltageSeries;
use crate::simulation::step_simulation;
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
        .map(|&position| editor.edit(world, position, false))
        .collect();
    editor.release(world);
    pans
}

fn count<T: Component>(world: &World) -> usize {
    world.query::<&T>().iter().count()
}

fn connected(world: &World, from: Entity, to: Entity) -> bool {
    world
        .query::<&Connection>()
        .iter()
        .any(|(_, connection)| connection.from == from && connection.to == to)
}

fn neuron(world: &mut World, position: Vec3) -> Entity {
    Element::ExcitatoryNeuron.spawn(world, position)
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
        .with::<&CompartmentCurrent>()
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
    let grown_from_dendrite = world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
        .iter()
        .any(|(_, cable)| {
            world.get::<&Dendrite>(cable.from).is_ok()
                && world.get::<&Dendrite>(cable.to).is_err()
                && world.get::<&Compartment>(cable.to).is_ok()
        });
    assert!(grown_from_dendrite);
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
fn tapping_an_element_while_adding_selects_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.5, 0.0), false);
    editor.select_pressed();
    editor.release(&world);
    assert_eq!(editor.selection, Some(a));
    assert_eq!(count::<LeakyNeuron>(&world), 1);

    editor.edit(&mut world, at(20.0, 0.0), false);
    editor.select_pressed();
    editor.release(&world);
    assert_eq!(editor.selection, Some(a));
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn holding_an_element_while_adding_moves_it() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let b = neuron(&mut world, at(0.0, 10.0));
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.edit(&mut world, at(0.0, 1.5), false);
    assert!(editor.hold(&world, at(0.0, 1.5)));
    assert_eq!(editor.connection_being_drawn(&world), None);
    drag(&mut editor, &mut world, &[at(0.0, 10.0), at(5.0, 3.5)]);
    assert_eq!(position(&world, a), at(5.0, 2.0));
    assert_eq!(editor.selection, Some(a));
    assert!(!connected(&world, a, b));
    assert_eq!(count::<LeakyNeuron>(&world), 2);
}

#[test]
fn holding_after_placing_moves_the_new_element() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::CurrentSource));
    editor.edit(&mut world, at(0.0, 0.0), false);
    assert!(editor.hold(&world, at(0.0, 0.0)));
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
    assert!(editor.hold(&world, at(0.0, 0.5)));
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
fn erasing_the_selection_takes_its_dendrites_connections_and_voltmeters() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.compartments = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let a = world.query::<&LeakyNeuron>().iter().next().unwrap().0;
    let b = neuron(&mut world, at(0.0, 10.0));
    spawn_synapse(&mut world, a, b);
    spawn_voltmeter(&mut world, a, at(1.0, 0.0));

    editor.selection = Some(a);
    editor.erase_selection(&mut world);
    assert_eq!(editor.selection, None);
    assert!(world.contains(b));
    assert_eq!(count::<LeakyNeuron>(&world), 1);
    assert_eq!(count::<Dendrite>(&world), 0);
    assert_eq!(count::<Connection>(&world), 0);
    assert_eq!(count::<Voltmeter>(&world), 0);
}

#[test]
fn voltmeters_attach_to_the_neuron_under_the_pointer() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Voltmeter);
    drag(&mut editor, &mut world, &[at(10.0, 10.0)]);
    assert_eq!(count::<Voltmeter>(&world), 0);

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
fn selecting_drags_the_element_under_the_pointer() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
    let pans = drag(&mut editor, &mut world, &[at(0.5, 0.0), at(5.5, 3.0)]);
    assert_eq!(pans, [None, None]);
    assert_eq!(editor.selection, Some(a));
    assert_eq!(world.get::<&Position>(a).unwrap().position, at(5.0, 3.0));
}

#[test]
fn dragging_empty_space_pans_and_keeps_the_selection() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
    editor.selection = Some(a);
    let pans = drag(&mut editor, &mut world, &[at(10.0, 10.0), at(12.0, 10.0)]);
    assert_eq!(pans, [None, Some(Pan(at(2.0, 0.0)))]);
    assert_eq!(editor.selection, Some(a));
}

#[test]
fn the_editor_knows_whether_a_press_landed_on_the_background() {
    let mut world = World::new();
    neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
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
    let mut editor = editor(Tool::Select);

    drag(&mut editor, &mut world, &[at(12.0, 13.0), at(13.0, 15.0)]);
    let size = world
        .get::<&VoltmeterSize>(voltmeter)
        .map(|size| (size.width, size.height))
        .unwrap();
    assert_eq!(size, (10.0, 5.0));
    assert_eq!(
        world.get::<&Position>(voltmeter).unwrap().position,
        at(10.5, 5.0)
    );

    drag(&mut editor, &mut world, &[at(10.0, 9.0), at(11.0, 9.0)]);
    assert_eq!(editor.selection, Some(voltmeter));
    assert_eq!(
        world.get::<&Position>(voltmeter).unwrap().position,
        at(11.5, 5.0)
    );
}

#[test]
fn selected_notes_are_resized_by_their_corners() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
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
    assert_eq!(editor.selection, Some(note));

    let [bottom_left, top_right] = [bottom_left + at(1.0, 1.0), top_right + at(1.0, 1.0)];
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
    let mut editor = editor(Tool::Select);
    drag(&mut editor, &mut world, &[at(1.2, 1.0)]);
    assert_eq!(editor.selection, Some(inside));
    drag(&mut editor, &mut world, &[at(-1.5, -2.0)]);
    assert_eq!(editor.selection, Some(note));
}

#[test]
fn viewers_touch_sensors_and_pan_everywhere_else() {
    let mut world = World::new();
    Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);

    assert_eq!(editor.view(&mut world, at(0.5, 0.0)), None);
    assert_eq!(editor.view(&mut world, at(5.0, 5.0)), None);
    assert_eq!(editor.touch(), Some(at(5.0, 5.0)));
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
        let fired = world.get::<&GeneratorDynamics>(sensor).unwrap().fired;
        step_simulation(world, &mut 0.0, 1, editor.touch());
        fired
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
    let mut editor = editor(Tool::Select);

    editor.view(&mut world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    editor.release(&world);
    assert_eq!(editor.selection, Some(target));

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
    let mut editor = editor(Tool::Select);
    editor.selection = Some(target);

    editor.view(&mut world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    assert_eq!(editor.selection, Some(target));
}

#[test]
fn holding_an_element_grabs_it() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);

    editor.view(&mut world, at(10.0, 10.0));
    assert!(!editor.hold(&world, at(10.0, 10.0)));
    editor.release(&world);

    editor.view(&mut world, at(0.5, 0.0));
    assert!(editor.hold(&world, at(0.5, 0.0)));
    assert_eq!(editor.selection, Some(target));
    assert_eq!(editor.edit(&mut world, at(3.5, 2.0), false), None);
    assert_eq!(
        world.get::<&Position>(target).unwrap().position,
        at(3.0, 2.0)
    );
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
    drag(&mut editor(Tool::Select), world, &path);
}

fn position(world: &World, entity: Entity) -> Vec3 {
    world.get::<&Position>(entity).unwrap().position
}

fn cables_out_of(world: &World, from: Entity) -> Vec<Entity> {
    world
        .query::<&Connection>()
        .with::<&CompartmentCurrent>()
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
    chain.windows(2).all(|pair| {
        let length = position(world, pair[0]).distance(position(world, pair[1]));
        (length - SEGMENT_LENGTH).abs() < 0.25 * SEGMENT_LENGTH
    })
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
    let mut editor = editor(Tool::Select);
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
    assert_eq!(entities(&world), HashSet::from([b]));
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

    let (restored, _) = first_within::<LeakyNeuron>(&world, at(0.0, 10.0), 0.1).unwrap();
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
    let mut editor = editor(Tool::Select);
    editor.compartments = true;
    let excitatory = Tool::Place(Element::ExcitatoryNeuron);
    let mut layouts = vec![layout(&world)];
    let edits: [(Tool, &[Vec3]); 7] = [
        (excitatory, &[at(0.0, 0.0)]),
        (excitatory, &[at(0.0, 20.0)]),
        (Tool::StaticConnection, &[at(0.0, 0.5), at(0.0, 19.5)]),
        (Tool::Axon, &[at(0.0, 20.5), at(20.0, 20.0)]),
        (Tool::Select, &[at(0.0, 0.5), at(-5.0, 3.5)]),
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
    let mut editor = editor(Tool::Select);
    let grown = layout(&world);
    let stretched = step(
        &mut editor,
        &mut world,
        Tool::Select,
        &[at(0.0, 0.0), at(0.0, -30.0)],
    );
    let compressed = step(
        &mut editor,
        &mut world,
        Tool::Select,
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
    let mut editor = editor(Tool::Select);
    drag(&mut editor, &mut world, &[at(12.0, 13.0), at(13.0, 15.0)]);
    drag(&mut editor, &mut world, &[at(10.0, 9.0), at(11.0, 9.0)]);
    world
        .get::<&mut VoltageSeries>(voltmeter)
        .unwrap()
        .spike_times
        .push(1.0);

    editor.undo(&mut world);
    editor.undo(&mut world);

    assert_eq!(position(&world, voltmeter), at(10.0, 5.0));
    let size = world.get::<&VoltmeterSize>(voltmeter).unwrap();
    assert_eq!((size.width, size.height), (8.0, 4.0));
    let series = world.get::<&VoltageSeries>(voltmeter).unwrap();
    assert_eq!(series.spike_times, [1.0]);
}

#[test]
fn undo_puts_back_a_resized_note() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
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
    let mut editor = editor(Tool::Select);
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
