use std::collections::HashSet;

use glam::Vec3;
use hecs::{Component, Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::dendrites::{compartment_tree, spawn_dendrites};
use crate::elements::{spawn_cable, spawn_compartment, spawn_synapse, spawn_voltmeter, Element};
use crate::measurement::voltmeter::{Corner, Frame, Voltmeter};
use crate::rendering::bounds::note_corners;
use crate::rendering::{compartment_radii, connection_paths, distance_to_segment, Theme};
use crate::simulation::{fire_nearby, touch_sensor_near};
use crate::tools::{ConnectionTool, StimulationTool, Tool};
use drag::Drag;
use history::{History, Recording, Snapshot};

mod drag;
mod history;
#[cfg(test)]
mod tests;

const CONNECTION_REACH: f32 = 2.0 * NODE_RADIUS;
const VOLTMETER_REACH: f32 = NODE_RADIUS;
const CORNER_REACH: f32 = 1.0;
const DENDRITE_GAP: f32 = 2.0 * SEGMENT_LENGTH;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pan(pub Vec3);

#[derive(Default)]
pub struct Editor {
    pub tool: Tool,
    pub compartments: bool,
    pub selection: Option<Entity>,
    connection_tool: Option<ConnectionTool>,
    stimulation_tool: Option<StimulationTool>,
    previous_creation: Option<Entity>,
    pressed: Option<Entity>,
    move_origin: Option<Vec3>,
    dragging: Option<Drag>,
    resizing: Option<(Entity, Corner, Vec3)>,
    recording: Option<Recording>,
    adjusting: Option<Snapshot>,
    history: History,
}

impl Editor {
    pub fn touch(&self) -> Option<Vec3> {
        self.stimulation_tool.as_ref().map(|touch| touch.position)
    }

    pub fn connection_being_drawn(&self, world: &World) -> Option<(Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        let drawn = match self.tool {
            Tool::StaticConnection => true,
            Tool::Place(_) => !self.grows_axon(world, connection.from),
            _ => false,
        };
        drawn.then_some((connection.start, connection.end))
    }

    pub fn release(&mut self, world: &World) {
        self.stimulation_tool = None;
        self.connection_tool = None;
        self.previous_creation = None;
        self.pressed = None;
        self.move_origin = None;
        self.dragging = None;
        self.resizing = None;
        if let Some(recording) = self.recording.take() {
            self.history.record(recording, world);
        }
    }

    pub fn forget_world(&mut self) {
        *self = Editor {
            tool: self.tool,
            compartments: self.compartments,
            ..Default::default()
        };
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self, world: &mut World) {
        self.release(world);
        self.settle();
        self.history.undo(world);
    }

    pub fn redo(&mut self, world: &mut World) {
        self.release(world);
        self.settle();
        self.history.redo(world);
    }

    pub fn adjust(
        &mut self,
        world: &World,
        entity: Entity,
        settled: bool,
        adjust: impl FnOnce() -> bool,
    ) {
        if self
            .adjusting
            .as_ref()
            .is_some_and(|adjusting| adjusting.entity != entity)
        {
            self.settle();
        }
        let ongoing = self.adjusting.is_some();
        let before = self
            .adjusting
            .take()
            .or_else(|| Snapshot::of(world, entity));
        if adjust() || ongoing {
            self.adjusting = before;
        }
        if settled {
            self.settle();
        }
    }

    fn settle(&mut self) {
        if let Some(adjusting) = self.adjusting.take() {
            self.history.record_snapshot(adjusting);
        }
    }

    pub fn erase_selection(&mut self, world: &mut World) {
        let Some(selected) = self.selection.take() else {
            return;
        };
        if world.get::<&Deletable>(selected).is_err() {
            return;
        }
        self.settle();
        let mut recording = Recording::start(world);
        let elements: HashSet<Entity> = compartment_tree(world, selected)
            .into_iter()
            .chain([selected])
            .collect();
        for entity in elements {
            recording.take(world, entity);
        }
        let dangling: Vec<Entity> = world
            .query::<&Connection>()
            .with::<&Deletable>()
            .iter()
            .filter(|(_, connection)| {
                !world.contains(connection.from) || !world.contains(connection.to)
            })
            .map(|(connection, _)| connection)
            .collect();
        for connection in dangling {
            recording.take(world, connection);
        }
        self.history.record(recording, world);
    }

    pub fn edit(&mut self, world: &mut World, position: Vec3, shift_down: bool) -> Option<Pan> {
        self.settle();
        let mut recording = self
            .recording
            .take()
            .unwrap_or_else(|| Recording::start(world));
        let pan = self.use_tool(world, &mut recording, position, shift_down);
        self.recording = Some(recording);
        pan
    }

    fn use_tool(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        position: Vec3,
        shift_down: bool,
    ) -> Option<Pan> {
        match self.tool {
            Tool::Select => return self.select(world, recording, position),
            Tool::Place(element) => self.add(world, recording, element, position, shift_down),
            Tool::Voltmeter => self.place_voltmeter(world, position),
            Tool::StaticConnection => self.connect(world, position, shift_down),
            Tool::Axon => self.grow_axon(world, position, shift_down),
            Tool::Erase => erase(world, recording, position),
            Tool::Stimulate => self.stimulate(world, position),
        }
        None
    }

    pub fn view(&mut self, world: &mut World, position: Vec3) -> Option<Pan> {
        let touching = self.stimulation_tool.is_some()
            || (self.move_origin.is_none() && touch_sensor_near(world, position));
        if touching {
            self.stimulate(world, position);
            return None;
        }
        match self.move_origin {
            Some(origin) => Some(Pan(position - origin)),
            None => {
                self.move_origin = Some(position);
                None
            }
        }
    }

    fn stimulate(&mut self, world: &mut World, position: Vec3) {
        if self.stimulation_tool.is_none() {
            fire_nearby(world, position);
        }
        self.stimulation_tool = Some(StimulationTool { position });
    }

    pub fn pressed_background(&self) -> bool {
        self.move_origin.is_some()
    }

    pub fn tap(&mut self, world: &World, position: Vec3) {
        if self.stimulation_tool.is_some() {
            return;
        }
        match entity_at(world, &voltmeter_frames(world), position) {
            Some((entity, _)) => self.select_entity(entity),
            None => self.selection = None,
        }
    }

    pub fn empty_at(&self, world: &World, position: Vec3) -> bool {
        entity_at(world, &voltmeter_frames(world), position).is_none()
    }

    fn select_entity(&mut self, entity: Entity) {
        self.selection = Some(entity);
        self.tool = Tool::Select;
    }

    pub fn hold(&mut self, world: &World, position: Vec3, select: bool) -> bool {
        if self.stimulation_tool.is_some() {
            return false;
        }
        let pressed = self.pressed.and_then(|pressed| {
            let pressed_position = world.get::<&Position>(pressed).ok()?.position;
            Some((pressed, pressed_position))
        });
        let Some((entity, entity_position)) =
            pressed.or_else(|| entity_at(world, &voltmeter_frames(world), position))
        else {
            return false;
        };
        self.move_origin = None;
        self.connection_tool = None;
        if select {
            self.select_entity(entity);
        }
        self.grab(world, entity, entity_position, position);
        true
    }

    fn grab(&mut self, world: &World, entity: Entity, entity_position: Vec3, position: Vec3) {
        let offset = (entity_position - position).with_y(0.0);
        self.dragging = Some(Drag::new(world, entity, offset));
    }

    fn too_near_previous(&self, world: &World, position: Vec3, minimum_distance: f32) -> bool {
        self.previous_creation
            .and_then(|previous| world.get::<&Position>(previous).ok().map(|p| p.position))
            .is_some_and(|previous| previous.distance(position) < minimum_distance)
    }

    fn grows_axon(&self, world: &World, from: Entity) -> bool {
        self.compartments && world.get::<&NeuronType>(from).is_ok()
    }

    fn add(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        element: Element,
        position: Vec3,
        shift_down: bool,
    ) {
        if let Some(drag) = &mut self.dragging {
            drag.move_to(world, recording, position);
        } else if let Some(connection) = &self.connection_tool {
            if self.grows_axon(world, connection.from) {
                self.extend_axon(world, position, shift_down);
            } else {
                self.extend_connection(world, position, shift_down);
            }
        } else if self.previous_creation.is_some() {
            self.place(world, element, position);
        } else if self.pressed.is_none() {
            self.press_to_add(world, element, position);
        }
    }

    fn press_to_add(&mut self, world: &mut World, element: Element, position: Vec3) {
        let candidates = if self.compartments {
            axon_sources(world)
        } else {
            sources(world)
        };
        let connection = start_connection(candidates, position);
        let compartments = positions_with::<Compartment>(world);
        let under = entity_at(world, &voltmeter_frames(world), position)
            .or_else(|| nearest_within(compartments, position, CONNECTION_REACH))
            .map(|(entity, _)| entity);
        self.pressed = connection
            .as_ref()
            .map(|connection| connection.from)
            .or(under);
        if self.pressed.is_some() {
            self.previous_creation = connection.as_ref().map(|connection| connection.from);
            self.connection_tool = connection;
            return;
        }
        self.place(world, element, position);
        if element.is_input() {
            self.connection_tool = self.previous_creation.map(|from| ConnectionTool {
                start: position,
                end: position,
                from,
            });
        }
    }

    fn crowds_previous(&self, world: &World, element: Element, position: Vec3) -> bool {
        match self.previous_creation {
            Some(previous) if element.is_neuron() && self.compartments => {
                dendrites_would_overlap(world, previous, position)
            }
            _ => self.too_near_previous(world, position, MIN_CREATION_DISTANCE_DEFAULT),
        }
    }

    fn place(&mut self, world: &mut World, element: Element, position: Vec3) {
        if self.crowds_previous(world, element, position) {
            return;
        }
        let entity = element.spawn(world, position);
        if element.is_neuron() && self.compartments {
            spawn_dendrites(world, entity);
        }
        if element == Element::Note {
            self.select_entity(entity);
        }
        self.previous_creation = Some(entity);
    }

    fn place_voltmeter(&mut self, world: &mut World, position: Vec3) {
        if self.too_near_previous(world, position, MIN_CREATION_DISTANCE_DEFAULT) {
            return;
        }
        let target = first_within::<LeakyNeuron>(world, position, VOLTMETER_REACH)
            .or_else(|| first_within::<Compartment>(world, position, VOLTMETER_REACH));
        if let Some((target, target_position)) = target {
            self.previous_creation =
                Some(spawn_voltmeter(world, target, target_position + Vec3::X));
        }
    }

    fn connect(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        if self.connection_tool.is_some() {
            self.extend_connection(world, position, shift_down);
        } else {
            self.connection_tool = start_connection(sources(world), position);
        }
    }

    fn extend_connection(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        connect_to_neuron_near(world, connection, position, shift_down, spawn_synapse);
        connection.end = position;
    }

    fn grow_axon(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        if self.connection_tool.is_some() {
            self.extend_axon(world, position, shift_down);
            return;
        }
        self.connection_tool = start_connection(axon_sources(world), position);
        if let Some(connection) = &self.connection_tool {
            self.previous_creation = Some(connection.from);
        }
    }

    fn extend_axon(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        connection.end = position;
        let reached_neuron = connect_to_neuron_near(
            world,
            connection,
            position,
            shift_down,
            |world, from, to| lay_axon(world, from, to, true),
        );
        let from = connection.from;
        if reached_neuron || self.too_near_previous(world, position, MIN_CREATION_DISTANCE_AXON) {
            return;
        }
        let Ok(neuron_type) = world.get::<&NeuronType>(from).map(|t| *t) else {
            return;
        };
        let compartment = spawn_compartment(world, position, neuron_type);
        lay_axon(world, from, compartment, false);
        self.previous_creation = Some(compartment);
        self.connection_tool = Some(ConnectionTool {
            start: position,
            end: position,
            from: compartment,
        });
    }

    fn select(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        position: Vec3,
    ) -> Option<Pan> {
        if let Some(drag) = &mut self.dragging {
            drag.move_to(world, recording, position);
        } else if let Some((resized, corner, offset)) = self.resizing {
            resize(world, recording, resized, corner, position + offset);
        } else if let Some(origin) = self.move_origin {
            return Some(Pan(position - origin));
        } else {
            self.pick(world, position);
        }
        None
    }

    fn pick(&mut self, world: &World, position: Vec3) {
        let frames = voltmeter_frames(world);
        let selected = self
            .selection
            .and_then(|selected| Some((selected, resizable(world, selected)?)));
        self.resizing = selected
            .into_iter()
            .chain(
                frames
                    .iter()
                    .map(|&(voltmeter, _, frame)| (voltmeter, frame)),
            )
            .find_map(|(entity, frame)| {
                let corner = frame.corner_near(position, CORNER_REACH)?;
                Some((entity, corner, frame.corner(corner) - position))
            });
        if self.resizing.is_some() {
            return;
        }
        match entity_at(world, &frames, position) {
            Some((entity, entity_position)) => {
                self.selection = Some(entity);
                self.grab(world, entity, entity_position, position);
            }
            None => self.move_origin = Some(position),
        }
    }
}

fn dendrites_would_overlap(world: &World, neuron: Entity, position: Vec3) -> bool {
    let Ok(soma) = world.get::<&Position>(neuron).map(|soma| soma.position) else {
        return false;
    };
    let tree: Vec<Vec3> = compartment_tree(world, neuron)
        .into_iter()
        .filter_map(|dendrite| world.get::<&Position>(dendrite).ok().map(|p| p.position))
        .collect();
    let low = tree.iter().fold(soma, |low, &position| low.min(position));
    let high = tree.iter().fold(soma, |high, &position| high.max(position));
    let footprint = high - low + Vec3::splat(DENDRITE_GAP);
    let offset = (position - soma).abs();
    offset.x < footprint.x && offset.z < footprint.z
}

fn voltmeter_frames(world: &World) -> Vec<(Entity, Vec3, Frame)> {
    world
        .query::<(&Position, &VoltmeterSize)>()
        .with::<&Voltmeter>()
        .iter()
        .map(|(voltmeter, (voltmeter_position, size))| {
            let frame = Frame::new(voltmeter_position.position, size);
            (voltmeter, voltmeter_position.position, frame)
        })
        .collect()
}

fn entity_at(
    world: &World,
    frames: &[(Entity, Vec3, Frame)],
    position: Vec3,
) -> Option<(Entity, Vec3)> {
    frames
        .iter()
        .find(|(_, _, frame)| frame.contains(position))
        .map(|&(voltmeter, voltmeter_position, _)| (voltmeter, voltmeter_position))
        .or_else(|| element_near(world, position))
        .or_else(|| note_under(world, position))
}

fn positions_with<T: Component>(world: &World) -> Vec<(Entity, Vec3)> {
    world
        .query::<&Position>()
        .with::<&T>()
        .iter()
        .map(|(entity, position)| (entity, position.position))
        .collect()
}

fn first_within<T: Component>(world: &World, position: Vec3, reach: f32) -> Option<(Entity, Vec3)> {
    positions_with::<T>(world)
        .into_iter()
        .find(|(_, candidate)| candidate.distance(position) < reach)
}

fn nearest_within(
    candidates: Vec<(Entity, Vec3)>,
    position: Vec3,
    reach: f32,
) -> Option<(Entity, Vec3)> {
    candidates
        .into_iter()
        .min_by(|(_, a), (_, b)| a.distance(position).total_cmp(&b.distance(position)))
        .filter(|(_, nearest)| nearest.distance(position) < reach)
}

fn sources(world: &World) -> Vec<(Entity, Vec3)> {
    [
        positions_with::<LeakyNeuron>(world),
        positions_with::<CurrentClamp>(world),
        positions_with::<GeneratorDynamics>(world),
    ]
    .concat()
}

fn axon_sources(world: &World) -> Vec<(Entity, Vec3)> {
    [
        positions_with::<StaticConnectionSource>(world),
        sources(world),
    ]
    .concat()
}

fn lay_axon(world: &mut World, from: Entity, to: Entity, directional: bool) -> Entity {
    let ends = [from, to].map(|end| world.get::<&Position>(end).map(|p| p.position));
    let neuron_type = world
        .get::<&NeuronType>(from)
        .map(|t| *t)
        .unwrap_or_default();
    let mut previous = from;
    if let [Ok(start), Ok(end)] = ends {
        let segments = (start.distance(end) / SEGMENT_LENGTH).round() as usize;
        for step in 1..segments {
            let position = start.lerp(end, step as f32 / segments as f32);
            let compartment = spawn_compartment(world, position, neuron_type);
            spawn_cable(world, previous, compartment, false);
            previous = compartment;
        }
    }
    spawn_cable(world, previous, to, directional)
}

fn start_connection(sources: Vec<(Entity, Vec3)>, position: Vec3) -> Option<ConnectionTool> {
    nearest_within(sources, position, CONNECTION_REACH).map(|(from, start)| ConnectionTool {
        start,
        end: position,
        from,
    })
}

fn connect_to_neuron_near(
    world: &mut World,
    connection: &mut ConnectionTool,
    position: Vec3,
    shift_down: bool,
    spawn: impl FnOnce(&mut World, Entity, Entity) -> Entity,
) -> bool {
    let neurons = positions_with::<LeakyNeuron>(world);
    let Some((target, target_position)) = nearest_within(neurons, position, CONNECTION_REACH)
    else {
        return false;
    };
    let exists = world
        .query::<&Connection>()
        .iter()
        .any(|(_, existing)| existing.from == connection.from && existing.to == target);
    if !exists && connection.from != target {
        spawn(world, connection.from, target);
    }
    if !shift_down {
        connection.start = target_position;
        connection.from = target;
    }
    true
}

fn element_near(world: &World, position: Vec3) -> Option<(Entity, Vec3)> {
    let elements = world
        .query::<&Position>()
        .without::<&Annotation>()
        .iter()
        .map(|(entity, element)| (entity, element.position))
        .collect();
    nearest_within(elements, position, SELECTION_RANGE)
}

fn note_under(world: &World, position: Vec3) -> Option<(Entity, Vec3)> {
    world
        .query::<(&Position, &NoteSize)>()
        .with::<&Annotation>()
        .iter()
        .filter(|(_, (note, size))| {
            let offset = position - note.position;
            offset.x.abs() <= size.height * 0.5 && offset.z.abs() <= size.width * 0.5
        })
        .max_by_key(|(entity, _)| entity.id())
        .map(|(entity, (note, _))| (entity, note.position))
}

fn resizable(world: &World, entity: Entity) -> Option<Frame> {
    if let Some(frame) = Frame::of(world, entity) {
        return Some(frame);
    }
    let note = world.get::<&Position>(entity).ok()?;
    let size = world.get::<&NoteSize>(entity).ok()?;
    let [bottom_left, _] = note_corners(note.position, &size);
    Some(Frame {
        bottom_left,
        width: size.width,
        height: size.height,
    })
}

fn resize(
    world: &mut World,
    recording: &mut Recording,
    entity: Entity,
    corner: Corner,
    position: Vec3,
) {
    let Some(frame) = resizable(world, entity) else {
        return;
    };
    recording.edit(world, entity);
    let resized = frame.resized(corner, position);
    let moved_to = if let Ok(mut size) = world.get::<&mut NoteSize>(entity) {
        *size = NoteSize {
            width: resized.width,
            height: resized.height,
        };
        resized.point(0.5, 0.5)
    } else {
        if let Ok(mut size) = world.get::<&mut VoltmeterSize>(entity) {
            *size = resized.size();
        }
        resized.position()
    };
    if let Ok(mut moved) = world.get::<&mut Position>(entity) {
        moved.position = moved_to;
    }
}

fn erase(world: &mut World, recording: &mut Recording, position: Vec3) {
    let cut = connections_near(world, position);
    let cut_off = cut.iter().filter_map(|&connection| {
        let to = world.get::<&Connection>(connection).ok()?.to;
        world.get::<&Compartment>(to).is_ok().then_some(to)
    });
    let near: Vec<Entity> = world
        .query::<&Position>()
        .with::<&Deletable>()
        .iter()
        .filter(|(_, element)| element.position.distance(position) < ERASE_RADIUS)
        .map(|(entity, _)| entity)
        .chain(cut_off)
        .collect();
    let elements: HashSet<Entity> = near
        .iter()
        .flat_map(|&entity| compartment_tree(world, entity))
        .chain(near.iter().copied())
        .collect();
    for entity in elements {
        recording.take(world, entity);
    }

    let dangling: Vec<Entity> = world
        .query::<&Connection>()
        .with::<&Deletable>()
        .iter()
        .filter(|(_, connection)| {
            world.get::<&Position>(connection.from).is_err()
                || world.get::<&Position>(connection.to).is_err()
        })
        .map(|(entity, _)| entity)
        .collect();
    for connection in cut.into_iter().chain(dangling) {
        recording.take(world, connection);
    }
}

fn connections_near(world: &World, position: Vec3) -> Vec<Entity> {
    let paths = connection_paths(
        world,
        &compartment_radii(world),
        &Theme::default().palette(),
    );
    world
        .query::<&Connection>()
        .with::<&Deletable>()
        .iter()
        .filter(|(entity, connection)| {
            let (Ok(from), Ok(to)) = (
                world.get::<&Position>(connection.from),
                world.get::<&Position>(connection.to),
            ) else {
                return false;
            };
            let distance = match paths.get(entity) {
                Some(path) => path.distance_to(position),
                None => distance_to_segment(from.position, to.position, position),
            };
            distance.is_some_and(|distance| distance < ERASE_RADIUS)
        })
        .map(|(entity, _)| entity)
        .collect()
}
