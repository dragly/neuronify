use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use glam::Vec3;
use hecs::Entity;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, Touch, TouchPhase, WindowEvent};
use visula::{
    winit::keyboard::ModifiersKeyState, CustomEvent, Expression, InstanceBuffer,
    LineGeometry, LineMaterial, Lines, MeshPipeline, RenderData, Renderable, ShadowRenderData,
    SphereGeometry, SphereMaterial, Spheres,
};

use neuronify_core::rendering::gpu_types::{ConnectionData, Sphere};
use neuronify_core::simulation::{apply_spatial_forces, integrate_motion};
use neuronify_core::{
    Compartment, CompartmentCurrent, Connection, ConnectionColor, Deletable, GeneratorDynamics,
    Keyboard, LeakyDynamics, LeakyNeuron, Mouse, NeuronType, Position, Selectable, SpatialDynamics,
    StaticConnectionSource, Tool, VisualRadius, CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE,
    COUPLING_CAPACITANCE, ERASE_RADIUS, FHN_CDT, FPS_LOW_PASS_FACTOR, LIF_DT,
    MIN_CREATION_DISTANCE_AXON, MIN_CREATION_DISTANCE_DEFAULT, NODE_RADIUS, PHYSICS_DT,
    SELECTION_RANGE,
};
use crate::constants::{COMBAT_DT, GAME_SPEED};
use crate::rendering::colors::glial_color;

use crate::components::*;
use crate::constants::*;
use crate::spawning;
use crate::rendering;
use crate::simulation::{boundary, cleanup, combat, cytokines, dev_scenarios, economy, metabolism, ownership, production, scenarios, setup, transport, victory};
use crate::simulation::pathfinding::HexGrid;
use crate::simulation::scenarios::ScenarioId;
use crate::tools::*;
use crate::ui::{main_menu, sidebar};

enum ConnectResult {
    Done,          // connected to target — stop the chain
    Continue,      // keep building
    FundsBlocked,  // can't afford — show feedback, stay put
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    /// Showing the scenario briefing overlay before gameplay begins.
    Briefing {
        name: String,
        briefing: String,
        objective: String,
    },
    InGame,
}

pub struct GameApp {
    pub spheres: Spheres,
    pub sphere_buffer: InstanceBuffer<Sphere>,
    pub connection_lines: Lines,
    pub connection_spheres: Spheres,
    pub connection_buffer: InstanceBuffer<ConnectionData>,
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
    pub selected_entities: Vec<Entity>,
    pub attack_mode: bool,
    pub funds_flash_timer: f64,
    pub funds_blocked_entity: Option<Entity>,
    pub connection_consumed_this_press: bool,
    /// Active touch points by touch ID → current screen position.
    pub touches: HashMap<u64, PhysicalPosition<f64>>,
    pub current_scenario: ScenarioId,
    pub sidebar_icons: Option<crate::ui::sidebar::SidebarIcons>,
    pub game_state: GameState,
    pub menu_scenarios: Vec<main_menu::ScenarioEntry>,
    pub menu_selected: Option<usize>,
    /// Embedded SVG content that will be loaded when `pending_new_game` fires.
    pub pending_svg_content: Option<&'static str>,
    /// When true, the next update loads the Voronoi terrain scenario.
    pub pending_voronoi_scenario: bool,
    /// Pathfinding grid rebuilt each frame from current neuroblast positions.
    pub hex_grid: HexGrid,
    /// When true, the next right-click or left-click sets the move destination
    /// for all selected neuroblasts.
    pub awaiting_move_destination: bool,
    /// Accumulated wall-clock seconds not yet consumed by combat ticks.
    pub combat_accumulator: f32,
    pub microglia_mesh: MeshPipeline,
    pub astrocyte_mesh: MeshPipeline,
    pub macrophage_mesh: MeshPipeline,
    pub dendrite_cylinders: visula::Cylinders,
    pub dendrite_buffer: InstanceBuffer<rendering::CylinderData>,
    pub health_bar_meshes: [MeshPipeline; 4],
    pub energy_bar_meshes: [MeshPipeline; 2],
    pub terrain_mesh: MeshPipeline,
    /// Terrain hex data for the loaded scenario, used for movement speed/passability.
    /// Empty map = no terrain effects (e.g. in the non-SVG default scenario).
    pub scenario_terrain: std::collections::HashMap<(i32,i32), crate::map::HexTerrain>,
    /// Parsed win condition for the current SVG scenario, if any.
    pub victory_condition: Option<victory::VictoryCondition>,
    /// Countdown (seconds) until the next victory check.
    pub victory_check_timer: f64,
    /// True once the victory condition has been satisfied.
    pub victory_achieved: bool,
    /// Elapsed in-game seconds since the scenario started.
    pub game_elapsed: f64,
    /// Dev mode enabled via `--dev` CLI flag.
    pub dev_mode: bool,
    /// Pending dev stage to apply after the base SVG loads.
    pub pending_dev_stage: Option<dev_scenarios::DevStage>,
}

/// Build the main-menu scenario list from SVG content embedded in the binary.
fn load_menu_scenarios() -> Vec<main_menu::ScenarioEntry> {
    let mut entries = Vec::new();
    for &content in crate::map::embedded_scenario_svgs() {
        match crate::map::parse_scenario_svg(content) {
            Ok(map) => entries.push(main_menu::ScenarioEntry {
                meta: map.meta,
                svg_content: content,
                dev_stage: None,
            }),
            Err(e) => log::warn!("Failed to parse embedded scenario: {}", e),
        }
    }
    entries
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
    pub fn new(application: &mut visula::Application, dev_mode: bool) -> GameApp {
        application.camera_controller.enabled = false;
        application.camera_controller.target_transform.center = Vec3::new(0.0, 0.0, 0.0);
        // 45° below horizontal, looking along +z. sin(45°) = cos(45°) ≈ 0.7071.
        application.camera_controller.target_transform.forward =
            Vec3::new(-0.3536, -0.7071, 0.6124);
        application.camera_controller.target_transform.distance = 300.0;
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

        let microglia_mesh =
            rendering::create_microglia_pipeline(&application.rendering_descriptor()).unwrap();
        let astrocyte_mesh =
            rendering::create_astrocyte_pipeline(&application.rendering_descriptor()).unwrap();
        let macrophage_mesh =
            rendering::create_macrophage_pipeline(&application.rendering_descriptor()).unwrap();
        let dendrite_buffer = InstanceBuffer::<rendering::CylinderData>::new(&application.device);
        let dendrite_cylinders =
            rendering::create_dendrite_pipeline(&application.rendering_descriptor(), &dendrite_buffer).unwrap();
        let health_bar_meshes =
            rendering::create_health_bar_pipelines(&application.rendering_descriptor()).unwrap();
        let energy_bar_meshes =
            rendering::create_energy_bar_pipelines(&application.rendering_descriptor()).unwrap();
        let terrain_mesh =
            rendering::create_terrain_pipeline(&application.rendering_descriptor()).unwrap();

        let world = hecs::World::new();
        let dish = setup::PetriDish {
            center: Vec3::ZERO,
            radius: PETRI_DISH_RADIUS,
        };

        // Load scenario metadata from SVG files for the main menu.
        let mut menu_scenarios = load_menu_scenarios();
        if dev_mode {
            menu_scenarios.extend(dev_scenarios::load_dev_scenario_entries());
        }

        GameApp {
            spheres,
            sphere_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
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
            keyboard: Keyboard { shift_down: false, ctrl_down: false },
            time: 0.0,
            iterations: 2,
            fps: 60.0,
            last_update: Utc::now(),
            move_origin: None,

            pending_new_game: false,
            pending_exit_game: false,
            p1_economy: PlayerEconomy::default(),
            placement_preview: None,
            selected_entities: Vec::new(),
            attack_mode: false,
            funds_flash_timer: 0.0,
            funds_blocked_entity: None,
            connection_consumed_this_press: false,
            touches: HashMap::new(),
            current_scenario: ScenarioId::Default,
            sidebar_icons: None,
            game_state: GameState::MainMenu,
            menu_scenarios,
            menu_selected: None,
            pending_svg_content: None,
            pending_voronoi_scenario: false,
            hex_grid: HexGrid::new(HEX_GRID_CELL_SIZE),
            awaiting_move_destination: false,
            combat_accumulator: 0.0,
            microglia_mesh,
            astrocyte_mesh,
            macrophage_mesh,
            dendrite_cylinders,
            dendrite_buffer,
            health_bar_meshes,
            energy_bar_meshes,
            terrain_mesh,
            scenario_terrain: std::collections::HashMap::new(),
            victory_condition: None,
            victory_check_timer: 1.0,
            victory_achieved: false,
            game_elapsed: 0.0,
            dev_mode,
            pending_dev_stage: None,
        }
    }

    /// Unproject a screen-space pixel position (physical pixels) onto the y=0 world plane.
    fn screen_to_world(application: &visula::Application, px: f32, py: f32) -> Option<Vec3> {
        let ndc_x = 2.0 * px / application.config.width as f32 - 1.0;
        let ndc_y = 1.0 - 2.0 * py / application.config.height as f32;
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
        if ray_world.y >= 0.0 {
            return None;
        }
        let t = -ray_origin.y / ray_world.y;
        Some(ray_origin + t * ray_world)
    }

    fn mouse_world_position(&self, application: &visula::Application) -> Option<Vec3> {
        let p = self.mouse.position?;
        Self::screen_to_world(application, p.x as f32, p.y as f32)
    }

    /// Pan the camera by a touchpad pixel delta, converting to world-space offset.
    fn pan_camera_by_pixels(application: &mut visula::Application, dx: f32, dy: f32) {
        let cx = application.config.width as f32 / 2.0;
        let cy = application.config.height as f32 / 2.0;
        if let (Some(p0), Some(p1)) = (
            Self::screen_to_world(application, cx, cy),
            Self::screen_to_world(application, cx + dx, cy + dy),
        ) {
            let offset = p1 - p0;
            application.camera_controller.target_transform.center -= offset;
            application.camera_controller.current_transform.center =
                application.camera_controller.target_transform.center;
        }
    }

    /// Build an axon or glial process connection chain.
    fn handle_connection_tool(
        &mut self,
        mouse_position: Vec3,
        previous_too_near: bool,
        is_glial_process: bool,
        just_pressed: bool,
    ) -> ConnectResult {
        let snap = 2.0 * NODE_RADIUS;
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

                // Only snap to a source on an explicit click — not on cursor movement.
                if just_pressed {
                    self.connection_tool =
                        find_nearest_within(&source_candidates, mouse_position).map(
                            |(id, position)| ConnectionTool {
                                start: position,
                                end: mouse_position,
                                from: id,
                                waypoints: Vec::new(),
                            },
                        );
                    if let Some(ct) = &self.connection_tool {
                        self.previous_creation = Some(PreviousCreation { entity: ct.from });
                    }
                }
                ConnectResult::Continue
            }
            Some(ct) => {
                ct.end = mouse_position;

                // Target candidates differ by tool.
                // Always exclude the source entity to prevent self-connections: when the source
                // is a neuron soma, it would otherwise be the nearest target on the very first
                // frame, immediately completing a self-loop and locking out further drawing.
                let source_entity = ct.from;
                let target_candidates: Vec<(Entity, Vec3, f32)> = if is_glial_process {
                    // GlialProcess: connect to neurons only (vessels are now terrain, not entities)
                    world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
                        .iter()
                        .filter(|(e, _)| *e != source_entity)
                        .map(|(e, p)| (e, p.position, snap))
                        .collect()
                } else {
                    // Axon: connect to neurons only (not vessels, not glial cells)
                    world
                        .query::<&Position>()
                        .with::<&LeakyNeuron>()
                        .iter()
                        .filter(|(e, _)| *e != source_entity)
                        .map(|(e, p)| (e, p.position, snap))
                        .collect()
                };

                let nearest_target = find_nearest_within(&target_candidates, mouse_position);

                match nearest_target {
                    Some((id, target_pos)) => {
                        if is_glial_process {
                            let from_pos = world
                                .get::<&Position>(ct.from)
                                .map(|p| p.position)
                                .unwrap_or(target_pos);

                            // For non-compartment targets (neuron somas, blood vessels) always
                            // place a fixed bridge compartment just outside the target's surface.
                            let target_is_compartment = world.get::<&Compartment>(id).is_ok();
                            let target_vr = world
                                .get::<&VisualRadius>(id)
                                .map(|vr| vr.radius)
                                .unwrap_or(NODE_RADIUS);
                            let bridge_dist = target_vr + NODE_RADIUS * 0.5;
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
                                let bridge = world.spawn((
                                    Position { position: bridge_pos },
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
                                    GlialProcess,
                                    ConnectionColor(glial_color()),
                                ));
                                let already = world
                                    .query::<&Connection>()
                                    .iter()
                                    .any(|(_, c)| c.from == ct.from && c.to == bridge);
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
                                directional: false,
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
                        } else {
                            // Axon tool: spawn a growth cone that travels to the target,
                            // passing through any intermediate waypoints the player painted.
                            let from_pos = world
                                .get::<&Position>(ct.from)
                                .map(|p| p.position)
                                .unwrap_or(target_pos);
                            let neuron_type = world
                                .get::<&NeuronType>(ct.from)
                                .map(|t| (*t).clone())
                                .unwrap_or(NeuronType::Excitatory);
                            spawning::spawn_growth_cone(
                                world,
                                ct.from,
                                from_pos,
                                target_pos,
                                Some(id),
                                neuron_type,
                                ct.waypoints.clone(),
                                crate::components::PlayerId::Player1,
                            );
                            ConnectResult::Done
                        }
                    }
                    None => {
                        if is_glial_process {
                            // GlialProcess: click in empty space to lay a process compartment.
                            if previous_too_near {
                                return ConnectResult::Continue;
                            }
                            if !economy::try_spend_blocks(economy, COMPARTMENT_SPAWN_COST) {
                                return ConnectResult::FundsBlocked;
                            }
                            let neuron_type =
                                if let Ok(neuron_type) = world.get::<&NeuronType>(ct.from) {
                                    (*neuron_type).clone()
                                } else {
                                    NeuronType::Excitatory
                                };
                            let compartment_builder = world.spawn((
                                Position { position: mouse_position },
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
                                GlialProcess,
                                ConnectionColor(glial_color()),
                            ));
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
                            self.previous_creation =
                                Some(PreviousCreation { entity: compartment_builder });
                            ct.start = mouse_position;
                            ct.from = compartment_builder;
                            ConnectResult::Continue
                        } else {
                            // Axon tool: add a waypoint when the cursor has moved far enough
                            // from the last one, giving meaningful spacing between paint marks.
                            let axon_paint_spacing = GROWTH_CONE_COMP_SPACING;
                            let last_pos = ct.waypoints.last().copied().unwrap_or_else(|| {
                                world
                                    .get::<&Position>(ct.from)
                                    .map(|p| p.position)
                                    .unwrap_or(ct.start)
                            });
                            if mouse_position.distance(last_pos) >= axon_paint_spacing {
                                // Block building through impassable terrain (vessels, scars, CSF).
                                let passable = crate::simulation::pathfinding::terrain_at(
                                    mouse_position,
                                    &self.scenario_terrain,
                                )
                                .map(|t| t.passable)
                                .unwrap_or(true);
                                if passable {
                                    ct.waypoints.push(mouse_position);
                                }
                            }
                            ConnectResult::Continue
                        }
                    }
                }
            }
        }
    }

    /// Set `entity` as the manual attack target for all selected player-owned `MobileUnit`s.
    fn apply_attack_target(&mut self, target: Entity) {
        for &e in &self.selected_entities {
            if let Ok(ownership) = self.world.get::<&Ownership>(e) {
                if ownership.player == PlayerId::Player1 {
                    if let Ok(mut m) = self.world.get::<&mut MobileUnit>(e) {
                        m.manual_target = Some(target);
                        m.target = None; // reset AI target so next frame picks up manual_target
                    }
                }
            }
        }
    }

    /// Handle right-click:
    /// - If a connection is in progress (axon/glial) → cancel it.
    /// - If selected entities include neuroblasts → set their move destination.
    /// - Otherwise → set attack target for selected player mobile units.
    fn handle_right_click(&mut self, mouse_pos: Vec3) {
        // Right-click cancels any in-progress connection drawing.
        if self.connection_tool.is_some() {
            self.connection_tool = None;
            self.previous_creation = None;
            return;
        }
        // Check if any selected entities are neuroblasts.
        let has_neuroblasts = self
            .selected_entities
            .iter()
            .any(|&e| self.world.get::<&Neuroblast>(e).is_ok());

        if has_neuroblasts || self.awaiting_move_destination {
            self.awaiting_move_destination = false;
            let neuroblasts: Vec<Entity> = self
                .selected_entities
                .iter()
                .copied()
                .filter(|&e| self.world.get::<&Neuroblast>(e).is_ok())
                .collect();
            for entity in neuroblasts {
                production::set_neuroblast_destination(
                    &mut self.world,
                    &self.hex_grid,
                    entity,
                    mouse_pos,
                    &self.scenario_terrain,
                );
            }
            return;
        }

        let clicked = self
            .world
            .query::<&Position>()
            .iter()
            .min_by(|a, b| nearest(&mouse_pos, a, b))
            .and_then(|(id, pos)| {
                if mouse_pos.distance(pos.position) < SELECTION_RANGE {
                    Some(id)
                } else {
                    None
                }
            });
        if let Some(entity) = clicked {
            self.apply_attack_target(entity);
        }
        self.attack_mode = false;
    }

    fn handle_tool(&mut self, application: &mut visula::Application, just_pressed: bool) {
        if !self.mouse.left_down {
            // Mouse released — if we were painting an axon, spawn the growth cone now.
            if matches!(self.tool, GameTool::Axon) {
                if let Some(ct) = self.connection_tool.take() {
                    if !ct.waypoints.is_empty() {
                        let from_pos = self
                            .world
                            .get::<&Position>(ct.from)
                            .map(|p| p.position)
                            .unwrap_or(ct.start);
                        let neuron_type = self
                            .world
                            .get::<&NeuronType>(ct.from)
                            .map(|t| (*t).clone())
                            .unwrap_or(NeuronType::Excitatory);

                        // Snap endpoint to a nearby neuron if in range.
                        let last_pos = ct.end;
                        let target_snap = 2.0 * NODE_RADIUS;
                        let source = ct.from;
                        let target_candidates: Vec<(Entity, Vec3, f32)> = self
                            .world
                            .query::<&Position>()
                            .with::<&LeakyNeuron>()
                            .iter()
                            .filter(|(e, _)| *e != source)
                            .map(|(e, p)| (e, p.position, target_snap))
                            .collect();
                        let (target_pos, target_entity) =
                            find_nearest_within(&target_candidates, last_pos)
                                .map(|(id, pos)| (pos, Some(id)))
                                .unwrap_or((last_pos, None));

                        spawning::spawn_growth_cone(
                            &mut self.world,
                            ct.from,
                            from_pos,
                            target_pos,
                            target_entity,
                            neuron_type,
                            ct.waypoints,
                            crate::components::PlayerId::Player1,
                        );
                    }
                }
            }

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
                        let clicked = self
                            .world
                            .query::<&Position>()
                            .iter()
                            .min_by(|a, b| nearest(&mouse_position, a, b))
                            .and_then(|(id, pos)| {
                                if mouse_position.distance(pos.position) < SELECTION_RANGE {
                                    Some(id)
                                } else {
                                    None
                                }
                            });
                        if let Some(entity) = clicked {
                            // Consume the click so mouse-drag doesn't re-trigger selection.
                            self.move_origin = Some(mouse_position);
                            if self.attack_mode {
                                // Attack-mode: set as manual target for all selected player units.
                                self.apply_attack_target(entity);
                                self.attack_mode = false;
                            } else if self.keyboard.shift_down {
                                // Shift-click: toggle entity in/out of selection.
                                if self.selected_entities.contains(&entity) {
                                    self.selected_entities.retain(|&e| e != entity);
                                } else {
                                    self.selected_entities.push(entity);
                                }
                            } else {
                                // Plain click: replace selection.
                                self.selected_entities = vec![entity];
                            }
                        } else {
                            if !self.attack_mode {
                                self.selected_entities.clear();
                            }
                            self.attack_mode = false;
                            self.move_origin = Some(mouse_position);
                        }
                    }
                }
            }
            GameTool::Axon => {
                if self.connection_consumed_this_press && self.connection_tool.is_none() {
                    return;
                }
                match self.handle_connection_tool(mouse_position, previous_too_near, false, just_pressed) {
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
                match self.handle_connection_tool(mouse_position, previous_too_near, true, just_pressed) {
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
            if self.pending_voronoi_scenario {
                self.pending_voronoi_scenario = false;
                use neuronify_game_lib::voronoi_map::game_integration;
                let model = game_integration::load_voronoi_model();

                // Build terrain mesh.
                game_integration::build_voronoi_terrain_mesh(
                    &mut self.terrain_mesh, &model, &application.device,
                );

                // Set up entities, terrain hex lookup, and victory condition.
                use crate::simulation::voronoi_scenario;
                let (victory_cond, name, briefing, objective) =
                    voronoi_scenario::setup_voronoi_scenario(
                        &mut self.world,
                        &model,
                        &mut self.scenario_terrain,
                    );

                self.petri_dish.radius = 10000.0; // No dish boundary — the map itself is the boundary.
                self.victory_condition = victory_cond;
                self.victory_achieved = false;
                self.victory_check_timer = 1.0;
                self.game_elapsed = 0.0;
                application.camera_controller.target_transform.center = Vec3::ZERO;
                application.camera_controller.target_transform.forward = Vec3::new(-0.3536, -0.7071, 0.6124);
                application.camera_controller.target_transform.distance = 350.0;
                application.camera_controller.current_transform =
                    application.camera_controller.target_transform.clone();
                self.game_state = GameState::Briefing { name, briefing, objective };
            } else if let Some(svg_content) = self.pending_svg_content.take() {
                if let Err(e) = scenarios::setup_scenario_from_svg(&mut self.world, &mut self.petri_dish, svg_content) {
                    log::error!("Failed to load SVG scenario: {}", e);
                    scenarios::setup_scenario(&mut self.world, &self.petri_dish, self.current_scenario);
                }
                // Build terrain mesh and store terrain data for movement lookups.
                let briefing_state = match crate::map::parse_scenario_svg(svg_content) {
                    Ok(map) => {
                        rendering::build_terrain_mesh(
                            &mut self.terrain_mesh, &map, &application.device,
                        );
                        self.victory_condition = victory::parse_victory(&map.meta.victory);
                        self.scenario_terrain = map.terrain;
                        GameState::Briefing {
                            name: map.meta.name,
                            briefing: map.meta.briefing,
                            objective: map.meta.objective,
                        }
                    }
                    Err(e) => {
                        log::warn!("Terrain mesh skipped: {}", e);
                        GameState::InGame
                    }
                };
                // Apply dev stage modifications on top of the base SVG.
                if let Some(stage) = self.pending_dev_stage.take() {
                    dev_scenarios::apply_dev_stage(
                        &mut self.world,
                        &mut self.p1_economy,
                        stage,
                    );
                }
                self.victory_achieved = false;
                self.victory_check_timer = 1.0;
                self.game_elapsed = 0.0;
                // Aim camera along +z at 30° below horizontal to show the rotated SVG map.
                application.camera_controller.target_transform.center = Vec3::ZERO;
                application.camera_controller.target_transform.forward = Vec3::new(-0.3536, -0.7071, 0.6124);
                application.camera_controller.target_transform.distance = 300.0;
                application.camera_controller.current_transform =
                    application.camera_controller.target_transform.clone();
                self.game_state = briefing_state;
            } else {
                scenarios::setup_scenario(&mut self.world, &self.petri_dish, self.current_scenario);
                self.game_state = GameState::InGame;
            }
            self.tool = GameTool::Select;
            self.p1_economy = PlayerEconomy::default();
            self.selected_entities.clear();
            self.attack_mode = false;
        }

        if self.pending_exit_game {
            std::process::exit(0);
        }

        // Pause all simulation while in the main menu or showing the briefing.
        if matches!(self.game_state, GameState::MainMenu | GameState::Briefing { .. }) {
            return;
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
        transport::tick_glial_connect_neurons(&mut self.world);
        transport::harvest_glucose(&mut self.world, &self.scenario_terrain, frame_dt);
        transport::spawn_lactate_packets(&mut self.world, frame_dt);
        cytokines::emit_cytokines(&mut self.world, frame_dt as f32);
        cytokines::tick_cytokines(&mut self.world, frame_dt as f32);
        cytokines::activate_macrophages(&mut self.world);

        // Victory condition — checked once per second for performance.
        self.game_elapsed += frame_dt;
        self.victory_check_timer -= frame_dt;
        if self.victory_check_timer <= 0.0 {
            self.victory_check_timer = 1.0;
            if !self.victory_achieved {
                if let Some(cond) = &self.victory_condition {
                    if victory::check_victory(&self.world, cond) {
                        self.victory_achieved = true;
                    }
                }
            }
        }
        transport::move_lactate_packets(&mut self.world, frame_dt);
        economy::glial_contribute_blocks(&mut self.world, frame_dt, &mut self.p1_economy);
        metabolism::metabolic_drain(&mut self.world, frame_dt);
        metabolism::apply_dormancy(&mut self.world);
        metabolism::resource_flow(&mut self.world, frame_dt);
        cleanup::cleanup_orphans(&mut self.world);
        ownership::update_ownership(&mut self.world);

        // Combat systems use a fixed timestep (COMBAT_DT) so simulation speed is
        // identical on every machine regardless of display refresh rate.
        // Elapsed wall-clock time is accumulated and consumed in whole ticks;
        // the remainder carries over to the next frame.
        let elapsed = (Utc::now() - self.last_update)
            .num_microseconds()
            .unwrap_or(0) as f32
            * 1e-6;
        // Cap accumulated time to 200 ms to prevent a spiral of death after stalls.
        self.combat_accumulator = (self.combat_accumulator + elapsed).min(0.2);

        while self.combat_accumulator >= COMBAT_DT {
            self.combat_accumulator -= COMBAT_DT;
            let sim_dt = COMBAT_DT * GAME_SPEED;

            // Production / migration / maturation systems
            self.hex_grid.rebuild(&self.world);
            production::tick_production(&mut self.world, sim_dt);
            production::move_neuroblasts(&mut self.world, &mut self.hex_grid, &self.scenario_terrain, sim_dt);
            production::tick_neuroblast_repulsion(&mut self.world, sim_dt);
            production::tick_maturation(&mut self.world, sim_dt);
            production::advance_growth_cones(&mut self.world, &mut self.p1_economy, &self.scenario_terrain, sim_dt);

            combat::tick_dying_units(&mut self.world, sim_dt);
            combat::tick_slow_effects(&mut self.world, sim_dt);
            combat::apply_neuron_spawning(&mut self.world, sim_dt);
            combat::apply_enemy_spawn_points(&mut self.world, sim_dt);
            combat::move_mobile_units(&mut self.world, &self.scenario_terrain, sim_dt);
            combat::apply_unit_repulsion(&mut self.world);
            combat::apply_unit_repulsion(&mut self.world); // second pass for complete separation
            combat::apply_axon_cutting(&mut self.world, sim_dt);
            combat::apply_neuron_engulfment(&mut self.world, sim_dt);
            combat::apply_burst_attacks(&mut self.world, sim_dt);
            combat::advance_attack_projectiles(&mut self.world, sim_dt);
            combat::despawn_dead(&mut self.world);
        }

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

                // Draw each segment: source → waypoints → cursor.
                let mut prev = from_pos;
                for &wp in &ct.waypoints {
                    connections.push(ConnectionData {
                        position_a: prev,
                        position_b: wp,
                        strength: 1.0,
                        directional: 0.0,
                        start_color: preview_color,
                        end_color: preview_color,
                        _padding: Default::default(),
                    });
                    // Ghost sphere at each intermediate waypoint.
                    spheres.push(Sphere {
                        position: wp,
                        color: preview_color * 0.6,
                        radius: NODE_RADIUS * 0.35,
                        _padding: Default::default(),
                    });
                    prev = wp;
                }
                connections.push(ConnectionData {
                    position_a: prev,
                    position_b: ct.end,
                    strength: 1.0,
                    directional: 0.0, // ghost sphere handles the endpoint marker
                    start_color: preview_color,
                    end_color: preview_color,
                    _padding: Default::default(),
                });
            }
        }

        rendering::update_microglia_mesh(
            &mut self.microglia_mesh,
            &self.world,
            &application.device,
            self.time as f32,
        );
        rendering::update_astrocyte_mesh(
            &mut self.astrocyte_mesh,
            &self.world,
            &application.device,
            self.time as f32,
        );
        rendering::update_macrophage_mesh(
            &mut self.macrophage_mesh,
            &self.world,
            &application.device,
            self.time as f32,
        );
        let dendrite_cyls = rendering::collect_dendrite_cylinders(&self.world);
        self.dendrite_buffer.update(&application.device, &application.queue, &dendrite_cyls);
        let cam_forward = application.camera_controller.current_transform.forward;
        rendering::update_health_bar_meshes(
            &mut self.health_bar_meshes,
            &self.world,
            &application.device,
            cam_forward,
        );
        rendering::update_energy_bar_meshes(
            &mut self.energy_bar_meshes,
            &self.world,
            &application.device,
            self.selected_entities.first().copied(),
            cam_forward,
        );

        // Petri dish and decorations
        connections.extend(rendering::collect_petri_dish(&self.petri_dish));

        self.sphere_buffer
            .update(&application.device, &application.queue, &spheres);
        self.connection_buffer
            .update(&application.device, &application.queue, &connections);

        // FPS tracking
        let new_fps = 1.0
            / ((Utc::now() - self.last_update).num_nanoseconds().unwrap() as f64 * 1e-9)
                .max(0.0000001);
        self.fps = (1.0 - FPS_LOW_PASS_FACTOR) * self.fps + FPS_LOW_PASS_FACTOR * new_fps;
        self.last_update = Utc::now();
    }

    fn render(&mut self, data: &mut RenderData) {
        self.terrain_mesh.render(data);
        self.spheres.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
        self.dendrite_cylinders.render(data);
        self.microglia_mesh.render(data);
        self.astrocyte_mesh.render(data);
        self.macrophage_mesh.render(data);
        for mesh in &mut self.health_bar_meshes {
            mesh.render(data);
        }
        for mesh in &mut self.energy_bar_meshes {
            mesh.render(data);
        }
    }

    fn render_shadow(&mut self, data: &mut ShadowRenderData) {
        // Spheres and lines support shadow rendering via the Renderable trait.
        self.spheres.render_shadow(data);
        self.connection_spheres.render_shadow(data);
        self.dendrite_cylinders.render_shadow(data);
    }

    fn gui(&mut self, _application: &visula::Application, context: &egui::Context) {
        // Main menu: show scenario selector, skip game UI.
        if self.game_state == GameState::MainMenu {
            match main_menu::draw_main_menu(context, &self.menu_scenarios, &mut self.menu_selected, self.dev_mode) {
                main_menu::MenuAction::StartScenario(svg_content) => {
                    self.pending_svg_content = Some(svg_content);
                    self.pending_dev_stage = None;
                    self.pending_new_game = true;
                }
                main_menu::MenuAction::StartDevScenario(svg_content, stage) => {
                    self.pending_svg_content = Some(svg_content);
                    self.pending_dev_stage = Some(stage);
                    self.pending_new_game = true;
                }
                main_menu::MenuAction::StartVoronoiScenario => {
                    self.pending_voronoi_scenario = true;
                    self.pending_new_game = true;
                }
                main_menu::MenuAction::Exit => {
                    self.pending_exit_game = true;
                }
                main_menu::MenuAction::None => {}
            }
            return;
        }

        // Briefing overlay — shown before gameplay begins; sim is paused.
        if let GameState::Briefing { name, briefing, objective } = self.game_state.clone() {
            let dark = egui::Color32::from_rgba_premultiplied(10, 12, 18, 230);
            let amber = egui::Color32::from_rgb(255, 190, 60);
            let dim = egui::Color32::from_rgb(160, 160, 160);
            egui::Area::new(egui::Id::new("briefing_overlay"))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(context, |ui| {
                    egui::Frame::new()
                        .fill(dark)
                        .inner_margin(egui::Margin::symmetric(32, 24))
                        .corner_radius(egui::CornerRadius::same(4))
                        .show(ui, |ui| {
                            ui.set_max_width(520.0);
                            // Title
                            ui.label(
                                egui::RichText::new(name.to_uppercase())
                                    .color(amber)
                                    .size(22.0)
                                    .monospace(),
                            );
                            ui.add_space(12.0);
                            // Briefing section
                            ui.label(
                                egui::RichText::new("SITUATION")
                                    .color(dim)
                                    .size(11.0)
                                    .monospace(),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(&briefing)
                                    .color(egui::Color32::from_rgb(210, 210, 210))
                                    .size(14.0)
                                    .monospace(),
                            );
                            ui.add_space(14.0);
                            // Objective section
                            ui.label(
                                egui::RichText::new("OBJECTIVE")
                                    .color(dim)
                                    .size(11.0)
                                    .monospace(),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(&objective)
                                    .color(egui::Color32::from_rgb(120, 220, 80))
                                    .size(14.0)
                                    .monospace(),
                            );
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                let btn = egui::Button::new(
                                    egui::RichText::new("[ BEGIN ]")
                                        .color(egui::Color32::BLACK)
                                        .size(15.0)
                                        .monospace(),
                                )
                                .fill(amber)
                                .min_size(egui::Vec2::new(140.0, 36.0));
                                if ui.add(btn).clicked() {
                                    self.game_state = GameState::InGame;
                                }
                            });
                        });
                });
            return;
        }

        // Victory screen overlay — shown on top of the normal game UI.
        if self.victory_achieved {
            egui::Window::new("Victory!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(context, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("OBJECTIVE COMPLETE")
                                .color(egui::Color32::from_rgb(120, 220, 80))
                                .size(24.0),
                        );
                        ui.add_space(6.0);
                        let mins = (self.game_elapsed / 60.0) as u32;
                        let secs = (self.game_elapsed % 60.0) as u32;
                        ui.label(format!("Time: {:02}:{:02}", mins, secs));
                        ui.add_space(10.0);
                        if ui.button("Return to Menu").clicked() {
                            self.game_state = GameState::MainMenu;
                            self.victory_achieved = false;
                        }
                    });
                });
        }

        let mut p1_neurons = 0u32;
        let mut p1_energy = 0.0f64;
        for (_, (ownership, metab)) in self.world.query::<(&Ownership, &MetabolicState)>().iter() {
            if ownership.player == PlayerId::Player1 {
                p1_neurons += 1;
                p1_energy += metab.energy;
            }
        }

        if self.sidebar_icons.is_none() {
            self.sidebar_icons = Some(sidebar::SidebarIcons::new(context));
        }
        let mut pending_produce: Option<ProducibleItem> = None;
        let mut pending_cancel = false;
        let mut pending_menu = false;
        let queue_snapshot = production::queue_snapshot(&self.world);
        let tool_before = self.tool.clone();
        sidebar::draw_sidebar(
            context,
            &mut self.tool,
            p1_neurons,
            p1_energy,
            self.p1_economy.building_blocks,
            self.funds_flash_timer > 0.0,
            &mut pending_menu,
            &mut self.pending_exit_game,
            self.sidebar_icons.as_ref().unwrap(),
            &mut pending_produce,
            &mut pending_cancel,
            &queue_snapshot,
        );

        if pending_menu {
            self.game_state = GameState::MainMenu;
        }

        // Cancel in-progress connection drawing if the tool changed.
        if self.tool != tool_before {
            self.connection_tool = None;
            self.previous_creation = None;
        }

        // Handle produce button click — append to queue if room and funds available.
        if let Some(item) = pending_produce {
            if let Some(origin) = production::origin_entity(&self.world) {
                let queue_len = self.world
                    .get::<&ProductionQueue>(origin)
                    .map(|q| q.items.len())
                    .unwrap_or(0);
                if queue_len < QUEUE_MAX_SIZE {
                    if economy::try_spend_blocks(&mut self.p1_economy, item.cost()) {
                        if let Ok(mut queue) = self.world.get::<&mut ProductionQueue>(origin) {
                            queue.items.push_back(QueuedItem {
                                item,
                                timer: 0.0,
                                duration: item.build_duration(),
                            });
                        }
                    } else {
                        self.funds_flash_timer = 0.5;
                    }
                }
            }
        }

        // Handle cancel — remove last queued item and refund cost.
        if pending_cancel {
            if let Some(origin) = production::origin_entity(&self.world) {
                if let Ok(mut queue) = self.world.get::<&mut ProductionQueue>(origin) {
                    if let Some(last) = queue.items.pop_back() {
                        economy::add_blocks(&mut self.p1_economy, last.item.cost());
                    }
                }
            }
        }

        // Remove any stale selected entities (despawned during combat).
        self.selected_entities.retain(|&e| self.world.contains(e));

        // Info panel for selected entities.
        if !self.selected_entities.is_empty() {
            let action = crate::ui::info_panel::draw_info_panel(
                context,
                &self.world,
                &self.selected_entities,
                self.attack_mode,
            );
            match action {
                crate::ui::info_panel::InfoPanelAction::EnterAttackMode => {
                    self.attack_mode = true;
                }
                crate::ui::info_panel::InfoPanelAction::ClearManualTargets => {
                    for &e in &self.selected_entities {
                        if let Ok(mut m) = self.world.get::<&mut MobileUnit>(e) {
                            m.manual_target = None;
                        }
                    }
                }
                crate::ui::info_panel::InfoPanelAction::GoToDestination => {
                    self.awaiting_move_destination = true;
                }
                crate::ui::info_panel::InfoPanelAction::PlantNeuroblast => {
                    for &e in &self.selected_entities {
                        if self.world.get::<&Neuroblast>(e).is_ok()
                            && self.world.get::<&MovePath>(e).is_err()
                        {
                            let cell_type = self.world
                                .get::<&Neuroblast>(e)
                                .ok()
                                .map(|nb| nb.cell_type)
                                .unwrap_or(ProducibleCell::ExcitatoryNeuroblast);
                            let _ = self.world.remove_one::<Neuroblast>(e);
                            let _ = self.world.insert(e, (MaturingNeuron {
                                timer: 0.0,
                                duration: 2.0,
                                cell_type,
                            },));
                        }
                    }
                }
                crate::ui::info_panel::InfoPanelAction::None => {}
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
                // just_pressed is true only on the initial press, not on release.
                self.handle_tool(application, self.mouse.left_down);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Right,
                    ..
                },
                ..
            } => {
                if let Some(mouse_pos) = self.mouse_world_position(application) {
                    self.handle_right_click(mouse_pos);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                self.keyboard.shift_down = state.lshift_state() == ModifiersKeyState::Pressed
                    || state.rshift_state() == ModifiersKeyState::Pressed;
                self.keyboard.ctrl_down = state.lcontrol_state() == ModifiersKeyState::Pressed
                    || state.rcontrol_state() == ModifiersKeyState::Pressed;
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
                self.handle_tool(application, false);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                match delta {
                    // Physical mouse wheel → zoom.
                    visula::winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        let scroll = *y;
                        application.camera_controller.target_transform.distance *=
                            1.0 - scroll * 0.1;
                        application.camera_controller.target_transform.distance = application
                            .camera_controller
                            .target_transform
                            .distance
                            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
                    }
                    // Touchpad two-finger swipe → pan; Ctrl+swipe → zoom (pinch workaround).
                    visula::winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        if self.keyboard.ctrl_down {
                            // Ctrl+scroll is the standard compositor mapping for touchpad pinch
                            // on Linux (e.g. via libinput-gestures or KDE/GNOME settings).
                            let scroll = pos.y as f32 / 100.0;
                            application.camera_controller.target_transform.distance *=
                                1.0 - scroll * 0.5;
                            application.camera_controller.target_transform.distance = application
                                .camera_controller
                                .target_transform
                                .distance
                                .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
                        } else {
                            Self::pan_camera_by_pixels(application, pos.x as f32, pos.y as f32);
                        }
                    }
                }
            }
            // Touchpad pinch gesture (macOS / iOS) → zoom.
            Event::WindowEvent {
                event: WindowEvent::PinchGesture { delta, .. },
                ..
            } => {
                let zoom = *delta as f32;
                application.camera_controller.target_transform.distance *= 1.0 - zoom * 0.5;
                application.camera_controller.target_transform.distance = application
                    .camera_controller
                    .target_transform
                    .distance
                    .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
            }
            // Touchscreen multi-touch: two fingers pan + pinch zoom.
            Event::WindowEvent {
                event: WindowEvent::Touch(Touch { id, phase, location, .. }),
                ..
            } => {
                match phase {
                    TouchPhase::Started => {
                        self.touches.insert(*id, *location);
                    }
                    TouchPhase::Moved => {
                        if let Some(&prev) = self.touches.get(id) {
                            if self.touches.len() == 2 {
                                // Find the other finger's current position.
                                let other = self
                                    .touches
                                    .iter()
                                    .find(|(&fid, _)| fid != *id)
                                    .map(|(_, &pos)| pos);

                                if let Some(other) = other {
                                    if self.keyboard.ctrl_down {
                                        // Ctrl + two fingers = zoom (pinch distance ratio).
                                        let old_d = ((prev.x - other.x).powi(2)
                                            + (prev.y - other.y).powi(2))
                                        .sqrt();
                                        let new_d = ((location.x - other.x).powi(2)
                                            + (location.y - other.y).powi(2))
                                        .sqrt();
                                        if old_d > 0.0 {
                                            let scale = (new_d / old_d) as f32;
                                            application
                                                .camera_controller
                                                .target_transform
                                                .distance /= scale;
                                            application
                                                .camera_controller
                                                .target_transform
                                                .distance = application
                                                .camera_controller
                                                .target_transform
                                                .distance
                                                .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
                                            application
                                                .camera_controller
                                                .current_transform
                                                .distance = application
                                                .camera_controller
                                                .target_transform
                                                .distance;
                                        }
                                    } else {
                                        // Two fingers without Ctrl = pan (centroid delta).
                                        let old_cx = (prev.x + other.x) / 2.0;
                                        let old_cy = (prev.y + other.y) / 2.0;
                                        let new_cx = (location.x + other.x) / 2.0;
                                        let new_cy = (location.y + other.y) / 2.0;
                                        let dx = (new_cx - old_cx) as f32;
                                        let dy = (new_cy - old_cy) as f32;
                                        Self::pan_camera_by_pixels(application, dx, dy);
                                    }
                                }
                            }
                            self.touches.insert(*id, *location);
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.touches.remove(id);
                    }
                }
            }
            _ => {}
        }
    }
}
