use std::collections::HashSet;

use glam::Vec3;
use hecs::{Component, Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::dendrites::{dendrite_tree, spawn_dendrites};
use crate::elements::{spawn_cable, spawn_compartment, spawn_synapse, spawn_voltmeter, Element};
use crate::measurement::voltmeter::{Corner, Frame, Voltmeter};
use crate::rendering::{compartment_radii, connection_paths, distance_to_segment, Theme};
use crate::simulation::touch_sensor_near;
use crate::tools::{ConnectionTool, StimulationTool, Tool};
use drag::Drag;

mod drag;
#[cfg(test)]
mod tests;

const CONNECTION_REACH: f32 = 2.0 * NODE_RADIUS;
const ELEMENT_ERASE_REACH: f32 = 1.5 * NODE_RADIUS;
const VOLTMETER_REACH: f32 = NODE_RADIUS;
const CORNER_REACH: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pan(pub Vec3);

#[derive(Default)]
pub struct Editor {
    pub tool: Tool,
    pub dendrites: bool,
    pub selection: Option<Entity>,
    connection_tool: Option<ConnectionTool>,
    stimulation_tool: Option<StimulationTool>,
    previous_creation: Option<Entity>,
    move_origin: Option<Vec3>,
    dragging: Option<Drag>,
    resizing: Option<(Entity, Corner)>,
}

impl Editor {
    pub fn touch(&self) -> Option<Vec3> {
        self.stimulation_tool.as_ref().map(|touch| touch.position)
    }

    pub fn connection_being_drawn(&self) -> Option<(Vec3, Vec3)> {
        match (self.tool, &self.connection_tool) {
            (Tool::StaticConnection, Some(connection)) => Some((connection.start, connection.end)),
            _ => None,
        }
    }

    pub fn release(&mut self) {
        self.stimulation_tool = None;
        self.connection_tool = None;
        self.previous_creation = None;
        self.move_origin = None;
        self.dragging = None;
        self.resizing = None;
    }

    pub fn edit(&mut self, world: &mut World, position: Vec3, shift_down: bool) -> Option<Pan> {
        match self.tool {
            Tool::Select => return self.select(world, position),
            Tool::Place(element) => self.place(world, element, position),
            Tool::Voltmeter => self.place_voltmeter(world, position),
            Tool::StaticConnection => self.connect(world, position, shift_down),
            Tool::Axon => self.grow_axon(world, position, shift_down),
            Tool::Erase => erase(world, position),
            Tool::Stimulate => self.stimulation_tool = Some(StimulationTool { position }),
        }
        None
    }

    pub fn view(&mut self, world: &World, position: Vec3) -> Option<Pan> {
        let touching = self.stimulation_tool.is_some()
            || (self.move_origin.is_none() && touch_sensor_near(world, position));
        if touching {
            self.stimulation_tool = Some(StimulationTool { position });
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

    fn too_near_previous(&self, world: &World, position: Vec3) -> bool {
        let minimum_distance = match self.tool {
            Tool::Axon => MIN_CREATION_DISTANCE_AXON,
            _ => MIN_CREATION_DISTANCE_DEFAULT,
        };
        self.previous_creation
            .and_then(|previous| world.get::<&Position>(previous).ok().map(|p| p.position))
            .is_some_and(|previous| previous.distance(position) < minimum_distance)
    }

    fn place(&mut self, world: &mut World, element: Element, position: Vec3) {
        if self.too_near_previous(world, position) {
            return;
        }
        let entity = element.spawn(world, position);
        if element.is_neuron() && self.dendrites {
            spawn_dendrites(world, entity);
        }
        if element == Element::Note {
            self.selection = Some(entity);
        }
        self.previous_creation = Some(entity);
    }

    fn place_voltmeter(&mut self, world: &mut World, position: Vec3) {
        if self.too_near_previous(world, position) {
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
        match &mut self.connection_tool {
            Some(connection) => {
                connect_to_neuron_near(world, connection, position, shift_down, spawn_synapse);
                connection.end = position;
            }
            None => self.connection_tool = start_connection(sources(world), position),
        }
    }

    fn grow_axon(&mut self, world: &mut World, position: Vec3, shift_down: bool) {
        let Some(connection) = &mut self.connection_tool else {
            let axon_sources = [
                positions_with::<StaticConnectionSource>(world),
                sources(world),
            ];
            self.connection_tool = start_connection(axon_sources.concat(), position);
            if let Some(connection) = &self.connection_tool {
                self.previous_creation = Some(connection.from);
            }
            return;
        };
        connection.end = position;
        let reached_neuron = connect_to_neuron_near(
            world,
            connection,
            position,
            shift_down,
            |world, from, to| spawn_cable(world, from, to, true),
        );
        let from = connection.from;
        if reached_neuron || self.too_near_previous(world, position) {
            return;
        }
        let Ok(neuron_type) = world.get::<&NeuronType>(from).map(|t| *t) else {
            return;
        };
        let compartment = spawn_compartment(world, position, neuron_type);
        spawn_cable(world, from, compartment, false);
        self.previous_creation = Some(compartment);
        self.connection_tool = Some(ConnectionTool {
            start: position,
            end: position,
            from: compartment,
        });
    }

    fn select(&mut self, world: &mut World, position: Vec3) -> Option<Pan> {
        if let Some(drag) = &self.dragging {
            drag.move_to(world, position);
        } else if let Some((voltmeter, corner)) = self.resizing {
            resize(world, voltmeter, corner, position);
        } else if let Some(origin) = self.move_origin {
            return Some(Pan(position - origin));
        } else {
            self.pick(world, position);
        }
        None
    }

    fn pick(&mut self, world: &World, position: Vec3) {
        let frames: Vec<_> = world
            .query::<(&Position, &VoltmeterSize)>()
            .with::<&Voltmeter>()
            .iter()
            .map(|(voltmeter, (voltmeter_position, size))| {
                let frame = Frame::new(voltmeter_position.position, size);
                (voltmeter, voltmeter_position.position, frame)
            })
            .collect();
        self.resizing = frames.iter().find_map(|(voltmeter, _, frame)| {
            Some((*voltmeter, frame.corner_near(position, CORNER_REACH)?))
        });
        if self.resizing.is_some() {
            return;
        }
        let picked = frames
            .iter()
            .find(|(_, _, frame)| frame.contains(position))
            .map(|&(voltmeter, voltmeter_position, _)| (voltmeter, voltmeter_position))
            .or_else(|| element_near(world, position))
            .or_else(|| note_under(world, position));
        match picked {
            Some((entity, entity_position)) => {
                self.selection = Some(entity);
                let offset = (entity_position - position).with_y(0.0);
                self.dragging = Some(Drag::new(world, entity, offset));
            }
            None => {
                self.selection = None;
                self.move_origin = Some(position);
            }
        }
    }
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

fn resize(world: &mut World, voltmeter: Entity, corner: Corner, position: Vec3) {
    let Some(frame) = Frame::of(world, voltmeter) else {
        return;
    };
    let resized = frame.resized(corner, position);
    if let Ok(mut size) = world.get::<&mut VoltmeterSize>(voltmeter) {
        *size = resized.size();
    }
    if let Ok(mut voltmeter_position) = world.get::<&mut Position>(voltmeter) {
        voltmeter_position.position = resized.position();
    }
}

fn erase(world: &mut World, position: Vec3) {
    let near: Vec<Entity> = world
        .query::<&Position>()
        .with::<&Deletable>()
        .iter()
        .filter(|(_, element)| element.position.distance(position) < ELEMENT_ERASE_REACH)
        .map(|(entity, _)| entity)
        .collect();
    let erased: HashSet<Entity> = near
        .iter()
        .flat_map(|&entity| dendrite_tree(world, entity))
        .chain(near.iter().copied())
        .collect();
    for entity in erased {
        world.despawn(entity).unwrap();
    }

    let paths = connection_paths(
        world,
        &compartment_radii(world),
        &Theme::default().palette(),
    );
    let connections: Vec<Entity> = world
        .query::<&Connection>()
        .with::<&Deletable>()
        .iter()
        .filter(|(entity, connection)| {
            let (Ok(from), Ok(to)) = (
                world.get::<&Position>(connection.from),
                world.get::<&Position>(connection.to),
            ) else {
                return true;
            };
            let distance = match paths.get(entity) {
                Some(path) => path.distance_to(position),
                None => distance_to_segment(from.position, to.position, position),
            };
            distance.is_some_and(|distance| distance < ERASE_RADIUS)
        })
        .map(|(entity, _)| entity)
        .collect();
    for connection in connections {
        world.despawn(connection).unwrap();
    }
}
