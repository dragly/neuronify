use crate::measurement::voltmeter::RollingWindow;
use crate::measurement::voltmeter::VoltageMeasurement;
use crate::measurement::voltmeter::VoltageSeries;
use crate::measurement::voltmeter::Voltmeter;
use crate::serialization::{LoadContext, SaveContext};
use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;
use cgmath::Vector4;
use chrono::{DateTime, Duration, Utc};
use egui::Color32;
use egui::LayerId;
use egui::Pos2;
use egui_plot::PlotBounds;
use egui_plot::{Line, PlotPoints};
use glam::Quat;
use glam::Vec3;
use hecs::serialize::column::*;
use hecs::Entity;
use js_sys::Uint8Array;
use postcard::ser_flavors::Flavor;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use visula::create_window;
#[cfg(target_arch = "wasm32")]
use visula::winit::platform::web::EventLoopExtWebSys;
use visula::winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
};
use visula::{
    create_event_loop, initialize_logger, winit::keyboard::ModifiersKeyState, Application,
    CustomEvent, InstanceBuffer, LineDelegate, Lines, RenderData, Renderable, RunConfig,
    Simulation, SphereDelegate, Spheres, Vector3,
};
use visula_derive::Instance;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopWindowTarget;

pub mod components;
pub mod legacy;
pub mod measurement;
pub mod serialization;

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    CurrentSource,
    TouchSensor,
    RegularSpikeGenerator,
    PoissonGenerator,
    Voltmeter,
    StaticConnection,
    Axon,
    Erase,
    Stimulate,
}

#[derive(Clone, Debug, PartialEq)]
enum ToolCategory {
    Interaction,
    Neurons,
    Connections,
}

impl ToolCategory {
    fn label(&self) -> &str {
        match self {
            ToolCategory::Interaction => "Interaction",
            ToolCategory::Neurons => "Neurons",
            ToolCategory::Connections => "Connections",
        }
    }
    fn tools(&self) -> Vec<(Tool, &str)> {
        match self {
            ToolCategory::Interaction => vec![
                (Tool::Select, "Select"),
                (Tool::Stimulate, "Stimulate"),
                (Tool::Erase, "Erase"),
            ],
            ToolCategory::Neurons => vec![
                (Tool::ExcitatoryNeuron, "Excitatory Neuron"),
                (Tool::InhibitoryNeuron, "Inhibitory Neuron"),
                (Tool::CurrentSource, "Current Source"),
                (Tool::TouchSensor, "Touch Sensor"),
                (Tool::RegularSpikeGenerator, "Spike Generator"),
                (Tool::PoissonGenerator, "Poisson Generator"),
                (Tool::Voltmeter, "Voltmeter"),
            ],
            ToolCategory::Connections => vec![
                (Tool::StaticConnection, "Static Connection"),
                (Tool::Axon, "Axon"),
            ],
        }
    }
}

const TOOL_CATEGORIES: [ToolCategory; 3] = [
    ToolCategory::Interaction,
    ToolCategory::Neurons,
    ToolCategory::Connections,
];

const NODE_RADIUS: f32 = 1.0;
const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NeuronType {
    Excitatory,
    Inhibitory,
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
pub struct StaticConnectionSource {}

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

#[derive(Clone, Debug)]
pub struct ConnectionTool {
    pub start: Vec3,
    pub end: Vec3,
    pub from: Entity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub strength: f64,
    pub directional: bool,
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
    pub delta_position: Option<PhysicalPosition<f64>>,
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
    pub edit_enabled: bool,
    pub last_touch_points: Option<((f64, f64), (f64, f64))>,
    pub move_origin: Option<Vec3>,
    pub active_entity: Option<Entity>,
    pub dragging_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub resizing_voltmeter: Option<(Entity, ResizeCorner)>,
}

#[derive(Clone, Copy, Debug)]
pub enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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

/// Evaluate a quadratic Bezier curve at parameter t in [0,1].
fn quadratic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
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

fn within_selection_range(
    mouse_position: Vec3,
    (id, position): (Entity, &Position),
) -> Option<(Entity, Vec3)> {
    if mouse_position.distance(position.position) < 0.9 * NODE_RADIUS {
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

        let mut world = hecs::World::new();

        // Load legacy .nfy file from command line argument if provided
        #[cfg(not(target_arch = "wasm32"))]
        {
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let path = &args[1];
                match std::fs::read_to_string(path) {
                    Ok(contents) => match legacy::parse_legacy_nfy(&contents) {
                        Ok(sim) => {
                            log::info!("Loaded legacy simulation from {}: {} nodes, {} edges",
                                path, sim.nodes.len(), sim.edges.len());
                            legacy::spawn::spawn_legacy_simulation(&mut world, &sim);
                        }
                        Err(e) => log::error!("Failed to parse legacy file {}: {}", path, e),
                    },
                    Err(e) => log::error!("Failed to read file {}: {}", path, e),
                }
            }
        }

        Neuronify {
            spheres,
            sphere_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
            tool: Tool::Select,
            previous_creation: None,
            connection_tool: None,
            stimulation_tool: None,
            world,
            time: 0.0,
            mouse: Mouse {
                left_down: false,
                position: None,
                delta_position: None,
            },
            keyboard: Keyboard { shift_down: false },
            iterations: 4,
            last_update: Utc::now(),
            fps: 60.0,
            edit_enabled: true,
            last_touch_points: None,
            move_origin: None,
            active_entity: None,
            dragging_entity: None,
            drag_offset: Vec3::ZERO,
            resizing_voltmeter: None,
        }
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        let Neuronify {
            tool,
            mouse,
            connection_tool,
            stimulation_tool,
            world,
            previous_creation,
            move_origin,
            active_entity,
            dragging_entity,
            drag_offset,
            resizing_voltmeter,
            ..
        } = self;
        if !mouse.left_down {
            *stimulation_tool = None;
            *connection_tool = None;
            *previous_creation = None;
            *move_origin = None;
            *dragging_entity = None;
            *resizing_voltmeter = None;
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
                let neuron_type = if self.tool == Tool::InhibitoryNeuron {
                    NeuronType::Inhibitory
                } else {
                    NeuronType::Excitatory
                };
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    components::LIFNeuron::default(),
                    components::LIFDynamics::default(),
                    components::LeakCurrent::default(),
                    neuron_type,
                    Deletable {},
                ));
                if self.tool == Tool::InhibitoryNeuron {
                    world
                        .insert_one(entity, components::Inhibitory)
                        .unwrap();
                }
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::CurrentSource => {
                if previous_too_near {
                    return;
                }
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    components::CurrentClamp::default(),
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::TouchSensor => {
                if previous_too_near {
                    return;
                }
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    components::TouchSensor,
                    components::GeneratorDynamics::default(),
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::RegularSpikeGenerator => {
                if previous_too_near {
                    return;
                }
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    components::RegularSpikeGenerator::default(),
                    components::GeneratorDynamics::default(),
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::PoissonGenerator => {
                if previous_too_near {
                    return;
                }
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    components::PoissonGenerator::default(),
                    components::GeneratorDynamics::default(),
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::StaticConnection => {
                if let Some(ct) = connection_tool {
                    // Find nearest target (LIF neuron)
                    let target_candidates: Vec<(Entity, Vec3)> = world
                        .query::<&Position>()
                        .with::<&components::LIFNeuron>()
                        .iter()
                        .map(|(e, p)| (e, p.position))
                        .collect();
                    let nearest_target = target_candidates
                        .iter()
                        .min_by(|a, b| {
                            a.1.distance(mouse_position)
                                .partial_cmp(&b.1.distance(mouse_position))
                                .unwrap_or(Ordering::Equal)
                        })
                        .and_then(|(id, pos)| {
                            if pos.distance(mouse_position) < 2.0 * NODE_RADIUS {
                                Some((*id, *pos))
                            } else {
                                None
                            }
                        });
                    if let Some((id, position)) = nearest_target {
                        let new_connection = Connection {
                            from: ct.from,
                            to: id,
                            strength: 1.0,
                            directional: true,
                        };
                        let connection_exists =
                            world.query::<&Connection>().iter().any(|(_, c)| {
                                c.from == new_connection.from && c.to == new_connection.to
                            });
                        if !connection_exists && ct.from != id {
                            world.spawn((
                                new_connection,
                                components::CurrentSynapse::default(),
                                Deletable {},
                            ));
                        }
                        if !self.keyboard.shift_down {
                            ct.start = position;
                            ct.from = id;
                        }
                    }
                    ct.end = mouse_position;
                } else {
                    // Find nearest connectable source (LIF neuron or current clamp)
                    let source_candidates: Vec<(Entity, Vec3)> = {
                        let mut candidates: Vec<_> = world
                            .query::<&Position>()
                            .with::<&components::LIFNeuron>()
                            .iter()
                            .map(|(e, p)| (e, p.position))
                            .collect();
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&components::CurrentClamp>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&components::GeneratorDynamics>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates
                    };
                    *connection_tool = source_candidates
                        .iter()
                        .min_by(|a, b| {
                            a.1.distance(mouse_position)
                                .partial_cmp(&b.1.distance(mouse_position))
                                .unwrap_or(Ordering::Equal)
                        })
                        .and_then(|(id, pos)| {
                            if pos.distance(mouse_position) < 2.0 * NODE_RADIUS {
                                Some((*id, *pos))
                            } else {
                                None
                            }
                        })
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
            }
            Tool::Voltmeter => {
                if previous_too_near {
                    return;
                }
                // Find nearest LIF neuron
                let result: Option<(Entity, Vec3)> = world
                    .query::<&Position>()
                    .with::<&components::LIFNeuron>()
                    .iter()
                    .filter_map(|(entity, position)| {
                        let distance = position.position.distance(mouse_position);
                        if distance < NODE_RADIUS {
                            Some((entity, position.position))
                        } else {
                            None
                        }
                    })
                    .next();
                let Some((target, position)) = result else {
                    return;
                };
                let voltmeter = world.spawn((
                    Voltmeter {},
                    Position {
                        position: position
                            + Vec3 {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                    },
                    VoltageSeries {
                        measurements: RollingWindow::new(100000),
                        spike_times: Vec::new(),
                    },
                    Connection {
                        from: target,
                        to: Entity::DANGLING,
                        strength: 1.0,
                        directional: true,
                    },
                    components::VoltmeterSize::default(),
                    Deletable {},
                ));
                // Fix self-reference: connection.to should point to voltmeter itself
                if let Ok(mut conn) = world.get::<&mut Connection>(voltmeter) {
                    conn.to = voltmeter;
                }
                *previous_creation = Some(PreviousCreation { entity: voltmeter });
            }
            Tool::Select => match mouse.left_down {
                true => {
                    // If already dragging an entity, move it (with offset)
                    if let Some(entity) = *dragging_entity {
                        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                            pos.position = mouse_position + *drag_offset;
                            pos.position.y = 0.0;
                        }
                    } else if let Some((entity, corner)) = *resizing_voltmeter {
                        // Resize voltmeter by dragging corner.
                        // Anchor the opposite corner so only the dragged corner moves.
                        // Read current state into locals to release borrows before writing.
                        let current = world
                            .get::<&Position>(entity)
                            .ok()
                            .map(|p| p.position)
                            .and_then(|vpos| {
                                world
                                    .get::<&components::VoltmeterSize>(entity)
                                    .ok()
                                    .map(|s| (vpos, s.width, s.height))
                            });
                        if let Some((vpos, w, h)) = current {
                            let bl = vpos + Vec3::new(-h * 0.5, 0.0, 0.0);
                            let anchor = match corner {
                                ResizeCorner::TopLeft => bl + Vec3::new(0.0, 0.0, w),
                                ResizeCorner::TopRight => bl,
                                ResizeCorner::BottomLeft => {
                                    bl + Vec3::new(h, 0.0, w)
                                }
                                ResizeCorner::BottomRight => bl + Vec3::new(h, 0.0, 0.0),
                            };
                            let new_width = match corner {
                                ResizeCorner::TopRight | ResizeCorner::BottomRight => {
                                    (mouse_position.z - anchor.z).max(2.0)
                                }
                                ResizeCorner::TopLeft | ResizeCorner::BottomLeft => {
                                    (anchor.z - mouse_position.z).max(2.0)
                                }
                            };
                            let new_height = match corner {
                                ResizeCorner::TopLeft | ResizeCorner::TopRight => {
                                    (mouse_position.x - anchor.x).max(1.0)
                                }
                                ResizeCorner::BottomLeft | ResizeCorner::BottomRight => {
                                    (anchor.x - mouse_position.x).max(1.0)
                                }
                            };
                            let new_bl = match corner {
                                ResizeCorner::TopLeft => Vec3::new(
                                    anchor.x,
                                    0.0,
                                    mouse_position.z.min(anchor.z - 2.0),
                                ),
                                ResizeCorner::TopRight => anchor,
                                ResizeCorner::BottomLeft => Vec3::new(
                                    mouse_position.x.min(anchor.x - 1.0),
                                    0.0,
                                    mouse_position.z.min(anchor.z - 2.0),
                                ),
                                ResizeCorner::BottomRight => Vec3::new(
                                    mouse_position.x.min(anchor.x - 1.0),
                                    0.0,
                                    anchor.z,
                                ),
                            };
                            let new_pos = new_bl + Vec3::new(new_height * 0.5, 0.0, 0.0);
                            // All reads done, borrows released — now write
                            if let Ok(mut size) = world
                                .get::<&mut components::VoltmeterSize>(entity)
                            {
                                size.width = new_width;
                                size.height = new_height;
                            }
                            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                                pos.position = new_pos;
                            }
                        }
                    } else {
                        match *move_origin {
                            Some(origin) => {
                                let center = mouse_position - origin;
                                application.camera_controller.center -=
                                    Vector3::new(center.x, center.y, center.z);
                            }
                            None => {
                                // Collect voltmeter bounds to avoid holding borrows
                                let voltmeter_bounds: Vec<_> = world
                                    .query::<(&Voltmeter, &Position)>()
                                    .iter()
                                    .filter_map(|(vid, (_, pos))| {
                                        world
                                            .get::<&components::VoltmeterSize>(vid)
                                            .ok()
                                            .map(|size| (vid, pos.position, size.width, size.height))
                                    })
                                    .collect();

                                // Check if clicking near a voltmeter corner for resize
                                let mut found_corner = false;
                                let corner_threshold = 1.0_f32;
                                for (vid, vpos, w, h) in &voltmeter_bounds {
                                    let bl = *vpos + Vec3::new(-h * 0.5, 0.0, 0.0);
                                    let corners = [
                                        (bl + Vec3::new(*h, 0.0, 0.0), ResizeCorner::TopLeft),
                                        (bl + Vec3::new(*h, 0.0, *w), ResizeCorner::TopRight),
                                        (bl, ResizeCorner::BottomLeft),
                                        (bl + Vec3::new(0.0, 0.0, *w), ResizeCorner::BottomRight),
                                    ];
                                    for (corner_pos, corner_type) in &corners {
                                        let dist = Vec3::new(
                                            mouse_position.x - corner_pos.x,
                                            0.0,
                                            mouse_position.z - corner_pos.z,
                                        )
                                        .length();
                                        if dist < corner_threshold {
                                            *resizing_voltmeter = Some((*vid, *corner_type));
                                            found_corner = true;
                                            break;
                                        }
                                    }
                                    if found_corner {
                                        break;
                                    }
                                }

                                if !found_corner {
                                    // Check if clicking inside a voltmeter's trace area
                                    let mut found_voltmeter = false;
                                    for (vid, vpos, w, h) in &voltmeter_bounds {
                                        let bl = *vpos + Vec3::new(-h * 0.5, 0.0, 0.0);
                                        if mouse_position.x >= bl.x
                                            && mouse_position.x <= bl.x + h
                                            && mouse_position.z >= bl.z
                                            && mouse_position.z <= bl.z + w
                                        {
                                            *active_entity = Some(*vid);
                                            *dragging_entity = Some(*vid);
                                            *drag_offset = *vpos - mouse_position;
                                            drag_offset.y = 0.0;
                                            found_voltmeter = true;
                                            break;
                                        }
                                    }

                                    if !found_voltmeter {
                                        if let Some((entity, entity_pos)) = world
                                            .query::<&Position>()
                                            .iter()
                                            .min_by(|a, b| nearest(&mouse_position, a, b))
                                            .and_then(|v| {
                                                within_selection_range(mouse_position, v)
                                            })
                                        {
                                            *active_entity = Some(entity);
                                            *dragging_entity = Some(entity);
                                            *drag_offset = entity_pos - mouse_position;
                                            drag_offset.y = 0.0;
                                        } else {
                                            *active_entity = None;
                                            *move_origin = Some(mouse_position);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false => {
                    *move_origin = None;
                    *dragging_entity = None;
                    *resizing_voltmeter = None;
                }
            },
            Tool::Axon => match connection_tool {
                None => {
                    // Find nearest connectable source (LIF neuron, generator, current clamp, or HH neuron)
                    let source_candidates: Vec<(Entity, Vec3)> = {
                        let mut candidates: Vec<_> = world
                            .query::<&Position>()
                            .with::<&StaticConnectionSource>()
                            .iter()
                            .map(|(e, p)| (e, p.position))
                            .collect();
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&components::LIFNeuron>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&components::CurrentClamp>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&components::GeneratorDynamics>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates
                    };
                    *connection_tool = source_candidates
                        .iter()
                        .min_by(|a, b| {
                            a.1.distance(mouse_position)
                                .partial_cmp(&b.1.distance(mouse_position))
                                .unwrap_or(Ordering::Equal)
                        })
                        .and_then(|(id, pos)| {
                            if pos.distance(mouse_position) < 2.0 * NODE_RADIUS {
                                Some((*id, *pos))
                            } else {
                                None
                            }
                        })
                        .map(|(id, position)| ConnectionTool {
                            start: position,
                            end: mouse_position,
                            from: id,
                        });
                    if let Some(ct) = connection_tool {
                        self.previous_creation = Some(PreviousCreation { entity: ct.from });
                    }
                }
                Some(ct) => {
                    ct.end = mouse_position;
                    // Find nearest connectable target (LIF neuron only, not compartments)
                    let target_candidates: Vec<(Entity, Vec3)> = world
                        .query::<&Position>()
                        .with::<&components::LIFNeuron>()
                        .iter()
                        .map(|(e, p)| (e, p.position))
                        .collect();
                    let nearest_target = target_candidates
                        .iter()
                        .min_by(|a, b| {
                            a.1.distance(mouse_position)
                                .partial_cmp(&b.1.distance(mouse_position))
                                .unwrap_or(Ordering::Equal)
                        })
                        .and_then(|(id, pos)| {
                            if pos.distance(mouse_position) < 2.0 * NODE_RADIUS {
                                Some((*id, *pos))
                            } else {
                                None
                            }
                        });

                    match nearest_target {
                        Some((id, position)) => {
                            let new_connection = Connection {
                                from: ct.from,
                                to: id,
                                strength: 1.0,
                                directional: true,
                            };
                            let connection_exists =
                                world.query::<&Connection>().iter().any(|(_, c)| {
                                    c.from == new_connection.from && c.to == new_connection.to
                                });
                            if !connection_exists && ct.from != id {
                                world.spawn((
                                    new_connection,
                                    Deletable {},
                                    CompartmentCurrent { capacitance: 1.0 },
                                ));
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
                                let new_connection = Connection {
                                    from: ct.from,
                                    to: compartment,
                                    strength: 1.0,
                                    directional: false,
                                };
                                world.spawn((
                                    new_connection,
                                    Deletable {},
                                    CompartmentCurrent { capacitance: 1.0 },
                                ));
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

    pub fn save(&self, path: PathBuf) {
        let mut context = SaveContext;
        let mut serializer = postcard::Serializer {
            output: postcard::ser_flavors::StdVec::new(),
        };
        serialize(&self.world, &mut context, &mut serializer).unwrap();
        let mut writer = std::fs::File::create(path.with_extension("neuronify")).unwrap();
        writer
            .write_all(&serializer.output.finalize().unwrap())
            .unwrap();
    }

    pub fn load_legacy_string(&mut self, contents: &str) {
        match legacy::parse_legacy_nfy(contents) {
            Ok(sim) => {
                self.world.clear();
                self.time = 0.0;
                legacy::spawn::spawn_legacy_simulation(&mut self.world, &sim);
            }
            Err(e) => log::error!("Failed to parse legacy file: {}", e),
        }
    }

    pub fn loadfile(&mut self, path: PathBuf) {
        let mut context = LoadContext::new();
        let reader = std::fs::File::open(path).unwrap();
        let mut bufreader = BufReader::new(reader);
        let mut bytes: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut bytes).unwrap();
        let mut deserializer = postcard::Deserializer::from_bytes(&bytes);
        self.world = deserialize(&mut context, &mut deserializer).unwrap();
    }

    pub fn from_slice(application: &mut visula::Application, bytes: &[u8]) -> Neuronify {
        let mut neuronify = Neuronify::new(application);
        let mut context = LoadContext::new();
        let mut deserializer = postcard::Deserializer::from_bytes(bytes);
        neuronify.world = deserialize(&mut context, &mut deserializer).unwrap();
        neuronify.edit_enabled = false;
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

fn blue() -> Vec3 {
    srgb(30, 102, 245)
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
fn yellow() -> Vec3 {
    srgb(223, 142, 29)
}
fn orange() -> Vec3 {
    srgb(254, 100, 11)
}
fn neurocolor(neuron_type: &NeuronType, value: f32) -> Vec3 {
    let v = 1.0 / (1.0 + (-5.0 * (value - 0.5)).exp());
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
        let cdt = 0.01;

        // Touch sensor stimulation: when Stimulate tool is active near a TouchSensor, fire it
        if let Some(stim) = stimulation_tool {
            let touch_entities: Vec<hecs::Entity> = world
                .query::<(&Position, &components::TouchSensor)>()
                .iter()
                .filter(|(_, (pos, _))| pos.position.distance(stim.position) < 2.0 * NODE_RADIUS)
                .map(|(e, _)| e)
                .collect();
            for entity in touch_entities {
                if let Ok(mut dynamics) = world.get::<&mut components::GeneratorDynamics>(entity) {
                    dynamics.fired = true;
                    dynamics.time_since_fire = 0.0;
                }
            }
        }

        // LIF simulation step — uses dt=0.0001 (0.1ms) matching old C++ Neuronify
        {
            let lif_dt = 0.0001;
            for _ in 0..self.iterations {
                legacy::step::lif_step(world, lif_dt, *time);
                *time += lif_dt;
            }
        }

        // Bridge: LIF neuron fires → inject current into connected Compartments
        {
            let fire_injections: Vec<(Entity, f64)> = world
                .query::<&Connection>()
                .iter()
                .filter_map(|(_, conn)| {
                    let just_fired = world
                        .get::<&components::LIFDynamics>(conn.from)
                        .map(|d| d.time_since_fire == 0.0)
                        .unwrap_or(false);
                    if !just_fired {
                        return None;
                    }
                    if world.get::<&Compartment>(conn.to).is_err() {
                        return None;
                    }
                    let current = if world.get::<&components::Inhibitory>(conn.from).is_ok() {
                        -3000.0
                    } else {
                        3000.0
                    };
                    Some((conn.to, current * conn.strength))
                })
                .collect();

            for (target, current) in fire_injections {
                if let Ok(mut compartment) = world.get::<&mut Compartment>(target) {
                    compartment.injected_current += current;
                }
            }
        }

        // HH Compartment simulation
        for _ in 0..self.iterations {
            for (_, compartment) in world.query_mut::<&mut Compartment>() {
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
                    if let Ok(compartment_from) = world.get::<&Compartment>(connection.from)
                    {
                        let voltage_diff = compartment_from.voltage - compartment_to.voltage;
                        let delta_voltage = voltage_diff / current.capacitance;
                        let new_compartment_to = new_compartments
                            .get_mut(&connection.to)
                            .expect("Could not get new compartment");
                        new_compartment_to.voltage += delta_voltage * cdt;
                        let new_compartment_from = new_compartments
                            .get_mut(&connection.from)
                            .expect("Could not get new compartment");
                        new_compartment_from.voltage -= delta_voltage * cdt;
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
                    if f_a.is_nan() || f_b.is_nan() || f_c.is_nan() {
                        continue;
                    }
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
        }

        // LIF neuron spheres
        let lif_neuron_spheres: Vec<Sphere> = world
            .query::<(
                &components::LIFNeuron,
                &components::LIFDynamics,
                &Position,
            )>()
            .iter()
            .map(|(_entity, (neuron, dynamics, position))| {
                let value = ((dynamics.voltage - neuron.resting_potential)
                    / (neuron.threshold - neuron.resting_potential))
                    .clamp(0.0, 1.0) as f32;
                let is_inhibitory = world.get::<&components::Inhibitory>(_entity).is_ok();
                let color = if is_inhibitory {
                    value * mantle() + (1.0 - value) * red()
                } else {
                    value * base() + (1.0 - value) * blue()
                };
                Sphere {
                    position: position.position,
                    color,
                    radius: NODE_RADIUS,
                    _padding: Default::default(),
                }
            })
            .collect();

        let current_clamp_spheres: Vec<Sphere> = world
            .query::<&Position>()
            .with::<&components::CurrentClamp>()
            .iter()
            .map(|(_entity, position)| Sphere {
                position: position.position,
                color: yellow(),
                radius: NODE_RADIUS,
                _padding: Default::default(),
            })
            .collect();

        let generator_spheres: Vec<Sphere> = world
            .query::<(&Position, &components::GeneratorDynamics)>()
            .iter()
            .map(|(_entity, (position, _))| Sphere {
                position: position.position,
                color: orange(),
                radius: NODE_RADIUS,
                _padding: Default::default(),
            })
            .collect();

        let compartment_spheres: Vec<Sphere> = world
            .query::<(&Compartment, &Position, &NeuronType)>()
            .iter()
            .map(|(_entity, (compartment, position, neuron_type))| {
                let value = ((compartment.voltage + 10.0) / 120.0) as f32;
                Sphere {
                    position: position.position,
                    color: neurocolor(neuron_type, value),
                    radius: 0.3 * NODE_RADIUS,
                    _padding: Default::default(),
                }
            })
            .collect();

        // Trigger spheres: small spheres traveling along connections during synaptic delay
        let trigger_spheres: Vec<Sphere> = world
            .query::<(&components::CurrentSynapse, &Connection)>()
            .iter()
            .flat_map(|(_entity, (synapse, connection))| {
                let start = world
                    .get::<&Position>(connection.from)
                    .map(|p| p.position)
                    .unwrap_or(Vec3::ZERO);
                let end = world
                    .get::<&Position>(connection.to)
                    .map(|p| p.position)
                    .unwrap_or(Vec3::ZERO);
                let diff = end - start;
                synapse
                    .triggers
                    .iter()
                    .map(move |&trigger_time| {
                        let fire_time = trigger_time - synapse.delay;
                        let progress = if synapse.delay > 0.0 {
                            ((synapse.time - fire_time) / synapse.delay).clamp(0.0, 1.0) as f32
                        } else {
                            1.0
                        };
                        Sphere {
                            position: start + diff * progress,
                            color: crust(),
                            radius: NODE_RADIUS * 0.5,
                            _padding: Default::default(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut spheres = Vec::new();
        spheres.extend(lif_neuron_spheres.iter());
        spheres.extend(current_clamp_spheres.iter());
        spheres.extend(generator_spheres.iter());
        spheres.extend(compartment_spheres.iter());
        spheres.extend(trigger_spheres.iter());

        // Collect connection info to detect reciprocal pairs
        let connection_info: Vec<(Entity, Entity, Entity, f32, bool)> = world
            .query::<&Connection>()
            .iter()
            .map(|(e, c)| (e, c.from, c.to, c.strength as f32, c.directional))
            .collect();

        // Build a set of (from, to) pairs to detect reciprocals
        let connection_pairs: std::collections::HashSet<(Entity, Entity)> = connection_info
            .iter()
            .map(|(_, from, to, _, _)| (*from, *to))
            .collect();

        let mut connections: Vec<ConnectionData> = Vec::new();

        for &(_edge_entity, from, to, strength, directional) in &connection_info {
            let start = world
                .get::<&Position>(from)
                .expect("Connection from broken")
                .position;
            let end = world
                .get::<&Position>(to)
                .expect("Connection to broken")
                .position;
            let value = |target: Entity| -> f32 {
                if let Ok(compartment) = world.get::<&Compartment>(target) {
                    ((compartment.voltage + 10.0) / 120.0) as f32
                } else if let Ok(dynamics) = world.get::<&components::LIFDynamics>(target) {
                    let neuron = world.get::<&components::LIFNeuron>(target).ok();
                    if let Some(neuron) = neuron {
                        ((dynamics.voltage - neuron.resting_potential)
                            / (neuron.threshold - neuron.resting_potential))
                            .clamp(0.0, 1.0) as f32
                    } else {
                        0.5
                    }
                } else {
                    1.0
                }
            };
            let start_value = value(to);
            let end_value = value(from);
            let (start_color, end_color) =
                if world.get::<&components::CurrentClamp>(from).is_ok() {
                    (yellow(), yellow())
                } else if world.get::<&components::GeneratorDynamics>(from).is_ok() {
                    (orange(), orange())
                } else if let Ok(neuron_type) = world.get::<&NeuronType>(from) {
                    (
                        neurocolor(&neuron_type, start_value),
                        neurocolor(&neuron_type, end_value),
                    )
                } else {
                    (crust(), crust())
                };

            let is_reciprocal = connection_pairs.contains(&(to, from));
            let dir_val = if directional { 1.0 } else { 0.0 };

            if is_reciprocal {
                // Bend to the right (relative to start→end direction)
                let segments = 16;
                let diff = end - start;
                // Right perpendicular in the xz ground plane: cross(diff, up)
                let up = Vec3::new(0.0, 1.0, 0.0);
                let right = diff.cross(up);
                let bend_amount = 0.2 * diff.length();
                let control = (start + end) * 0.5 + right.normalize_or_zero() * bend_amount;

                for i in 0..segments {
                    let t0 = i as f32 / segments as f32;
                    let t1 = (i + 1) as f32 / segments as f32;
                    let p0 = quadratic_bezier(start, control, end, t0);
                    let p1 = quadratic_bezier(start, control, end, t1);
                    let c0 = start_color.lerp(end_color, t0);
                    let c1 = start_color.lerp(end_color, t1);
                    let is_last = i == segments - 1;
                    connections.push(ConnectionData {
                        position_a: p0,
                        position_b: p1,
                        strength,
                        directional: if is_last { dir_val } else { 0.0 },
                        start_color: c0,
                        end_color: c1,
                        _padding: Default::default(),
                    });
                }
            } else {
                connections.push(ConnectionData {
                    position_a: start,
                    position_b: end,
                    strength,
                    directional: dir_val,
                    start_color,
                    end_color,
                    _padding: Default::default(),
                });
            }
        }

        if self.tool == Tool::StaticConnection {
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

        // Voltmeter traces as 3D lines
        for (voltmeter_id, _) in world.query::<&Voltmeter>().iter() {
            // Find the VoltageSeries + Connection on this voltmeter entity
            let (series, spike_times, voltmeter_pos, trace_width, trace_height) = {
                let Ok(series) = world.get::<&VoltageSeries>(voltmeter_id) else {
                    continue;
                };
                let Ok(pos) = world.get::<&Position>(voltmeter_id) else {
                    continue;
                };
                let size = world
                    .get::<&components::VoltmeterSize>(voltmeter_id)
                    .ok();
                let tw = size.as_ref().map(|s| s.width).unwrap_or(8.0);
                let th = size.as_ref().map(|s| s.height).unwrap_or(4.0);
                // Clone the data we need so we can release the borrows
                let measurements: Vec<_> = series
                    .measurements
                    .iter()
                    .map(|m| (m.time, m.voltage))
                    .collect();
                let spikes = series.spike_times.clone();
                let vpos = pos.position;
                (measurements, spikes, vpos, tw, th)
            };

            if series.len() < 2 {
                continue;
            }

            // Trace dimensions in world units
            let time_window = 1.0_f64 / 3.0; // seconds of data to show
            let v_min = -100.0_f64; // mV
            let v_max = 50.0_f64; // mV

            let latest_time = series.last().map(|(t, _)| *t).unwrap_or(0.0);
            let start_time = latest_time - time_window;

            // Bottom-left origin of the trace: offset from voltmeter position
            // x-axis points up on screen, so bottom-left is below the position
            let bottom_left_origin = voltmeter_pos + Vec3::new(-trace_height * 0.5, 0.0, 0.0);

            let green = srgb(64, 160, 43);

            // Draw border frame
            let bottom_left = bottom_left_origin;
            let bottom_right = bottom_left_origin + Vec3::new(0.0, 0.0, trace_width);
            let top_left = bottom_left_origin + Vec3::new(trace_height, 0.0, 0.0);
            let top_right = bottom_left_origin + Vec3::new(trace_height, 0.0, trace_width);
            let frame_color = srgb(80, 80, 100);
            for (a, b) in [
                (top_left, top_right),
                (top_right, bottom_right),
                (bottom_right, bottom_left),
                (bottom_left, top_left),
            ] {
                connections.push(ConnectionData {
                    position_a: a,
                    position_b: b,
                    strength: 1.0,
                    directional: 0.0,
                    start_color: frame_color,
                    end_color: frame_color,
                    _padding: Default::default(),
                });
            }

            // Draw voltage trace
            let visible: Vec<_> = series
                .iter()
                .filter(|(t, _)| *t >= start_time)
                .collect();

            for window in visible.windows(2) {
                let (t0, v0) = window[0];
                let (t1, v1) = window[1];

                let z0 = ((t0 - start_time) / time_window) as f32 * trace_width;
                let z1 = ((t1 - start_time) / time_window) as f32 * trace_width;
                let x0 = ((v0 - v_min) / (v_max - v_min)) as f32 * trace_height;
                let x1 = ((v1 - v_min) / (v_max - v_min)) as f32 * trace_height;

                let p0 = bottom_left_origin + Vec3::new(x0, 0.0, z0);
                let p1 = bottom_left_origin + Vec3::new(x1, 0.0, z1);

                connections.push(ConnectionData {
                    position_a: p0,
                    position_b: p1,
                    strength: 1.0,
                    directional: 0.0,
                    start_color: green,
                    end_color: green,
                    _padding: Default::default(),
                });
            }

            // Draw vertical spike markers
            for spike_time in &spike_times {
                if *spike_time < start_time || *spike_time > latest_time {
                    continue;
                }
                let z = ((spike_time - start_time) / time_window) as f32 * trace_width;
                let top = bottom_left_origin + Vec3::new(trace_height, 0.0, z);
                let bottom = bottom_left_origin + Vec3::new(0.0, 0.0, z);
                connections.push(ConnectionData {
                    position_a: top,
                    position_b: bottom,
                    strength: 1.0,
                    directional: 0.0,
                    start_color: green,
                    end_color: green,
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

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        egui::Area::new("edit_button_area")
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .show(context, |ui| {
                ui.toggle_value(&mut self.edit_enabled, "Edit").clicked();
            });
        if self.edit_enabled {
            #[cfg(not(target_arch = "wasm32"))]
            egui::TopBottomPanel::top("top_panel").show(context, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.save(path);
                            }
                        }

                        if ui.button("Open").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.loadfile(path);
                            }
                        }
                    });
                    ui.menu_button("Examples", |ui| {
                        ui.menu_button("Tutorial", |ui| {
                            if ui.button("1 - Intro").clicked() {
                                self.load_legacy_string(include_str!("../examples/tutorial_1_intro.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("2 - Circuits").clicked() {
                                self.load_legacy_string(include_str!("../examples/tutorial_2_circuits.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("3 - Creation").clicked() {
                                self.load_legacy_string(include_str!("../examples/tutorial_3_creation.nfy"));
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Neurons", |ui| {
                            if ui.button("Leaky").clicked() {
                                self.load_legacy_string(include_str!("../examples/leaky.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Inhibitory").clicked() {
                                self.load_legacy_string(include_str!("../examples/inhibitory.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Adaptation").clicked() {
                                self.load_legacy_string(include_str!("../examples/adaptation.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Burst").clicked() {
                                self.load_legacy_string(include_str!("../examples/burst.nfy"));
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Circuits", |ui| {
                            if ui.button("Input Summation").clicked() {
                                self.load_legacy_string(include_str!("../examples/input_summation.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Prolonged Activity").clicked() {
                                self.load_legacy_string(include_str!("../examples/prolonged_activity.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Disinhibition").clicked() {
                                self.load_legacy_string(include_str!("../examples/disinhibition.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Recurrent Inhibition").clicked() {
                                self.load_legacy_string(include_str!("../examples/recurrent_inhibition.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Reciprocal Inhibition").clicked() {
                                self.load_legacy_string(include_str!("../examples/reciprocal_inhibition.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Lateral Inhibition").clicked() {
                                self.load_legacy_string(include_str!("../examples/lateral_inhibition.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Lateral Inhibition 1").clicked() {
                                self.load_legacy_string(include_str!("../examples/lateral_inhibition_1.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Lateral Inhibition 2").clicked() {
                                self.load_legacy_string(include_str!("../examples/lateral_inhibition_2.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Two Neuron Oscillator").clicked() {
                                self.load_legacy_string(include_str!("../examples/two_neuron_oscillator.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Rhythm Transformation").clicked() {
                                self.load_legacy_string(include_str!("../examples/rythm_transformation.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Types of Inhibition").clicked() {
                                self.load_legacy_string(include_str!("../examples/types_of_inhibition.nfy"));
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Textbook", |ui| {
                            if ui.button("IF Response").clicked() {
                                self.load_legacy_string(include_str!("../examples/if_response.nfy"));
                                ui.close_menu();
                            }
                            if ui.button("Refractory Period").clicked() {
                                self.load_legacy_string(include_str!("../examples/refractory_period.nfy"));
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Items", |ui| {
                            if ui.button("Generators").clicked() {
                                self.load_legacy_string(include_str!("../examples/generators.nfy"));
                                ui.close_menu();
                            }
                        });
                    });
                });
            });
            egui::Window::new("Elements").show(context, |ui| {
                for category in &TOOL_CATEGORIES {
                    ui.collapsing(category.label(), |ui| {
                        for (tool_value, label) in category.tools() {
                            ui.selectable_value(&mut self.tool, tool_value, label);
                        }
                    });
                }
            });
            egui::Window::new("Settings").show(context, |ui| {
                ui.label(format!("FPS: {:.0}", self.fps));
                ui.label("Simulation speed");
                ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
            });
            if let Some(active_entity) = self.active_entity {
                egui::Window::new("Selection").show(context, |ui| {
                    // Hodgkin-Huxley compartment
                    if let Ok(compartment) = self.world.get::<&Compartment>(active_entity) {
                        ui.collapsing("Compartment", |ui| {
                            egui::Grid::new("compartment_state").show(ui, |ui| {
                                ui.label("Voltage:");
                                ui.label(format!("{:.2} mV", compartment.voltage));
                                ui.end_row();
                                ui.label("m:");
                                ui.label(format!("{:.2}", compartment.m));
                                ui.end_row();
                                ui.label("n:");
                                ui.label(format!("{:.2}", compartment.n));
                                ui.end_row();
                                ui.label("h:");
                                ui.label(format!("{:.2}", compartment.h));
                                ui.end_row();
                            });
                        });
                    }
                    // LIF neuron
                    if let Ok(mut neuron) =
                        self.world
                            .get::<&mut components::LIFNeuron>(active_entity)
                    {
                        let is_inhibitory = self
                            .world
                            .get::<&components::Inhibitory>(active_entity)
                            .is_ok();
                        let label = if is_inhibitory {
                            "LIF Neuron (Inhibitory)"
                        } else {
                            "LIF Neuron (Excitatory)"
                        };
                        ui.collapsing(label, |ui| {
                            egui::Grid::new("neuron_settings").show(ui, |ui| {
                                ui.label("Threshold:");
                                ui.add(egui::Slider::new(
                                    &mut neuron.threshold,
                                    -0.08..=-0.03,
                                ).suffix(" V"));
                                ui.end_row();
                                ui.label("Resting potential:");
                                ui.add(egui::Slider::new(
                                    &mut neuron.resting_potential,
                                    -0.09..=-0.05,
                                ).suffix(" V"));
                                ui.end_row();
                                ui.label("Capacitance:");
                                ui.add(
                                    egui::Slider::new(&mut neuron.capacitance, 1e-11..=1e-9)
                                        .suffix(" F"),
                                );
                                ui.end_row();
                            });
                        });
                    }
                    if let Ok(dynamics) = self
                        .world
                        .get::<&components::LIFDynamics>(active_entity)
                    {
                        ui.collapsing("Dynamics", |ui| {
                            egui::Grid::new("neuron_dynamics").show(ui, |ui| {
                                ui.label("Voltage:");
                                ui.label(format!("{:.4} V", dynamics.voltage));
                                ui.end_row();
                                ui.label("Fired:");
                                ui.label(format!("{}", dynamics.fired));
                                ui.end_row();
                            });
                        });
                    }
                    // Current clamp
                    if let Ok(mut clamp) = self
                        .world
                        .get::<&mut components::CurrentClamp>(active_entity)
                    {
                        ui.collapsing("Current Source", |ui| {
                            egui::Grid::new("clamp_settings").show(ui, |ui| {
                                ui.label("Current:");
                                ui.add(
                                    egui::Slider::new(&mut clamp.current_output, 0.0..=1e-8)
                                        .suffix(" A"),
                                );
                                ui.end_row();
                            });
                        });
                    }
                    // Regular Spike Generator
                    if let Ok(mut gen) = self
                        .world
                        .get::<&mut components::RegularSpikeGenerator>(active_entity)
                    {
                        ui.collapsing("Spike Generator", |ui| {
                            egui::Grid::new("spike_gen_settings").show(ui, |ui| {
                                ui.label("Frequency:");
                                ui.add(
                                    egui::Slider::new(&mut gen.frequency, 1.0..=200.0)
                                        .suffix(" Hz"),
                                );
                                ui.end_row();
                            });
                        });
                    }
                    // Poisson Generator
                    if let Ok(mut gen) = self
                        .world
                        .get::<&mut components::PoissonGenerator>(active_entity)
                    {
                        ui.collapsing("Poisson Generator", |ui| {
                            egui::Grid::new("poisson_gen_settings").show(ui, |ui| {
                                ui.label("Rate:");
                                ui.add(
                                    egui::Slider::new(&mut gen.rate, 1.0..=200.0)
                                        .suffix(" Hz"),
                                );
                                ui.end_row();
                            });
                        });
                    }
                    // Voltmeter
                    if self.world.get::<&Voltmeter>(active_entity).is_ok() {
                        ui.collapsing("Voltmeter", |ui| {
                            if let Ok(mut size) = self
                                .world
                                .get::<&mut components::VoltmeterSize>(
                                    active_entity,
                                )
                            {
                                egui::Grid::new("voltmeter_size_settings").show(ui, |ui| {
                                    ui.label("Width:");
                                    ui.add(egui::Slider::new(&mut size.width, 2.0..=20.0));
                                    ui.end_row();
                                    ui.label("Height:");
                                    ui.add(egui::Slider::new(&mut size.height, 1.0..=10.0));
                                    ui.end_row();
                                });
                            }
                            if let Ok(series) =
                                self.world.get::<&VoltageSeries>(active_entity)
                            {
                                if let Some(last) = series.measurements.last() {
                                    ui.label(format!("Voltage: {:.2} mV", last.voltage));
                                }
                            }
                        });
                    }
                });
            }
        }
    }

    fn handle_event(&mut self, application: &mut visula::Application, event: &Event<CustomEvent>) {
        if let Event::WindowEvent { window_id, .. } = event {
            if &application.window.id() != window_id {
                return;
            }
        }
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
                self.mouse.delta_position = None;
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
                self.mouse.delta_position = self.mouse.position.map(|previous| {
                    PhysicalPosition::new(position.x - previous.x, position.y - previous.y)
                });
                self.mouse.position = Some(*position);
                self.handle_tool(application);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                application.camera_controller.distance *= 1.0 - scroll * 0.1;
                application.camera_controller.distance =
                    application.camera_controller.distance.clamp(5.0, 200.0);
            }
            _ => {}
        }
    }
}

struct Bundle {
    application: Application,
    simulation: Neuronify,
}

#[wasm_bindgen]
pub struct WasmWrapper {
    event_loop: EventLoop<CustomEvent>,
    bundles: Vec<Bundle>,
}

#[wasm_bindgen]
pub async fn initialize() -> WasmWrapper {
    initialize_logger();
    let event_loop = create_event_loop();
    let bundles: Vec<Bundle> = Vec::new();
    WasmWrapper {
        event_loop,
        bundles,
    }
}

#[wasm_bindgen]
pub async fn load(wrapper: &mut WasmWrapper, canvas: &str, url: &str) -> Result<(), JsValue> {
    let window = create_window(
        RunConfig {
            canvas_name: canvas.to_owned(),
        },
        &wrapper.event_loop,
    );
    let mut application = pollster::block_on(async { Application::new(Arc::new(window)).await });

    let mut opts = RequestInit::new();
    opts.method("GET");
    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().ok_or("No global `window` exists")?;
    let response_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response_value.dyn_into()?;
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = Uint8Array::new(&buffer);
    let vec = uint8_array.to_vec();
    let simulation = Neuronify::from_slice(&mut application, &vec);
    wrapper.bundles.push(Bundle {
        application,
        simulation,
    });
    Ok(())
}

#[wasm_bindgen]
pub async fn start(mut wrapper: WasmWrapper) -> Result<(), JsValue> {
    let _event_handler = move |event, target: &EventLoopWindowTarget<CustomEvent>| {
        for bundle in wrapper.bundles.iter_mut() {
            let application = &mut bundle.application;
            let simulation = &mut bundle.simulation;
            if !application.handle_event(&event) {
                simulation.handle_event(application, &event);
            }
            if let Event::WindowEvent { ref event, .. } = event {
                match event {
                    WindowEvent::RedrawRequested => {
                        application.update();
                        simulation.update(application);
                        application.render(simulation);

                        application.window.borrow_mut().request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        }
    };
    #[cfg(target_arch = "wasm32")]
    wrapper.event_loop.spawn(_event_handler);
    Ok(())
}

pub fn run() {
    visula::run(Neuronify::new);
}
