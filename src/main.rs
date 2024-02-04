use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;
use glam::Quat;
use glam::Vec3;
use hecs::Entity;
use std::collections::HashMap;
use std::iter::FromIterator;
use strum::EnumIter;
use strum::IntoEnumIterator;
use visula::Renderable;
use visula::{
    CustomEvent, InstanceBuffer, LineDelegate, Lines, RenderData, SphereDelegate, Spheres,
};
use visula_derive::Instance;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
};

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum Tool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    StaticConnection,
    LearningConnection,
    Erase,
    Stimulate,
}

const NODE_RADIUS: f32 = 1.0;
const ERASE_RADIUS: f32 = 2.0 * NODE_RADIUS;
const SIGMA: f32 = 3.0 * NODE_RADIUS;

#[derive(Clone, Debug)]
enum NeuronType {
    Excitatory,
    Inhibitory,
}

#[derive(Clone, Debug)]
struct NeuronDynamics {
    voltage: f64,
    input_current: f64,
    refraction: f64,
    fired: bool,
    last_fired: Option<f64>,
}

#[derive(Clone, Debug)]
struct Neuron {
    initial_voltage: f64,
    reset_potential: f64,
    resting_potential: f64,
    leak_tau: f64,
    threshold: f64,
    input_tau: f64,
    ty: NeuronType,
    dynamics: HashMap<Entity, NeuronDynamics>,
}

#[derive(Clone, Debug)]
struct Brain {}

#[derive(Clone, Debug)]
struct Selectable {
    selected: bool,
}

struct LearningSynapse {}

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
    pub fn new(ty: NeuronType, brains: &[Entity]) -> Neuron {
        let dynamics = HashMap::from_iter(brains.iter().map(|e| {
            (
                *e,
                NeuronDynamics {
                    refraction: 0.0,
                    voltage: 0.0,
                    input_current: 0.0,
                    fired: false,
                    last_fired: None,
                },
            )
        }));
        Neuron {
            initial_voltage: 0.0,
            leak_tau: 1.0,
            reset_potential: -70.0,
            resting_potential: -70.0,
            threshold: 30.0,
            input_tau: 0.1,
            ty,
            dynamics,
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
    brain: Entity,
}

impl Trigger {
    pub fn new(time: f64, current: f64, connection: Entity, brain: Entity) -> Trigger {
        Trigger {
            total: time,
            remaining: time,
            current,
            connection,
            brain,
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
struct Connection {
    from: Entity,
    to: Entity,
    strength: f64,
}

#[derive(Clone, Debug)]
pub struct Stimulate {
    pub injected_current: f64,
}

impl Stimulate {
    pub fn new() -> Stimulate {
        Stimulate {
            injected_current: 0.0,
        }
    }
}

impl Default for Stimulate {
    fn default() -> Self {
        Self::new()
    }
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
    _padding: [f32; 2],
}

struct Mouse {
    left_down: bool,
    position: Option<PhysicalPosition<f64>>,
}

struct Keyboard {
    shift_down: bool,
}

struct Simulation {
    brains: Vec<Entity>,
    selected_brain: Entity,
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
}

#[derive(Debug)]
struct Error {}

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

impl Simulation {
    fn new(application: &mut visula::Application) -> Simulation {
        application.camera_controller.enabled = false;

        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let connection_buffer = InstanceBuffer::<ConnectionData>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let connection = connection_buffer.instance();

        let spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereDelegate {
                position: sphere.position.clone(),
                radius: NODE_RADIUS.into(),
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

        let mut world = hecs::World::new();

        let brains = vec![world.spawn((Brain {},))];
        let player_brain = *brains.first().expect("No brains were added!");

        Simulation {
            brains,
            selected_brain: player_brain,
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
        }
    }

    fn handle_tool(&mut self, application: &visula::Application) {
        let Simulation {
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
        match tool {
            Tool::ExcitatoryNeuron | Tool::InhibitoryNeuron => {
                if previous_creation.is_none()
                    || previous_creation
                        .as_ref()
                        .unwrap()
                        .position
                        .distance(mouse_position)
                        > minimum_distance
                {
                    match self.tool {
                        Tool::ExcitatoryNeuron => {
                            self.world.spawn((
                                Position {
                                    position: mouse_position,
                                },
                                Neuron::new(NeuronType::Excitatory, &self.brains),
                                Deletable {},
                                Stimulate::new(),
                                Selectable { selected: false },
                            ));
                        }
                        Tool::InhibitoryNeuron => {
                            self.world.spawn((
                                Position {
                                    position: mouse_position,
                                },
                                Neuron::new(NeuronType::Inhibitory, &self.brains),
                                Deletable {},
                                Stimulate::new(),
                                Selectable { selected: false },
                            ));
                        }
                        _ => {}
                    }
                    self.previous_creation = Some(PreviousCreation {
                        position: mouse_position,
                    });
                }
            }
            Tool::StaticConnection | Tool::LearningConnection => {
                let nearest = world
                    .query::<&Position>()
                    .with::<&Neuron>()
                    .iter()
                    .min_by(|(_, x), (_, y)| {
                        mouse_position
                            .distance(x.position)
                            .partial_cmp(&mouse_position.distance(y.position))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .and_then(|(id, position)| {
                        if mouse_position.distance(position.position) < 1.5 * NODE_RADIUS {
                            Some((id, position.position))
                        } else {
                            None
                        }
                    });
                if let Some(ct) = connection_tool {
                    if let Some((id, position)) = nearest {
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
                        let connection_exists =
                            world.query::<&Connection>().iter().any(|(_, c)| {
                                c.from == new_connection.from && c.to == new_connection.to
                            });
                        if !connection_exists && ct.from != id {
                            if *tool == Tool::StaticConnection {
                                world.spawn((new_connection, Deletable {}));
                            } else {
                                world.spawn((new_connection, Deletable {}, LearningSynapse {}));
                            };
                        }
                        if !self.keyboard.shift_down {
                            ct.start = position;
                            ct.from = id;
                        }
                    }
                    ct.end = mouse_position;
                } else if let Some((id, position)) = nearest {
                    *connection_tool = Some(ConnectionTool {
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
}

impl visula::Simulation for Simulation {
    type Error = Error;
    fn update(&mut self, application: &visula::Application) {
        let Simulation {
            connection_tool,
            world,
            time,
            stimulation_tool,
            selected_brain,
            ..
        } = self;
        let dt = 0.001;

        for (_, (position, stimulate)) in world.query_mut::<(&Position, &mut Stimulate)>() {
            if let Some(stim) = stimulation_tool {
                let mouse_distance = position.position.distance(stim.position);
                stimulate.injected_current = (2000.0
                    * (-mouse_distance * mouse_distance / (2.0 * SIGMA * SIGMA)).exp())
                    as f64;
            } else {
                stimulate.injected_current = 0.0;
            }
        }

        for _ in 0..self.iterations {
            for (_, (neuron, stimulate)) in world.query_mut::<(&mut Neuron, &Stimulate)>() {
                for dynamics in neuron.dynamics.values_mut() {
                    dynamics.input_current =
                        dynamics.input_current - dynamics.input_current * dt / neuron.input_tau;
                    let leak_current =
                        (neuron.resting_potential - dynamics.voltage) / neuron.leak_tau;
                    let other_currents = if dynamics.refraction <= 0.0 {
                        dynamics.input_current + stimulate.injected_current
                    } else {
                        0.0
                    };
                    let current = leak_current + other_currents;
                    dynamics.voltage = (dynamics.voltage + current * dt).clamp(-200.0, 200.0);
                    if dynamics.refraction <= 0.0 && dynamics.voltage > neuron.threshold {
                        dynamics.fired = true;
                        dynamics.last_fired = Some(*time);
                        dynamics.refraction = 0.2;
                        dynamics.voltage = neuron.initial_voltage;
                        dynamics.voltage = neuron.reset_potential;
                    }
                }
            }
            let new_triggers: Vec<(Entity, Entity, f64)> = world
                .query::<&Connection>()
                .iter()
                .flat_map(|(connection_entity, connection)| {
                    let neuron_from = world
                        .get::<&Neuron>(connection.from)
                        .expect("Connection from does not exist!");
                    let base_current = match &neuron_from.ty {
                        NeuronType::Excitatory => 3000.0,
                        NeuronType::Inhibitory => -3000.0,
                    };
                    let current = connection.strength * base_current;
                    let mut triggers = vec![];
                    for (brain, dynamics) in neuron_from.dynamics.iter() {
                        if dynamics.fired {
                            triggers.push((connection_entity, *brain, current));
                        }
                    }
                    triggers
                })
                .collect();
            for (connection_entity, brain, current) in new_triggers {
                world.spawn((Trigger::new(0.3, current, connection_entity, brain),));
            }

            for (_, trigger) in world.query_mut::<&mut Trigger>() {
                trigger.decrement(dt);
            }

            let triggers_to_delete: Vec<Entity> = world
                .query::<&Trigger>()
                .iter()
                .filter_map(|(entity, trigger)| {
                    if trigger.done() {
                        let connection = world.get::<&Connection>(trigger.connection).unwrap();
                        let mut neuron_to = world.get::<&mut Neuron>(connection.to).unwrap();
                        neuron_to
                            .dynamics
                            .get_mut(&trigger.brain)
                            .unwrap()
                            .input_current += trigger.current;
                        neuron_to
                            .dynamics
                            .get_mut(&trigger.brain)
                            .unwrap()
                            .input_current = neuron_to.dynamics[&trigger.brain]
                            .input_current
                            .clamp(-20000.0, 20000.0);
                        Some(entity)
                    } else {
                        None
                    }
                })
                .collect();
            for entity in triggers_to_delete {
                world.despawn(entity).expect("Could not delete entity!");
            }

            for (_, neuron) in world.query_mut::<&mut Neuron>() {
                for dynamics in neuron.dynamics.values_mut() {
                    dynamics.fired = false;
                    dynamics.refraction -= dt;
                }
            }

            *time += dt;
        }

        let neuron_spheres: Vec<Sphere> = world
            .query::<(&Neuron, &Position)>()
            .iter()
            .map(|(_entity, (neuron, position))| {
                let value = ((neuron.dynamics[selected_brain].voltage + 100.0) / 150.0)
                    .clamp(0.0, 1.0) as f32;
                let color = match neuron.ty {
                    NeuronType::Excitatory => Vec3::new(value / 2.0, value, 0.95),
                    NeuronType::Inhibitory => Vec3::new(0.95, value / 2.0, value),
                };
                Sphere {
                    position: position.position,
                    color,
                    _padding: Default::default(),
                }
            })
            .collect();
        let trigger_spheres: Vec<Sphere> = world
            .query::<&Trigger>()
            .iter()
            .filter_map(|(_entity, trigger)| {
                if trigger.brain != *selected_brain {
                    return None;
                }
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
                    _padding: Default::default(),
                })
            })
            .collect();

        let mut spheres = Vec::new();
        spheres.extend(neuron_spheres.iter());
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

    fn gui(&mut self, context: &egui::Context) {
        egui::Window::new("Settings").show(context, |ui| {
            ui.label("Tool");
            for value in Tool::iter() {
                ui.selectable_value(&mut self.tool, value.clone(), format!("{:?}", &value));
            }
            ui.label("Simulation speed");
            ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
        });
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
                self.keyboard.shift_down = state.shift();
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

fn main() {
    visula::run(Simulation::new); //.expect("Initializing simulation failed"));
}
