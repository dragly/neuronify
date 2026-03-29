use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use glam::Vec3;
use hecs::Entity;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use visula::{
    winit::keyboard::ModifiersKeyState, CustomEvent, InstanceBuffer, LineDelegate, Lines,
    MeshPipeline, RenderData, Renderable, SphereDelegate, Spheres,
};

use neuronify_core::rendering::gpu_types::{ConnectionData, Sphere};
use neuronify_core::simulation::{apply_spatial_forces, integrate_motion};
use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, Deletable, GeneratorDynamics, Keyboard,
    LeakyDynamics, LeakyNeuron, Mouse, NeuronType, PoissonGenerator, Position,
    RegularSpikeGenerator, Selectable, SpatialDynamics, StaticConnectionSource, Tool,
    CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE, COUPLING_CAPACITANCE, ERASE_RADIUS, FHN_CDT,
    FPS_LOW_PASS_FACTOR, LIF_DT, MIN_CREATION_DISTANCE_AXON, MIN_CREATION_DISTANCE_DEFAULT,
    NODE_RADIUS, PHYSICS_DT, SELECTION_RANGE, TARGET_FRAME_MS,
};

use crate::components::*;
use crate::constants::*;
use crate::rendering;
use crate::simulation::{ai::AiState, game, motor};
use crate::spawning;
use crate::tools::*;
use crate::ui::sidebar;

pub struct GameApp {
    pub spheres: Spheres,
    pub sphere_buffer: InstanceBuffer<Sphere>,
    pub connection_lines: Lines,
    pub connection_spheres: Spheres,
    pub connection_buffer: InstanceBuffer<ConnectionData>,
    pub vessel_mesh: MeshPipeline,
    pub world: hecs::World,
    pub petri_dish: game::PetriDish,
    pub ai_state: AiState,
    pub tool: GameTool,
    pub connection_tool: Option<ConnectionTool>,
    pub previous_creation: Option<PreviousCreation>,
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub time: f64,
    pub iterations: u32,
    pub fps: f64,
    pub last_update: DateTime<Utc>,
    pub move_origin: Option<Vec3>,
    pub dragging_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub pending_new_game: bool,
    pub pending_exit_game: bool,
    pub p1_economy: PlayerEconomy,
    pub p2_economy: PlayerEconomy,
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
        .unwrap_or(Ordering::Equal)
}

impl GameApp {
    pub fn new(application: &mut visula::Application) -> GameApp {
        application.camera_controller.enabled = false;
        application.camera_controller.target_transform.center = Vec3::new(0.0, 0.0, 0.0);
        application.camera_controller.target_transform.forward =
            Vec3::new(0.3, -1.0, 0.0).normalize();
        application.camera_controller.target_transform.distance = 150.0;
        application.camera_controller.current_transform =
            application.camera_controller.target_transform.clone();

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
                width: connection.strength.clone() * 0.3,
                color: connection.start_color.clone(),
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

        let mut vessel_mesh =
            rendering::create_vessel_pipeline(&application.rendering_descriptor()).unwrap();

        let mut world = hecs::World::new();
        let dish = game::PetriDish {
            center: Vec3::ZERO,
            radius: PETRI_DISH_RADIUS,
        };
        game::setup_game(&mut world, &dish);
        rendering::update_vessel_mesh(&mut vessel_mesh, &world, &application.device);

        GameApp {
            spheres,
            sphere_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
            vessel_mesh,
            world,
            petri_dish: dish,
            ai_state: AiState::new(),
            tool: GameTool::Select,
            connection_tool: None,
            previous_creation: None,
            mouse: Mouse {
                left_down: false,
                position: None,
                delta_position: None,
            },
            keyboard: Keyboard { shift_down: false },
            time: 0.0,
            iterations: 4,
            fps: 60.0,
            last_update: Utc::now(),
            move_origin: None,
            dragging_entity: None,
            drag_offset: Vec3::ZERO,
            pending_new_game: false,
            pending_exit_game: false,
            p1_economy: PlayerEconomy::default(),
            p2_economy: PlayerEconomy::default(),
            placement_preview: None,
        }
    }

    fn mouse_world_position(&self, application: &visula::Application) -> Option<Vec3> {
        let mouse_physical_position = self.mouse.position?;
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
        Some(ray_origin + t * ray_world)
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            self.connection_tool = None;
            self.previous_creation = None;
            self.move_origin = None;
            self.dragging_entity = None;
            // Keep placement_preview for hover effect
            return;
        }

        let mouse_position = match self.mouse_world_position(application) {
            Some(p) => p,
            None => return,
        };

        // Update placement preview
        self.placement_preview = Some(mouse_position);

        let minimum_distance = match self.tool {
            GameTool::Axon => MIN_CREATION_DISTANCE_AXON,
            _ => MIN_CREATION_DISTANCE_DEFAULT,
        };
        let previous_too_near = if let Some(pc) = &self.previous_creation {
            if let Ok(position) = self.world.get::<&Position>(pc.entity) {
                position.position.distance(mouse_position) < minimum_distance
            } else {
                false
            }
        } else {
            false
        };

        match &self.tool {
            GameTool::ExcitatoryNeuron | GameTool::InhibitoryNeuron => {
                if previous_too_near {
                    return;
                }
                if !game::is_within_build_range(&self.world, mouse_position, PlayerId::Player1) {
                    return;
                }
                if !game::try_spend_blocks(&mut self.p1_economy, NEURON_SPAWN_COST) {
                    return;
                }
                let neuron_type = if self.tool == GameTool::InhibitoryNeuron {
                    NeuronType::Inhibitory
                } else {
                    NeuronType::Excitatory
                };
                let entity = spawning::spawn_neuron(
                    &mut self.world,
                    mouse_position,
                    neuron_type,
                    PlayerId::Player1,
                );
                self.previous_creation = Some(PreviousCreation { entity });
            }
            GameTool::MembraneSegment => {
                if previous_too_near {
                    return;
                }
                if !game::is_within_build_range(&self.world, mouse_position, PlayerId::Player1) {
                    return;
                }
                if !game::try_spend_blocks(&mut self.p1_economy, MEMBRANE_SPAWN_COST) {
                    return;
                }
                let entity = self.world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    MembraneSegment,
                    SpatialDynamics {
                        velocity: Vec3::ZERO,
                        acceleration: Vec3::ZERO,
                    },
                    Ownership {
                        player: PlayerId::Player1,
                    },
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            GameTool::SpikeGenerator | GameTool::PoissonGenerator => {
                if previous_too_near {
                    return;
                }
                if !game::is_within_build_range(&self.world, mouse_position, PlayerId::Player1) {
                    return;
                }
                if !game::try_spend_blocks(&mut self.p1_economy, GENERATOR_SPAWN_COST) {
                    return;
                }
                let entity = if self.tool == GameTool::SpikeGenerator {
                    self.world.spawn((
                        Position {
                            position: mouse_position,
                        },
                        RegularSpikeGenerator::default(),
                        GeneratorDynamics::default(),
                        MetabolicState::default(),
                        Ownership {
                            player: PlayerId::Player1,
                        },
                        SpatialDynamics {
                            velocity: Vec3::ZERO,
                            acceleration: Vec3::ZERO,
                        },
                        Deletable {},
                    ))
                } else {
                    self.world.spawn((
                        Position {
                            position: mouse_position,
                        },
                        PoissonGenerator::default(),
                        GeneratorDynamics::default(),
                        MetabolicState::default(),
                        Ownership {
                            player: PlayerId::Player1,
                        },
                        SpatialDynamics {
                            velocity: Vec3::ZERO,
                            acceleration: Vec3::ZERO,
                        },
                        Deletable {},
                    ))
                };
                self.previous_creation = Some(PreviousCreation { entity });
            }
            GameTool::MotorCilia => {
                // Attach motor cilia to nearest owned neuron
                let nearest = self
                    .world
                    .query::<(&Position, &Ownership)>()
                    .with::<&LeakyNeuron>()
                    .without::<&MotorCilia>()
                    .iter()
                    .filter(|(_, (_, o))| o.player == PlayerId::Player1)
                    .min_by(|a, b| {
                        a.1 .0
                            .position
                            .distance(mouse_position)
                            .partial_cmp(&b.1 .0.position.distance(mouse_position))
                            .unwrap_or(Ordering::Equal)
                    })
                    .and_then(|(e, (p, _))| {
                        if p.position.distance(mouse_position) < NODE_RADIUS * 3.0 {
                            Some(e)
                        } else {
                            None
                        }
                    });
                if let Some(entity) = nearest {
                    let pos = self
                        .world
                        .get::<&Position>(entity)
                        .map(|p| p.position)
                        .unwrap_or(Vec3::ZERO);
                    if !game::try_spend_blocks(&mut self.p1_economy, MOTOR_CILIA_COST) {
                        return;
                    }
                    // Direction: from neuron toward mouse click
                    let dir = (mouse_position - pos).normalize_or_zero();
                    let _ = self.world.insert_one(
                        entity,
                        MotorCilia {
                            direction: dir,
                            strength: MOTOR_THRUST_STRENGTH,
                        },
                    );
                }
            }
            GameTool::ActivitySensor | GameTool::ChemicalSensor | GameTool::TouchSensor => {
                if previous_too_near {
                    return;
                }
                if !game::is_within_build_range(&self.world, mouse_position, PlayerId::Player1) {
                    return;
                }
                if !game::try_spend_blocks(&mut self.p1_economy, SENSOR_COST) {
                    return;
                }
                let sensor_type = match self.tool {
                    GameTool::ActivitySensor => SensorType::Activity,
                    GameTool::ChemicalSensor => SensorType::Chemical,
                    GameTool::TouchSensor => SensorType::Touch,
                    _ => unreachable!(),
                };
                let entity = spawning::spawn_neuron(
                    &mut self.world,
                    mouse_position,
                    NeuronType::Excitatory,
                    PlayerId::Player1,
                );
                let _ = self.world.insert_one(
                    entity,
                    SensorNeuron {
                        sensor_type,
                        sensitivity: SENSOR_SENSITIVITY,
                        gain: SENSOR_GAIN,
                    },
                );
                self.previous_creation = Some(PreviousCreation { entity });
            }
            GameTool::GlialCell => {
                if previous_too_near {
                    return;
                }
                // Glial cells must be placed within a blood vessel's supply_radius
                if !game::is_within_vessel_range(&self.world, mouse_position, PlayerId::Player1) {
                    return;
                }
                if !game::try_spend_blocks(&mut self.p1_economy, GLIAL_COST) {
                    return;
                }
                let entity = self.world.spawn((
                    Position {
                        position: mouse_position,
                    },
                    GlialCell::default(),
                    Ownership {
                        player: PlayerId::Player1,
                    },
                    SpatialDynamics {
                        velocity: Vec3::ZERO,
                        acceleration: Vec3::ZERO,
                    },
                    Deletable {},
                ));
                self.previous_creation = Some(PreviousCreation { entity });
            }
            GameTool::Erase => {
                let to_delete: Vec<Entity> = self
                    .world
                    .query::<&Position>()
                    .with::<&Deletable>()
                    .iter()
                    .filter_map(|(entity, position)| {
                        if position.position.distance(mouse_position) < NODE_RADIUS * 1.5 {
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect();
                for entity in to_delete {
                    let _ = self.world.despawn(entity);
                }
                let connections_to_delete: Vec<Entity> = self
                    .world
                    .query::<&Connection>()
                    .with::<&Deletable>()
                    .iter()
                    .filter_map(|(entity, connection)| {
                        if let (Ok(from), Ok(to)) = (
                            self.world.get::<&Position>(connection.from),
                            self.world.get::<&Position>(connection.to),
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
                    .collect();
                for entity in connections_to_delete {
                    let _ = self.world.despawn(entity);
                }
            }
            GameTool::Select => {
                if let Some(entity) = self.dragging_entity {
                    // Don't drag anchored entities
                    if self.world.get::<&Anchored>(entity).is_ok() {
                        return;
                    }
                    if let Ok(mut pos) = self.world.get::<&mut Position>(entity) {
                        pos.position = mouse_position + self.drag_offset;
                        pos.position.y = 0.0;
                    }
                } else {
                    match self.move_origin {
                        Some(origin) => {
                            let center = mouse_position - origin;
                            application.camera_controller.target_transform.center -=
                                Vec3::new(center.x, center.y, center.z);
                            application.camera_controller.current_transform.center =
                                application.camera_controller.target_transform.center;
                        }
                        None => {
                            if let Some((entity, entity_pos)) = self
                                .world
                                .query::<&Position>()
                                .iter()
                                .min_by(|a, b| nearest(&mouse_position, a, b))
                                .and_then(|(id, pos)| {
                                    if mouse_position.distance(pos.position) < SELECTION_RANGE {
                                        Some((id, pos.position))
                                    } else {
                                        None
                                    }
                                })
                            {
                                self.dragging_entity = Some(entity);
                                self.drag_offset = entity_pos - mouse_position;
                                self.drag_offset.y = 0.0;
                            } else {
                                self.move_origin = Some(mouse_position);
                            }
                        }
                    }
                }
            }
            GameTool::Axon => {
                let world = &mut self.world;
                let economy = &mut self.p1_economy;
                match &mut self.connection_tool {
                    None => {
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
                                    .with::<&GeneratorDynamics>()
                                    .iter()
                                    .map(|(e, p)| (e, p.position)),
                            );
                            candidates.extend(
                                world
                                    .query::<&Position>()
                                    .with::<&StaticConnectionSource>()
                                    .iter()
                                    .map(|(e, p)| (e, p.position)),
                            );
                            candidates
                        };
                        self.connection_tool = source_candidates
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
                        if let Some(ct) = &self.connection_tool {
                            self.previous_creation = Some(PreviousCreation { entity: ct.from });
                        }
                    }
                    Some(ct) => {
                        ct.end = mouse_position;
                        // Check if near an existing neuron to connect to
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
                                // Cost for compartment
                                if !game::try_spend_blocks(economy, COMPARTMENT_SPAWN_COST) {
                                    return;
                                }
                                // Membrane blocking
                                let from_pos = world
                                    .get::<&Position>(ct.from)
                                    .map(|p| p.position)
                                    .unwrap_or(Vec3::ZERO);
                                if game::membrane_blocks_path(world, from_pos, mouse_position) {
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
                                            velocity: Vec3::ZERO,
                                            acceleration: Vec3::ZERO,
                                        },
                                    ));
                                    world.spawn((
                                        Connection {
                                            from: ct.from,
                                            to: compartment,
                                            strength: 1.0,
                                            directional: false,
                                        },
                                        Deletable {},
                                        CompartmentCurrent {
                                            capacitance: COUPLING_CAPACITANCE,
                                        },
                                    ));
                                    self.previous_creation = Some(PreviousCreation {
                                        entity: compartment,
                                    });
                                    ct.start = mouse_position;
                                    ct.from = compartment;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl visula::Simulation for GameApp {
    type Error = Error;

    fn clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: neuronify_core::rendering::srgb_component(30) as f64,
            g: neuronify_core::rendering::srgb_component(30) as f64,
            b: neuronify_core::rendering::srgb_component(46) as f64,
            a: 1.0,
        }
    }

    fn update(&mut self, application: &mut visula::Application) {
        if self.pending_new_game {
            self.pending_new_game = false;
            self.world.clear();
            game::setup_game(&mut self.world, &self.petri_dish);
            rendering::update_vessel_mesh(&mut self.vessel_mesh, &self.world, &application.device);
            self.ai_state = AiState::new();
            self.tool = GameTool::Select;
            self.p1_economy = PlayerEconomy::default();
            self.p2_economy = PlayerEconomy::default();
        }

        if self.pending_exit_game {
            std::process::exit(0);
        }

        // LIF substeps
        let lif_dt = LIF_DT;
        for _ in 0..self.iterations {
            neuronify_core::lif_step(&mut self.world, lif_dt, self.time);
            self.time += lif_dt;
        }

        // FHN + physics substeps
        let fire_window = self.iterations as f64 * lif_dt;
        let mut recently_fired: HashSet<Entity> = self
            .world
            .query::<&LeakyDynamics>()
            .iter()
            .filter(|(_, d)| d.time_since_fire < fire_window)
            .map(|(e, _)| e)
            .collect();
        for (e, d) in self.world.query::<&GeneratorDynamics>().iter() {
            if d.time_since_fire < fire_window {
                recently_fired.insert(e);
            }
        }

        for _ in 0..self.iterations {
            neuronify_core::fhn_step(&mut self.world, FHN_CDT, &recently_fired);
            apply_spatial_forces(&mut self.world);
            integrate_motion(&mut self.world, PHYSICS_DT);
        }

        // Game systems (once per frame)
        let frame_dt = self.iterations as f64 * LIF_DT;
        game::enforce_petri_boundary(&mut self.world, &self.petri_dish);
        game::apply_sensors(&mut self.world);
        game::glial_gather_resources(&mut self.world, frame_dt);
        game::glial_contribute_blocks(
            &mut self.world,
            frame_dt,
            &mut self.p1_economy,
            &mut self.p2_economy,
        );
        game::glial_distribute_atp(&mut self.world, frame_dt);
        game::metabolic_drain(&mut self.world, frame_dt);
        game::apply_dormancy(&mut self.world);
        game::resource_flow(&mut self.world, frame_dt);
        game::depolarization_block(&mut self.world, frame_dt);
        game::apply_substrate_zones(&mut self.world, frame_dt);
        game::cleanup_orphans(&mut self.world);
        game::update_ownership(&mut self.world);

        // Motor systems
        motor::apply_motor_forces(&mut self.world, frame_dt);
        motor::enforce_anchored(&mut self.world);

        // AI opponent
        crate::simulation::ai::ai_tick_for_player(
            &mut self.world,
            &mut self.ai_state,
            &self.petri_dish,
            PlayerId::Player2,
            &mut self.p2_economy,
        );

        // Collect rendering data
        let mut spheres = rendering::collect_game_spheres(&self.world);
        let placement_spheres = rendering::collect_placement_preview(&self.tool, &self.placement_preview);
        spheres.extend(placement_spheres.iter());

        // Connections: use core's collect_connections with a neutral tool, then add axon preview
        let mut connections = neuronify_core::rendering::collect_connections(
            &self.world,
            &Tool::Select, // won't draw any tool preview
            &None,
        );

        // Add axon tool preview line
        if self.tool == GameTool::Axon {
            if let Some(ref ct) = self.connection_tool {
                connections.push(ConnectionData {
                    position_a: ct.start,
                    position_b: ct.end,
                    strength: 1.0,
                    directional: 1.0,
                    start_color: Vec3::new(0.8, 0.8, 0.8),
                    end_color: Vec3::new(0.8, 0.8, 0.8),
                    _padding: Default::default(),
                });
            }
        }

        // Petri dish and decorations
        connections.extend(rendering::collect_petri_dish(&self.petri_dish));
        connections.extend(rendering::collect_vessel_supply_rings(&self.world));
        connections.extend(rendering::collect_glial_vessel_links(&self.world));
        connections.extend(rendering::collect_substrate_zone_rings(&self.world));

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);
        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

        // FPS tracking
        let time_diff = Utc::now() - self.last_update;
        #[cfg(not(target_arch = "wasm32"))]
        if time_diff < Duration::milliseconds(TARGET_FRAME_MS) {
            std::thread::sleep(std::time::Duration::from_millis(
                (Duration::milliseconds(TARGET_FRAME_MS) - time_diff).num_milliseconds() as u64,
            ))
        }
        let new_fps = 1.0
            / ((Utc::now() - self.last_update).num_nanoseconds().unwrap() as f64 * 1e-9)
                .max(0.0000001);
        self.fps = (1.0 - FPS_LOW_PASS_FACTOR) * self.fps + FPS_LOW_PASS_FACTOR * new_fps;
        self.last_update = Utc::now();
    }

    fn render(&mut self, data: &mut RenderData) {
        self.vessel_mesh.render(data);
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
    }

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        // Gather stats
        let mut p1_neurons = 0u32;
        let mut p1_energy = 0.0f64;
        let mut p2_neurons = 0u32;
        let mut p2_energy = 0.0f64;
        for (_, (ownership, metab)) in self.world.query::<(&Ownership, &MetabolicState)>().iter() {
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

        sidebar::draw_sidebar(
            context,
            &mut self.tool,
            p1_neurons,
            p1_energy,
            self.p1_economy.building_blocks,
            p2_neurons,
            p2_energy,
            self.p2_economy.building_blocks,
            &mut self.pending_new_game,
            &mut self.pending_exit_game,
        );
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
                // Update placement preview on mouse move
                if let Some(pos) = self.mouse_world_position(application) {
                    self.placement_preview = Some(pos);
                }
                self.handle_tool(application);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let scroll = match delta {
                    visula::winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    visula::winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
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
