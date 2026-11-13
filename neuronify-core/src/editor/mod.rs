use std::collections::HashSet;
use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Component, Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::dendrites::{compartment_tree, spawn_dendrites};
use crate::elements::{
    spawn_axon_cable, spawn_cable, spawn_compartment, spawn_connection, spawn_dendrite_compartment,
    Element, Meter,
};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::visual_input::{visual_input_frame, VisualInput};
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::bounds::note_corners;
use crate::rendering::{compartment_radii, connection_paths, distance_to_segment, Theme};
use crate::simulation::{touch_sensor_near, Touch};
use crate::tips::{onward, tip_at};
use crate::tools::{
    ConnectionHint, ConnectionMode, ConnectionOffer, ConnectionTool, Stimulation, StimulationTool,
    Tool,
};
use attach::{cable_spot, Spot};
use drag::Drag;
use history::{History, Recording, Snapshot};

mod attach;
mod drag;
mod history;
mod targets;
#[cfg(test)]
mod tests;

pub use attach::{Bead, GLIDE_DURATION};
pub use targets::{Shape, Targets};

const CONNECTION_REACH: f32 = 2.0 * NODE_RADIUS;
const DWELL_SLACK: f32 = 0.5;
const STAYING_REACH: f32 = 1.5;
const SHARPEST_BEND: f32 = std::f32::consts::FRAC_PI_4;
const SHARPEST_ENDING_TURN: f32 = std::f32::consts::FRAC_PI_4;
const CORNER_REACH: f32 = 1.0;
const SPEAKER_OFFSET: Vec3 = Vec3::new(3.0, 0.0, 0.0);
const CONNECTION_SELECTION_RANGE: f32 = 0.6 * NODE_RADIUS;
const DENDRITE_GAP: f32 = 2.0 * SEGMENT_LENGTH;
const HINT_DURATION: Seconds = Seconds(3.0);
const HINT_FADE: Seconds = Seconds(0.25);
const HINT_LENGTH: f32 = 4.0 * NODE_RADIUS;
const HINT_SWAY: f32 = NODE_RADIUS;
const HINT_SEGMENTS: usize = 8;
const HINT_WIGGLES_PER_SECOND: f32 = 0.7;
const HINT_WIGGLE_LAG: f32 = 1.5;
const HINTED_NEURONS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pan(pub Vec3);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedNeuron {
    pub neuron: Entity,
    pub painted: bool,
}

#[derive(Clone, Copy, Debug)]
struct Dwell {
    anchor: Vec3,
    since: Seconds,
    neuron: Option<Entity>,
    spent: bool,
}

#[derive(Default)]
pub struct Editor {
    pub tool: Tool,
    pub compartments: bool,
    pub stimulation: Stimulation,
    pub selection: Option<Entity>,
    pub touch_reach: f32,
    pressed_sensor: bool,
    pulling: bool,
    placed: Option<Entity>,
    connected: HashSet<Entity>,
    connected_earlier: usize,
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
    settings_changed: bool,
    shortcut: Option<Meter>,
    wired: bool,
    presses: Vec<Vec3>,
    placed_neuron: Option<PlacedNeuron>,
    ending: Option<Entity>,
    drawn: HashSet<Entity>,
    spot: Option<Spot>,
    attached: HashSet<Entity>,
    dwell: Option<Dwell>,
    soma_ahead: Option<Entity>,
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
            _ if self.pulling => true,
            Tool::StaticConnection => true,
            Tool::Place(_) => !self.grows_axon(world, connection.from),
            tool => tool.meter().is_some(),
        };
        let end = self
            .ending
            .and_then(|ending| world.get::<&Position>(ending).ok())
            .map_or(connection.end, |ending| ending.position);
        let clear_of_source = connection.start.distance(end) > TERMINAL_INSET;
        (drawn && clear_of_source).then_some((connection.start, end))
    }

    pub fn cable_being_drawn(&self, world: &World) -> Option<(Entity, Vec3, Vec3)> {
        let (start, end) = self.connection_being_drawn(world)?;
        let from = self.connection_tool.as_ref()?.from;
        let wire = self.tool.meter().is_some() || Meter::of(world, from).is_some();
        (!wire).then_some((from, start, end))
    }

    pub fn axon_being_grown(&self, world: &World) -> Option<(Entity, Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        let grows = match self.tool {
            Tool::Axon | Tool::MyelinatedAxon => true,
            Tool::Place(_) => self.grows_axon(world, connection.from),
            _ => false,
        };
        let waits = self.soma_ahead.is_some();
        (grows && !waits).then_some((connection.from, connection.start, connection.end))
    }

    pub fn axon_waiting(&self, world: &World) -> Option<(Entity, Vec3, Vec3)> {
        let soma = self.soma_ahead?;
        let connection = self.connection_tool.as_ref()?;
        let start = world.get::<&Position>(soma).ok()?.position;
        Some((soma, start, connection.end))
    }

    pub fn dendrite_being_grown(&self) -> Option<(Entity, Vec3, Vec3)> {
        let connection = self.connection_tool.as_ref()?;
        (self.tool == Tool::Dendrite).then_some((connection.from, connection.start, connection.end))
    }

    pub fn connection_mode(&self, world: &World) -> ConnectionMode {
        match self.tool.connects() {
            true => ConnectionMode::of(world),
            false => ConnectionMode::One,
        }
    }

    pub fn set_connection_mode(&mut self, world: &mut World, mode: ConnectionMode) {
        if mode != ConnectionMode::of(world) {
            mode.set_in(world);
            self.settings_changed = true;
        }
    }

    pub fn ending(&self) -> Option<Entity> {
        self.connection_tool.as_ref().and(self.ending)
    }

    pub fn dwell(&mut self, world: &mut World, now: Seconds) -> Option<Entity> {
        let pointer = self
            .connection_tool
            .as_ref()
            .map(|connection| connection.end);
        let Some(pointer) = pointer.filter(|_| self.connection_mode(world) != ConnectionMode::One)
        else {
            self.dwell = None;
            return None;
        };
        let neuron = self.ending_neuron(world);
        let slack = DWELL_SLACK * self.touch_reach;
        let needed = self.connection_mode(world).dwell();
        if self
            .dwell
            .is_none_or(|dwell| dwell.anchor.distance(pointer) > slack)
        {
            self.dwell = Some(Dwell {
                anchor: pointer,
                since: now,
                neuron,
                spent: false,
            });
        }
        let dwell = self.dwell.as_mut()?;
        if dwell.neuron != neuron {
            dwell.neuron = neuron;
            dwell.since = now;
        }
        if dwell.spent || neuron.is_none() || now - dwell.since < needed {
            return None;
        }
        dwell.spent = true;
        let ending = self.ending.take()?;
        let spot = self.spot.take();
        self.attach(world, ending, spot);
        Some(ending)
    }

    pub fn dwelling(&self, world: &World, now: Seconds) -> Option<Seconds> {
        if self.connection_mode(world) == ConnectionMode::One {
            return None;
        }
        let neuron = self.ending_neuron(world)?;
        let held = self
            .dwell
            .filter(|dwell| !dwell.spent && dwell.neuron == Some(neuron));
        Some(held.map_or(Seconds(0.0), |dwell| now - dwell.since))
    }

    fn ending_neuron(&self, world: &World) -> Option<Entity> {
        let (soma, _) = soma_of(world, self.ending()?)?;
        Some(soma)
    }

    pub fn new_ending(&self, world: &World) -> Option<(Entity, Entity)> {
        let (from, _, _) = self.axon_being_grown(world)?;
        let ending = self
            .ending
            .filter(|&ending| !connected(world, from, ending))?;
        Some((from, ending))
    }

    pub fn bead(&self, world: &World) -> Option<Bead> {
        let connection = self.connection_tool.as_ref()?;
        let spot = self.spot?;
        Some(Bead {
            from: connection.from,
            onto: spot.nearest(world),
            at: spot.at(world)?,
            radius: spot.radius,
        })
    }

    pub fn drawn(&self) -> &HashSet<Entity> {
        &self.drawn
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

    pub fn offer_connection(&mut self, world: &World, now: Seconds) {
        if self.shortcut.is_some() || self.pressed_sensor {
            return;
        }
        let pressed = self
            .connection_tool
            .as_ref()
            .map(|connection| connection.from)
            .filter(|&from| self.pressed == Some(from));
        let learned = self.connected_earlier + self.connected.len() >= HINTED_NEURONS;
        let quiet = learned || alone(world);
        let offered = pressed
            .or(self.placed)
            .filter(|&from| !quiet || !world.satisfies::<&NeuronType>(from));
        if let Some(from) = offered {
            self.offer = Some(ConnectionOffer { from, since: now });
        }
    }

    fn note_connected(&mut self, world: &World, recording: &Recording) {
        for entity in recording.spawned(world) {
            let Ok(connection) = world.get::<&Connection>(entity) else {
                continue;
            };
            let wire = [connection.from, connection.to]
                .into_iter()
                .any(|end| Meter::of(world, end).is_some());
            if !connection.directional || wire {
                continue;
            }
            let cable = world.satisfies::<&Cable>(entity);
            let ends = [(!cable).then_some(connection.from), Some(connection.to)];
            self.connected.extend(
                ends.into_iter()
                    .flatten()
                    .filter(|&end| world.satisfies::<&NeuronType>(end)),
            );
        }
    }

    pub fn connection_hint(&mut self, world: &World, now: Seconds) -> Option<ConnectionHint> {
        let age = now - self.offer.as_ref()?.since;
        if age >= HINT_DURATION {
            self.offer = None;
            return None;
        }
        if !matches!(self.tool, Tool::Place(_)) && self.tool.meter().is_none() {
            return None;
        }
        let (from, start) = self.offered_source(world)?;
        let shown = (age.min(HINT_DURATION - age) / HINT_FADE).clamp(0.0, 1.0) as f32;
        let grown = 1.0 - (1.0 - shown).powi(3);
        let wiggle = age.0 as f32 * TAU * HINT_WIGGLES_PER_SECOND;
        let onward = onward(world, from);
        let across = Vec3::Y.cross(onward);
        let string = (0..=HINT_SEGMENTS)
            .map(|index| {
                let along = index as f32 / HINT_SEGMENTS as f32;
                let sway = (wiggle - HINT_WIGGLE_LAG * along).sin() * HINT_SWAY * along * along;
                start + (onward * HINT_LENGTH * along + across * sway) * grown
            })
            .collect();
        let above = onward * HINT_LENGTH + Vec3::X * (HINT_SWAY + NODE_RADIUS);
        Some(ConnectionHint {
            from,
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

    pub fn lift(&mut self, world: &mut World) -> Option<Entity> {
        let axon = self
            .connection_tool
            .as_ref()
            .filter(|_| self.tool != Tool::Dendrite);
        let mut formed = None;
        if let Some(axon) = axon {
            let measuring = self.tool.meter().is_some() || Meter::of(world, axon.from).is_some();
            if !measuring {
                formed = self
                    .ending
                    .filter(|&ending| self.form(world, ending, self.spot));
            } else if let Some(meter) = self.shortcut.filter(|_| !self.wired) {
                measure_where_released(world, meter, axon);
            }
        }
        if self.ending.is_none() {
            self.reach_release(world);
        }
        if let Some(held) = self.held {
            self.selection = Some(held);
        }
        self.release(world);
        formed
    }

    fn reach_release(&self, world: &mut World) {
        let stroke = self.axon_being_grown(world).or(self.dendrite_being_grown());
        let Some((tip, _, release)) = stroke else {
            return;
        };
        let new = self
            .recording
            .as_ref()
            .is_some_and(|recording| recording.is_new(tip));
        let Some((cable, parent)) = cable_into(world, tip).filter(|_| new) else {
            return;
        };
        let Ok(start) = world.get::<&Position>(parent).map(|p| p.position) else {
            return;
        };
        let length = start.distance(release) as f64 * MICROMETRES_PER_UNIT;
        let Ok(mut cable) = world.get::<&mut Cable>(cable) else {
            return;
        };
        if length <= cable.length {
            return;
        }
        cable.length = length;
        drop(cable);
        if let Ok(mut tip) = world.get::<&mut Position>(tip) {
            tip.position = release;
        }
    }

    pub fn take_presses(&mut self) -> Vec<Vec3> {
        std::mem::take(&mut self.presses)
    }

    pub fn take_placed_neuron(&mut self) -> Option<PlacedNeuron> {
        self.placed_neuron.take()
    }

    pub fn take_edited(&mut self) -> bool {
        std::mem::take(&mut self.edited)
    }

    pub fn mark_edited(&mut self) {
        self.edited = true;
    }

    pub fn mark_settings_changed(&mut self) {
        self.settings_changed = true;
    }

    pub fn take_settings_changed(&mut self) -> bool {
        std::mem::take(&mut self.settings_changed)
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
        self.pressed_sensor = false;
        self.pulling = false;
        self.placed = None;
        self.ending = None;
        self.spot = None;
        self.attached.clear();
        self.dwell = None;
        self.soma_ahead = None;
        self.drawn.clear();
        if let Some(recording) = self.recording.take() {
            self.note_connected(world, &recording);
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
            connected_earlier: self.connected_earlier + self.connected.len(),
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
        let before = self.adjusting.take().or_else(|| {
            let mut parts = compartment_tree(world, entity);
            parts.extend(crate::elements::axon_ending(world, entity));
            Snapshot::of(world, entity, parts)
        });
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
            .query::<(Entity, &Connection)>()
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

    pub fn edit(&mut self, world: &mut World, position: Vec3) -> Option<Pan> {
        self.settle();
        let pressed = self.recording.is_none();
        let mut recording = self
            .recording
            .take()
            .unwrap_or_else(|| Recording::start(world));
        let pan = if self.pulling {
            self.extend_connection(world, position);
            None
        } else if pressed && self.tool == Tool::Edit && self.pull_tip(world, position) {
            None
        } else {
            self.use_tool(world, &mut recording, position)
        };
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
    ) -> Option<Pan> {
        match self.tool {
            Tool::View => return self.view(world, position),
            Tool::Edit => return self.select(world, recording, position),
            Tool::Place(element) => self.add(world, recording, element, position),
            Tool::Voltmeter | Tool::SpikeDetector | Tool::RatePlot | Tool::Loudspeaker => {
                self.measure(world, position)
            }
            Tool::StaticConnection => self.connect(world, position),
            Tool::Axon | Tool::MyelinatedAxon => self.grow_axon(world, recording, position),
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
        self.pan(position)
    }

    pub fn pan(&mut self, position: Vec3) -> Option<Pan> {
        match self.move_origin {
            Some(origin) => Some(Pan(position - origin)),
            None => {
                self.move_origin = Some(position);
                None
            }
        }
    }

    fn pull_tip(&mut self, world: &World, position: Vec3) -> bool {
        let Some(tip) = self.tip_at(world, position) else {
            return false;
        };
        self.connection_tool = Some(tip);
        self.pulling = true;
        true
    }

    fn tip_at(&self, world: &World, position: Vec3) -> Option<ConnectionTool> {
        tip_at(world, self.selection, position, self.touch_reach).map(|tip| ConnectionTool {
            start: tip.center,
            end: position,
            from: tip.from,
        })
    }

    pub fn pulled_tip(&self) -> Option<Entity> {
        let connection = self.connection_tool.as_ref()?;
        let fresh = self.pulling || self.previous_creation == Some(connection.from);
        fresh.then_some(connection.from)
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

    pub fn snap_back(&mut self, world: &mut World) {
        if self.dragging.is_none() && self.resizing.is_none() {
            return;
        }
        if let Some(recording) = self.recording.take() {
            self.history.revert(recording, world);
        }
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
        if self.stimulation_tool.is_some() || self.pulling {
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
        self.compartments && world.satisfies::<hecs::Or<&LeakyNeuron, &Compartment>>(from)
    }

    fn add(
        &mut self,
        world: &mut World,
        recording: &mut Recording,
        element: Element,
        position: Vec3,
    ) {
        if let Some(drag) = &mut self.dragging {
            drag.move_to(world, recording, position);
        } else if let Some(connection) = &self.connection_tool {
            if self.grows_axon(world, connection.from) {
                self.extend_axon(world, recording, position);
            } else {
                self.extend_connection(world, position);
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
        let connection = start_connection(candidates, position)
            .or_else(|| self.pull_offer(world, position))
            .or_else(|| self.tip_at(world, position));
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
        if let Some(pressed) = self.pressed {
            self.press_sensor(world, pressed);
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

    fn press_sensor(&mut self, world: &World, entity: Entity) {
        if !has::<TouchSensor>(world, entity) {
            return;
        }
        if let Ok(sensor) = world.get::<&Position>(entity) {
            self.presses.push(sensor.position);
            self.pressed_sensor = true;
        }
    }

    pub fn way_to_next(&self, world: &World, position: Vec3) -> Option<f32> {
        let element = self.painting()?;
        self.spread(world, element, position)
    }

    fn spread(&self, world: &World, element: Element, position: Vec3) -> Option<f32> {
        let previous = self.previous_creation?;
        if element.is_neuron() && self.compartments {
            return dendrite_spread(world, previous, position);
        }
        let at = world.get::<&Position>(previous).ok()?.position;
        Some(at.distance(position) / MIN_CREATION_DISTANCE_DEFAULT)
    }

    fn crowds_previous(&self, world: &World, element: Element, position: Vec3) -> bool {
        self.spread(world, element, position)
            .is_some_and(|spread| spread < 1.0)
    }

    fn place(&mut self, world: &mut World, element: Element, position: Vec3) {
        if self.crowds_previous(world, element, position) {
            return;
        }
        let painted = self.painting().is_some();
        let entity = element.spawn(world, position);
        if element.is_neuron() {
            self.placed = Some(entity);
            self.placed_neuron = Some(PlacedNeuron {
                neuron: entity,
                painted,
            });
        }
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

    fn connect(&mut self, world: &mut World, position: Vec3) {
        if self.connection_tool.is_some() {
            self.extend_connection(world, position);
        } else {
            self.connection_tool = start_connection(sources(world), position)
                .or_else(|| self.tip_at(world, position))
                .or_else(|| {
                    meter_at(world, position).map(|(meter, start)| ConnectionTool {
                        start,
                        end: position,
                        from: meter,
                    })
                });
        }
    }

    fn extend_connection(&mut self, world: &mut World, position: Vec3) {
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
        connection.end = position;
        let from = connection.from;
        let neurons = positions_with::<LeakyNeuron>(world);
        let near = nearest_within(neurons, position, CONNECTION_REACH).map(|(neuron, _)| neuron);
        if near.is_none() {
            measure_with_meter_at(world, from, position);
        }
        self.ending = near.filter(|&neuron| neuron != from && !self.attached.contains(&neuron));
    }

    fn source(&self) -> Option<Entity> {
        self.connection_tool
            .as_ref()
            .map(|connection| connection.from)
    }

    fn form(&self, world: &mut World, ending: Entity, spot: Option<Spot>) -> bool {
        let Some(from) = self.source() else {
            return false;
        };
        if from == ending || connected(world, from, ending) {
            return false;
        }
        if self.axon_being_grown(world).is_some() {
            let synapse = lay_axon(world, from, ending, true, self.myelinates());
            if let Some(at) = spot.and_then(|spot| spot.at(world)) {
                glide(world, synapse, at);
            }
        } else {
            spawn_connection(world, from, ending);
        }
        true
    }

    fn attach(&mut self, world: &mut World, ending: Entity, spot: Option<Spot>) {
        self.form(world, ending, spot);
        let Some((soma, at)) = soma_of(world, ending) else {
            return;
        };
        self.attached.insert(soma);
        if self.connection_mode(world) != ConnectionMode::Chain {
            return;
        }
        self.soma_ahead = (ending != soma).then_some(soma);
        self.previous_creation = Some(soma);
        if let Some(connection) = &mut self.connection_tool {
            connection.from = soma;
            connection.start = at;
        }
    }

    fn wait_for_soma(&mut self, world: &World, position: Vec3) -> bool {
        let Some(soma) = self.soma_ahead else {
            return false;
        };
        let reached = Targets::new(world)
            .piece_near(position, self.touch_reach, |entity| entity == soma)
            .is_some();
        if reached {
            self.soma_ahead = None;
        }
        !reached
    }

    fn grow_axon(&mut self, world: &mut World, recording: &mut Recording, position: Vec3) {
        if self.connection_tool.is_some() {
            self.extend_axon(world, recording, position);
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

    fn extend_axon(&mut self, world: &mut World, recording: &mut Recording, position: Vec3) {
        let Some(connection) = &mut self.connection_tool else {
            return;
        };
        connection.end = position;
        if self.wait_for_soma(world, position) {
            self.ending = None;
            self.spot = None;
            return;
        }
        let ends = self.aim_axon(world, position) && self.straighten(world, recording, position);
        if !ends {
            self.grow_tip(world, recording, position);
        }
    }

    fn straighten(&mut self, world: &mut World, recording: &mut Recording, position: Vec3) -> bool {
        let Some(at) = self.ending_at(world) else {
            return true;
        };
        while let Some(tip) = self.source() {
            let Some(previous) = turns_sharply(world, tip, at) else {
                return true;
            };
            if !recording.is_new(tip) || leads_on(world, tip) {
                self.ending = None;
                self.spot = None;
                return false;
            }
            self.take_back(world, recording, tip);
            let Ok(start) = world.get::<&Position>(previous).map(|p| p.position) else {
                return true;
            };
            self.previous_creation = Some(previous);
            self.connection_tool = Some(ConnectionTool {
                start,
                end: position,
                from: previous,
            });
        }
        true
    }

    fn ending_at(&self, world: &World) -> Option<Vec3> {
        match self.spot {
            Some(spot) => spot.at(world),
            None => Some(world.get::<&Position>(self.ending?).ok()?.position),
        }
    }

    fn take_back(&mut self, world: &mut World, recording: &mut Recording, compartment: Entity) {
        for connection in connections_touching(world, compartment) {
            recording.take(world, connection);
        }
        recording.take(world, compartment);
        self.drawn.remove(&compartment);
    }

    fn aim_axon(&mut self, world: &World, position: Vec3) -> bool {
        let Some(from) = self.source() else {
            return true;
        };
        let targets = Targets::new(world);
        let mut skip = cable_network(world, from);
        for &neuron in &self.attached {
            skip.extend(cable_network(world, neuron));
        }
        let reach = self.touch_reach;
        let target = axon_target(world, &targets, &skip, from, position, reach).or_else(|| {
            let staying = cable_network(world, self.ending?);
            targets.piece_near(position, STAYING_REACH * reach, |entity| {
                staying.contains(&entity) && !skip.contains(&entity) && takes_axons(world, entity)
            })
        });
        let soma = target
            .map(|(target, _)| target)
            .filter(|&target| !has::<Compartment>(world, target));
        if let Some(soma) = soma {
            self.spot = None;
            self.ending = (soma != from).then_some(soma);
            return true;
        }
        let staying = self.ending.map(|ending| cable_network(world, ending));
        let accepts = |entity: Entity| !skip.contains(&entity) && takes_axons(world, entity);
        let spot = cable_spot(world, position, reach, accepts).or_else(|| {
            let staying = staying?;
            cable_spot(world, position, STAYING_REACH * reach, |entity| {
                staying.contains(&entity) && accepts(entity)
            })
        });
        self.spot = spot;
        self.ending = spot.map(|spot| spot.nearest(world));
        spot.is_some()
    }

    fn grow_tip(&mut self, world: &mut World, recording: &mut Recording, position: Vec3) {
        let myelinated = self.myelinates();
        let spacing = match myelinated {
            true => INTERNODE_LENGTH,
            false => MIN_CREATION_DISTANCE_AXON,
        };
        let Some(from) = self.source() else {
            return;
        };
        if self.too_near_previous(world, position, spacing) {
            return;
        }
        let Some(from) = self.back_up(world, recording, from, position, spacing) else {
            return;
        };
        let Ok(neuron_type) = world.get::<&NeuronType>(from).map(|t| *t) else {
            return;
        };
        let Some(steps) = steps_towards(world, from, position, spacing) else {
            return;
        };
        let mut previous = from;
        for &at in &steps {
            let compartment = spawn_compartment(world, at, neuron_type);
            lay_axon(world, previous, compartment, false, myelinated);
            previous = compartment;
        }
        self.grew(previous, steps, position);
    }

    fn grew(&mut self, tip: Entity, steps: Vec<Vec3>, position: Vec3) {
        self.previous_creation = Some(tip);
        self.connection_tool = Some(ConnectionTool {
            start: steps.last().copied().unwrap_or(position),
            end: position,
            from: tip,
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
        let Some(steps) = steps_towards(world, from, position, SEGMENT_LENGTH) else {
            return;
        };
        let Some(tip) = lay_dendrite(world, from, &steps) else {
            return;
        };
        self.grew(tip, steps, position);
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
            let Some((_, previous)) = cable_into(world, backed) else {
                break false;
            };
            let at = world.get::<&Position>(backed).ok()?.position;
            let before = world.get::<&Position>(previous).ok()?.position;
            let sharp = (before - at).angle_between(position - at) < SHARPEST_BEND;
            if !sharp || !recording.is_new(backed) || leads_on(world, backed) {
                break sharp;
            }
            self.take_back(world, recording, backed);
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
                    .filter(|&&(meter, _, _)| resizable(world, meter).is_some())
                    .map(|&(meter, _, frame)| (meter, frame)),
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

fn alone(world: &World) -> bool {
    let mut elements = world.query::<&Position>().without::<&Compartment>();
    elements.iter().nth(1).is_none()
}

fn dendrite_spread(world: &World, neuron: Entity, position: Vec3) -> Option<f32> {
    let soma = world.get::<&Position>(neuron).ok()?.position;
    let tree: Vec<Vec3> = compartment_tree(world, neuron)
        .into_iter()
        .filter_map(|dendrite| world.get::<&Position>(dendrite).ok().map(|p| p.position))
        .collect();
    let low = tree.iter().fold(soma, |low, &position| low.min(position));
    let high = tree.iter().fold(soma, |high, &position| high.max(position));
    let footprint = high - low + Vec3::splat(DENDRITE_GAP);
    let offset = (position - soma).abs();
    Some((offset.x / footprint.x).max(offset.z / footprint.z))
}

fn meter_frames(world: &World) -> Vec<(Entity, Vec3, Frame)> {
    let meters = world
        .query::<(Entity, (&Position, &MeterSize))>()
        .iter()
        .map(|(meter, (meter_position, size))| {
            let frame = Frame::new(meter_position.position, size);
            (meter, meter_position.position, frame)
        })
        .collect::<Vec<_>>();
    let visual_inputs = world
        .query::<(Entity, &Position)>()
        .with::<&VisualInput>()
        .iter()
        .map(|(input, center)| (input, center.position, visual_input_frame(center.position)))
        .collect::<Vec<_>>();
    [meters, visual_inputs].concat()
}

fn positions_with<T: Component>(world: &World) -> Vec<(Entity, Vec3)> {
    world
        .query::<(Entity, &Position)>()
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

fn steps_towards(world: &World, from: Entity, position: Vec3, spacing: f32) -> Option<Vec<Vec3>> {
    let start = world.get::<&Position>(from).ok()?.position;
    let direction = (position - start).normalize_or_zero();
    let steps = (start.distance(position) / spacing).floor().max(1.0) as usize;
    Some(
        (1..=steps)
            .map(|step| start + direction * spacing * step as f32)
            .collect(),
    )
}

fn lay_dendrite(world: &mut World, from: Entity, steps: &[Vec3]) -> Option<Entity> {
    let neuron_type = *world.get::<&NeuronType>(from).ok()?;
    let mut previous = from;
    for &position in steps {
        let compartment = spawn_dendrite_compartment(world, position, neuron_type);
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

fn connected(world: &World, from: Entity, to: Entity) -> bool {
    world
        .query::<&Connection>()
        .iter()
        .any(|connection| connection.from == from && connection.to == to)
}

fn cable_network(world: &World, start: Entity) -> HashSet<Entity> {
    let cables: Vec<(Entity, Entity)> = world
        .query::<&Connection>()
        .with::<&Cable>()
        .iter()
        .filter(|cable| !cable.directional)
        .map(|cable| (cable.from, cable.to))
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

fn connections_touching(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .query::<(Entity, &Connection)>()
        .iter()
        .filter(|(_, connection)| connection.from == entity || connection.to == entity)
        .map(|(connection, _)| connection)
        .collect()
}

fn cable_into(world: &World, tip: Entity) -> Option<(Entity, Entity)> {
    world
        .query::<(Entity, &Connection)>()
        .with::<&Cable>()
        .iter()
        .find(|(_, cable)| cable.to == tip && !cable.directional)
        .map(|(cable, connection)| (cable, connection.from))
}

fn glide(world: &mut World, synapse: Entity, from: Vec3) {
    let since = crate::input::now();
    world.insert_one(synapse, Gliding { from, since }).ok();
}

fn turns_sharply(world: &World, tip: Entity, at: Vec3) -> Option<Entity> {
    if !has::<Compartment>(world, tip) {
        return None;
    }
    let (_, previous) = cable_into(world, tip)?;
    let tip_at = world.get::<&Position>(tip).ok()?.position;
    let before = world.get::<&Position>(previous).ok()?.position;
    let turn = (tip_at - before).angle_between(at - tip_at);
    (turn > SHARPEST_ENDING_TURN).then_some(previous)
}

fn leads_on(world: &World, compartment: Entity) -> bool {
    world
        .query::<&Connection>()
        .iter()
        .any(|connection| connection.from == compartment)
}

fn soma_of(world: &World, entity: Entity) -> Option<(Entity, Vec3)> {
    let soma = match has::<LeakyNeuron>(world, entity) {
        true => entity,
        false => cable_network(world, entity)
            .into_iter()
            .find(|&member| has::<LeakyNeuron>(world, member))?,
    };
    let at = world.get::<&Position>(soma).ok()?.position;
    Some((soma, at))
}

fn has<T: Component>(world: &World, entity: Entity) -> bool {
    world.satisfies::<&T>(entity)
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
        !own.contains(&entity) && takes_axons(world, entity)
    })
}

fn takes_axons(world: &World, entity: Entity) -> bool {
    has::<LeakyNeuron>(world, entity) || has::<Dendrite>(world, entity)
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
        let connection = world.get::<&Connection>(connection).ok()?;
        let cable_beyond =
            !connection.directional && world.satisfies::<&Compartment>(connection.to);
        cable_beyond.then_some(connection.to)
    });
    let near: Vec<Entity> = world
        .query::<(Entity, &Position)>()
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
        .query::<(Entity, &Connection)>()
        .with::<&Deletable>()
        .iter()
        .filter(|(_, connection)| {
            !world.contains(connection.from) || !world.contains(connection.to)
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
        .query::<(Entity, &Connection)>()
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
