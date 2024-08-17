use chrono::{DateTime, Duration, Utc};
use hecs::serialize::column::*;
use js_sys::Uint8Array;
use postcard::ser_flavors::Flavor;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use visula::initialize_event_loop_and_window_with_config;
use visula::initialize_logger;
use visula::winit::keyboard::ModifiersKeyState;
use visula::Application;
use visula::RunConfig;
use visula::Simulation;
use visula::Vector3;
use wasm_bindgen::prelude::*;

use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use std::collections::HashMap;

use crate::measurement::voltmeter::VoltageSeries;

use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;
use cgmath::Vector4;
use egui::Color32;
use egui::LayerId;
use egui::Pos2;
use egui_plot::PlotBounds;
use egui_plot::{Line, PlotPoints};
use glam::Quat;
use glam::Vec3;
use hecs::Entity;

use crate::measurement::voltmeter::RollingWindow;
use crate::measurement::voltmeter::VoltageMeasurement;
use crate::measurement::voltmeter::Voltmeter;
use strum::EnumIter;
use strum::IntoEnumIterator;
use visula::winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
};
use visula::Renderable;
use visula::{
    CustomEvent, InstanceBuffer, LineDelegate, Lines, RenderData, SphereDelegate, Spheres,
};
use visula_derive::Instance;

pub mod measurement;
pub mod serialization;
use crate::serialization::{LoadContext, SaveContext};

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum Tool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    CurrentSource,
    Voltmeter,
    StaticConnection,
    LearningConnection,
    Axon,
    Erase,
    Stimulate,
}

const NODE_RADIUS: f32 = 1.0;
const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;
const SIGMA: f32 = 1.0 * NODE_RADIUS;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NeuronType {
    Excitatory,
    Inhibitory,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NeuronDynamics {
    pub voltage: f64,
    pub current: f64,
    pub refraction: f64,
    pub fired: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neuron {
    pub initial_voltage: f64,
    pub reset_potential: f64,
    pub resting_potential: f64,
    pub threshold: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SynapseCurrent {
    pub current: f64,
    pub tau: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompartmentCurrent {
    capacitance: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Selectable {
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentSource {
    pub current: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticConnectionSource {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LearningSynapse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreviousCreation {
    pub entity: Entity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deletable {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Vec3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpatialDynamics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

impl Default for Neuron {
    fn default() -> Self {
        Self::new()
    }
}

impl Neuron {
    pub fn new() -> Neuron {
        Neuron {
            initial_voltage: 0.0,
            reset_potential: -70.0,
            resting_potential: -70.0,
            threshold: 30.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionTool {
    pub start: Vec3,
    pub end: Vec3,
    pub from: Entity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trigger {
    pub total: f64,
    pub remaining: f64,
    pub current: f64,
    pub connection: Entity,
}

impl Trigger {
    pub fn new(time: f64, current: f64, connection: Entity) -> Trigger {
        Trigger {
            total: time,
            remaining: time,
            current,
            connection,
        }
    }
    pub fn decrement(&mut self, dt: f64) {
        self.remaining = (self.remaining - dt).max(0.0);
    }
    pub fn progress(&self) -> f64 {
        (self.total - self.remaining) / self.total
    }
    pub fn done(&self) -> bool {
        self.remaining <= 0.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeakCurrent {
    pub current: f64,
    pub tau: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub strength: f64,
    pub directional: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StimulateCurrent {
    pub current: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StimulationTool {
    pub position: Vec3,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Instance, Pod, Zeroable)]
pub struct Sphere {
    pub position: glam::Vec3,
    pub color: glam::Vec3,
    pub radius: f32,
    pub _padding: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub left_down: bool,
    pub position: Option<PhysicalPosition<f64>>,
}

pub struct Keyboard {
    pub shift_down: bool,
}

pub struct Neuronify {
    pub tool: Tool,
    pub previous_creation: Option<PreviousCreation>,
    pub connection_tool: Option<ConnectionTool>,
    pub stimulation_tool: Option<StimulationTool>,
    pub world: hecs::World,
    pub time: f64,
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub spheres: Spheres,
    pub sphere_buffer: InstanceBuffer<Sphere>,
    pub connection_lines: Lines,
    pub connection_spheres: Spheres,
    pub connection_buffer: InstanceBuffer<ConnectionData>,
    pub iterations: u32,
    pub last_update: DateTime<Utc>,
    pub fps: f64,
}

#[derive(Debug)]
pub struct Error {}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct ConnectionData {
    pub start_color: Vec3,
    pub end_color: Vec3,
    pub position_a: Vec3,
    pub position_b: Vec3,
    pub strength: f32,
    pub directional: f32,
    pub _padding: [f32; 2],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct LineData {
    pub start: Vec3,
    pub end: Vec3,
    pub _padding: [f32; 2],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct MeshInstanceData {
    pub position: Vec3,
    pub _padding: f32,
    pub rotation: Quat,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Compartment {
    voltage: f64,
    m: f64,
    h: f64,
    n: f64,
    influence: f64,
    capacitance: f64,
    injected_current: f64,
}

fn nearest(
    mouse_position: &Vec3,
    (_, x): &(Entity, &Position),
    (_, y): &(Entity, &Position),
) -> Ordering {
    mouse_position
        .distance(x.position)
        .partial_cmp(&mouse_position.distance(y.position))
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn within_attachment_range(
    mouse_position: Vec3,
    (id, position): (Entity, &Position),
) -> Option<(Entity, Vec3)> {
    if mouse_position.distance(position.position) < 1.5 * NODE_RADIUS {
        Some((id, position.position))
    } else {
        None
    }
}

impl Neuronify {
    pub fn new(application: &mut visula::Application) -> Neuronify {
        application.camera_controller.enabled = false;
        application.camera_controller.center = Vector3::new(0.0, 0.0, 0.0);
        application.camera_controller.forward = Vector3::new(1.0, -1.0, 0.0);
        application.camera_controller.distance = 50.0;

        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let connection_buffer = InstanceBuffer::<ConnectionData>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let connection = connection_buffer.instance();

        let spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereDelegate {
                position: sphere.position.clone(),
                radius: sphere.radius,
                color: sphere.color,
            },
        )
        .unwrap();

        let connection_vector = connection.position_b.clone() - connection.position_a.clone();
        // TODO: Add normalize function to expressions
        let connection_endpoint = connection.position_a.clone() + connection_vector.clone()
            - connection.directional.clone() * connection_vector.clone()
                / connection_vector.clone().length()
                * NODE_RADIUS
                * 2.0;
        let connection_lines = Lines::new(
            &application.rendering_descriptor(),
            &LineDelegate {
                start: connection.position_a.clone(),
                end: connection_endpoint.clone(),
                width: 0.3.into(),
                start_color: connection.start_color.clone(),
                end_color: connection.end_color.clone(),
            },
        )
        .unwrap();

        let connection_spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereDelegate {
                position: connection_endpoint,
                radius: connection.directional.clone() * (0.5 * NODE_RADIUS),
                color: Vec3::new(136.0 / 255.0, 57.0 / 255.0, 239.0 / 255.0).into(),
            },
        )
        .unwrap();

        let world = hecs::World::new();

        Neuronify {
            spheres,
            sphere_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
            tool: Tool::ExcitatoryNeuron,
            previous_creation: None,
            connection_tool: None,
            stimulation_tool: None,
            world,
            time: 0.0,
            mouse: Mouse {
                left_down: false,
                position: None,
            },
            keyboard: Keyboard { shift_down: false },
            iterations: 4,
            last_update: Utc::now(),
            fps: 60.0,
        }
    }

    fn handle_tool(&mut self, application: &visula::Application) {
        let Neuronify {
            tool,
            mouse,
            connection_tool,
            stimulation_tool,
            world,
            previous_creation,
            ..
        } = self;
        if !mouse.left_down {
            *stimulation_tool = None;
            *connection_tool = None;
            *previous_creation = None;
            return;
        }
        let mouse_physical_position = match mouse.position {
            Some(p) => p,
            None => {
                return;
            }
        };
        let screen_position = cgmath::Vector4 {
            x: 2.0 * mouse_physical_position.x as f32 / application.config.width as f32 - 1.0,
            y: 1.0 - 2.0 * mouse_physical_position.y as f32 / application.config.height as f32,
            z: 1.0,
            w: 1.0,
        };
        let ray_clip = cgmath::Vector4 {
            x: screen_position.x,
            y: screen_position.y,
            z: -1.0,
            w: 1.0,
        };
        let aspect_ratio = application.config.width as f32 / application.config.height as f32;
        let inv_projection = application
            .camera_controller
            .projection_matrix(aspect_ratio)
            .invert()
            .unwrap();

        let ray_eye = inv_projection * ray_clip;
        let ray_eye = cgmath::Vector4 {
            x: ray_eye.x,
            y: ray_eye.y,
            z: -1.0,
            w: 0.0,
        };
        let inv_view_matrix = application
            .camera_controller
            .view_matrix()
            .invert()
            .unwrap();
        let ray_world = inv_view_matrix * ray_eye;
        let ray_world = cgmath::Vector3 {
            x: ray_world.x,
            y: ray_world.y,
            z: ray_world.z,
        }
        .normalize();
        let ray_origin = application.camera_controller.position();
        let t = -ray_origin.y / ray_world.y;
        let intersection = ray_origin + t * ray_world;
        let mouse_position = Vec3::new(intersection.x, intersection.y, intersection.z);

        let minimum_distance = match tool {
            Tool::Axon => 2.0 * NODE_RADIUS,
            _ => 6.0 * NODE_RADIUS,
        };
        let previous_too_near = if let Some(pc) = previous_creation {
            if let Ok(position) = world.get::<&Position>(pc.entity) {
                position.position.distance(mouse_position) < minimum_distance
            } else {
                false
            }
        } else {
            false
        };
        match tool {
            Tool::ExcitatoryNeuron | Tool::InhibitoryNeuron => {
                if previous_too_near {
                    return;
                }
                let dynamics = NeuronDynamics {
                    current: 0.0,
                    refraction: 0.0,
                    voltage: 0.0,
                    fired: false,
                };
                let leak_current = LeakCurrent {
                    current: 0.0,
                    tau: 1.0,
                };
                let stimulate_current = StimulateCurrent { current: 0.0 };
                let new_id = match self.tool {
                    Tool::ExcitatoryNeuron => Some(world.spawn((
                        Position {
                            position: mouse_position + 0.4 * Vec3::Y,
                        },
                        Neuron::new(),
                        NeuronType::Excitatory,
                        StaticConnectionSource {},
                        dynamics,
                        leak_current,
                        Deletable {},
                        stimulate_current,
                        Selectable { selected: false },
                    ))),
                    Tool::InhibitoryNeuron => Some(world.spawn((
                        Position {
                            position: mouse_position,
                        },
                        Neuron::new(),
                        NeuronType::Inhibitory,
                        StaticConnectionSource {},
                        dynamics,
                        leak_current,
                        Deletable {},
                        stimulate_current,
                        Selectable { selected: false },
                    ))),
                    _ => None,
                };
                if let Some(id) = new_id {
                    self.previous_creation = Some(PreviousCreation { entity: id });
                }
            }
            Tool::CurrentSource => {
                if previous_too_near {
                    return;
                }
                let id = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    StaticConnectionSource {},
                    CurrentSource { current: 150.0 },
                    Deletable {},
                    Selectable { selected: false },
                ));
                self.previous_creation = Some(PreviousCreation { entity: id });
            }
            Tool::StaticConnection | Tool::LearningConnection => {
                if let Some(ct) = connection_tool {
                    let nearest_target = world
                        .query::<&Position>()
                        .with::<&Neuron>()
                        .iter()
                        .min_by(|a, b| nearest(&mouse_position, a, b))
                        .and_then(|v| within_attachment_range(mouse_position, v));
                    if let Some((id, position)) = nearest_target {
                        let strength = if *tool == Tool::StaticConnection {
                            1.0
                        } else {
                            0.0
                        };
                        let new_connection = Connection {
                            from: ct.from,
                            to: id,
                            strength,
                            directional: true,
                        };
                        let connection_exists =
                            world.query::<&Connection>().iter().any(|(_, c)| {
                                c.from == new_connection.from && c.to == new_connection.to
                            });
                        if !connection_exists && ct.from != id {
                            let synapse_current = SynapseCurrent {
                                current: 0.0,
                                tau: 0.1,
                            };
                            if world.get::<&CurrentSource>(ct.from).is_ok() {
                                world.spawn((new_connection, Deletable {}, synapse_current));
                            } else if world.get::<&Neuron>(ct.from).is_ok() {
                                if *tool == Tool::StaticConnection {
                                    world.spawn((new_connection, Deletable {}, synapse_current));
                                } else {
                                    world.spawn((
                                        new_connection,
                                        Deletable {},
                                        synapse_current,
                                        LearningSynapse {},
                                    ));
                                };
                            }
                        }
                        if !self.keyboard.shift_down {
                            ct.start = position;
                            ct.from = id;
                        }
                    }
                    ct.end = mouse_position;
                } else {
                    *connection_tool = world
                        .query::<&Position>()
                        .with::<&StaticConnectionSource>()
                        .iter()
                        .min_by(|a, b| nearest(&mouse_position, a, b))
                        .and_then(|v| within_attachment_range(mouse_position, v))
                        .map(|(id, position)| ConnectionTool {
                            start: position,
                            end: mouse_position,
                            from: id,
                        });
                }
            }
            Tool::Stimulate => {
                *stimulation_tool = Some(StimulationTool {
                    position: mouse_position,
                })
            }
            Tool::Erase => {
                let to_delete = world
                    .query::<&Position>()
                    .with::<&Deletable>()
                    .iter()
                    .filter_map(|(entity, position)| {
                        let distance = position.position.distance(mouse_position);
                        if distance < NODE_RADIUS * 1.5 {
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Entity>>();
                for entity in to_delete {
                    world.despawn(entity).unwrap();
                }
                let connections_to_delete = world
                    .query::<&Connection>()
                    .with::<&Deletable>()
                    .iter()
                    .filter_map(|(entity, connection)| {
                        if let (Ok(from), Ok(to)) = (
                            world.get::<&Position>(connection.from),
                            world.get::<&Position>(connection.to),
                        ) {
                            let a = from.position;
                            let b = to.position;
                            let p = mouse_position;
                            let ab = b - a;
                            let ap = p - a;
                            let t = ap.dot(ab) / ab.dot(ab);
                            let d = t * ab;
                            let point_on_line = a + d;
                            let distance_from_line = p.distance(point_on_line);
                            if distance_from_line < ERASE_RADIUS && (0.0..=1.0).contains(&t) {
                                Some(entity)
                            } else {
                                None
                            }
                        } else {
                            Some(entity)
                        }
                    })
                    .collect::<Vec<Entity>>();
                for connection in connections_to_delete {
                    world.despawn(connection).unwrap();
                }
                let triggers_to_delete = world
                    .query::<&Trigger>()
                    .iter()
                    .filter_map(|(entity, trigger)| {
                        if world.get::<&Connection>(trigger.connection).is_err() {
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Entity>>();
                for trigger in triggers_to_delete {
                    world.despawn(trigger).unwrap();
                }
            }
            Tool::Voltmeter => {
                if previous_too_near {
                    return;
                }
                let result = world
                    .query::<&Position>()
                    .with::<&Neuron>()
                    .iter()
                    .find_map(|(entity, position)| {
                        let distance = position.position.distance(mouse_position);
                        if distance < NODE_RADIUS {
                            Some((entity, position.clone()))
                        } else {
                            None
                        }
                    });
                let Some((target, position)) = result else {
                    return;
                };
                let voltmeter = world.spawn((
                    Voltmeter {},
                    Position {
                        position: position.position
                            + Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 0.0,
                            },
                    },
                ));
                *previous_creation = Some(PreviousCreation { entity: voltmeter });
                world.spawn((
                    VoltageSeries {
                        measurements: RollingWindow::new(100000),
                    },
                    Connection {
                        from: target,
                        to: voltmeter,
                        strength: 1.0,
                        directional: true,
                    },
                ));
            }
            Tool::Select => {
                for (_, (selectable, position)) in world.query_mut::<(&mut Selectable, &Position)>()
                {
                    let distance = position.position.distance(mouse_position);
                    if distance < NODE_RADIUS {
                        selectable.selected = true;
                    }
                }
            }
            Tool::Axon => match connection_tool {
                None => {
                    *connection_tool = world
                        .query::<&Position>()
                        .with::<&StaticConnectionSource>()
                        .iter()
                        .min_by(|a, b| nearest(&mouse_position, a, b))
                        .and_then(|v| within_attachment_range(mouse_position, v))
                        .map(|(id, position)| ConnectionTool {
                            start: position,
                            end: mouse_position,
                            from: id,
                        });
                    match connection_tool {
                        Some(ct) => {
                            self.previous_creation = Some(PreviousCreation { entity: ct.from });
                        }
                        None => {}
                    }
                }
                Some(ct) => {
                    ct.end = mouse_position;
                    let nearest_target = world
                        .query::<&Position>()
                        .with::<&Neuron>()
                        .iter()
                        .min_by(|a, b| nearest(&mouse_position, a, b))
                        .and_then(|v| within_attachment_range(mouse_position, v));

                    match nearest_target {
                        Some((id, position)) => {
                            let strength = 1.0;
                            let new_connection = Connection {
                                from: ct.from,
                                to: id,
                                strength,
                                directional: true,
                            };
                            let synapse_current = SynapseCurrent {
                                current: 0.0,
                                tau: 0.1,
                            };
                            let connection_exists =
                                world.query::<&Connection>().iter().any(|(_, c)| {
                                    c.from == new_connection.from && c.to == new_connection.to
                                });
                            if !connection_exists && ct.from != id {
                                world.spawn((new_connection, Deletable {}, synapse_current));
                            }
                            if !self.keyboard.shift_down {
                                ct.start = position;
                                ct.from = id;
                            }
                        }
                        None => {
                            if previous_too_near {
                                return;
                            }
                            let neuron_type =
                                if let Ok(neuron_type) = world.get::<&NeuronType>(ct.from) {
                                    Some((*neuron_type).clone())
                                } else {
                                    None
                                };
                            if let Some(neuron_type) = neuron_type {
                                let compartment = world.spawn((
                                    Position {
                                        position: mouse_position,
                                    },
                                    neuron_type,
                                    Compartment {
                                        voltage: 100.0,
                                        m: 0.084073044,
                                        h: 0.45317015,
                                        n: 0.38079754,
                                        influence: 0.0,
                                        capacitance: 4.0,
                                        injected_current: 0.0,
                                    },
                                    StaticConnectionSource {},
                                    Deletable {},
                                    Selectable { selected: false },
                                    SpatialDynamics {
                                        velocity: Vec3::new(0.0, 0.0, 0.0),
                                        acceleration: Vec3::new(0.0, 0.0, 0.0),
                                    },
                                ));
                                let strength = 1.0;
                                let new_connection = Connection {
                                    from: ct.from,
                                    to: compartment,
                                    strength,
                                    directional: false,
                                };
                                let compartment_current = CompartmentCurrent { capacitance: 0.1 };
                                world.spawn((new_connection, Deletable {}, compartment_current));
                                self.previous_creation = Some(PreviousCreation {
                                    entity: compartment,
                                });
                                *connection_tool = Some(ConnectionTool {
                                    start: mouse_position,
                                    end: mouse_position,
                                    from: compartment,
                                });
                            }
                        }
                    }
                }
            },
        }
    }

    pub fn save(&self) {
        let mut context = SaveContext;
        let mut serializer = postcard::Serializer {
            output: postcard::ser_flavors::StdVec::new(),
        };
        serialize(&self.world, &mut context, &mut serializer).unwrap();
        let mut writer = std::fs::File::create("output.neuronify").unwrap();
        writer
            .write_all(&serializer.output.finalize().unwrap())
            .unwrap();
    }

    pub fn loadfile(&mut self) {
        let mut context = LoadContext::new();
        let reader = std::fs::File::open("output.neuronify").unwrap();
        let mut bufreader = BufReader::new(reader);
        let mut bytes: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut bytes).unwrap();
        let mut deserializer = postcard::Deserializer::from_bytes(&bytes);
        self.world = deserialize(&mut context, &mut deserializer).unwrap();
    }

    pub fn load(application: &mut visula::Application, bytes: &[u8]) -> Neuronify {
        let mut neuronify = Neuronify::new(application);
        let mut context = LoadContext::new();
        let mut deserializer = postcard::Deserializer::from_bytes(bytes);
        neuronify.world = deserialize(&mut context, &mut deserializer).unwrap();
        neuronify
    }
}

fn srgb_component(value: u8) -> f32 {
    (value as f32 / 255.0 + 0.055_f32).powf(2.44) / 1.055
}

fn srgb(red: u8, green: u8, blue: u8) -> Vec3 {
    Vec3::new(
        srgb_component(red),
        srgb_component(green),
        srgb_component(blue),
    )
}

fn red() -> Vec3 {
    srgb(210, 15, 57)
}

fn maroon() -> Vec3 {
    srgb(230, 69, 83)
}

fn blue() -> Vec3 {
    srgb(30, 102, 245)
}

fn lavender() -> Vec3 {
    srgb(114, 135, 253)
}
fn pink() -> Vec3 {
    srgb(234, 118, 203)
}
fn base() -> Vec3 {
    srgb(239, 241, 245)
}
fn mantle() -> Vec3 {
    srgb(230, 233, 239)
}
fn crust() -> Vec3 {
    srgb(220, 224, 232)
}
fn neurocolor(neuron_type: &NeuronType, value: f32) -> Vec3 {
    let v = 1.0 / (1.0 + (-4.9 * (-1.0 + 2.0 * value)).exp());
    match *neuron_type {
        NeuronType::Excitatory => v * base() + (1.0 - v) * blue(),
        NeuronType::Inhibitory => v * mantle() + (1.0 - v) * red(),
    }
}

impl visula::Simulation for Neuronify {
    type Error = Error;
    fn clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: srgb_component(30) as f64,
            g: srgb_component(30) as f64,
            b: srgb_component(46) as f64,
            a: 1.0,
        }
    }
    fn update(&mut self, application: &mut visula::Application) {
        let Neuronify {
            connection_tool,
            world,
            time,
            stimulation_tool,
            ..
        } = self;
        let dt = 0.001;

        for (_, (position, stimulate)) in world.query_mut::<(&Position, &mut StimulateCurrent)>() {
            if let Some(stim) = stimulation_tool {
                let mouse_distance = position.position.distance(stim.position);
                stimulate.current = (300.0
                    * (-mouse_distance * mouse_distance / (2.0 * SIGMA * SIGMA)).exp())
                    as f64;
            } else {
                stimulate.current = 0.0;
            }
        }

        for _ in 0..self.iterations {
            for (_, dynamics) in world.query_mut::<&mut NeuronDynamics>() {
                dynamics.current = 0.0;
            }
            for (_, (leak, neuron, dynamics)) in
                world.query_mut::<(&mut LeakCurrent, &Neuron, &mut NeuronDynamics)>()
            {
                leak.current = (neuron.resting_potential - dynamics.voltage) / leak.tau;
                dynamics.current += leak.current;
            }
            for (_, (dynamics, stimulate)) in
                world.query_mut::<(&mut NeuronDynamics, &StimulateCurrent)>()
            {
                dynamics.current += stimulate.current;
            }
            for (_, synapse) in world.query_mut::<&mut SynapseCurrent>() {
                synapse.current -= synapse.current * dt / synapse.tau;
            }

            for (_, (synapse, connection)) in
                world.query::<(&mut SynapseCurrent, &Connection)>().iter()
            {
                if let Ok(source) = world.get::<&CurrentSource>(connection.from) {
                    synapse.current = source.current;
                }
                if let (Ok(compartment), Ok(neuron_type)) = (
                    world.get::<&Compartment>(connection.from),
                    world.get::<&NeuronType>(connection.from),
                ) {
                    let current = (compartment.voltage - 50.0).clamp(0.0, 200.0);
                    let sign = match *neuron_type {
                        NeuronType::Excitatory => 1.0,
                        NeuronType::Inhibitory => -1.0,
                    };
                    synapse.current += sign * current;
                }
            }
            for (_, (synapse, connection)) in world.query::<(&SynapseCurrent, &Connection)>().iter()
            {
                let mut dynamics = world.get::<&mut NeuronDynamics>(connection.to).unwrap();
                if dynamics.refraction <= 0.0 {
                    dynamics.current += synapse.current;
                }
            }
            for (_, (dynamics, neuron)) in world.query_mut::<(&mut NeuronDynamics, &Neuron)>() {
                dynamics.voltage = (dynamics.voltage + dynamics.current * dt).clamp(-200.0, 200.0);
                if dynamics.refraction <= 0.0 && dynamics.voltage > neuron.threshold {
                    dynamics.fired = true;
                    dynamics.refraction = 0.2;
                    dynamics.voltage = neuron.initial_voltage;
                    dynamics.voltage = neuron.reset_potential;
                }
            }
            for (_, compartment) in world.query_mut::<&mut Compartment>() {
                let cdt = 10.0 * dt;
                let v = compartment.voltage;

                let sodium_activation_alpha = 0.1 * (25.0 - v) / ((2.5 - 0.1 * v).exp() - 1.0);
                let sodium_activation_beta = 4.0 * (-v / 18.0).exp();
                let sodium_inactivation_alpha = 0.07 * (-v / 20.0).exp();
                let sodium_inactivation_beta = 1.0 / ((3.0 - 0.1 * v).exp() + 1.0);

                let mut m = compartment.m;
                let alpham = sodium_activation_alpha;
                let betam = sodium_activation_beta;
                let dm = cdt * (alpham * (1.0 - m) - betam * m);
                let mut h = compartment.h;
                let alphah = sodium_inactivation_alpha;
                let betah = sodium_inactivation_beta;
                let dh = cdt * (alphah * (1.0 - h) - betah * h);

                m += dm;
                h += dh;

                m = m.clamp(0.0, 1.0);
                h = h.clamp(0.0, 1.0);

                let g_na = 120.0;

                let ena = 115.0;

                let m3 = m * m * m;

                let sodium_current = -g_na * m3 * h * (compartment.voltage - ena);

                let potassium_activation_alpha =
                    0.01 * (10.0 - v) / ((1.0 - (0.1 * v)).exp() - 1.0);
                let potassium_activation_beta = 0.125 * (-v / 80.0).exp();

                let mut n = compartment.n;
                let alphan = potassium_activation_alpha;
                let betan = potassium_activation_beta;
                let dn = cdt * (alphan * (1.0 - n) - betan * n);

                n += dn;
                n = n.clamp(0.0, 1.0);

                let g_k = 36.0;
                let ek = -12.0;
                let n4 = n * n * n * n;

                let potassium_current = -g_k * n4 * (compartment.voltage - ek);

                let e_m = 10.6;
                let leak_conductance = 1.3;
                let leak_current = -leak_conductance * (compartment.voltage - e_m);

                let current = sodium_current
                    + potassium_current
                    + leak_current
                    + compartment.injected_current;
                let delta_voltage = current / compartment.capacitance;

                compartment.n = n;
                compartment.m = m;
                compartment.h = h;
                compartment.voltage += delta_voltage * cdt;
                compartment.voltage = compartment.voltage.clamp(-50.0, 200.0);
                compartment.injected_current -= 1.0 * compartment.injected_current * cdt;
            }

            let mut new_compartments: HashMap<Entity, Compartment> = world
                .query::<&Compartment>()
                .iter()
                .map(|(entity, &compartment)| (entity, compartment))
                .collect();
            for (_, (connection, current)) in
                world.query::<(&Connection, &CompartmentCurrent)>().iter()
            {
                if let Ok(compartment_to) = world.get::<&Compartment>(connection.to) {
                    if let Ok(dynamics_from) = world.get::<&NeuronDynamics>(connection.from) {
                        if dynamics_from.fired {
                            let new_compartment_to = new_compartments
                                .get_mut(&connection.to)
                                .expect("Could not get new compartment");
                            new_compartment_to.injected_current += 150.0;
                        }
                    } else if let Ok(compartment_from) = world.get::<&Compartment>(connection.from)
                    {
                        let voltage_diff = compartment_from.voltage - compartment_to.voltage;
                        let delta_voltage = voltage_diff / current.capacitance;
                        let new_compartment_to = new_compartments
                            .get_mut(&connection.to)
                            .expect("Could not get new compartment");
                        new_compartment_to.voltage += delta_voltage * dt;
                        let new_compartment_from = new_compartments
                            .get_mut(&connection.from)
                            .expect("Could not get new compartment");
                        new_compartment_from.voltage -= delta_voltage * dt;
                    }
                }
            }
            let positions: Vec<(Entity, Position)> = world
                .query::<&Position>()
                .iter()
                .map(|(e, p)| (e.to_owned(), p.to_owned()))
                .collect();
            for (id, (position, dynamics)) in world.query_mut::<(&Position, &mut SpatialDynamics)>()
            {
                for (other_id, other_position) in &positions {
                    if id == *other_id {
                        continue;
                    }
                    let from = position.position;
                    let to = other_position.position;
                    let r2 = from.distance_squared(to);
                    let target2 = (2.0 * NODE_RADIUS).powi(2);
                    let d = (to - from).normalize();
                    let force = 5.0 * (r2 - target2).min(0.0) * d;
                    dynamics.acceleration += force;
                }
            }
            let connections: Vec<(Entity, Connection)> = world
                .query::<&Connection>()
                .iter()
                .map(|(e, c)| (e.to_owned(), c.to_owned()))
                .collect();

            for (connection_id_1, connection_1) in &connections {
                for (connection_id_2, connection_2) in &connections {
                    if connection_id_1 == connection_id_2 {
                        continue;
                    }
                    if connection_1.to != connection_2.from {
                        continue;
                    }
                    let to_1 = world.get::<&Position>(connection_1.to).unwrap().position;
                    let from_1 = world.get::<&Position>(connection_1.from).unwrap().position;
                    let to_2 = world.get::<&Position>(connection_2.to).unwrap().position;
                    let from_2 = world.get::<&Position>(connection_2.from).unwrap().position;
                    let target = 1.0;
                    let dir_ab = (to_1 - from_1).normalize();
                    let dir_bc = (to_2 - from_2).normalize();
                    let dot = dir_ab.dot(dir_bc);
                    let diff = target - dot;
                    let p_a = (dir_ab.cross((dir_ab).cross(dir_bc))).normalize();
                    let p_c = (dir_bc.cross((dir_ab).cross(dir_bc))).normalize();
                    let k = 1.0;
                    let f_a = k * diff / dir_ab.length() * p_a;
                    let f_c = k * diff / dir_bc.length() * p_c;
                    let f_b = -f_a - f_c;
                    if let Ok(mut dynamics_a) = world.get::<&mut SpatialDynamics>(connection_1.from)
                    {
                        dynamics_a.acceleration += f_a;
                    }
                    if let Ok(mut dynamics_b) = world.get::<&mut SpatialDynamics>(connection_1.to) {
                        dynamics_b.acceleration += f_b;
                    }
                    if let Ok(mut dynamics_c) = world.get::<&mut SpatialDynamics>(connection_2.to) {
                        dynamics_c.acceleration += f_c;
                    }
                }
            }

            for (compartment_id, new_compartment) in new_compartments {
                let mut old_compartment = world
                    .get::<&mut Compartment>(compartment_id)
                    .expect("Could not find compartment");
                *old_compartment = new_compartment;
            }
            let new_triggers: Vec<(Entity, f64)> = world
                .query::<&Connection>()
                .with::<&SynapseCurrent>()
                .iter()
                .flat_map(|(connection_entity, connection)| {
                    let mut triggers = vec![];
                    if let (Ok(_neuron_from), Ok(neuron_from_type)) = (
                        world.get::<&Neuron>(connection.from),
                        world.get::<&NeuronType>(connection.from),
                    ) {
                        if let Ok(dynamics_from) = world.get::<&NeuronDynamics>(connection.from) {
                            let base_current = match *neuron_from_type {
                                NeuronType::Excitatory => 3000.0,
                                NeuronType::Inhibitory => -3000.0,
                            };
                            let current = connection.strength * base_current;
                            if dynamics_from.fired {
                                triggers.push((connection_entity, current));
                            }
                        }
                    }
                    triggers
                })
                .collect();
            for (connection_entity, current) in new_triggers {
                world.spawn((Trigger::new(0.3, current, connection_entity),));
            }

            for (_, connection) in world
                .query::<&Connection>()
                .with::<&CompartmentCurrent>()
                .iter()
            {
                if let (Ok(from), Ok(to)) = (
                    world.get::<&Position>(connection.from),
                    world.get::<&Position>(connection.to),
                ) {
                    let r2 = from.position.distance_squared(to.position);
                    let d = to.position - from.position;
                    let target_length = 2.0 * NODE_RADIUS;
                    let force = 10.0 * (r2 - target_length.powi(2)) * d.normalize();
                    if let Ok(mut dynamics_from) =
                        world.get::<&mut SpatialDynamics>(connection.from)
                    {
                        dynamics_from.acceleration += force;
                    }
                    if let Ok(mut dynamics_to) = world.get::<&mut SpatialDynamics>(connection.to) {
                        dynamics_to.acceleration -= force;
                    }
                }
            }

            for (_, trigger) in world.query_mut::<&mut Trigger>() {
                trigger.decrement(dt);
            }

            let triggers_to_delete: Vec<Entity> = world
                .query::<&Trigger>()
                .iter()
                .filter_map(|(entity, trigger)| {
                    if trigger.done() {
                        let mut synapse = world
                            .get::<&mut SynapseCurrent>(trigger.connection)
                            .unwrap();
                        synapse.current = trigger.current;
                        Some(entity)
                    } else {
                        None
                    }
                })
                .collect();

            for entity in triggers_to_delete {
                world.despawn(entity).expect("Could not delete entity!");
            }

            for (_, dynamics) in world.query_mut::<&mut NeuronDynamics>() {
                dynamics.fired = false;
                dynamics.refraction -= dt;
            }

            for (_, (position, dynamics)) in
                world.query_mut::<(&mut Position, &mut SpatialDynamics)>()
            {
                let gravity = -position.position.y;
                dynamics.acceleration += Vec3::new(0.0, gravity, 0.0);
                dynamics.velocity += dynamics.acceleration * dt as f32;
                position.position += dynamics.velocity * dt as f32;
                dynamics.acceleration = Vec3::new(0.0, 0.0, 0.0);
                dynamics.velocity -= dynamics.velocity * dt as f32;
            }

            let mut updates = HashMap::new();
            for (entity, (_, connection)) in world.query::<(&VoltageSeries, &Connection)>().iter() {
                let dynamics = world
                    .get::<&NeuronDynamics>(connection.from)
                    .expect("Connection with voltage series does not come from neuron");
                updates.insert(
                    entity,
                    VoltageMeasurement {
                        voltage: dynamics.voltage,
                        time: *time,
                    },
                );
            }
            for (entity, value) in updates {
                world
                    .get::<&mut VoltageSeries>(entity)
                    .unwrap()
                    .measurements
                    .push(value);
            }

            *time += dt;
        }

        let neuron_spheres: Vec<Sphere> = world
            .query::<(&Neuron, &NeuronDynamics, &Position, &NeuronType)>()
            .iter()
            .map(|(_entity, (_neuron, dynamics, position, neuron_type))| {
                let value = ((dynamics.voltage + 100.0) / 150.0).clamp(0.0, 1.0) as f32;
                Sphere {
                    position: position.position,
                    color: neurocolor(neuron_type, value),
                    radius: NODE_RADIUS,
                    _padding: Default::default(),
                }
            })
            .collect();

        let compartment_spheres: Vec<Sphere> = world
            .query::<(&Compartment, &Position, &NeuronType)>()
            .iter()
            .map(|(_entity, (compartment, position, neuron_type))| {
                let value = ((compartment.voltage + 10.0) / 120.0) as f32;
                let _color = match neuron_type {
                    NeuronType::Excitatory => Vec3::new(value / 2.0, value, 0.95),
                    NeuronType::Inhibitory => Vec3::new(0.95, value / 2.0, value),
                };
                Sphere {
                    position: position.position,
                    color: neurocolor(neuron_type, value),
                    radius: 0.3 * NODE_RADIUS,
                    _padding: Default::default(),
                }
            })
            .collect();

        let source_spheres: Vec<Sphere> = world
            .query::<&Position>()
            .with::<&CurrentSource>()
            .iter()
            .map(|(_entity, position)| {
                let color = Vec3::new(0.8, 0.8, 0.1);
                Sphere {
                    position: position.position,
                    color,
                    radius: NODE_RADIUS,
                    _padding: Default::default(),
                }
            })
            .collect();

        let trigger_spheres: Vec<Sphere> = world
            .query::<&Trigger>()
            .iter()
            .map(|(_entity, trigger)| {
                let connection = world
                    .get::<&Connection>(trigger.connection)
                    .expect("Connection from broken");
                let start = world
                    .get::<&Position>(connection.from)
                    .expect("Connection from broken")
                    .position;
                let end = world
                    .get::<&Position>(connection.to)
                    .expect("Connection to broken")
                    .position;
                let diff = end - start;
                let position = start + diff * trigger.progress() as f32;
                Sphere {
                    position,
                    color: Vec3::new(0.8, 0.9, 0.9),
                    radius: NODE_RADIUS * 0.5,
                    _padding: Default::default(),
                }
            })
            .collect();

        let mut spheres = Vec::new();
        spheres.extend(neuron_spheres.iter());
        spheres.extend(compartment_spheres.iter());
        spheres.extend(source_spheres.iter());
        spheres.extend(trigger_spheres.iter());

        let mut connections: Vec<ConnectionData> = world
            .query::<&Connection>()
            .iter()
            .map(|(_, connection)| {
                let start = world
                    .get::<&Position>(connection.from)
                    .expect("Connection from broken")
                    .position;
                let end = world
                    .get::<&Position>(connection.to)
                    .expect("Connection to broken")
                    .position;
                let start_value = match world.get::<&Compartment>(connection.from) {
                    Ok(compartment) => ((compartment.voltage + 10.0) / 120.0) as f32,
                    Err(_) => 1.0,
                };
                let end_value = match world.get::<&Compartment>(connection.to) {
                    Ok(compartment) => ((compartment.voltage + 10.0) / 120.0) as f32,
                    Err(_) => 1.0,
                };
                let start_color = match world.get::<&NeuronType>(connection.from) {
                    Ok(neuron_type) => neurocolor(&neuron_type, start_value),
                    Err(_) => Vec3::new(1.0, 0.0, 1.0),
                };
                let end_color = match world.get::<&NeuronType>(connection.from) {
                    Ok(neuron_type) => neurocolor(&neuron_type, end_value),
                    Err(_) => Vec3::new(1.0, 0.0, 1.0),
                };
                ConnectionData {
                    position_a: start,
                    position_b: end,
                    strength: connection.strength as f32,
                    directional: match connection.directional {
                        true => 1.0,
                        false => 0.0,
                    },
                    start_color,
                    end_color,
                    _padding: Default::default(),
                }
            })
            .collect();

        if self.tool == Tool::StaticConnection || self.tool == Tool::LearningConnection {
            if let Some(connection) = &connection_tool {
                connections.push(ConnectionData {
                    position_a: connection.start,
                    position_b: connection.end,
                    strength: 1.0,
                    directional: 1.0,
                    start_color: Vec3::new(0.8, 0.8, 0.8),
                    end_color: Vec3::new(0.8, 0.8, 0.8),
                    _padding: Default::default(),
                });
            }
        }

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);

        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

        let time_diff = Utc::now() - self.last_update;
        #[cfg(not(target_arch = "wasm32"))]
        if time_diff < Duration::milliseconds(16) {
            thread::sleep(std::time::Duration::from_millis(
                (Duration::milliseconds(16) - time_diff).num_milliseconds() as u64,
            ))
        }
        let low_pass_factor = 0.05;
        let new_fps = 1.0
            / ((Utc::now() - self.last_update).num_nanoseconds().unwrap() as f64 * 1e-9)
                .max(0.0000001);
        self.fps = (1.0 - low_pass_factor) * self.fps + low_pass_factor * new_fps;
        self.last_update = Utc::now();
    }

    fn render(&mut self, data: &mut RenderData) {
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
    }

    fn gui(&mut self, application: &visula::Application, context: &egui::Context) {
        egui::Window::new("Settings").show(context, |ui| {
            ui.label(format!("FPS: {:.0}", self.fps));
            ui.label("Tool");
            for value in Tool::iter() {
                ui.selectable_value(&mut self.tool, value.clone(), format!("{:?}", &value));
            }
            ui.label("Simulation speed");
            ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
            if ui.button("Save").clicked() {
                self.save();
            }
            if ui.button("Load").clicked() {
                self.loadfile();
            }
        });

        for (voltmeter_id, _voltmeter) in self.world.query::<&Voltmeter>().iter() {
            for (_, (series, connection)) in
                self.world.query::<(&VoltageSeries, &Connection)>().iter()
            {
                if connection.to != voltmeter_id {
                    continue;
                }
                let Ok(position) = self.world.get::<&Position>(connection.from) else {
                    log::error!("Position not found for entity");
                    continue;
                };
                let id = egui::Id::new(voltmeter_id);
                egui::Window::new("Voltmeter")
                    .id(id)
                    .resizable(true)
                    .show(context, |ui| {
                        let line_points: PlotPoints = series
                            .measurements
                            .iter()
                            .map(|m| [m.time, m.voltage])
                            .collect();
                        let (min_x, max_x) = {
                            match series.measurements.last().map(|m| m.time) {
                                Some(t) => (t - 5.0, t),
                                None => (-5.0, 0.0),
                            }
                        };
                        let line = Line::new(line_points);
                        egui_plot::Plot::new("Voltage")
                            .show(ui, |plot_ui| {
                                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                    [min_x, -100.0],
                                    [max_x, 100.0],
                                ));
                                plot_ui.line(line)
                            })
                            .response
                    });

                let mut start = Pos2::new(0.0, 0.0);
                context.memory(|memory| {
                    let rect = memory
                        .area_rect(id)
                        .expect("Could not find id of window that was just created");
                    start = rect.center();
                });
                let width = application.config.width as f32;
                let height = application.config.height as f32;
                let position_2d_pre = application
                    .camera_controller
                    .uniforms(width / height)
                    .model_view_projection_matrix
                    * Vector4::new(
                        position.position.x,
                        position.position.y,
                        position.position.z,
                        1.0,
                    );

                let position_2d = position_2d_pre / position_2d_pre.w;

                let line_end = (
                    width / application.window.scale_factor() as f32 * (position_2d[0] + 1.0) / 2.0,
                    height / application.window.scale_factor() as f32
                        * (((0.0 - position_2d[1]) + 1.0) / 2.0),
                )
                    .into();
                context
                    .layer_painter(LayerId::background())
                    .line_segment([start, line_end], (1.0, Color32::WHITE)); // Adjust color and line thickness as needed
            }
        }
    }

    fn handle_event(&mut self, application: &mut visula::Application, event: &Event<CustomEvent>) {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                self.mouse.left_down = *state == ElementState::Pressed;
                self.handle_tool(application);
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                self.keyboard.shift_down = state.lshift_state() == ModifiersKeyState::Pressed
                    || state.rshift_state() == ModifiersKeyState::Pressed;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                self.mouse.position = Some(*position);
                self.handle_tool(application);
            }
            _ => {}
        }
    }
}

#[wasm_bindgen]
pub async fn load(url: &str) -> Result<(), JsValue> {
    initialize_logger();
    let (event_loop, window) = initialize_event_loop_and_window_with_config(RunConfig {
        canvas_name: "canvas".to_owned(),
    });
    let main_window_id = window.id();
    let mut application =
        pollster::block_on(async { Application::new(Arc::new(window), &event_loop).await });

    let mut opts = RequestInit::new();
    opts.method("GET");
    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().ok_or("No global `window` exists")?;
    let response_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response_value.dyn_into()?;
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = Uint8Array::new(&buffer);
    let vec = uint8_array.to_vec();
    let mut simulation = Neuronify::load(&mut application, &vec);
    event_loop
        .run(move |event, target| {
            if !application.handle_event(&event) {
                simulation.handle_event(&mut application, &event);
            }
            if let Event::WindowEvent { window_id, event } = event {
                if main_window_id != window_id {
                    return;
                }
                match event {
                    WindowEvent::RedrawRequested => {
                        application.update();
                        simulation.update(&mut application);
                        application.render(&mut simulation);

                        application.window.borrow_mut().request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        })
        .expect("Event loop failed to run");
    Ok(())
}
