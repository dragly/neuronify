use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use glam::Vec3;
use hecs::Entity;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use visula::{
    winit::keyboard::ModifiersKeyState, CustomEvent, Expression, InstanceBuffer, LineGeometry,
    LineMaterial, Lines, MeshPipeline, RenderData, Renderable, SphereGeometry, SphereMaterial,
    Spheres, UniformBuffer,
};

use neuronify_core::rendering::gpu_types::{ConnectionData, Sphere};
use neuronify_core::simulation::{apply_spatial_forces, integrate_motion};
use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, ConnectionColor, Deletable, GeneratorDynamics,
    Keyboard, LeakyDynamics, LeakyNeuron, Mouse, NeuronType, Position, Selectable, SpatialDynamics,
    StaticConnectionSource, Tool, VisualRadius, CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE,
    COUPLING_CAPACITANCE, ERASE_RADIUS, FHN_CDT, FPS_LOW_PASS_FACTOR, LIF_DT,
    MIN_CREATION_DISTANCE_AXON, MIN_CREATION_DISTANCE_DEFAULT, NODE_RADIUS, PHYSICS_DT,
    SELECTION_RANGE, TARGET_FRAME_MS,
};
use crate::rendering::colors::glial_color;

use crate::components::*;
use crate::constants::*;
use crate::rendering;
use crate::simulation::{boundary, cleanup, economy, metabolism, ownership, setup, transport};
use crate::spawning;
use crate::tools::*;
use crate::ui::sidebar;

enum ConnectResult {
    Done,          // connected to target — stop the chain
    Continue,      // keep building
    FundsBlocked,  // can't afford — show feedback, stay put
}

pub struct GameApp {
    pub spheres: Spheres,
    pub sphere_buffer: InstanceBuffer<Sphere>,
    pub connection_lines: Lines,
    pub connection_spheres: Spheres,
    pub connection_buffer: InstanceBuffer<ConnectionData>,
    pub vessel_mesh: MeshPipeline,
    pub particle_spheres: Spheres,
    #[allow(dead_code)]
    pub particle_buffer: InstanceBuffer<rendering::BloodParticle>,
    pub vessel_time_buffer: UniformBuffer<rendering::VesselTime>,
    pub world: hecs::World,
    pub petri_dish: setup::PetriDish,
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

    pub pending_new_game: bool,
    pub pending_exit_game: bool,
    pub p1_economy: PlayerEconomy,
    pub placement_preview: Option<Vec3>,
    pub selected_entity: Option<Entity>,
    pub funds_flash_timer: f64,
    pub funds_blocked_entity: Option<Entity>,
    pub connection_consumed_this_press: bool,
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

/// Find the nearest candidate within a snap radius.
/// Each candidate has (entity, position, snap_radius).
fn find_nearest_within(
    candidates: &[(Entity, Vec3, f32)],
    mouse_position: Vec3,
) -> Option<(Entity, Vec3)> {
    candidates
        .iter()
        .filter(|(_, pos, snap)| pos.distance(mouse_position) < *snap)
        .min_by(|a, b| {
            a.1.distance(mouse_position)
                .partial_cmp(&b.1.distance(mouse_position))
                .unwrap_or(Ordering::Equal)
        })
        .map(|(id, pos, _)| (*id, *pos))
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
            &SphereGeometry {
                position: sphere.position.clone(),
                radius: sphere.radius,
                color: sphere.color,
            },
            &SphereMaterial {
                color: Expression::InstanceColor.lit(),
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
                color: Expression::InstanceColor.lit(),
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
                color: Expression::InstanceColor.lit(),
            },
        )
        .unwrap();

        let mut vessel_mesh =
            rendering::create_vessel_pipeline(&application.rendering_descriptor()).unwrap();

        let vessel_time_buffer = UniformBuffer::<rendering::VesselTime>::new(&application.device);
        let particle_buffer = InstanceBuffer::<rendering::BloodParticle>::new(&application.device);
        let particle_spheres = rendering::create_particle_pipeline(
            &application.rendering_descriptor(),
            &particle_buffer,
            &vessel_time_buffer,
        )
        .unwrap();

        let mut world = hecs::World::new();
        let dish = setup::PetriDish {
            center: Vec3::ZERO,
            radius: PETRI_DISH_RADIUS,
        };
        setup::setup_game(&mut world, &dish);
        rendering::update_vessel_mesh(&mut vessel_mesh, &world, &application.device);

        let particles = rendering::generate_vessel_particles(&world);
        particle_buffer.update(&application.device, &application.queue, &particles);

        GameApp {
            spheres,
            sphere_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
            vessel_mesh,
            particle_spheres,
            particle_buffer,
            vessel_time_buffer,
            world,
            petri_dish: dish,
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

            pending_new_game: false,
            pending_exit_game: false,
            p1_economy: PlayerEconomy::default(),
            placement_preview: None,
            selected_entity: None,
            funds_flash_timer: 0.0,
            funds_blocked_entity: None,
            connection_consumed_this_press: false,
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

    /// Build an axon or glial process connection chain.
    fn handle_connection_tool(
        &mut self,
        mouse_position: Vec3,
        previous_too_near: bool,
        is_glial_process: bool,
    ) -> ConnectResult {
        let snap = 2.0 * NODE_RADIUS;
        let vessel_snap = BLOOD_VESSEL_SNAP_RADIUS;
        let world = &mut self.world;
        let economy = &mut self.p1_economy;

        match &mut self.connection_tool {
            None => {
                // Source candidates differ by tool
                let source_candidates: Vec<(Entity, Vec3, f32)> = if is_glial_process {
                    // GlialProcess tool: start from glial cells or glial process compartments
                    let mut candidates: Vec<_> = world
                        .query::<(&Position, &GlialCell)>()
                        .iter()
                        .map(|(e, (p, _))| (e, p.position, snap))
                        .collect();
                    candidates.extend(
                        world
                            .query::<(&Position, &GlialProcess)>()
                            .iter()
                            .map(|(e, (p, _))| (e, p.position, snap)),
                    );
                    candidates
                } else {
                    // Axon tool: start from neurons, generators, or neuronal compartments
                    // (exclude GlialProcess compartments)
                    let mut candidates: Vec<_> = world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
                        .iter()
                        .map(|(e, p)| (e, p.position, snap))
                        .collect();
                    candidates.extend(
                        world
                            .query::<&Position>()
                            .with::<&GeneratorDynamics>()
                            .iter()
                            .map(|(e, p)| (e, p.position, snap)),
                    );
                    candidates.extend(
                        world
                            .query::<&Position>()
                            .with::<&StaticConnectionSource>()
                            .iter()
                            .filter(|(e, _)| world.get::<&GlialProcess>(*e).is_err())
                            .filter(|(e, _)| world.get::<&GlialCell>(*e).is_err())
                            .map(|(e, p)| (e, p.position, snap)),
                    );
                    candidates
                };

                self.connection_tool = find_nearest_within(&source_candidates, mouse_position).map(
                    |(id, position)| ConnectionTool {
                        start: position,
                        end: mouse_position,
                        from: id,
                    },
                );
                if let Some(ct) = &self.connection_tool {
                    self.previous_creation = Some(PreviousCreation { entity: ct.from });
                }
                ConnectResult::Continue
            }
            Some(ct) => {
                ct.end = mouse_position;

                // Target candidates differ by tool
                let target_candidates: Vec<(Entity, Vec3, f32)> = if is_glial_process {
                    // GlialProcess: connect to blood vessels or neurons
                    let mut targets: Vec<_> = world
                        .query::<&Position>()
                        .with::<&VesselAnchor>()
                        .iter()
                        .map(|(e, p)| (e, p.position, vessel_snap))
                        .collect();
                    targets.extend(
                        world
                            .query::<&Position>()
                            .with::<&LeakyNeuron>()
                            .iter()
                            .map(|(e, p)| (e, p.position, snap)),
                    );
                    targets
                } else {
                    // Axon: connect to neurons only (not vessels, not glial cells)
                    world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
                        .iter()
                        .map(|(e, p)| (e, p.position, snap))
                        .collect()
                };

                let nearest_target = find_nearest_within(&target_candidates, mouse_position);

                match nearest_target {
                    Some((id, target_pos)) => {
                        let from_pos = world
                            .get::<&Position>(ct.from)
                            .map(|p| p.position)
                            .unwrap_or(target_pos);

                        // For non-compartment targets (neuron somas, blood vessels) always
                        // place a fixed bridge compartment just outside the target's surface.
                        // This compartment has no SpatialDynamics so physics can't drag it in.
                        let target_is_compartment =
                            world.get::<&Compartment>(id).is_ok();
                        let target_vr = world
                            .get::<&VisualRadius>(id)
                            .map(|vr| vr.radius)
                            .unwrap_or(NODE_RADIUS);

                        // Bridge distance: just outside the target surface.
                        let bridge_dist = target_vr + NODE_RADIUS * 0.5;
                        // Need a bridge if the target is a soma/vessel OR ct.from is too far away.
                        let needs_bridge = !target_is_compartment
                            || from_pos.distance(target_pos) > 2.0 * NODE_RADIUS * 1.5;

                        let final_from = if needs_bridge {
                            if !economy::try_spend_blocks(economy, COMPARTMENT_SPAWN_COST) {
                                return ConnectResult::FundsBlocked;
                            }
                            let dir = (from_pos - target_pos).normalize_or_zero();
                            let bridge_pos = target_pos + dir * bridge_dist;
                            let neuron_type = world
                                .get::<&NeuronType>(ct.from)
                                .map(|t| (*t).clone())
                                .unwrap_or(NeuronType::Excitatory);
                            // No SpatialDynamics — this compartment sits fixed at the surface.
                            let bridge = world.spawn((
                                Position {
                                    position: bridge_pos,
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
                            ));
                            if is_glial_process {
                                let _ = world.insert_one(bridge, GlialProcess);
                                let _ = world.insert_one(bridge, ConnectionColor(glial_color()));
                            }
                            let already = world.query::<&Connection>().iter().any(|(_, c)| {
                                c.from == ct.from && c.to == bridge
                            });
                            if !already && ct.from != bridge {
                                world.spawn((
                                    Connection {
                                        from: ct.from,
                                        to: bridge,
                                        strength: 1.0,
                                        directional: false,
                                    },
                                    Deletable {},
                                    CompartmentCurrent {
                                        capacitance: COUPLING_CAPACITANCE,
                                    },
                                ));
                            }
                            bridge
                        } else {
                            ct.from
                        };

                        let new_connection = Connection {
                            from: final_from,
                            to: id,
                            strength: 1.0,
                            directional: !is_glial_process,
                        };
                        let connection_exists =
                            world.query::<&Connection>().iter().any(|(_, c)| {
                                c.from == new_connection.from && c.to == new_connection.to
                            });
                        if !connection_exists && final_from != id {
                            world.spawn((
                                new_connection,
                                Deletable {},
                                CompartmentCurrent {
                                    capacitance: COUPLING_CAPACITANCE,
                                },
                            ));
                        }
                        ConnectResult::Done
                    }
                    None => {
                        if previous_too_near {
                            return ConnectResult::Continue;
                        }
                        if !economy::try_spend_blocks(economy, COMPARTMENT_SPAWN_COST) {
                            return ConnectResult::FundsBlocked;
                        }
                        let neuron_type = if let Ok(neuron_type) = world.get::<&NeuronType>(ct.from)
                        {
                            (*neuron_type).clone()
                        } else {
                            NeuronType::Excitatory
                        };
                        let compartment_builder = world.spawn((
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
                        // Mark glial process compartments
                        if is_glial_process {
                            let _ = world.insert_one(compartment_builder, GlialProcess);
                            let _ = world.insert_one(compartment_builder, ConnectionColor(glial_color()));
                        }
                        world.spawn((
                            Connection {
                                from: ct.from,
                                to: compartment_builder,
                                strength: 1.0,
                                directional: false,
                            },
                            Deletable {},
                            CompartmentCurrent {
                                capacitance: COUPLING_CAPACITANCE,
                            },
                        ));
                        self.previous_creation = Some(PreviousCreation {
                            entity: compartment_builder,
                        });
                        ct.start = mouse_position;
                        ct.from = compartment_builder;
                        ConnectResult::Continue
                    }
                }
            }
        }
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            self.connection_tool = None;
            self.previous_creation = None;
            self.move_origin = None;

            self.funds_blocked_entity = None;
            self.connection_consumed_this_press = false;
            return;
        }

        let mouse_position = match self.mouse_world_position(application) {
            Some(p) => p,
            None => return,
        };

        self.placement_preview = Some(mouse_position);

        let minimum_distance = match self.tool {
            GameTool::Axon | GameTool::GlialProcess => MIN_CREATION_DISTANCE_AXON,
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
                if !economy::try_spend_blocks(&mut self.p1_economy, NEURON_SPAWN_COST) {
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
            GameTool::GlialCell => {
                if previous_too_near {
                    return;
                }
                if !economy::try_spend_blocks(&mut self.p1_economy, GLIAL_COST) {
                    return;
                }
                let entity =
                    spawning::spawn_glial(&mut self.world, mouse_position, PlayerId::Player1, 3);
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
                match self.move_origin {
                    Some(origin) => {
                        let center = mouse_position - origin;
                        application.camera_controller.target_transform.center -=
                            Vec3::new(center.x, center.y, center.z);
                        application.camera_controller.current_transform.center =
                            application.camera_controller.target_transform.center;
                    }
                    None => {
                        if let Some((entity, _entity_pos)) = self
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
                            self.selected_entity = Some(entity);
                        } else {
                            self.selected_entity = None;
                            self.move_origin = Some(mouse_position);
                        }
                    }
                }
            }
            GameTool::Axon => {
                if self.connection_consumed_this_press && self.connection_tool.is_none() {
                    return;
                }
                match self.handle_connection_tool(mouse_position, previous_too_near, false) {
                    ConnectResult::Done => {
                        self.connection_tool = None;
                        self.funds_blocked_entity = None;
                        self.connection_consumed_this_press = true;
                    }
                    ConnectResult::FundsBlocked => {
                        self.funds_flash_timer = 0.5;
                        self.funds_blocked_entity = self.previous_creation.as_ref().map(|pc| pc.entity);
                    }
                    ConnectResult::Continue => {
                        self.funds_blocked_entity = None;
                    }
                }
            }
            GameTool::GlialProcess => {
                if self.connection_consumed_this_press && self.connection_tool.is_none() {
                    return;
                }
                match self.handle_connection_tool(mouse_position, previous_too_near, true) {
                    ConnectResult::Done => {
                        self.connection_tool = None;
                        self.funds_blocked_entity = None;
                        self.connection_consumed_this_press = true;
                    }
                    ConnectResult::FundsBlocked => {
                        self.funds_flash_timer = 0.5;
                        self.funds_blocked_entity = self.previous_creation.as_ref().map(|pc| pc.entity);
                    }
                    ConnectResult::Continue => {
                        self.funds_blocked_entity = None;
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
            setup::setup_game(&mut self.world, &self.petri_dish);
            rendering::update_vessel_mesh(&mut self.vessel_mesh, &self.world, &application.device);
            self.tool = GameTool::Select;
            self.p1_economy = PlayerEconomy::default();
            self.selected_entity = None;
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
        self.funds_flash_timer = (self.funds_flash_timer - frame_dt).max(0.0);
        boundary::enforce_petri_boundary(&mut self.world, &self.petri_dish);
        transport::spawn_glucose_packets(&mut self.world, frame_dt);
        transport::move_glucose_packets(&mut self.world, frame_dt);
        transport::spawn_lactate_packets(&mut self.world, frame_dt);
        transport::move_lactate_packets(&mut self.world, frame_dt);
        economy::glial_contribute_blocks(&mut self.world, frame_dt, &mut self.p1_economy);
        metabolism::metabolic_drain(&mut self.world, frame_dt);
        metabolism::apply_dormancy(&mut self.world);
        metabolism::resource_flow(&mut self.world, frame_dt);
        cleanup::cleanup_orphans(&mut self.world);
        ownership::update_ownership(&mut self.world);

        // Collect rendering data
        let connection_preview_end = if matches!(self.tool, GameTool::Axon | GameTool::GlialProcess) {
            self.connection_tool.as_ref().map(|ct| ct.end)
        } else {
            None
        };
        let mut spheres = rendering::collect_game_spheres(&self.world, self.funds_blocked_entity);
        let placement_spheres = rendering::collect_placement_preview(
            &self.tool,
            &self.placement_preview,
            connection_preview_end,
        );
        spheres.extend(placement_spheres.iter());

        let mut connections =
            neuronify_core::rendering::collect_connections(&self.world, &Tool::Select, &None);

        // Connection tool preview line — no directional arrow; ghost sphere (above) marks the end.
        if matches!(self.tool, GameTool::Axon | GameTool::GlialProcess) {
            if let Some(ref ct) = self.connection_tool {
                let from_pos = self
                    .world
                    .get::<&Position>(ct.from)
                    .map(|p| p.position)
                    .unwrap_or(ct.start);
                let preview_color = if self.tool == GameTool::GlialProcess {
                    Vec3::new(0.4, 0.8, 0.4)
                } else {
                    Vec3::new(0.8, 0.8, 0.8)
                };
                connections.push(ConnectionData {
                    position_a: from_pos,
                    position_b: ct.end,
                    strength: 1.0,
                    directional: 0.0, // ghost sphere handles the endpoint marker
                    start_color: preview_color,
                    end_color: preview_color,
                    _padding: Default::default(),
                });
            }
        }

        // Petri dish and decorations
        connections.extend(rendering::collect_petri_dish(&self.petri_dish));

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);
        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

        self.vessel_time_buffer.update(
            &application.queue,
            &rendering::VesselTime {
                time: self.time as f32,
                _padding: Default::default(),
            },
        );

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
        self.particle_spheres.render(data);
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
    }

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        let mut p1_neurons = 0u32;
        let mut p1_energy = 0.0f64;
        for (_, (ownership, metab)) in self.world.query::<(&Ownership, &MetabolicState)>().iter() {
            if ownership.player == PlayerId::Player1 {
                p1_neurons += 1;
                p1_energy += metab.energy;
            }
        }

        sidebar::draw_sidebar(
            context,
            &mut self.tool,
            p1_neurons,
            p1_energy,
            self.p1_economy.building_blocks,
            self.funds_flash_timer > 0.0,
            &mut self.pending_new_game,
            &mut self.pending_exit_game,
        );

        // Info panel for selected entity
        if let Some(entity) = self.selected_entity {
            if self.world.contains(entity) {
                crate::ui::info_panel::draw_info_panel(context, &self.world, entity);
            } else {
                self.selected_entity = None;
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
