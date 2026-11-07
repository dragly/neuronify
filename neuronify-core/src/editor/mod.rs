use std::collections::HashSet;
use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Component, Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::dendrites::{compartment_tree, spawn_dendrites};
use crate::elements::{
    spawn_axon_cable, spawn_cable, spawn_compartment, spawn_connection, Element, Meter,
};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::visual_input::{visual_input_frame, VisualInput};
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::bounds::note_corners;
use crate::rendering::{compartment_radii, connection_paths, distance_to_segment, Theme};
use crate::simulation::{touch_sensor_near, Touch};
use crate::tools::{
    ConnectionHint, ConnectionOffer, ConnectionTool, Stimulation, StimulationTool, Tool,
};
use drag::Drag;
use history::{History, Recording, Snapshot};

mod drag;
mod history;
mod targets;
#[cfg(test)]
mod tests;

pub use targets::{Shape, Targets};

const CONNECTION_REACH: f32 = 2.0 * NODE_RADIUS;
const STAYING_REACH: f32 = 1.5;
const SHARPEST_BEND: f32 = std::f32::consts::FRAC_PI_4;
const CORNER_REACH: f32 = 1.0;
const SPEAKER_OFFSET: Vec3 = Vec3::new(3.0, 0.0, 0.0);
const CONNECTION_SELECTION_RANGE: f32 = 0.6 * NODE_RADIUS;
const DENDRITE_GAP: f32 = 2.0 * SEGMENT_LENGTH;
const HINT_SECONDS: f64 = 2.0;
const HINT_FADE_SECONDS: f64 = 0.25;
const HINT_LENGTH: f32 = 4.0 * NODE_RADIUS;
const HINT_SWAY: f32 = NODE_RADIUS;
const HINT_SEGMENTS: usize = 8;
const HINT_WIGGLES_PER_SECOND: f32 = 0.7;
const HINT_WIGGLE_LAG: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pan(pub Vec3);

#[derive(Default)]
pub struct Editor {
    pub tool: Tool,
    pub compartments: bool,
    pub stimulation: Stimulation,
    pub selection: Option<Entity>,
    pub touch_reach: f32,
    connection_tool: Option<ConnectionTool>,
    offer: Option<ConnectionOffer>,
    stimulation_tool: Option<StimulationTool>,
    previous_creation: Option<Entity>,
    pressed: Option<Entity>,
    pressed_connection: Option<Entity>,
    move_origin: Option<Vec3>,
    dragging: Option<Drag>,
    held: Option<Entity>,
    resizing: Option<(Entity, Corner, Vec3)>,
    recording: Option<Recording>,
    adjusting: Option<Snapshot>,
    history: History,
    edited: bool,
    shortcut: Option<Meter>,
    wired: bool,
    presses: Vec<Vec3>,
    ending: Option<Entity>,
    formed: bool,
    drawn: HashSet<Entity>,
}

impl Editor {
    pub fn touch(&self) -> Option<Touch> {
        self.stimulation_tool.as_ref().map(|touch| Touch {
            position: touch.position,
            current: self.stimulation.current,
        })
    }

    pub fn connection_being_drawn(&self, world: &World) -> Option<(Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        let drawn = match self.tool {
            Tool::StaticConnection => true,
            Tool::Place(_) => !self.grows_axon(world, connection.from),
            tool => tool.meter().is_some(),
        };
        let clear_of_source = connection.start.distance(connection.end) > TERMINAL_INSET;
        (drawn && clear_of_source).then_some((connection.start, connection.end))
    }

    pub fn axon_being_grown(&self, world: &World) -> Option<(Entity, Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        let grows = match self.tool {
            Tool::Axon | Tool::MyelinatedAxon => true,
            Tool::Place(_) => self.grows_axon(world, connection.from),
            _ => false,
        };
        grows.then_some((connection.from, connection.start, connection.end))
    }

    pub fn dendrite_being_grown(&self) -> Option<(Entity, Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        (self.tool == Tool::Dendrite).then_some((connection.from, connection.start, connection.end))
    }

    pub fn ending(&self) -> Option<Entity> {
        self.connection_tool.as_ref().and(self.ending)
    }

    pub fn new_ending(&self, world: &World) -> Option<(Entity, Entity)> {
        let axon = self.connection_tool.as_ref()?;
        let ending = self.ending.filter(|&ending| {
            has::<Compartment>(world, ending) && !connected(world, axon.from, ending)
        })?;
        Some((axon.from, ending))
    }

    pub fn drawn(&self) -> &HashSet<Entity> {
        &self.drawn
    }

    pub fn take_formed(&mut self) -> bool {
        std::mem::take(&mut self.formed)
    }

    pub fn painting(&self) -> Option<Element> {
        let Tool::Place(element) = self.tool else {
            return None;
        };
        let painting = element.is_neuron()
            && self.previous_creation.is_some()
            && self.connection_tool.is_none()
            && self.dragging.is_none();
        painting.then_some(element)
    }

    pub fn offer_connection(&mut self, now: f64) {
        let Some(connection) = &self.connection_tool else {
            return;
        };
        if self.shortcut.is_some() {
            return;
        }
        if self.pressed == Some(connection.from) {
            self.offer = Some(ConnectionOffer {
                from: connection.from,
                since: now,
            });
        }
    }

    pub fn connection_hint(&mut self, world: &World, now: f64) -> Option<ConnectionHint> {
        let age = now - self.offer.as_ref()?.since;
        if age >= HINT_SECONDS {
            self.offer = None;
            return None;
        }
        if !matches!(self.tool, Tool::Place(_)) && self.tool.meter().is_none() {
            return None;
        }
        let (_, start) = self.offered_source(world)?;
        let shown = (age.min(HINT_SECONDS - age) / HINT_FADE_SECONDS).clamp(0.0, 1.0) as f32;
        let grown = 1.0 - (1.0 - shown).powi(3);
        let wiggle = age as f32 * TAU * HINT_WIGGLES_PER_SECOND;
        let mut string: Vec<Vec3> = (0..=HINT_SEGMENTS)
            .map(|index| {
                let along = index as f32 / HINT_SEGMENTS as f32;
                let phase = wiggle - HINT_WIGGLE_LAG * along;
                let sway = Vec3::new(phase.sin(), 0.0, (1.3 * phase).sin());
                start + (Vec3::Y * HINT_LENGTH * along + sway * HINT_SWAY * along * along) * grown
            })
            .collect();
        let bead = string[HINT_SEGMENTS];
        let pull = (bead - string[HINT_SEGMENTS - 1]).normalize_or_zero();
        string.push(bead + pull * TERMINAL_INSET);
        let above = Vec3::Y * HINT_LENGTH + Vec3::X * (HINT_SWAY + NODE_RADIUS);
        Some(ConnectionHint {
            string,
            label: start + above * grown,
            shown,
        })
    }

    fn offered_source(&self, world: &World) -> Option<(Entity, Vec3)> {
        let from = self.offer.as_ref()?.from;
        let start = world.get::<&Position>(from).ok()?.position;
        Some((from, start))
    }

    fn pull_offer(&self, world: &World, position: Vec3) -> Option<ConnectionTool> {
        let (from, start) = self.offered_source(world)?;
        (position.distance(start) < HINT_LENGTH + HINT_SWAY).then_some(ConnectionTool {
            start,
            end: position,
            from,
        })
    }

    pub fn lift(&mut self, world: &mut World) {
        let axon = self
            .connection_tool
            .as_ref()
            .filter(|_| self.tool != Tool::Dendrite);
        if let Some(axon) = axon {
            let measuring = self.tool.meter().is_some() || Meter::of(world, axon.from).is_some();
            if !measuring {
                if let Some((from, ending)) = self.new_ending(world) {
                    lay_axon(world, from, ending, true, self.myelinates());
                }
            } else if let Some(meter) = self.shortcut.filter(|_| !self.wired) {
                measure_where_released(world, meter, axon);
            }
        }
        if let Some(held) = self.held {
            self.selection = Some(held);
        }
        self.release(world);
    }

    pub fn take_presses(&mut self) -> Vec<Vec3> {
        std::mem::take(&mut self.presses)
    }

    pub fn take_edited(&mut self) -> bool {
        std::mem::take(&mut self.edited)
    }

    pub fn mark_edited(&mut self) {
        self.edited = true;
    }

    pub fn release(&mut self, world: &World) {
        self.stimulation_tool = None;
        self.connection_tool = None;
        self.previous_creation = None;
        self.pressed = None;
        self.pressed_connection = None;
        self.move_origin = None;
        self.dragging = None;
        self.held = None;
        self.resizing = None;
        self.shortcut = None;
        self.wired = false;
        self.ending = None;
        self.formed = false;
        self.drawn.clear();
        if let Some(recording) = self.recording.take() {
            self.edited |= self.history.record(recording, world);
        }
    }

    pub fn sync_selection(&mut self, world: &mut World) {
        self.selection = self.selection.filter(|&selected| world.contains(selected));
        if let Some(selected) = self.selection {
            if Selected::entity(world) != Some(selected) {
                Selected::set(world, Some(selected));
            }
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
        self.edited |= self.history.undo(world);
    }

    pub fn redo(&mut self, world: &mut World) {
        self.release(world);
        self.settle();
        self.edited |= self.history.redo(world);
    }

    pub fn prepare(&mut self, world: &mut World, preparation: Preparation) {
        self.release(world);
        self.settle();
        let mut recording = Recording::start(world);
        let prepared: Vec<Entity> = world
            .iter()
            .filter(|entity| {
                entity.has::<Cable>() || entity.has::<Preparation>() || entity.has::<Temperature>()
            })
            .map(|entity| entity.entity())
            .collect();
        for entity in prepared {
            recording.edit(world, entity);
        }
        crate::simulation::model::prepare(world, preparation);
        self.edited |= self.history.record(recording, world);
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
            .or_else(|| Snapshot::of(world, entity, compartment_tree(world, entity)));
        let adjusted = adjust();
        self.edited |= adjusted;
        if adjusted || ongoing {
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
        take_connections(world, &mut recording, dangling);
        self.edited |= self.history.record(recording, world);
    }

    pub fn edit(&mut self, world: &mut World, position: Vec3, shift_down: bool) -> Option<Pan> {
        self.settle();
        let mut recording = self
            .recording
            .take()
            .unwrap_or_else(|| Recording::start(world));
        let pan = self.use_tool(world, &mut recording, position, shift_down);
        self.hold_drawing(world, &recording);
        self.recording = Some(recording);
        pan
    }

    fn hold_drawing(&mut self, world: &World, recording: &Recording) {
        let drawing = self.axon_being_grown(world).or(self.dendrite_being_grown());
        let Some((from, _, _)) = drawing else {
            return;
        };
        let drawn = recording
            .spawned(world)
            .chain([from])
            .filter(|&entity| has::<Compartment>(world, entity));
        self.drawn.extend(drawn);
    }

    fn use_tool(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        position: Vec3,
        shift_down: bool,
    ) -> Option<Pan> {
        match self.tool {
            Tool::View => return self.view(world, position),
            Tool::Edit => return self.select(world, recording, position),
            Tool::Place(element) => self.add(world, recording, element, position, shift_down),
            Tool::Voltmeter | Tool::SpikeDetector | Tool::RatePlot | Tool::Loudspeaker => {
                self.measure(world, position)
            }
            Tool::StaticConnection => self.connect(world, position, shift_down),
            Tool::Axon | Tool::MyelinatedAxon => {
                self.grow_axon(world, recording, position, shift_down)
            }
            Tool::Dendrite => self.grow_dendrite(world, recording, position),
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

    fn stimulate(&mut self, _world: &mut World, position: Vec3) {
        if self.stimulation_tool.is_none() {
            self.presses.push(position);
        }
        self.stimulation_tool = Some(StimulationTool { position });
    }

    pub fn pressed_background(&self) -> bool {
        self.move_origin.is_some()
    }

    pub fn tap_while_editing(&mut self) -> bool {
        match self.pressed_connection {
            Some(connection) => {
                self.selection = Some(connection);
                false
            }
            None => self.pressed_background(),
        }
    }

    pub fn tap(&mut self, world: &World, position: Vec3) {
        if self.stimulation_tool.is_some() {
            return;
        }
        let targets = Targets::new(world);
        let tapped = targets
            .at(position)
            .map(|(entity, _)| entity)
            .or_else(|| connection_at(world, position))
            .or_else(|| {
                self.hold_target(&targets, position)
                    .map(|(entity, _)| entity)
            });
        match tapped {
            Some(entity) => self.select_entity(entity),
            None => self.selection = None,
        }
    }

    pub fn empty_at(&self, world: &World, position: Vec3) -> bool {
        self.hold_target(&Targets::new(world), position).is_none()
            && connection_at(world, position).is_none()
    }

    fn select_entity(&mut self, entity: Entity) {
        self.selection = Some(entity);
        if self.tool.edits() {
            self.tool = Tool::Edit;
        }
    }

    pub fn hold_candidate(&self, world: &World, position: Vec3) -> Option<(Entity, Vec3)> {
        if self.stimulation_tool.is_some() {
            return None;
        }
        let pressed = self.pressed.and_then(|pressed| {
            let pressed_position = world.get::<&Position>(pressed).ok()?.position;
            Some((pressed, pressed_position))
        });
        pressed.or_else(|| self.hold_target(&Targets::new(world), position))
    }

    pub fn hold(&mut self, world: &World, position: Vec3, select: bool) -> bool {
        let Some((entity, entity_position)) = self.hold_candidate(world, position) else {
            return false;
        };
        self.move_origin = None;
        self.connection_tool = None;
        self.drawn.clear();
        if select {
            self.held = Some(entity);
        }
        if self.tool == Tool::View {
            self.tool = Tool::Edit;
        }
        self.grab(world, entity, entity_position, position);
        true
    }

    pub fn aims(&self) -> bool {
        let erasing = self.tool == Tool::Erase && self.recording.is_some();
        self.dragging.is_some()
            || self.resizing.is_some()
            || self.connection_tool.is_some()
            || self.painting().is_some()
            || erasing
    }

    pub fn picked(&self) -> Option<Entity> {
        self.dragging.as_ref().map(Drag::entity)
    }

    pub fn hold_target(&self, targets: &Targets, position: Vec3) -> Option<(Entity, Vec3)> {
        targets.near(position, self.touch_reach)
    }

    pub fn move_held(&mut self) {
        if self.held.take().is_some() {
            self.selection = None;
        }
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
        self.compartments
            && world
                .satisfies::<hecs::Or<&LeakyNeuron, &Compartment>>(from)
                .unwrap_or(false)
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
                self.extend_axon(world, recording, position, shift_down);
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
        let connection =
            start_connection(candidates, position).or_else(|| self.pull_offer(world, position));
        self.offer = None;
        let compartments = positions_with::<Compartment>(world);
        let under = Targets::new(world)
            .at(position)
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

    fn measure(&mut self, world: &mut World, position: Vec3) {
        let Some(meter) = self.tool.meter() else {
            return;
        };
        if self.connection_tool.is_some() {
            self.extend_wire(world, position);
            return;
        }
        if self.pressed.is_some() {
            return;
        }
        let pulled = self.pull_offer(world, position);
        self.offer = None;
        if let Some(wire) = pulled.or_else(|| start_wire(world, meter, position)) {
            if Meter::of(world, wire.from).is_none() {
                self.shortcut = Some(meter);
            }
            self.pressed = Some(wire.from);
            self.connection_tool = Some(wire);
            return;
        }
        let placed = spawn_meter_centered(world, meter, position);
        self.previous_creation = Some(placed);
        self.connection_tool = Some(ConnectionTool {
            start: position,
            end: position,
            from: placed,
        });
    }

    fn extend_wire(&mut self, world: &mut World, position: Vec3) {
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        connection.end = position;
        let from = connection.from;
        if self.wired {
            return;
        }
        self.wired = match Meter::of(world, from) {
            Some(meter) => measure_neuron_near(world, meter, from, position),
            None => measure_with_meter_at(world, from, position),
        };
    }

    fn connect(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        if self.connection_tool.is_some() {
            self.extend_connection(world, position, shift_down);
        } else {
            self.connection_tool = start_connection(sources(world), position).or_else(|| {
                meter_at(world, position).map(|(meter, start)| ConnectionTool {
                    start,
                    end: position,
                    from: meter,
                })
            });
        }
    }

    fn extend_connection(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        let from_meter = self
            .connection_tool
            .as_ref()
            .is_some_and(|connection| Meter::of(world, connection.from).is_some());
        if from_meter {
            self.extend_wire(world, position);
            return;
        }
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        if !connect_to_neuron_near(world, connection, position, shift_down, spawn_connection) {
            measure_with_meter_at(world, connection.from, position);
        }
        connection.end = position;
    }

    fn grow_axon(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        position: Vec3,
        shift_down: bool,
    ) {
        if self.connection_tool.is_some() {
            self.extend_axon(world, recording, position, shift_down);
            return;
        }
        self.connection_tool = start_connection(axon_sources(world), position);
        if let Some(connection) = &self.connection_tool {
            self.previous_creation = Some(connection.from);
        }
    }

    fn myelinates(&self) -> bool {
        self.tool == Tool::MyelinatedAxon
    }

    fn extend_axon(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        position: Vec3,
        shift_down: bool,
    ) {
        let myelinated = self.myelinates();
        let spacing = match myelinated {
            true => INTERNODE_LENGTH,
            false => MIN_CREATION_DISTANCE_AXON,
        };
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        connection.end = position;
        let from = connection.from;
        let targets = Targets::new(world);
        let own = cable_network(world, from);
        let reach = self.touch_reach;
        let target = axon_target(world, &targets, &own, from, position, reach).or_else(|| {
            let staying = cable_network(world, self.ending?);
            targets.piece_near(position, STAYING_REACH * reach, |entity| {
                staying.contains(&entity)
                    && (entity == from
                        || (!own.contains(&entity) && has::<Compartment>(world, entity)))
            })
        });
        let on_soma = target.is_some_and(|(target, _)| !has::<Compartment>(world, target));
        self.ending = match target {
            Some((target, _)) if target == from => self.ending.filter(|&ending| ending == from),
            Some((target, target_position)) if on_soma => {
                if !connected(world, from, target) {
                    lay_axon(world, from, target, true, myelinated);
                    self.formed = true;
                }
                if !shift_down {
                    connection.start = target_position;
                    connection.from = target;
                }
                Some(target)
            }
            target => target.map(|(target, _)| target),
        };
        if target.is_some() || self.too_near_previous(world, position, spacing) {
            return;
        }
        let Some(from) = self.back_up(world, recording, from, position, spacing) else {
            return;
        };
        let Ok(neuron_type) = world.get::<&NeuronType>(from).map(|t| *t) else {
            return;
        };
        let compartment = spawn_compartment(world, position, neuron_type);
        lay_axon(world, from, compartment, false, myelinated);
        self.previous_creation = Some(compartment);
        self.connection_tool = Some(ConnectionTool {
            start: position,
            end: position,
            from: compartment,
        });
    }

    fn grow_dendrite(&mut self, world: &mut World, recording: &mut Recording, position: Vec3) {
        let Some(connection) = &mut self.connection_tool else {
            self.connection_tool = start_connection(dendrite_sources(world), position);
            self.previous_creation = self.connection_tool.as_ref().map(|dendrite| dendrite.from);
            return;
        };
        connection.end = position;
        let from = connection.from;
        if self.too_near_previous(world, position, SEGMENT_LENGTH) {
            return;
        }
        let Some(from) = self.back_up(world, recording, from, position, SEGMENT_LENGTH) else {
            return;
        };
        let Some(tip) = lay_dendrite(world, from, position) else {
            return;
        };
        self.previous_creation = Some(tip);
        self.connection_tool = Some(ConnectionTool {
            start: position,
            end: position,
            from: tip,
        });
    }

    fn back_up(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        tip: Entity,
        position: Vec3,
        spacing: f32,
    ) -> Option<Entity> {
        let mut backed = tip;
        let sharp = loop {
            let Some((cable, previous)) = cable_into(world, backed) else {
                break false;
            };
            let at = world.get::<&Position>(backed).ok()?.position;
            let before = world.get::<&Position>(previous).ok()?.position;
            let sharp = (before - at).angle_between(position - at) < SHARPEST_BEND;
            if !sharp || !recording.is_new(backed) {
                break sharp;
            }
            recording.take(world, cable);
            recording.take(world, backed);
            self.drawn.remove(&backed);
            backed = previous;
        };
        let at = world.get::<&Position>(backed).ok()?.position;
        if !sharp && (backed == tip || at.distance(position) >= spacing) {
            return Some(backed);
        }
        self.previous_creation = Some(backed);
        self.connection_tool = Some(ConnectionTool {
            start: at,
            end: position,
            from: backed,
        });
        None
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
        let frames = meter_frames(world);
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
        let targets = Targets::new(world);
        let connection = connection_at(world, position);
        let picked = match targets.at(position) {
            None if connection.is_none() => self.hold_target(&targets, position),
            picked => picked,
        };
        match picked {
            Some((entity, entity_position)) => {
                match self.selection {
                    Some(_) => self.selection = Some(entity),
                    None => self.held = Some(entity),
                }
                self.grab(world, entity, entity_position, position);
            }
            None => {
                self.pressed_connection = connection;
                self.move_origin = Some(position);
            }
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

fn meter_frames(world: &World) -> Vec<(Entity, Vec3, Frame)> {
    let meters = world
        .query::<(&Position, &MeterSize)>()
        .iter()
        .map(|(meter, (meter_position, size))| {
            let frame = Frame::new(meter_position.position, size);
            (meter, meter_position.position, frame)
        })
        .collect::<Vec<_>>();
    let visual_inputs = world
        .query::<&Position>()
        .with::<&VisualInput>()
        .iter()
        .map(|(input, center)| (input, center.position, visual_input_frame(center.position)))
        .collect::<Vec<_>>();
    [meters, visual_inputs].concat()
}

fn positions_with<T: Component>(world: &World) -> Vec<(Entity, Vec3)> {
    world
        .query::<&Position>()
        .with::<&T>()
        .iter()
        .map(|(entity, position)| (entity, position.position))
        .collect()
}

fn nearest_within(
    candidates: impl IntoIterator<Item = (Entity, Vec3)>,
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

fn dendrite_sources(world: &World) -> Vec<(Entity, Vec3)> {
    [
        positions_with::<LeakyNeuron>(world),
        positions_with::<Dendrite>(world),
    ]
    .concat()
}

fn lay_dendrite(world: &mut World, from: Entity, end: Vec3) -> Option<Entity> {
    let start = world.get::<&Position>(from).ok()?.position;
    let neuron_type = *world.get::<&NeuronType>(from).ok()?;
    let segments = ((start.distance(end) / SEGMENT_LENGTH).round() as usize).max(1);
    let mut previous = from;
    for step in 1..=segments {
        let position = start.lerp(end, step as f32 / segments as f32);
        let compartment = spawn_compartment(world, position, neuron_type);
        world.insert_one(compartment, Dendrite).unwrap();
        spawn_cable(world, previous, compartment, false);
        previous = compartment;
    }
    Some(previous)
}

fn lay_axon(
    world: &mut World,
    from: Entity,
    to: Entity,
    directional: bool,
    myelinated: bool,
) -> Entity {
    let ends = [from, to].map(|end| world.get::<&Position>(end).map(|p| p.position));
    let neuron_type = world
        .get::<&NeuronType>(from)
        .map(|t| *t)
        .unwrap_or_default();
    if myelinated && directional {
        let mut terminal_start = from;
        if let [Ok(start), Ok(end)] = ends {
            let internodes = ((start.distance(end) - SEGMENT_LENGTH) / INTERNODE_LENGTH).floor();
            if internodes >= 1.0 {
                let position = start + (end - start).normalize() * internodes * INTERNODE_LENGTH;
                terminal_start = spawn_compartment(world, position, neuron_type);
                lay_axon(world, from, terminal_start, false, true);
            }
        }
        return lay_axon(world, terminal_start, to, true, false);
    }
    let spacing = match myelinated {
        true => INTERNODE_LENGTH,
        false => SEGMENT_LENGTH,
    };
    let mut previous = from;
    if let [Ok(start), Ok(end)] = ends {
        let segments = (start.distance(end) / spacing).round() as usize;
        for step in 1..segments {
            let position = start.lerp(end, step as f32 / segments as f32);
            let compartment = spawn_compartment(world, position, neuron_type);
            spawn_axon_cable(world, previous, compartment, myelinated);
            previous = compartment;
        }
    }
    match directional {
        true => spawn_cable(world, previous, to, true),
        false => spawn_axon_cable(world, previous, to, myelinated),
    }
}

fn start_connection(sources: Vec<(Entity, Vec3)>, position: Vec3) -> Option<ConnectionTool> {
    nearest_within(sources, position, CONNECTION_REACH).map(|(from, start)| ConnectionTool {
        start,
        end: position,
        from,
    })
}

fn meter_at(world: &World, position: Vec3) -> Option<(Entity, Vec3)> {
    let framed = meter_frames(world)
        .into_iter()
        .find(|&(meter, _, frame)| frame.contains(position) && Meter::of(world, meter).is_some())
        .map(|(meter, _, _)| (meter, position));
    framed.or_else(|| {
        nearest_within(
            positions_with::<Loudspeaker>(world),
            position,
            CONNECTION_REACH,
        )
    })
}

fn measurable_near(world: &World, meter: Meter, position: Vec3) -> Option<(Entity, Vec3)> {
    let candidates = [
        positions_with::<LeakyNeuron>(world),
        positions_with::<Compartment>(world),
    ]
    .concat()
    .into_iter()
    .filter(|&(target, _)| meter.measures(world, target));
    nearest_within(candidates, position, CONNECTION_REACH)
}

fn start_wire(world: &World, meter: Meter, position: Vec3) -> Option<ConnectionTool> {
    meter_at(world, position)
        .or_else(|| measurable_near(world, meter, position))
        .map(|(from, start)| ConnectionTool {
            start,
            end: position,
            from,
        })
}

fn measures(world: &World, meter: Entity, target: Entity) -> bool {
    measured(world, meter)
        .iter()
        .any(|&(_, measured)| measured == target)
}

fn measure_neuron_near(world: &mut World, meter: Meter, entity: Entity, position: Vec3) -> bool {
    let Some((target, _)) = measurable_near(world, meter, position) else {
        return false;
    };
    if measures(world, entity, target) {
        return false;
    }
    meter.wire(world, target, entity);
    true
}

fn measure_with_meter_at(world: &mut World, source: Entity, position: Vec3) -> bool {
    let Some((entity, _)) = meter_at(world, position) else {
        return false;
    };
    let Some(meter) = Meter::of(world, entity) else {
        return false;
    };
    if measures(world, entity, source) || !meter.measures(world, source) {
        return false;
    }
    meter.wire(world, source, entity);
    true
}

fn spawn_meter_centered(world: &mut World, meter: Meter, position: Vec3) -> Entity {
    let width = meter.size().map_or(0.0, |size| size.width);
    meter.spawn(world, position - Vec3::Z * width * 0.5)
}

fn measure_where_released(world: &mut World, meter: Meter, wire: &ConnectionTool) {
    let placed = if wire.end.distance(wire.start) < CONNECTION_REACH {
        let offset = match meter {
            Meter::Loudspeaker => SPEAKER_OFFSET,
            _ => Vec3::X,
        };
        meter.spawn(world, wire.start + offset)
    } else if Targets::new(world).at(wire.end).is_none() {
        spawn_meter_centered(world, meter, wire.end)
    } else {
        return;
    };
    meter.wire(world, wire.from, placed);
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
    if !connected(world, connection.from, target) && connection.from != target {
        spawn(world, connection.from, target);
    }
    if !shift_down {
        connection.start = target_position;
        connection.from = target;
    }
    true
}

fn connected(world: &World, from: Entity, to: Entity) -> bool {
    world
        .query::<&Connection>()
        .iter()
        .any(|(_, connection)| connection.from == from && connection.to == to)
}

fn cable_network(world: &World, start: Entity) -> HashSet<Entity> {
    let cables: Vec<(Entity, Entity)> = world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .filter(|(_, cable)| !cable.directional)
        .map(|(_, cable)| (cable.from, cable.to))
        .collect();
    let mut network = HashSet::from([start]);
    let mut frontier = vec![start];
    while let Some(node) = frontier.pop() {
        for &(from, to) in &cables {
            let neighbour = if node == from {
                to
            } else if node == to {
                from
            } else {
                continue;
            };
            if network.insert(neighbour) {
                frontier.push(neighbour);
            }
        }
    }
    network
}

fn cable_into(world: &World, tip: Entity) -> Option<(Entity, Entity)> {
    world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .find(|(_, cable)| cable.to == tip && !cable.directional)
        .map(|(cable, connection)| (cable, connection.from))
}

fn has<T: Component>(world: &World, entity: Entity) -> bool {
    world.satisfies::<&T>(entity).unwrap_or(false)
}

fn axon_target(
    world: &World,
    targets: &Targets,
    own: &HashSet<Entity>,
    from: Entity,
    position: Vec3,
    reach: f32,
) -> Option<(Entity, Vec3)> {
    let on_source = world
        .get::<&Position>(from)
        .ok()
        .filter(|_| !has::<Compartment>(world, from))
        .map(|source| (from, source.position))
        .filter(|&(_, at)| {
            at.distance(position) < CONNECTION_REACH
                || targets
                    .piece_near(position, reach, |entity| entity == from)
                    .is_some()
        });
    if on_source.is_some() {
        return on_source;
    }
    targets.piece_near(position, reach, |entity| {
        !own.contains(&entity)
            && (has::<LeakyNeuron>(world, entity) || has::<Compartment>(world, entity))
    })
}

fn connection_at(world: &World, position: Vec3) -> Option<Entity> {
    let hidden = hidden_connections(world);
    connection_paths(
        world,
        &compartment_radii(world),
        &Theme::default().palette(),
    )
    .into_iter()
    .filter(|(connection, _)| !hidden.contains(connection))
    .filter_map(|(connection, path)| {
        let to_ends = path.points.iter().map(|point| point.distance(position));
        let distance = to_ends.chain(path.distance_to(position)).reduce(f32::min)?;
        (distance < CONNECTION_SELECTION_RANGE + path.radius).then_some((connection, distance))
    })
    .min_by(|(_, a), (_, b)| a.total_cmp(b))
    .map(|(connection, _)| connection)
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
        if let Ok(mut size) = world.get::<&mut MeterSize>(entity) {
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
    take_connections(world, recording, cut.into_iter().chain(dangling).collect());
}

fn take_connections(world: &mut World, recording: &mut Recording, connections: Vec<Entity>) {
    for connection in connections {
        recording.take(world, connection);
    }
}

fn connections_near(world: &World, position: Vec3) -> Vec<Entity> {
    let paths = connection_paths(
        world,
        &compartment_radii(world),
        &Theme::default().palette(),
    );
    let hidden = hidden_connections(world);
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
            if hidden.contains(entity) {
                return false;
            }
            let distance = match paths.get(entity) {
                Some(path) => path.distance_to(position),
                None => distance_to_segment(from.position, to.position, position),
            };
            distance.is_some_and(|distance| distance < ERASE_RADIUS)
        })
        .map(|(entity, _)| entity)
        .collect()
}
