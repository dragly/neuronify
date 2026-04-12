use glam::{Quat, Vec3};
use visula::winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use visula::{
    CustomEvent, Expression, InstanceBuffer, LineGeometry, LineMaterial, Lines, MeshGeometry,
    MeshMaterial, MeshPipeline, RenderData, Renderable, SphereGeometry, SphereMaterial, Spheres,
};
use wgpu::util::DeviceExt;

use neuronify_core::rendering::gpu_types::{ConnectionData, Sphere};
use neuronify_game_lib::voronoi_map::{
    map_gen,
    mesh_builder,
    terrain_model::*,
};

use super::editor;
use super::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Orbit,
    EditVertex,
    PaintTerrain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    FullMap,
    TopDown,
    TileCatalog,
}

#[derive(Debug)]
pub struct Error {}

pub struct MapEditorApp {
    // Data model
    pub model: MapModel,

    // Rendering
    pub terrain_mesh: MeshPipeline,
    pub catalog_mesh: MeshPipeline,
    pub wireframe_lines: Lines,
    pub wireframe_buffer: InstanceBuffer<ConnectionData>,
    pub edit_spheres: Spheres,
    pub edit_sphere_buffer: InstanceBuffer<Sphere>,

    // Editor state
    pub mode: Mode,
    pub view: View,
    pub selected_terrain: EditorTerrain,
    pub show_wireframe: bool,
    pub height_only: bool,

    // Camera
    pub theta: f32,
    pub phi: f32,
    pub dist: f32,
    pub camera_center: Vec3,
    pub dragging_camera: bool,
    pub panning_camera: bool,
    /// World-space point on the ground plane that should stay under the cursor.
    pub pan_grab_point: Option<Vec3>,
    pub last_mouse_x: f64,
    pub last_mouse_y: f64,

    // Edit state
    pub picked_world_key: Option<String>,
    pub pick_screen_y: f64,
    /// The tile's affine transform at pick time: c1=(pb-pa), c2 computed from pc.
    /// Used to convert world XZ deltas to local (lx, ly) deltas.
    pub pick_c1: [f32; 2],
    pub pick_c2: [f32; 2],
    /// Previous world-space ground hit for incremental XZ delta.
    pub pick_prev_world: Option<Vec3>,
    pub mesh_dirty: bool,
    pub dots_dirty: bool,
    pub catalog_dirty: bool,
    pub wireframe_dirty: bool,

    // Mouse tracking
    pub mouse_pos: Option<(f64, f64)>,
    pub mouse_left_down: bool,
    pub ctrl_down: bool,

    // Cached mesh data for editing
    pub cached_mesh_data: Option<mesh_builder::MeshData>,
    pub cached_catalog_data: Option<mesh_builder::MeshData>,
}

impl MapEditorApp {
    pub fn new(application: &mut visula::Application) -> Self {
        application.camera_controller.enabled = false;
        application.camera_controller.target_transform.center = Vec3::new(0.0, 0.0, 0.0);
        application.camera_controller.target_transform.forward =
            Vec3::new(-0.3536, -0.7071, 0.6124);
        application.camera_controller.target_transform.distance = 350.0;
        application.camera_controller.current_transform =
            application.camera_controller.target_transform.clone();

        let wireframe_buffer = InstanceBuffer::<ConnectionData>::new(&application.device);
        let wf = wireframe_buffer.instance();
        let wireframe_lines = Lines::new(
            &application.rendering_descriptor(),
            &LineGeometry {
                start: wf.position_a.clone(),
                end: wf.position_b.clone(),
                width: Expression::from(0.3),
                color: wf.start_color.clone(),
            },
            &LineMaterial {
                color: Expression::InputColor.lit(),
            },
        )
        .unwrap();

        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let edit_spheres = Spheres::new(
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

        let terrain_mesh = MeshPipeline::new(
            &application.rendering_descriptor(),
            &MeshGeometry {
                position: Vec3::ZERO.into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::ONE.into(),
            },
            &MeshMaterial {
                color: Expression::InputColor.lit(),
            },
        )
        .unwrap();

        let catalog_mesh = MeshPipeline::new(
            &application.rendering_descriptor(),
            &MeshGeometry {
                position: Vec3::ZERO.into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::ONE.into(),
            },
            &MeshMaterial {
                color: Expression::InputColor.lit(),
            },
        )
        .unwrap();

        let model = map_gen::generate_scenario_map();

        let app = MapEditorApp {
            model,
            terrain_mesh,
            catalog_mesh,
            wireframe_lines,
            wireframe_buffer,
            edit_spheres,
            edit_sphere_buffer: sphere_buffer,
            mode: Mode::Orbit,
            view: View::FullMap,
            selected_terrain: EditorTerrain::Vessel,
            show_wireframe: false,
            height_only: false,
            theta: 0.8,
            phi: 0.6,
            dist: 350.0,
            camera_center: Vec3::ZERO,
            dragging_camera: false,
            panning_camera: false,
            pan_grab_point: None,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            picked_world_key: None,
            pick_screen_y: 0.0,
            pick_c1: [0.0; 2],
            pick_c2: [0.0; 2],
            pick_prev_world: None,
            mesh_dirty: true,
            dots_dirty: true,
            catalog_dirty: true,
            wireframe_dirty: true,
            mouse_pos: None,
            mouse_left_down: false,
            ctrl_down: false,
            cached_mesh_data: None,
            cached_catalog_data: None,
        };

        app
    }

    fn rebuild_mesh(&mut self, device: &wgpu::Device) {
        let data = mesh_builder::build_map_mesh(
            &self.model,
            map_gen::MAP_W,
            map_gen::MAP_H,
            map_gen::CELL_SPACING,
        );

        if data.vertices.is_empty() {
            self.terrain_mesh.vertex_count = 0;
            return;
        }

        self.terrain_mesh.vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("voronoi_terrain"),
                contents: bytemuck::cast_slice(&data.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.terrain_mesh.index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("voronoi_terrain_idx"),
                contents: bytemuck::cast_slice(&data.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        self.terrain_mesh.vertex_count = data.indices.len();

        self.cached_mesh_data = Some(data);
        self.mesh_dirty = false;
        self.dots_dirty = true;
        self.catalog_dirty = true;
        self.wireframe_dirty = true;
    }

    fn rebuild_catalog(&mut self, device: &wgpu::Device) {
        let data = mesh_builder::build_tile_catalog(&self.model);
        if data.vertices.is_empty() {
            self.catalog_mesh.vertex_count = 0;
            self.cached_catalog_data = None;
            self.catalog_dirty = false;
            return;
        }
        self.catalog_mesh.vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("catalog"),
                contents: bytemuck::cast_slice(&data.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.catalog_mesh.index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("catalog_idx"),
                contents: bytemuck::cast_slice(&data.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        self.catalog_mesh.vertex_count = data.indices.len();
        self.cached_catalog_data = Some(data);
        self.catalog_dirty = false;
        self.dots_dirty = true;
    }

    fn rebuild_wireframe(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let edges = mesh_builder::build_wireframe_edges(
            &self.model,
            map_gen::MAP_W,
            map_gen::MAP_H,
            map_gen::CELL_SPACING,
        );
        let line_color = Vec3::new(1.0, 0.7, 0.2);
        let line_data: Vec<ConnectionData> = edges
            .iter()
            .map(|(start, end)| ConnectionData {
                start_color: line_color,
                end_color: line_color,
                position_a: Vec3::new(start[0], start[1], start[2]),
                position_b: Vec3::new(end[0], end[1], end[2]),
                strength: 1.0,
                directional: 0.0,
                _padding: [0.0; 2],
            })
            .collect();
        self.wireframe_buffer.update(device, queue, &line_data);
        self.wireframe_dirty = false;
    }

    fn rebuild_edit_dots(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.mode != Mode::EditVertex {
            self.edit_sphere_buffer.update(device, queue, &[]);
            self.dots_dirty = false;
            return;
        }

        let data = match self.view {
            View::TileCatalog => self.cached_catalog_data.as_ref(),
            _ => self.cached_mesh_data.as_ref(),
        };
        let data = match data {
            Some(d) => d,
            None => {
                self.dots_dirty = false;
                return;
            }
        };

        let mut seen = std::collections::HashSet::new();
        let mut spheres = Vec::new();

        for vi in 0..data.vertex_world_keys.len() {
            let wk = &data.vertex_world_keys[vi];
            if wk == "wall" || seen.contains(wk) {
                continue;
            }
            let cls = match data.world_vert_cls.get(wk) {
                Some(c) if !c.is_empty() => c,
                _ => continue,
            };
            seen.insert(wk.clone());

            let v = &data.vertices[vi];
            let is_corner = cls
                .iter()
                .any(|c| matches!(c, VertexClassification::Corner { .. }));
            let is_edge = cls
                .iter()
                .any(|c| matches!(c, VertexClassification::Edge { .. }));
            let color = if is_corner {
                Vec3::new(0.4, 0.9, 1.0) // cyan for corners (shared per terrain type)
            } else if is_edge {
                Vec3::new(1.0, 0.8, 0.2) // yellow for edge vertices
            } else {
                Vec3::new(1.0, 1.0, 1.0) // white for interior
            };

            spheres.push(Sphere {
                position: Vec3::new(v.position[0], v.position[1] + 0.5, v.position[2]),
                color,
                radius: 1.2,
                _padding: 0.0,
            });
        }

        self.edit_sphere_buffer.update(device, queue, &spheres);
        self.dots_dirty = false;
    }

    fn update_camera(&self, application: &mut visula::Application) {
        let sin_phi = self.phi.sin();
        let cos_phi = self.phi.cos();
        let sin_theta = self.theta.sin();
        let cos_theta = self.theta.cos();

        let forward = Vec3::new(
            -sin_phi * cos_theta,
            -cos_phi,
            -sin_phi * sin_theta,
        );

        application.camera_controller.target_transform.center = self.camera_center;
        application.camera_controller.target_transform.forward = forward;
        application.camera_controller.target_transform.distance = self.dist;
        application.camera_controller.current_transform =
            application.camera_controller.target_transform.clone();
    }

    fn screen_to_world(application: &visula::Application, px: f32, py: f32) -> Option<Vec3> {
        let ndc_x = 2.0 * px / application.config.width as f32 - 1.0;
        let ndc_y = 1.0 - 2.0 * py / application.config.height as f32;
        let ray_clip = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let aspect = application.config.width as f32 / application.config.height as f32;
        let inv_proj = application
            .camera_controller
            .projection_matrix(aspect)
            .inverse();
        let ray_eye = inv_proj * ray_clip;
        let ray_eye = glam::Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
        let inv_view = application.camera_controller.view_matrix().inverse();
        let ray_world = inv_view * ray_eye;
        let ray_dir = Vec3::new(ray_world.x, ray_world.y, ray_world.z).normalize();
        let ray_origin = application.camera_controller.position();

        if ray_dir.y >= 0.0 {
            return None;
        }
        let t = -ray_origin.y / ray_dir.y;
        Some(ray_origin + t * ray_dir)
    }
}

impl visula::Simulation for MapEditorApp {
    type Error = Error;

    fn clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: 0.094,
            g: 0.094,
            b: 0.11,
            a: 1.0,
        }
    }

    fn update(&mut self, application: &mut visula::Application) {
        if self.mesh_dirty {
            self.rebuild_mesh(&application.device);
        }
        if self.catalog_dirty && self.view == View::TileCatalog {
            self.rebuild_catalog(&application.device);
        }
        if self.wireframe_dirty && self.show_wireframe {
            self.rebuild_wireframe(&application.device, &application.queue);
        }
        if self.dots_dirty {
            self.rebuild_edit_dots(&application.device, &application.queue);
        }
        self.update_camera(application);
    }

    fn render(&mut self, data: &mut RenderData) {
        match self.view {
            View::FullMap | View::TopDown => {
                self.terrain_mesh.render(data);
                if self.show_wireframe {
                    self.wireframe_lines.render(data);
                }
                if self.mode == Mode::EditVertex {
                    self.edit_spheres.render(data);
                }
            }
            View::TileCatalog => {
                self.catalog_mesh.render(data);
                if self.mode == Mode::EditVertex {
                    self.edit_spheres.render(data);
                }
            }
        }
    }

    fn gui(&mut self, application: &visula::Application, context: &egui::Context) {
        ui::draw_ui(self, application, context);
    }

    fn handle_event(
        &mut self,
        application: &mut visula::Application,
        event: &Event<CustomEvent>,
    ) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = *state == ElementState::Pressed;

                    if *button == MouseButton::Right {
                        if pressed {
                            self.panning_camera = true;
                            self.pan_grab_point = self.mouse_pos.and_then(|(mx, my)| {
                                Self::screen_to_world(application, mx as f32, my as f32)
                            });
                        } else {
                            self.panning_camera = false;
                            self.pan_grab_point = None;
                        }
                    }

                    if *button == MouseButton::Left {
                        self.mouse_left_down = pressed;

                        if pressed {
                            if self.mode == Mode::EditVertex {
                                // Try to pick a vertex from the active view's mesh data.
                                let active_data = match self.view {
                                    View::TileCatalog => &self.cached_catalog_data,
                                    _ => &self.cached_mesh_data,
                                };
                                if let Some((mx, my)) = self.mouse_pos {
                                    if let Some(wk) = editor::pick_vertex(
                                        application,
                                        active_data,
                                        mx as f32,
                                        my as f32,
                                    ) {
                                        self.picked_world_key = Some(wk.clone());
                                        self.pick_screen_y = my;
                                        self.pick_prev_world = Self::screen_to_world(
                                            application, mx as f32, my as f32,
                                        );
                                        // Store the tile's affine transform for
                                        // world → local conversion.
                                        if let Some(d) = active_data.as_ref() {
                                            if let Some(c1) = d.world_key_c1.get(&wk) {
                                                self.pick_c1 = *c1;
                                            }
                                            if let Some(c2) = d.world_key_c2.get(&wk) {
                                                self.pick_c2 = *c2;
                                            }
                                        }
                                        return;
                                    }
                                }
                            }

                            if self.mode == Mode::PaintTerrain {
                                if let Some((mx, my)) = self.mouse_pos {
                                    if let Some(world_pos) =
                                        Self::screen_to_world(application, mx as f32, my as f32)
                                    {
                                        editor::paint_cell(
                                            &mut self.model,
                                            world_pos,
                                            self.selected_terrain,
                                            map_gen::MAP_W,
                                            map_gen::MAP_H,
                                        );
                                        self.mesh_dirty = true;
                                    }
                                }
                                return;
                            }

                            // Start camera orbit.
                            self.dragging_camera = true;
                            if let Some((mx, my)) = self.mouse_pos {
                                self.last_mouse_x = mx;
                                self.last_mouse_y = my;
                            }
                        } else {
                            self.dragging_camera = false;
                            self.picked_world_key = None;
                        }
                    }
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let mx = position.x;
                    let my = position.y;
                    self.mouse_pos = Some((mx, my));

                    // Vertex dragging in edit mode.
                    // Height: from screen Y delta.
                    // XZ: raycast to ground, get world delta, invert the
                    //     tile's affine transform to get local (dlx, dly).
                    if self.mode == Mode::EditVertex {
                        if let Some(ref wk) = self.picked_world_key {
                            // Height from screen Y.
                            let screen_dy = (my - self.pick_screen_y) as f32;
                            self.pick_screen_y = my;
                            let delta_height = -screen_dy * 0.2;

                            // XZ from world raycast + inverse affine.
                            let mut delta_local_x = 0.0f32;
                            let mut delta_local_y = 0.0f32;
                            if let Some(prev) = self.pick_prev_world {
                                if let Some(curr) = Self::screen_to_world(
                                    application, mx as f32, my as f32,
                                ) {
                                    self.pick_prev_world = Some(curr);
                                    // World delta in the map's 2D plane.
                                    // world_x corresponds to map +X,
                                    // world_z corresponds to map -Y (negated).
                                    let dwx = curr.x - prev.x;
                                    let dwy = -(curr.z - prev.z); // map Y
                                    // Invert the 2x2 affine [c1x c2x; c1y c2y]
                                    // to get (dlx, dly) from (dwx, dwy).
                                    let [c1x, c1y] = self.pick_c1;
                                    let [c2x, c2y] = self.pick_c2;
                                    let det = c1x * c2y - c2x * c1y;
                                    if det.abs() > 1e-6 {
                                        delta_local_x = ( c2y * dwx - c2x * dwy) / det;
                                        delta_local_y = (-c1y * dwx + c1x * dwy) / det;
                                    }
                                }
                            }

                            if self.height_only {
                                delta_local_x = 0.0;
                                delta_local_y = 0.0;
                            }

                            let active_data = match self.view {
                                View::TileCatalog => &self.cached_catalog_data,
                                _ => &self.cached_mesh_data,
                            };
                            editor::drag_vertex(
                                &mut self.model,
                                wk,
                                delta_height,
                                delta_local_x,
                                delta_local_y,
                                active_data,
                            );
                            self.mesh_dirty = true;
                            return;
                        }
                    }

                    // Terrain painting while dragging.
                    if self.mode == Mode::PaintTerrain && self.mouse_left_down {
                        if let Some(world_pos) =
                            Self::screen_to_world(application, mx as f32, my as f32)
                        {
                            editor::paint_cell(
                                &mut self.model,
                                world_pos,
                                self.selected_terrain,
                                map_gen::MAP_W,
                                map_gen::MAP_H,
                            );
                            self.mesh_dirty = true;
                        }
                        return;
                    }

                    // Camera pan (right-click drag): keep the grab point
                    // pinned under the cursor.
                    if self.panning_camera {
                        if let Some(grab) = self.pan_grab_point {
                            if let Some(current) = Self::screen_to_world(application, mx as f32, my as f32) {
                                let delta = grab - current;
                                self.camera_center += delta;
                                // Immediately update the camera controller so
                                // that subsequent mouse events in the same
                                // frame see the new camera position (avoids
                                // jitter from stale projection).
                                self.update_camera(application);
                            }
                        }
                    }

                    // Camera orbit (left-click drag).
                    if self.dragging_camera {
                        let dx = mx - self.last_mouse_x;
                        let dy = my - self.last_mouse_y;
                        self.theta += dx as f32 * 0.008;
                        self.phi = (self.phi - dy as f32 * 0.008).clamp(0.15, 1.4);
                        self.last_mouse_x = mx;
                        self.last_mouse_y = my;
                    }
                }

                WindowEvent::ModifiersChanged(state) => {
                    use visula::winit::keyboard::ModifiersKeyState;
                    self.ctrl_down = state.lcontrol_state() == ModifiersKeyState::Pressed
                        || state.rcontrol_state() == ModifiersKeyState::Pressed;
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    // All scroll events → zoom. Two-finger pan deferred until
                    // winit has proper Wayland multitouch support.
                    let scroll = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 40.0,
                    };
                    self.dist *= 1.0 - scroll * 0.1;
                    self.dist = self.dist.clamp(50.0, 1200.0);
                }

                _ => {}
            },
            _ => {}
        }
    }
}
