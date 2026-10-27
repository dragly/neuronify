use super::*;

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
    editor.release();
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
    editor.dendrites = true;
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
    editor.release();
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
fn erasing_removes_what_is_under_the_pointer() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.dendrites = true;
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
fn selecting_empty_space_pans_and_clears_the_selection() {
    let mut world = World::new();
    let a = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
    editor.selection = Some(a);
    let pans = drag(&mut editor, &mut world, &[at(10.0, 10.0), at(12.0, 10.0)]);
    assert_eq!(pans, [None, Some(Pan(at(2.0, 0.0)))]);
    assert_eq!(editor.selection, None);
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

    assert_eq!(editor.view(&world, at(0.5, 0.0)), None);
    assert_eq!(editor.view(&world, at(5.0, 5.0)), None);
    assert_eq!(editor.touch(), Some(at(5.0, 5.0)));
    editor.release();

    assert_eq!(editor.view(&world, at(10.0, 10.0)), None);
    assert_eq!(editor.view(&world, at(11.0, 10.0)), Some(Pan(at(1.0, 0.0))));
    assert_eq!(editor.touch(), None);
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

#[test]
fn dendrites_move_along_with_a_dragged_neuron() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.dendrites = true;
    drag(&mut editor, &mut world, &[at(0.0, 0.0)]);
    let before: Vec<(Entity, Vec3)> = positions_with::<Dendrite>(&world);

    drag_in_steps(&mut world, at(0.0, 0.5), at(3.0, 4.5));

    for (dendrite, previous) in before {
        let moved = position(&world, dendrite) - previous;
        assert!(moved.distance(at(3.0, 4.0)) < 1e-4, "moved {moved}");
    }
}
