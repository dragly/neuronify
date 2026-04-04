use crate::components::*;
use crate::constants::*;
use crate::measurement::voltmeter::{RollingWindow, VoltageSeries, Voltmeter};
use crate::rendering::{
    collect_connections, collect_placement_preview, collect_spheres, collect_voltmeter_traces,
    ConnectionData, Sphere,
};
use crate::serialization::{LoadContext, SaveContext};
use crate::tools::*;
use chrono::{DateTime, Duration, Utc};
use glam::Vec3;
use hecs::serialize::column::*;
use hecs::Entity;
use postcard::ser_flavors::Flavor;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use visula::{
    winit::keyboard::ModifiersKeyState, CustomEvent, Expression, InstanceBuffer, LineGeometry,
    LineMaterial, Lines, RenderData, Renderable, SphereGeometry, SphereMaterial, Spheres,
};

use crate::input::{Keyboard, Mouse};
use crate::rendering::{
    collect_petri_dish, collect_resource_node_rings, collect_substrate_zone_rings,
};
use crate::simulation;
use crate::simulation::ai::AiState;
use crate::simulation::game::{self, PetriDish};

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
    pub petri_dish: Option<PetriDish>,
    pub current_player: PlayerId,
    pub ai_state: Option<AiState>,
    pub pending_new_game: bool,
    pub placement_preview: Option<Vec3>,
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

impl Neuronify {
    pub fn new(application: &mut visula::Application) -> Neuronify {
        application.camera_controller.enabled = false;
        application.camera_controller.target_transform.center = Vec3::new(0.0, 0.0, 0.0);
        application.camera_controller.target_transform.forward =
            Vec3::new(0.3, -1.0, 0.0).normalize();
        application.camera_controller.target_transform.distance = 50.0;
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
                color: Expression::InputColor.lit(),
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
                color: Expression::InputColor.lit(),
            },
        )
        .unwrap();

        let connection_spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: connection_endpoint,
                radius: connection.directional.clone() * (0.5 * NODE_RADIUS),
                color: Vec3::new(136.0 / 255.0, 57.0 / 255.0, 239.0 / 255.0).into(),
            },
            &SphereMaterial {
                color: Expression::InputColor.lit(),
            },
        )
        .unwrap();

        let mut world = hecs::World::new();

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
            keyboard: Keyboard { shift_down: false, ctrl_down: false },
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
            petri_dish: None,
            current_player: PlayerId::Player1,
            ai_state: None,
            pending_new_game: false,
            placement_preview: None,
        }
    }

    pub fn start_game(&mut self, application: &mut visula::Application) {
        let dish = PetriDish {
            center: Vec3::ZERO,
            radius: PETRI_DISH_RADIUS,
        };
        game::setup_game(&mut self.world, &dish);
        self.petri_dish = Some(dish);
        self.current_player = PlayerId::Player1;
        self.ai_state = Some(AiState::new());
        self.tool = Tool::Select;
        self.active_entity = None;
        // Zoom out to see the larger dish
        application.camera_controller.target_transform.distance = 150.0;
    }

    fn update_placement_preview(&mut self, application: &mut visula::Application) {
        let mouse_position = match self.mouse.position {
            Some(p) => {
                let ndc_x = 2.0 * p.x as f32 / application.config.width as f32 - 1.0;
                let ndc_y = 1.0 - 2.0 * p.y as f32 / application.config.height as f32;
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
            None => return,
        };
        self.placement_preview = Some(mouse_position);
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
            placement_preview: _,
            ..
        } = self;
        if !mouse.left_down {
            *stimulation_tool = None;
            *connection_tool = None;
            *previous_creation = None;
            *move_origin = None;
            *dragging_entity = None;
            *resizing_voltmeter = None;
            // Keep placement_preview for hover effect
            return;
        }
        let mouse_physical_position = match mouse.position {
            Some(p) => p,
            None => {
                return;
            }
        };
        let ndc_x = 2.0 * mouse_physical_position.x as f32 / application.config.width as f32 - 1.0;
        let ndc_y = 1.0 - 2.0 * mouse_physical_position.y as f32 / application.config.height as f32;
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
        let intersection = ray_origin + t * ray_world;
        let mouse_position = intersection;

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
            Tool::ExcitatoryNeuron | Tool::InhibitoryNeuron => {
                if previous_too_near {
                    return;
                }
                // Game mode: check proximity and cost
                if self.petri_dish.is_some() {
                    if !game::is_within_build_range(world, mouse_position, self.current_player) {
                        return;
                    }
                    if !game::try_spend_energy(
                        world,
                        mouse_position,
                        self.current_player,
                        NEURON_SPAWN_COST,
                    ) {
                        return;
                    }
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
                    LeakyNeuron::default(),
                    LeakyDynamics::default(),
                    LeakCurrent::default(),
                    neuron_type,
                    Deletable {},
                ));
                if self.tool == Tool::InhibitoryNeuron {
                    world.insert_one(entity, Inhibitory).unwrap();
                }
                if self.petri_dish.is_some() {
                    let _ = world.insert(
                        entity,
                        (
                            MetabolicState::default(),
                            SpatialDynamics {
                                velocity: Vec3::ZERO,
                                acceleration: Vec3::ZERO,
                            },
                            Ownership {
                                player: self.current_player,
                            },
                            DepolarizationBlock {
                                time_above_threshold: 0.0,
                                blocked: false,
                                recovery_timer: 0.0,
                            },
                        ),
                    );
                }
                self.previous_creation = Some(PreviousCreation { entity });
            }
            Tool::MembraneSegment => {
                if previous_too_near {
                    return;
                }
                if self.petri_dish.is_some() {
                    if !game::is_within_build_range(world, mouse_position, self.current_player) {
                        return;
                    }
                    if !game::try_spend_energy(
                        world,
                        mouse_position,
                        self.current_player,
                        MEMBRANE_SPAWN_COST,
                    ) {
                        return;
                    }
                }
                let entity = world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    MembraneSegment,
                    SpatialDynamics {
                        velocity: Vec3::ZERO,
                        acceleration: Vec3::ZERO,
                    },
                    Ownership {
                        player: self.current_player,
                    },
                    Deletable {},
                ));
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
                    CurrentClamp::default(),
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
                    TouchSensor,
                    GeneratorDynamics::default(),
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
                    RegularSpikeGenerator::default(),
                    GeneratorDynamics::default(),
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
                    PoissonGenerator::default(),
                    GeneratorDynamics::default(),
                    Deletable {},
                ));
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
                            world.spawn((new_connection, CurrentSynapse::default(), Deletable {}));
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
                    VoltmeterSize::default(),
                    Deletable {},
                ));
                if let Ok(mut conn) = world.get::<&mut Connection>(voltmeter) {
                    conn.to = voltmeter;
                }
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
                        let current = world
                            .get::<&Position>(entity)
                            .ok()
                            .map(|p| p.position)
                            .and_then(|vpos| {
                                world
                                    .get::<&VoltmeterSize>(entity)
                                    .ok()
                                    .map(|s| (vpos, s.width, s.height))
                            });
                        if let Some((vpos, w, h)) = current {
                            let bl = vpos + Vec3::new(-h * 0.5, 0.0, 0.0);
                            let anchor = match corner {
                                ResizeCorner::TopLeft => bl + Vec3::new(0.0, 0.0, w),
                                ResizeCorner::TopRight => bl,
                                ResizeCorner::BottomLeft => bl + Vec3::new(h, 0.0, w),
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
                                ResizeCorner::TopLeft => {
                                    Vec3::new(anchor.x, 0.0, mouse_position.z.min(anchor.z - 2.0))
                                }
                                ResizeCorner::TopRight => anchor,
                                ResizeCorner::BottomLeft => Vec3::new(
                                    mouse_position.x.min(anchor.x - 1.0),
                                    0.0,
                                    mouse_position.z.min(anchor.z - 2.0),
                                ),
                                ResizeCorner::BottomRight => {
                                    Vec3::new(mouse_position.x.min(anchor.x - 1.0), 0.0, anchor.z)
                                }
                            };
                            let new_pos = new_bl + Vec3::new(new_height * 0.5, 0.0, 0.0);
                            if let Ok(mut size) = world.get::<&mut VoltmeterSize>(entity) {
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
                                application.camera_controller.target_transform.center -=
                                    Vec3::new(center.x, center.y, center.z);
                                application.camera_controller.current_transform.center =
                                    application.camera_controller.target_transform.center;
                            }
                            None => {
                                let voltmeter_bounds: Vec<_> = world
                                    .query::<(&Voltmeter, &Position)>()
                                    .iter()
                                    .filter_map(|(vid, (_, pos))| {
                                        world.get::<&VoltmeterSize>(vid).ok().map(|size| {
                                            (vid, pos.position, size.width, size.height)
                                        })
                                    })
                                    .collect();

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
                                            .and_then(|v| within_selection_range(mouse_position, v))
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
                                world.spawn((
                                    new_connection,
                                    Deletable {},
                                    CompartmentCurrent {
                                        capacitance: COUPLING_CAPACITANCE,
                                    },
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
                            // Game mode: check cost for compartment
                            if self.petri_dish.is_some()
                                && !game::try_spend_energy(
                                    world,
                                    mouse_position,
                                    self.current_player,
                                    COMPARTMENT_SPAWN_COST,
                                )
                            {
                                return;
                            }
                            // Game mode: check membrane blocking
                            if self.petri_dish.is_some() {
                                let from_pos = world
                                    .get::<&Position>(ct.from)
                                    .map(|p| p.position)
                                    .unwrap_or(Vec3::ZERO);
                                if game::membrane_blocks_path(world, from_pos, mouse_position) {
                                    return;
                                }
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
                                        voltage: -10.0,
                                        m: -0.625,
                                        h: 0.0,
                                        n: 0.0,
                                        influence: 0.0,
                                        capacitance: 1.0,
                                        injected_current: 0.0,
                                        fire_impulse: 0.0,
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
                                    CompartmentCurrent {
                                        capacitance: COUPLING_CAPACITANCE,
                                    },
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
        match crate::legacy::parse_legacy_nfy(contents) {
            Ok(sim) => {
                self.world.clear();
                self.time = 0.0;
                crate::legacy::convert::spawn_legacy_simulation(&mut self.world, &sim);
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

impl visula::Simulation for Neuronify {
    type Error = Error;
    fn clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: crate::rendering::srgb_component(30) as f64,
            g: crate::rendering::srgb_component(30) as f64,
            b: crate::rendering::srgb_component(46) as f64,
            a: 1.0,
        }
    }
    fn update(&mut self, application: &mut visula::Application) {
        if self.pending_new_game {
            self.pending_new_game = false;
            self.start_game(application);
        }

        let Neuronify {
            connection_tool,
            world,
            time,
            stimulation_tool,
            ..
        } = self;

        simulation::stimulate_nearby(world, stimulation_tool);

        let lif_dt = LIF_DT;
        for _ in 0..self.iterations {
            simulation::lif_step(world, lif_dt, *time);
            *time += lif_dt;
        }

        let recently_fired: HashSet<Entity> = world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < self.iterations as f64 * lif_dt)
            .map(|(e, _)| e)
            .collect();

        for _ in 0..self.iterations {
            simulation::fhn_step(world, FHN_CDT, &recently_fired);
            simulation::apply_spatial_forces(world);
            simulation::integrate_motion(world, PHYSICS_DT);
        }

        // Game systems (once per frame)
        if let Some(ref dish) = self.petri_dish {
            let frame_dt = self.iterations as f64 * LIF_DT;
            game::enforce_petri_boundary(world, dish);
            game::harvest_resources(world, frame_dt);
            game::metabolic_drain(world, frame_dt);
            game::resource_flow(world, frame_dt);
            game::depolarization_block(world, frame_dt);
            game::apply_substrate_zones(world, frame_dt);
            game::check_starvation(world);
            game::cleanup_dead(world);
            game::update_ownership(world);

            // AI opponent
            if let Some(ref mut ai) = self.ai_state {
                simulation::ai::ai_tick(world, ai, dish);
            }
        }

        let mut spheres = collect_spheres(world);
        let placement_spheres = collect_placement_preview(&self.tool, &self.placement_preview);
        spheres.extend(placement_spheres.iter());
        
        let mut connections = collect_connections(world, &self.tool, connection_tool);
        connections.extend(collect_voltmeter_traces(world));
        if let Some(ref dish) = self.petri_dish {
            connections.extend(collect_petri_dish(dish));
            connections.extend(collect_resource_node_rings(world));
            connections.extend(collect_substrate_zone_rings(world));
        }

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);

        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

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
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
    }

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        let in_game = self.petri_dish.is_some();

        // RTS-style bottom command panel (game mode)
        if in_game {
            // Gather stats
            let mut p1_neurons = 0u32;
            let mut p1_energy = 0.0f64;
            let mut p2_neurons = 0u32;
            let mut p2_energy = 0.0f64;
            for (_, (ownership, metab)) in
                self.world.query::<(&Ownership, &MetabolicState)>().iter()
            {
                match ownership.player {
                    PlayerId::Player1 => {
                        p1_neurons += 1;
                        p1_energy += metab.energy;
                    }
                    PlayerId::Player2 => {
                        p2_neurons += 1;
                        p2_energy += metab.energy;
                    }
                }
            }

            egui::TopBottomPanel::bottom("game_command_panel")
                .min_height(80.0)
                .show(context, |ui| {
                    ui.horizontal(|ui| {
                        // Left: Status panel
                        ui.vertical(|ui| {
                            ui.set_min_width(180.0);
                            ui.colored_label(
                                egui::Color32::from_rgb(64, 160, 43),
                                egui::RichText::new(format!(
                                    "YOU: {} neurons | {:.0} energy",
                                    p1_neurons, p1_energy
                                ))
                                .strong(),
                            );
                            ui.colored_label(
                                egui::Color32::from_rgb(136, 57, 239),
                                format!("AI:  {} neurons | {:.0} energy", p2_neurons, p2_energy),
                            );
                        });

                        ui.separator();

                        // Center: Build tools as buttons (RTS-style)
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("BUILD")
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.horizontal(|ui| {
                                let game_tools: Vec<(Tool, &str, &str)> = vec![
                                    (Tool::ExcitatoryNeuron, "Excitatory", "25 energy"),
                                    (Tool::InhibitoryNeuron, "Inhibitory", "25 energy"),
                                    (Tool::Axon, "Axon", "3/seg"),
                                    (Tool::MembraneSegment, "Membrane", "15 energy"),
                                ];
                                for (tool, label, cost) in game_tools {
                                    let selected = self.tool == tool;
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("{}\n{}", label, cost)).small(),
                                    )
                                    .min_size(egui::vec2(72.0, 50.0))
                                    .selected(selected);
                                    if ui.add(btn).clicked() {
                                        self.tool = tool;
                                    }
                                }
                            });
                        });

                        ui.separator();

                        // Right: Action tools
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("ACTIONS")
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.horizontal(|ui| {
                                let action_tools: Vec<(Tool, &str)> = vec![
                                    (Tool::Select, "Select"),
                                    (Tool::Stimulate, "Stimulate"),
                                    (Tool::Erase, "Erase"),
                                ];
                                for (tool, label) in action_tools {
                                    let selected = self.tool == tool;
                                    let btn = egui::Button::new(egui::RichText::new(label).small())
                                        .min_size(egui::vec2(60.0, 50.0))
                                        .selected(selected);
                                    if ui.add(btn).clicked() {
                                        self.tool = tool;
                                    }
                                }
                            });
                        });

                        ui.separator();

                        // Far right: Game controls
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("GAME")
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new("New Game")
                                            .min_size(egui::vec2(60.0, 30.0)),
                                    )
                                    .clicked()
                                {
                                    self.pending_new_game = true;
                                }
                                if ui
                                    .add(egui::Button::new("Exit").min_size(egui::vec2(50.0, 30.0)))
                                    .clicked()
                                {
                                    self.petri_dish = None;
                                    self.world.clear();
                                }
                            });
                        });
                    });
                });
        }

        // Edit button (only in sandbox mode, not game mode)
        if !in_game {
            egui::Area::new("edit_button_area".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .show(context, |ui| {
                    ui.toggle_value(&mut self.edit_enabled, "Edit").clicked();
                });
        }
        if self.edit_enabled || in_game {
            #[cfg(not(target_arch = "wasm32"))]
            egui::TopBottomPanel::top("top_panel").show(context, |ui| {
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
                        ui.menu_button("Tutorial", |ui| {
                            if ui.button("1 - Intro").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/tutorial_1_intro.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("2 - Circuits").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/tutorial_2_circuits.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("3 - Creation").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/tutorial_3_creation.nfy"
                                ));
                                ui.close();
                            }
                        });
                        ui.menu_button("Neurons", |ui| {
                            if ui.button("Leaky").clicked() {
                                self.load_legacy_string(include_str!("../examples/leaky.nfy"));
                                ui.close();
                            }
                            if ui.button("Inhibitory").clicked() {
                                self.load_legacy_string(include_str!("../examples/inhibitory.nfy"));
                                ui.close();
                            }
                            if ui.button("Adaptation").clicked() {
                                self.load_legacy_string(include_str!("../examples/adaptation.nfy"));
                                ui.close();
                            }
                            if ui.button("Burst").clicked() {
                                self.load_legacy_string(include_str!("../examples/burst.nfy"));
                                ui.close();
                            }
                        });
                        ui.menu_button("Circuits", |ui| {
                            if ui.button("Input Summation").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/input_summation.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Prolonged Activity").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/prolonged_activity.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Disinhibition").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/disinhibition.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Recurrent Inhibition").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/recurrent_inhibition.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Reciprocal Inhibition").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/reciprocal_inhibition.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Lateral Inhibition").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/lateral_inhibition.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Lateral Inhibition 1").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/lateral_inhibition_1.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Lateral Inhibition 2").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/lateral_inhibition_2.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Two Neuron Oscillator").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/two_neuron_oscillator.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Rhythm Transformation").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/rythm_transformation.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Types of Inhibition").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/types_of_inhibition.nfy"
                                ));
                                ui.close();
                            }
                        });
                        ui.menu_button("Textbook", |ui| {
                            if ui.button("IF Response").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/if_response.nfy"
                                ));
                                ui.close();
                            }
                            if ui.button("Refractory Period").clicked() {
                                self.load_legacy_string(include_str!(
                                    "../examples/refractory_period.nfy"
                                ));
                                ui.close();
                            }
                        });
                        ui.menu_button("Items", |ui| {
                            if ui.button("Generators").clicked() {
                                self.load_legacy_string(include_str!("../examples/generators.nfy"));
                                ui.close();
                            }
                        });
                    });
                    ui.menu_button("Game", |ui| {
                        if ui.button("New Game").clicked() {
                            self.pending_new_game = true;
                            ui.close();
                        }
                        if self.petri_dish.is_some() && ui.button("Exit Game").clicked() {
                            self.petri_dish = None;
                            self.world.clear();
                            ui.close();
                        }
                    });
                });
            });
            if !in_game {
                egui::Window::new("Elements").show(context, |ui| {
                    for category in &TOOL_CATEGORIES {
                        ui.collapsing(category.label(), |ui| {
                            for (tool_value, label) in category.tools() {
                                ui.selectable_value(&mut self.tool, tool_value, label);
                            }
                        });
                    }
                });
            }
            egui::Window::new("Settings").show(context, |ui| {
                ui.label(format!("FPS: {:.0}", self.fps));
                ui.label("Simulation speed");
                ui.add(egui::Slider::new(&mut self.iterations, 1..=20));
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
                        let is_inhibitory = self.world.get::<&Inhibitory>(active_entity).is_ok();
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
                self.update_placement_preview(application);
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
