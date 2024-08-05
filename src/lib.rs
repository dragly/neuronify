use std::borrow::BorrowMut;
use visula::Simulation;
mod measurement;
use std::sync::Arc;
use visula::initialize_event_loop_and_window_with_config;
use visula::initialize_logger;
use visula::winit::keyboard::ModifiersKeyState;
use visula::Application;
use visula::RunConfig;
use wasm_bindgen::prelude::*;

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

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum Tool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    CurrentSource,
    Voltmeter,
    StaticConnection,
    LearningConnection,
    Erase,
    Stimulate,
}

const NODE_RADIUS: f32 = 1.0;
const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;
const SIGMA: f32 = 1.0 * NODE_RADIUS;

#[derive(Clone, Debug)]
enum NeuronType {
    Excitatory,
    Inhibitory,
}

#[derive(Clone, Debug)]
struct NeuronDynamics {
    voltage: f64,
    current: f64,
    refraction: f64,
    fired: bool,
}

#[derive(Clone, Debug)]
struct Neuron {
    initial_voltage: f64,
    reset_potential: f64,
    resting_potential: f64,
    threshold: f64,
    ty: NeuronType,
}

#[derive(Clone, Debug)]
struct SynapseCurrent {
    current: f64,
    tau: f64,
}

#[derive(Clone, Debug)]
struct Selectable {
    selected: bool,
}

#[derive(Clone, Debug)]
struct CurrentSource {
    current: f64,
}

#[derive(Clone, Debug)]
struct StaticSource {}

#[derive(Clone, Debug)]
struct LearningSynapse {}

#[derive(Clone, Debug)]
struct PreviousCreation {
    position: Vec3,
}

#[derive(Clone, Debug)]
struct Deletable {}

#[derive(Clone, Debug)]
pub struct Position {
    pub position: Vec3,
}

impl Neuron {
    pub fn new(ty: NeuronType) -> Neuron {
        Neuron {
            initial_voltage: 0.0,
            reset_potential: -70.0,
            resting_potential: -70.0,
            threshold: 30.0,
            ty,
        }
    }
}

#[derive(Clone, Debug)]
struct ConnectionTool {
    start: Vec3,
    end: Vec3,
    from: Entity,
}

#[derive(Clone, Debug)]
struct Trigger {
    total: f64,
    remaining: f64,
    current: f64,
    connection: Entity,
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

#[derive(Clone, Debug)]
struct LeakCurrent {
    current: f64,
    tau: f64,
}

#[derive(Clone, Debug)]
struct Connection {
    from: Entity,
    to: Entity,
    strength: f64,
}

#[derive(Clone, Debug)]
pub struct StimulateCurrent {
    pub current: f64,
}

#[derive(Clone, Debug)]
struct StimulationTool {
    position: Vec3,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Instance, Pod, Zeroable)]
struct Sphere {
    position: glam::Vec3,
    color: glam::Vec3,
    radius: f32,
    _padding: f32,
}

struct Mouse {
    left_down: bool,
    position: Option<PhysicalPosition<f64>>,
}

struct Keyboard {
    shift_down: bool,
}

pub struct Neuronify {
    tool: Tool,
    previous_creation: Option<PreviousCreation>,
    connection_tool: Option<ConnectionTool>,
    stimulation_tool: Option<StimulationTool>,
    world: hecs::World,
    time: f64,
    mouse: Mouse,
    keyboard: Keyboard,
    spheres: Spheres,
    sphere_buffer: InstanceBuffer<Sphere>,
    connection_lines: Lines,
    connection_spheres: Spheres,
    connection_buffer: InstanceBuffer<ConnectionData>,
    iterations: u32,
    window_y: f32,
}

#[derive(Debug)]
pub struct Error {}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
struct ConnectionData {
    position_a: Vec3,
    position_b: Vec3,
    strength: f32,
    _padding: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
struct LineData {
    start: Vec3,
    end: Vec3,
    _padding: [f32; 2],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
struct MeshInstanceData {
    position: Vec3,
    _padding: f32,
    rotation: Quat,
}

impl Neuronify {
    pub fn new(application: &mut visula::Application) -> Neuronify {
        application.camera_controller.enabled = false;

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
            - connection_vector.clone() / connection_vector.clone().length() * NODE_RADIUS * 2.0;
        let connection_lines = Lines::new(
            &application.rendering_descriptor(),
            &LineDelegate {
                start: connection.position_a.clone(),
                end: connection_endpoint.clone(),
                width: 0.2.into(),
                alpha: 1.0.into(),
            },
        )
        .unwrap();

        let connection_spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereDelegate {
                position: connection_endpoint,
                radius: (0.5 * NODE_RADIUS).into(),
                color: Vec3::new(1.0, 1.0, 1.0).into(),
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
            window_y: 0.0,
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

        let minimum_distance = 6.0 * NODE_RADIUS;
        let previous_too_near = if let Some(pc) = previous_creation {
            pc.position.distance(mouse_position) < minimum_distance
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
                match self.tool {
                    Tool::ExcitatoryNeuron => {
                        self.world.spawn((
                            Position {
                                position: mouse_position,
                            },
                            Neuron::new(NeuronType::Excitatory),
                            StaticSource {},
                            dynamics,
                            leak_current,
                            Deletable {},
                            stimulate_current,
                            Selectable { selected: false },
                        ));
                    }
                    Tool::InhibitoryNeuron => {
                        self.world.spawn((
                            Position {
                                position: mouse_position,
                            },
                            Neuron::new(NeuronType::Inhibitory),
                            StaticSource {},
                            dynamics,
                            leak_current,
                            Deletable {},
                            stimulate_current,
                            Selectable { selected: false },
                        ));
                    }
                    _ => {}
                }
                self.previous_creation = Some(PreviousCreation {
                    position: mouse_position,
                });
            }
            Tool::CurrentSource => {
                if previous_too_near {
                    return;
                }
                self.world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    StaticSource {},
                    CurrentSource { current: 200.0 },
                    Deletable {},
                    Selectable { selected: false },
                ));
                self.previous_creation = Some(PreviousCreation {
                    position: mouse_position,
                });
            }
            Tool::StaticConnection | Tool::LearningConnection => {
                let closest = |(_, x): &(Entity, &Position), (_, y): &(Entity, &Position)| {
                    mouse_position
                        .distance(x.position)
                        .partial_cmp(&mouse_position.distance(y.position))
                        .unwrap_or(std::cmp::Ordering::Equal)
                };
                let filter = |(id, position): (Entity, &Position)| {
                    if mouse_position.distance(position.position) < 1.5 * NODE_RADIUS {
                        Some((id, position.position))
                    } else {
                        None
                    }
                };
                if let Some(ct) = connection_tool {
                    let nearest_target = world
                        .query::<&Position>()
                        .with::<&Neuron>()
                        .iter()
                        .min_by(closest)
                        .and_then(filter);
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
                            if *tool == Tool::StaticConnection {
                                world.spawn((new_connection, Deletable {}, synapse_current));
                            } else {
                                world.spawn((
                                    new_connection,
                                    Deletable {},
                                    LearningSynapse {},
                                    synapse_current,
                                ));
                            };
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
                        .with::<&StaticSource>()
                        .iter()
                        .min_by(closest)
                        .and_then(filter)
                        .and_then(|(id, position)| {
                            Some(ConnectionTool {
                                start: position,
                                end: mouse_position,
                                from: id,
                            })
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
                            Some((entity.clone(), position.clone()))
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
                *previous_creation = Some(PreviousCreation {
                    position: position.position,
                });
                world.spawn((
                    VoltageSeries {
                        measurements: RollingWindow::new(100000),
                    },
                    Connection {
                        from: target,
                        to: voltmeter,
                        strength: 1.0,
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
        }
    }

    pub fn save() {}

    pub fn load(application: &mut visula::Application, url: &str) -> Neuronify {
        Neuronify::new(application)
    }
}

impl visula::Simulation for Neuronify {
    type Error = Error;
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
                stimulate.current = (600.0
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
            let new_triggers: Vec<(Entity, f64)> = world
                .query::<&Connection>()
                .with::<&SynapseCurrent>()
                .iter()
                .flat_map(|(connection_entity, connection)| {
                    let mut triggers = vec![];
                    if let Ok(neuron_from) = world.get::<&Neuron>(connection.from) {
                        if let Ok(dynamics_from) = world.get::<&NeuronDynamics>(connection.from) {
                            let base_current = match &neuron_from.ty {
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
            .query::<(&Neuron, &NeuronDynamics, &Position)>()
            .iter()
            .map(|(_entity, (neuron, dynamics, position))| {
                let value = ((dynamics.voltage + 100.0) / 150.0).clamp(0.0, 1.0) as f32;
                let color = match neuron.ty {
                    NeuronType::Excitatory => Vec3::new(value / 2.0, value, 0.95),
                    NeuronType::Inhibitory => Vec3::new(0.95, value / 2.0, value),
                };
                Sphere {
                    position: position.position,
                    color,
                    radius: NODE_RADIUS,
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
            .filter_map(|(_entity, trigger)| {
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
                Some(Sphere {
                    position,
                    color: Vec3::new(0.8, 0.9, 0.9),
                    radius: NODE_RADIUS * 0.5,
                    _padding: Default::default(),
                })
            })
            .collect();

        let mut spheres = Vec::new();
        spheres.extend(neuron_spheres.iter());
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
                ConnectionData {
                    position_a: start,
                    position_b: end,
                    strength: connection.strength as f32,
                    _padding: Default::default(),
                }
            })
            .collect();

        if let Some(connection) = &connection_tool {
            connections.push(ConnectionData {
                position_a: connection.start,
                position_b: connection.end,
                strength: 1.0,
                _padding: Default::default(),
            });
        }

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);

        self.connection_buffer
            .update(&application.device, &application.queue, &connections);
    }

    fn render(&mut self, data: &mut RenderData) {
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
    }

    fn gui(&mut self, application: &visula::Application, context: &egui::Context) {
        egui::Window::new("Settings").show(context, |ui| {
            ui.label("Tool");
            for value in Tool::iter() {
                ui.selectable_value(&mut self.tool, value.clone(), format!("{:?}", &value));
            }
            ui.label("Simulation speed");
            ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
        });

        for (voltmeter_id, voltmeter) in self.world.query::<&Voltmeter>().iter() {
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
pub fn load(url: &str) {
    initialize_logger();
    let (event_loop, window) = initialize_event_loop_and_window_with_config(RunConfig {
        canvas_name: "canvas".to_owned(),
    });
    let main_window_id = window.id();
    let mut application =
        pollster::block_on(async { Application::new(Arc::new(window), &event_loop).await });
    let mut simulation = Neuronify::load(&mut application, &url);
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
}
