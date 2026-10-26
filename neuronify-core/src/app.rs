use crate::camera;
use crate::constants::*;
use crate::editor::{Editor, Pan};
use crate::examples::Example;
use crate::gui;
#[cfg(not(target_arch = "wasm32"))]
use crate::gui::MenuChoice;
use crate::landing::{LandingChoice, LandingScreen};
use crate::rendering::lighting::{configure_lighting, set_theme_lighting, shaded};
use crate::rendering::{
    canvas_bounds, collect_scene, collect_voltmeter_traces, collect_voltmeter_wires,
    connection_being_drawn, create_floor, ConnectionData, CylinderData, GaugeRenderer,
    NoteRenderer, Sphere, Theme, TorusData,
};
use chrono::{DateTime, Duration, Utc};
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
    pub editor: Editor,
    pub theme: Theme,
    /// The theme the floor and lighting were last set up for
    applied_theme: Theme,
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
    pub load_error: Option<String>,
    pub note_renderer: NoteRenderer,
    pub gauge_renderer: GaugeRenderer,
    pub landing: Option<LandingScreen>,
    pub frame_scene_requested: bool,
}

#[derive(Debug)]
pub struct Error {}

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

impl Neuronify {
    pub fn new(application: &mut visula::Application) -> Neuronify {
        camera::set_up(application);

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
            editor: Editor::default(),
            theme,
            applied_theme: theme,
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

    fn sync_pointer_with_gui(&mut self, context: &egui::Context) {
        let held = context.input(|input| input.pointer.any_down() || input.any_touches());
        if !held {
            self.touches = Touches::default();
            self.mouse.left_down = false;
            self.editor.release();
        }
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            self.editor.release();
            return;
        }
        let Some(screen_position) = self.mouse.position else {
            return;
        };
        let position = camera::ground_position(application, screen_position);
        let requested_pan = if self.edit_enabled {
            self.editor
                .edit(&mut self.world, position, self.keyboard.shift_down)
        } else {
            self.editor.view(&self.world, position)
        };
        if let Some(Pan(offset)) = requested_pan {
            camera::pan(application, offset);
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
        self.editor.release();
        self.editor.selection = None;
        self.landing = None;
        self.frame_scene_requested = true;
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

        if let Some(position) = self.editor.touch() {
            simulation::stimulate_nearby(&mut self.world, position);
        }
        simulation::step_frame(&mut self.world, &mut self.time, self.iterations);

        if let Some(landing) = &mut self.landing {
            landing.update(self.iterations);
        }
        if self.frame_scene_requested {
            self.frame_scene_requested = false;
            camera::frame(application, canvas_bounds(&self.world));
        }

        let camera = &application.camera_controller.current_transform;
        application.light.shadow_center = camera.center;
        // visula blurs shadows over a fixed number of shadow map texels,
        // so a map that reaches well past the view gives softer shadows
        application.light.shadow_extent = (camera.distance * 2.0).clamp(50.0, 625.0);

        let world = &self.world;
        let palette = self.theme.palette();
        let scene = collect_scene(world, &palette);
        let mut connections = collect_voltmeter_wires(world, &palette);
        if let Some((start, end)) = self.editor.connection_being_drawn() {
            connections.push(connection_being_drawn(start, end, &palette));
        }
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
            gui::load_error(context, error);
        }
        if gui::corner_buttons(context, &mut self.edit_enabled) {
            self.landing = Some(LandingScreen::new(true));
        }
        if self.edit_enabled {
            #[cfg(not(target_arch = "wasm32"))]
            match gui::menu_bar(context) {
                Some(MenuChoice::Save(path)) => self.save(path),
                Some(MenuChoice::Open(path)) => self.loadfile(path),
                Some(MenuChoice::Example(example)) => self.load_example(example),
                None => {}
            }
            gui::elements_window(context, &mut self.editor);
            gui::settings_window(context, self.fps, &mut self.iterations, &mut self.theme);
            if let Some(selection) = self.editor.selection {
                gui::selection_window(context, &self.world, selection);
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
                        camera::pinch(application, previous, current)
                    }
                    TouchGesture::None => {}
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => camera::scroll(application, delta),
            _ => {}
        }
    }
}
