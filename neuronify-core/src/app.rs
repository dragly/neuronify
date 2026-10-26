use crate::components::*;
use crate::constants::*;
use crate::dendrites::{dendrite_tree, spawn_dendrites};
use crate::elements::{spawn_cable, spawn_compartment, spawn_synapse, spawn_voltmeter, Element};
use crate::examples::Example;
use crate::landing::{LandingChoice, LandingScreen};
use crate::measurement::voltmeter::{Corner, Frame, VoltageSeries, Voltmeter};
use crate::rendering::lighting::{configure_lighting, set_theme_lighting, shaded};
use crate::rendering::{
    canvas_bounds, collect_connections, collect_scene, collect_voltmeter_traces, compartment_radii,
    connection_paths, create_floor, ConnectionData, CylinderData, GaugeRenderer, NoteRenderer,
    Sphere, Theme, TorusData,
};
use crate::tools::*;
use chrono::{DateTime, Duration, Utc};
use glam::Vec3;
use hecs::Entity;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use visula::{
    winit::keyboard::ModifiersKeyState, CustomEvent, CylinderGeometry, CylinderMaterial, Cylinders,
    Expression, InstanceBuffer, LineGeometry, LineMaterial, Lines, MeshPipeline, RenderData,
    Renderable, ShadowRenderData, SphereGeometry, SphereMaterial, Spheres, Torus, TorusGeometry,
    TorusMaterial,
};

use crate::input::{Keyboard, Mouse, TouchGesture, Touches};
use crate::simulation;

pub struct Neuronify {
    pub tool: Tool,
    pub theme: Theme,
    /// The theme the floor and lighting were last set up for
    applied_theme: Theme,
    pub dendrites: bool,
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
    pub cylinders: Cylinders,
    pub cylinder_buffer: InstanceBuffer<CylinderData>,
    pub rings: Torus,
    pub ring_buffer: InstanceBuffer<TorusData>,
    pub floor: MeshPipeline,
    pub iterations: u32,
    pub last_update: DateTime<Utc>,
    pub fps: f64,
    pub edit_enabled: bool,
    pub touches: Touches,
    pub move_origin: Option<Vec3>,
    pub active_entity: Option<Entity>,
    pub dragging_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub resizing_voltmeter: Option<(Entity, Corner)>,
    pub load_error: Option<String>,
    pub note_renderer: NoteRenderer,
    pub gauge_renderer: GaugeRenderer,
    pub landing: Option<LandingScreen>,
    pub frame_scene_requested: bool,
}

#[derive(Debug)]
pub struct Error {}

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

fn within_selection_range(
    mouse_position: Vec3,
    (id, position): (Entity, &Position),
) -> Option<(Entity, Vec3)> {
    if mouse_position.distance(position.position) < SELECTION_RANGE {
        Some((id, position.position))
    } else {
        None
    }
}

fn ground_position(application: &visula::Application, position: PhysicalPosition<f64>) -> Vec3 {
    let ndc_x = 2.0 * position.x as f32 / application.config.width as f32 - 1.0;
    let ndc_y = 1.0 - 2.0 * position.y as f32 / application.config.height as f32;
    let ray_clip = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let aspect_ratio = application.config.width as f32 / application.config.height as f32;
    let inv_projection = application
        .camera_controller
        .projection_matrix(aspect_ratio)
        .inverse();

    let ray_eye = inv_projection * ray_clip;
    let ray_eye = glam::Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
    let inv_view_matrix = application.camera_controller.view_matrix().inverse();
    let ray_world = inv_view_matrix * ray_eye;
    let ray_world = Vec3::new(ray_world.x, ray_world.y, ray_world.z).normalize();
    let ray_origin = application.camera_controller.position();
    let t = -ray_origin.y / ray_world.y;
    ray_origin + t * ray_world
}

fn is_press_scroll_or_touch(event: &Event<CustomEvent>) -> bool {
    matches!(
        event,
        Event::WindowEvent {
            event: WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::Touch(_),
            ..
        }
    )
}

fn pan(application: &mut visula::Application, offset: Vec3) {
    let camera = &mut application.camera_controller;
    camera.target_transform.center -= offset;
    camera.current_transform.center = camera.target_transform.center;
}

fn gui_takes_press(application: &visula::Application, position: PhysicalPosition<f64>) -> bool {
    let context = application.egui_renderer.state.egui_ctx();
    let point = egui::pos2(position.x as f32, position.y as f32) / context.pixels_per_point();
    let radius = context.global_style().interaction.interact_radius;
    context.memory(|memory| {
        memory.areas().visible_layer_ids().into_iter().any(|layer| {
            layer.order != egui::Order::Background
                && memory
                    .area_rect(layer.id)
                    .is_some_and(|rect| rect.distance_to_pos(point) <= radius)
        })
    })
}

fn pinch(
    application: &mut visula::Application,
    previous: [PhysicalPosition<f64>; 2],
    current: [PhysicalPosition<f64>; 2],
) {
    let spread = |[a, b]: [PhysicalPosition<f64>; 2]| (a.x - b.x).hypot(a.y - b.y);
    let midpoint = |[a, b]: [PhysicalPosition<f64>; 2]| {
        PhysicalPosition::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
    };
    let anchor = ground_position(application, midpoint(previous));
    let camera = &mut application.camera_controller;
    if spread(previous) > 0.0 && spread(current) > 0.0 {
        let zoom = (spread(previous) / spread(current)) as f32;
        camera.current_transform.distance = (camera.current_transform.distance * zoom)
            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
        camera.target_transform.distance = camera.current_transform.distance;
    }
    let under_fingers = ground_position(application, midpoint(current));
    let camera = &mut application.camera_controller;
    camera.current_transform.center += anchor - under_fingers;
    camera.target_transform.center = camera.current_transform.center;
}

impl Neuronify {
    pub fn new(application: &mut visula::Application) -> Neuronify {
        application.camera_controller.enabled = false;
        application.camera_controller.target_transform.center = Vec3::new(0.0, 0.0, 0.0);
        application.camera_controller.target_transform.forward =
            Vec3::new(0.3, -1.0, 0.0).normalize();
        application.camera_controller.target_transform.distance = CAMERA_DEFAULT_DISTANCE;
        application.camera_controller.current_transform =
            application.camera_controller.target_transform.clone();

        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let connection_buffer = InstanceBuffer::<ConnectionData>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let connection = connection_buffer.instance();

        let spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: sphere.position.clone(),
                radius: sphere.radius,
                color: sphere.color,
            },
            &SphereMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let connection_vector = connection.position_b.clone() - connection.position_a.clone();
        let connection_endpoint = connection.position_a.clone() + connection_vector.clone()
            - connection.directional.clone() * connection_vector.clone()
                / connection_vector.clone().length()
                * NODE_RADIUS
                * 2.0;
        let connection_lines = Lines::new(
            &application.rendering_descriptor(),
            &LineGeometry {
                start: connection.position_a.clone(),
                end: connection_endpoint.clone(),
                width: connection.strength.clone() * 0.3,
                color: connection.start_color.clone(),
            },
            &LineMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let connection_spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: connection_endpoint,
                radius: connection.directional.clone() * (0.5 * NODE_RADIUS),
                color: crate::rendering::colors::terminal_color().into(),
            },
            &SphereMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let cylinder_buffer = InstanceBuffer::<CylinderData>::new(&application.device);
        let cylinder = cylinder_buffer.instance();
        let cylinders = Cylinders::new(
            &application.rendering_descriptor(),
            &CylinderGeometry {
                start: cylinder.start,
                end: cylinder.end,
                start_radius: cylinder.start_radius,
                end_radius: cylinder.end_radius,
                color: cylinder.color,
            },
            &CylinderMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let ring_buffer = InstanceBuffer::<TorusData>::new(&application.device);
        let ring = ring_buffer.instance();
        let rings = Torus::new(
            &application.rendering_descriptor(),
            &TorusGeometry {
                position: ring.position,
                major_radius: ring.major_radius,
                minor_radius: ring.minor_radius,
                rotation: ring.rotation,
                color: ring.color,
            },
            &TorusMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let theme = match application.window.theme() {
            Some(visula::winit::window::Theme::Light) => Theme::Light,
            _ => Theme::Dark,
        };
        let floor = create_floor(
            &application.rendering_descriptor(),
            &application.device,
            theme.palette().floor,
        )
        .unwrap();

        configure_lighting(application, theme);

        let mut world = hecs::World::new();
        #[allow(unused_mut)]
        let mut opened_file_from_command_line = false;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let path = &args[1];
                match std::fs::read_to_string(path) {
                    Ok(contents) => match crate::legacy::parse_legacy_nfy(&contents) {
                        Ok(sim) => {
                            log::info!(
                                "Loaded legacy simulation from {}: {} nodes, {} edges",
                                path,
                                sim.nodes.len(),
                                sim.edges.len()
                            );
                            crate::legacy::convert::spawn_legacy_simulation(&mut world, &sim);
                            opened_file_from_command_line = true;
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
            cylinders,
            cylinder_buffer,
            rings,
            ring_buffer,
            floor,
            tool: Tool::Select,
            theme,
            applied_theme: theme,
            dendrites: false,
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
            touches: Touches::default(),
            move_origin: None,
            active_entity: None,
            dragging_entity: None,
            drag_offset: Vec3::ZERO,
            resizing_voltmeter: None,
            load_error: None,
            note_renderer: NoteRenderer::new(application),
            gauge_renderer: GaugeRenderer::new(application),
            landing: (!opened_file_from_command_line).then(|| LandingScreen::new(false)),
            frame_scene_requested: opened_file_from_command_line,
        }
    }

    fn press_pointer(&mut self, application: &mut visula::Application, pressed: bool) {
        self.mouse.left_down = pressed;
        self.mouse.delta_position = None;
        self.handle_tool(application);
    }

    fn move_pointer(
        &mut self,
        application: &mut visula::Application,
        position: PhysicalPosition<f64>,
    ) {
        self.mouse.delta_position = self.mouse.position.map(|previous| {
            PhysicalPosition::new(position.x - previous.x, position.y - previous.y)
        });
        self.mouse.position = Some(position);
        self.handle_tool(application);
    }

    fn release_tools(&mut self) {
        self.stimulation_tool = None;
        self.connection_tool = None;
        self.previous_creation = None;
        self.move_origin = None;
        self.dragging_entity = None;
        self.resizing_voltmeter = None;
    }

    fn sync_pointer_with_gui(&mut self, context: &egui::Context) {
        let held = context.input(|input| input.pointer.any_down() || input.any_touches());
        if !held {
            self.touches = Touches::default();
            self.mouse.left_down = false;
            self.release_tools();
        }
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            self.release_tools();
            return;
        }
        let palette = self.theme.palette();
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
        let mouse_physical_position = match mouse.position {
            Some(p) => p,
            None => {
                return;
            }
        };
        let mouse_position = ground_position(application, mouse_physical_position);

        if !self.edit_enabled {
            let touching = stimulation_tool.is_some()
                || (move_origin.is_none() && simulation::touch_sensor_near(world, mouse_position));
            if touching {
                *stimulation_tool = Some(StimulationTool {
                    position: mouse_position,
                });
            } else {
                match *move_origin {
                    Some(origin) => pan(application, mouse_position - origin),
                    None => *move_origin = Some(mouse_position),
                }
            }
            return;
        }

        let minimum_distance = match tool {
            Tool::Axon => MIN_CREATION_DISTANCE_AXON,
            _ => MIN_CREATION_DISTANCE_DEFAULT,
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
            Tool::Place(element) => {
                if previous_too_near {
                    return;
                }
                let element = *element;
                let entity = element.spawn(world, mouse_position);
                if element.is_neuron() && self.dendrites {
                    spawn_dendrites(world, entity);
                }
                if element == Element::Note {
                    *active_entity = Some(entity);
                }
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::StaticConnection => {
                if let Some(ct) = connection_tool {
                    let target_candidates: Vec<(Entity, Vec3)> = world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
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
                            spawn_synapse(world, ct.from, id);
                        }
                        if !self.keyboard.shift_down {
                            ct.start = position;
                            ct.from = id;
                        }
                    }
                    ct.end = mouse_position;
                } else {
                    let source_candidates: Vec<(Entity, Vec3)> = {
                        let mut candidates: Vec<_> = world
                            .query::<&Position>()
                            .with::<&LeakyNeuron>()
                            .iter()
                            .map(|(e, p)| (e, p.position))
                            .collect();
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&CurrentClamp>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&GeneratorDynamics>()
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
                let to_delete: HashSet<Entity> = to_delete
                    .iter()
                    .flat_map(|&entity| dendrite_tree(world, entity))
                    .chain(to_delete.iter().copied())
                    .collect();
                for entity in to_delete {
                    world.despawn(entity).unwrap();
                }
                // Erase what is drawn, which for most connections is a bent path
                let paths = connection_paths(world, &compartment_radii(world), &palette);
                let connections_to_delete = world
                    .query::<&Connection>()
                    .with::<&Deletable>()
                    .iter()
                    .filter_map(|(entity, connection)| {
                        if let (Ok(from), Ok(to)) = (
                            world.get::<&Position>(connection.from),
                            world.get::<&Position>(connection.to),
                        ) {
                            let near = if let Some(path) = paths.get(&entity) {
                                path.distance_to(mouse_position)
                                    .is_some_and(|distance| distance < ERASE_RADIUS)
                            } else {
                                let a = from.position;
                                let b = to.position;
                                let p = mouse_position;
                                let ab = b - a;
                                let ap = p - a;
                                let t = ap.dot(ab) / ab.dot(ab);
                                let d = t * ab;
                                let point_on_line = a + d;
                                let distance_from_line = p.distance(point_on_line);
                                distance_from_line < ERASE_RADIUS && (0.0..=1.0).contains(&t)
                            };
                            near.then_some(entity)
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
                let result: Option<(Entity, Vec3)> = world
                    .query::<&Position>()
                    .with::<&LeakyNeuron>()
                    .iter()
                    .filter_map(|(entity, position)| {
                        let distance = position.position.distance(mouse_position);
                        if distance < NODE_RADIUS {
                            Some((entity, position.position))
                        } else {
                            None
                        }
                    })
                    .next()
                    .or_else(|| {
                        world
                            .query::<&Position>()
                            .with::<&Compartment>()
                            .iter()
                            .filter_map(|(entity, position)| {
                                let distance = position.position.distance(mouse_position);
                                if distance < NODE_RADIUS {
                                    Some((entity, position.position))
                                } else {
                                    None
                                }
                            })
                            .next()
                    });
                let Some((target, position)) = result else {
                    return;
                };
                let voltmeter = spawn_voltmeter(world, target, position + Vec3::X);
                *previous_creation = Some(PreviousCreation { entity: voltmeter });
            }
            Tool::Select => match mouse.left_down {
                true => {
                    if let Some(entity) = *dragging_entity {
                        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                            pos.position = mouse_position + *drag_offset;
                            pos.position.y = 0.0;
                        }
                    } else if let Some((entity, corner)) = *resizing_voltmeter {
                        if let Some(frame) = Frame::of(world, entity) {
                            let resized = frame.resized(corner, mouse_position);
                            if let Ok(mut size) = world.get::<&mut VoltmeterSize>(entity) {
                                *size = resized.size();
                            }
                            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                                pos.position = resized.position();
                            }
                        }
                    } else {
                        match *move_origin {
                            Some(origin) => pan(application, mouse_position - origin),
                            None => {
                                let frames: Vec<_> = world
                                    .query::<(&Position, &VoltmeterSize)>()
                                    .with::<&Voltmeter>()
                                    .iter()
                                    .map(|(voltmeter, (position, size))| {
                                        (
                                            voltmeter,
                                            position.position,
                                            Frame::new(position.position, size),
                                        )
                                    })
                                    .collect();
                                *resizing_voltmeter =
                                    frames.iter().find_map(|(voltmeter, _, frame)| {
                                        Some((*voltmeter, frame.corner_near(mouse_position, 1.0)?))
                                    });

                                if resizing_voltmeter.is_none() {
                                    let mut found_voltmeter = false;
                                    for (voltmeter, position, frame) in &frames {
                                        if frame.contains(mouse_position) {
                                            *active_entity = Some(*voltmeter);
                                            *dragging_entity = Some(*voltmeter);
                                            *drag_offset = *position - mouse_position;
                                            drag_offset.y = 0.0;
                                            found_voltmeter = true;
                                            break;
                                        }
                                    }

                                    if !found_voltmeter {
                                        // Notes lie under the circuit,
                                        // so anything near the pointer wins over the note around it
                                        let nearby = world
                                            .query::<&Position>()
                                            .without::<&Annotation>()
                                            .iter()
                                            .min_by(|a, b| nearest(&mouse_position, a, b))
                                            .and_then(|v| {
                                                within_selection_range(mouse_position, v)
                                            });
                                        let note = || {
                                            world
                                                .query::<(&Position, &NoteSize)>()
                                                .with::<&Annotation>()
                                                .iter()
                                                .filter(|(_, (position, size))| {
                                                    let offset = mouse_position - position.position;
                                                    offset.x.abs() <= size.height * 0.5
                                                        && offset.z.abs() <= size.width * 0.5
                                                })
                                                // The renderer draws later notes on top
                                                .max_by_key(|(entity, _)| entity.id())
                                                .map(|(entity, (position, _))| {
                                                    (entity, position.position)
                                                })
                                        };
                                        if let Some((entity, entity_pos)) = nearby.or_else(note) {
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
                                .with::<&LeakyNeuron>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&CurrentClamp>()
                                .iter()
                                .map(|(e, p)| (e, p.position)),
                        );
                        candidates.extend(
                            world
                                .query::<&Position>()
                                .with::<&GeneratorDynamics>()
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
                    let target_candidates: Vec<(Entity, Vec3)> = world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
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
                                spawn_cable(world, ct.from, id, true);
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
                            let neuron_type = world.get::<&NeuronType>(ct.from).ok().map(|t| *t);
                            if let Some(neuron_type) = neuron_type {
                                let compartment =
                                    spawn_compartment(world, mouse_position, neuron_type);
                                spawn_cable(world, ct.from, compartment, false);
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
        let bytes = crate::serialization::save(&self.world).unwrap();
        let mut writer = std::fs::File::create(path.with_extension("neuronify")).unwrap();
        writer.write_all(&bytes).unwrap();
    }

    pub fn load_example(&mut self, example: &Example) {
        match example.spawn() {
            Ok(world) => self.open_world(world, 0.0),
            Err(e) => log::error!("Failed to load example {}: {}", example.name, e),
        }
    }

    pub fn open_world(&mut self, world: hecs::World, time: f64) {
        self.world = world;
        self.time = time;
        self.release_tools();
        self.active_entity = None;
        self.landing = None;
        self.frame_scene_requested = true;
    }

    fn frame_scene(&self, application: &mut visula::Application) {
        let camera = &mut application.camera_controller;
        let (center, distance) = match canvas_bounds(&self.world) {
            Some(bounds) => {
                let height_on_screen = bounds.size().x;
                let width_on_screen = bounds.size().z;
                let aspect_ratio =
                    application.config.width as f32 / application.config.height.max(1) as f32;
                let tangent = (CAMERA_FIELD_OF_VIEW_DEGREES.to_radians() / 2.0).tan();
                let fit = (height_on_screen / tangent)
                    .max(width_on_screen / (tangent * aspect_ratio))
                    / 2.0
                    * CAMERA_FRAMING_MARGIN;
                let center = bounds.center();
                (Vec3::new(center.x, 0.0, center.z), fit)
            }
            None => (Vec3::ZERO, CAMERA_DEFAULT_DISTANCE),
        };
        camera.target_transform.center = center;
        camera.target_transform.distance =
            distance.clamp(CAMERA_DEFAULT_DISTANCE, CAMERA_MAX_DISTANCE);
        camera.current_transform = camera.target_transform.clone();
    }

    pub fn loadfile(&mut self, path: PathBuf) {
        let reader = std::fs::File::open(path).unwrap();
        let mut bufreader = BufReader::new(reader);
        let mut bytes: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut bytes).unwrap();
        if let Err(error) = self.load_from_bytes(&bytes) {
            log::error!("{error}");
            self.load_error = Some(error);
        }
    }

    /// Replaces the world with a legacy `.nfy` file or a `.neuronify` save file.
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let world = crate::serialization::load_from_bytes(bytes)?;
        self.open_world(world, 0.0);
        Ok(())
    }
}

impl visula::Simulation for Neuronify {
    fn clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: crate::rendering::srgb_component(30) as f64,
            g: crate::rendering::srgb_component(30) as f64,
            b: crate::rendering::srgb_component(46) as f64,
            a: 1.0,
        }
    }
    fn update(&mut self, application: &mut visula::Application) {
        #[cfg(target_arch = "wasm32")]
        if let Some(fetched) = crate::web::take_fetched_file() {
            if let Err(error) = fetched.and_then(|bytes| self.load_from_bytes(&bytes)) {
                log::error!("{error}");
                self.load_error = Some(error);
            }
        }

        if self.applied_theme != self.theme {
            self.floor = create_floor(
                &application.rendering_descriptor(),
                &application.device,
                self.theme.palette().floor,
            )
            .unwrap();
            set_theme_lighting(application, self.theme);
            self.applied_theme = self.theme;
        }

        simulation::stimulate_nearby(&mut self.world, &self.stimulation_tool);
        simulation::step_frame(&mut self.world, &mut self.time, self.iterations);

        if let Some(landing) = &mut self.landing {
            landing.update(self.iterations);
        }
        if self.frame_scene_requested {
            self.frame_scene_requested = false;
            self.frame_scene(application);
        }

        let camera = &application.camera_controller.current_transform;
        application.light.shadow_center = camera.center;
        // visula blurs shadows over a fixed number of shadow map texels,
        // so a map that reaches well past the view gives softer shadows
        application.light.shadow_extent = (camera.distance * 2.0).clamp(50.0, 625.0);

        let world = &self.world;
        let palette = self.theme.palette();
        let scene = collect_scene(world, &palette);
        let mut connections =
            collect_connections(world, &self.tool, &self.connection_tool, &palette);
        connections.extend(collect_voltmeter_traces(world));

        self.sphere_buffer
            .update(&application.device, &application.queue, &scene.spheres);

        self.cylinder_buffer
            .update(&application.device, &application.queue, &scene.cylinders);

        self.ring_buffer
            .update(&application.device, &application.queue, &scene.rings);

        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

        self.note_renderer.update(application, &self.world);
        self.gauge_renderer
            .update(application, &self.world, &palette);

        let time_diff = Utc::now() - self.last_update;
        #[cfg(not(target_arch = "wasm32"))]
        if time_diff < Duration::milliseconds(TARGET_FRAME_MS) {
            thread::sleep(std::time::Duration::from_millis(
                (Duration::milliseconds(TARGET_FRAME_MS) - time_diff).num_milliseconds() as u64,
            ))
        }
        let low_pass_factor = FPS_LOW_PASS_FACTOR;
        let new_fps = 1.0
            / ((Utc::now() - self.last_update).num_nanoseconds().unwrap() as f64 * 1e-9)
                .max(0.0000001);
        self.fps = (1.0 - low_pass_factor) * self.fps + low_pass_factor * new_fps;
        self.last_update = Utc::now();
    }

    fn render(&mut self, data: &mut RenderData) {
        self.floor.render(data);
        self.spheres.render(data);
        self.cylinders.render(data);
        self.rings.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
        self.note_renderer.render(data);
        self.gauge_renderer.render(data);
    }

    fn render_shadow(&mut self, data: &mut ShadowRenderData) {
        self.spheres.render_shadow(data);
        self.cylinders.render_shadow(data);
        self.rings.render_shadow(data);
    }

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        context.set_theme(match self.theme {
            Theme::Dark => egui::Theme::Dark,
            Theme::Light => egui::Theme::Light,
        });
        self.sync_pointer_with_gui(context);
        if let Some(landing) = &mut self.landing {
            match landing.show(context, self.theme) {
                Some(LandingChoice::Example { world, time }) => self.open_world(*world, time),
                Some(LandingChoice::EmptyCanvas) => self.open_world(hecs::World::new(), 0.0),
                Some(LandingChoice::BackToSimulation) => self.landing = None,
                None => {}
            }
            return;
        }
        if let Some(error) = &self.load_error {
            egui::Window::new("Could not load file").show(context, |ui| {
                ui.label(error);
            });
        }
        egui::Area::new("edit_button_area".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Examples").clicked() {
                        self.landing = Some(LandingScreen::new(true));
                    }
                    ui.toggle_value(&mut self.edit_enabled, "Edit");
                });
            });
        if self.edit_enabled {
            // show_inside needs a &mut Ui, which Simulation::gui does not hand us
            #[cfg(not(target_arch = "wasm32"))]
            #[allow(deprecated)]
            egui::Panel::top("top_panel").show(context, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
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
                        for category in crate::examples::EXAMPLE_CATEGORIES {
                            ui.menu_button(category.name, |ui| {
                                for example in category.examples {
                                    if ui.button(example.name).clicked() {
                                        self.load_example(example);
                                        ui.close();
                                    }
                                }
                            });
                        }
                    });
                });
            });
            egui::Window::new("Elements").show(context, |ui| {
                for category in &TOOL_CATEGORIES {
                    ui.collapsing(category.label(), |ui| {
                        for (tool_value, label) in category.tools() {
                            ui.selectable_value(&mut self.tool, tool_value, label);
                        }
                        if *category == ToolCategory::Neurons {
                            ui.checkbox(&mut self.dendrites, "With dendrites");
                        }
                    });
                }
            });
            egui::Window::new("Settings").show(context, |ui| {
                ui.label(format!("FPS: {:.0}", self.fps));
                ui.label("Simulation speed");
                ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
                ui.horizontal(|ui| {
                    ui.label("Theme");
                    ui.selectable_value(&mut self.theme, Theme::Dark, "Dark");
                    ui.selectable_value(&mut self.theme, Theme::Light, "Light");
                });
            });
            if let Some(active_entity) = self.active_entity {
                egui::Window::new("Selection").show(context, |ui| {
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
                    if let Ok(mut neuron) = self.world.get::<&mut LeakyNeuron>(active_entity) {
                        let is_inhibitory =
                            neuron_type(&self.world, active_entity) == NeuronType::Inhibitory;
                        let label = if is_inhibitory {
                            "LIF Neuron (Inhibitory)"
                        } else {
                            "LIF Neuron (Excitatory)"
                        };
                        ui.collapsing(label, |ui| {
                            egui::Grid::new("neuron_settings").show(ui, |ui| {
                                ui.label("Threshold:");
                                ui.add(
                                    egui::Slider::new(&mut neuron.threshold, -0.08..=-0.03)
                                        .suffix(" V"),
                                );
                                ui.end_row();
                                ui.label("Resting potential:");
                                ui.add(
                                    egui::Slider::new(&mut neuron.resting_potential, -0.09..=-0.05)
                                        .suffix(" V"),
                                );
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
                    if let Ok(dynamics) = self.world.get::<&LeakyDynamics>(active_entity) {
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
                    if let Ok(mut clamp) = self.world.get::<&mut CurrentClamp>(active_entity) {
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
                    if let Ok(mut gen) = self.world.get::<&mut RegularSpikeGenerator>(active_entity)
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
                    if let Ok(mut gen) = self.world.get::<&mut PoissonGenerator>(active_entity) {
                        ui.collapsing("Poisson Generator", |ui| {
                            egui::Grid::new("poisson_gen_settings").show(ui, |ui| {
                                ui.label("Rate:");
                                ui.add(egui::Slider::new(&mut gen.rate, 1.0..=200.0).suffix(" Hz"));
                                ui.end_row();
                            });
                        });
                    }
                    if self.world.get::<&Voltmeter>(active_entity).is_ok() {
                        ui.collapsing("Voltmeter", |ui| {
                            if let Ok(mut size) =
                                self.world.get::<&mut VoltmeterSize>(active_entity)
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
                            if let Ok(series) = self.world.get::<&VoltageSeries>(active_entity) {
                                if let Some(last) = series.measurements.last() {
                                    ui.label(format!("Voltage: {:.2} mV", last.voltage));
                                }
                            }
                        });
                    }
                    if self.world.get::<&Annotation>(active_entity).is_ok() {
                        ui.collapsing("Note", |ui| {
                            if let Ok(mut annotation) =
                                self.world.get::<&mut Annotation>(active_entity)
                            {
                                ui.label("Text:");
                                ui.add(
                                    egui::TextEdit::multiline(&mut annotation.text)
                                        .desired_rows(4)
                                        .desired_width(f32::INFINITY),
                                );
                            }
                            if let Ok(mut size) = self.world.get::<&mut NoteSize>(active_entity) {
                                egui::Grid::new("note_size_settings").show(ui, |ui| {
                                    ui.label("Width:");
                                    ui.add(egui::Slider::new(&mut size.width, 2.0..=30.0));
                                    ui.end_row();
                                    ui.label("Height:");
                                    ui.add(egui::Slider::new(&mut size.height, 1.0..=20.0));
                                    ui.end_row();
                                });
                            }
                        });
                    }
                });
            }
        }
    }

    fn handle_event(&mut self, application: &mut visula::Application, event: &Event<CustomEvent>) {
        if self.landing.is_some() && is_press_scroll_or_touch(event) {
            return;
        }
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
                self.press_pointer(application, *state == ElementState::Pressed);
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
                self.move_pointer(application, *position);
            }
            Event::WindowEvent {
                event: WindowEvent::Touch(touch),
                ..
            } => {
                if touch.phase == TouchPhase::Started
                    && gui_takes_press(application, touch.location)
                {
                    return;
                }
                match self.touches.update(touch) {
                    TouchGesture::Press(position) => {
                        self.move_pointer(application, position);
                        self.press_pointer(application, true);
                    }
                    TouchGesture::Drag(position) => self.move_pointer(application, position),
                    TouchGesture::Release => self.press_pointer(application, false),
                    TouchGesture::Pinch { previous, current } => {
                        pinch(application, previous, current)
                    }
                    TouchGesture::None => {}
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                application.camera_controller.target_transform.distance *= 1.0 - scroll * 0.1;
                application.camera_controller.target_transform.distance = application
                    .camera_controller
                    .target_transform
                    .distance
                    .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
            }
            _ => {}
        }
    }
}
