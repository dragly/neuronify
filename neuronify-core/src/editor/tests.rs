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
fn erasing_the_selection_takes_its_dendrites_connections_and_voltmeters() {
    let mut world = World::new();
    let mut editor = editor(Tool::Place(Element::ExcitatoryNeuron));
    editor.dendrites = true;
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
    editor.release();
    editor.edit(&mut world, at(10.0, 10.0), false);
    assert!(editor.pressed_background());
    editor.release();
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

#[test]
fn viewers_select_with_a_tap() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);

    editor.view(&world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    editor.release();
    assert_eq!(editor.selection, Some(target));

    editor.view(&world, at(10.0, 10.0));
    editor.tap(&world, at(10.0, 10.0));
    editor.release();
    assert_eq!(editor.selection, None);
}

#[test]
fn touching_a_sensor_leaves_the_selection_alone() {
    let mut world = World::new();
    let target = neuron(&mut world, at(10.0, 0.0));
    Element::TouchSensor.spawn(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);
    editor.selection = Some(target);

    editor.view(&world, at(0.5, 0.0));
    editor.tap(&world, at(0.5, 0.0));
    assert_eq!(editor.selection, Some(target));
}

#[test]
fn holding_an_element_grabs_it() {
    let mut world = World::new();
    let target = neuron(&mut world, at(0.0, 0.0));
    let mut editor = editor(Tool::Select);

    editor.view(&world, at(10.0, 10.0));
    assert!(!editor.hold(&world, at(10.0, 10.0)));
    editor.release();

    editor.view(&world, at(0.5, 0.0));
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
    editor.dendrites = true;
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
        dendrite_tree(world, soma)
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
    let dendrites = dendrite_tree(&world, source);
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
