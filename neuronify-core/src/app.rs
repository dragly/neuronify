use crate::camera;
use crate::editor::{Editor, Pan};
use crate::examples::Example;
use crate::frame_clock::FrameClock;
use crate::gui;
#[cfg(not(target_arch = "wasm32"))]
use crate::gui::MenuChoice;
use crate::landing::{LandingChoice, LandingScreen};
use crate::rendering::{canvas_bounds, SceneRenderer, Theme};
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use visula::{winit::keyboard::ModifiersKeyState, CustomEvent, RenderData, ShadowRenderData};

use crate::input::{Keyboard, Mouse, TouchGesture, Touches};
use crate::simulation;

pub struct Neuronify {
    pub edit_enabled: bool,
    pub landing: Option<LandingScreen>,
    editor: Editor,
    theme: Theme,
    world: hecs::World,
    time: f64,
    iterations: u32,
    mouse: Mouse,
    keyboard: Keyboard,
    touches: Touches,
    renderer: SceneRenderer,
    frame_clock: FrameClock,
    load_error: Option<String>,
    frame_scene_requested: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn world_from_command_line() -> Option<hecs::World> {
    let path = std::env::args().nth(1)?;
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| log::error!("Failed to read file {}: {}", path, e))
        .ok()?;
    let simulation = crate::legacy::parse_legacy_nfy(&contents)
        .map_err(|e| log::error!("Failed to parse legacy file {}: {}", path, e))
        .ok()?;
    log::info!(
        "Loaded legacy simulation from {}: {} nodes, {} edges",
        path,
        simulation.nodes.len(),
        simulation.edges.len()
    );
    let mut world = hecs::World::new();
    crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
    Some(world)
}

#[cfg(target_arch = "wasm32")]
fn world_from_command_line() -> Option<hecs::World> {
    None
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

        let theme = match application.window.theme() {
            Some(visula::winit::window::Theme::Light) => Theme::Light,
            _ => Theme::Dark,
        };
        let renderer = SceneRenderer::new(application, theme);

        let opened = world_from_command_line();
        Neuronify {
            edit_enabled: true,
            landing: opened.is_none().then(|| LandingScreen::new(false)),
            editor: Editor::default(),
            theme,
            frame_scene_requested: opened.is_some(),
            world: opened.unwrap_or_default(),
            time: 0.0,
            iterations: 4,
            mouse: Mouse::default(),
            keyboard: Keyboard::default(),
            touches: Touches::default(),
            renderer,
            frame_clock: FrameClock::default(),
            load_error: None,
        }
    }

    fn press_pointer(&mut self, application: &mut visula::Application, pressed: bool) {
        self.mouse.left_down = pressed;
        self.handle_tool(application);
    }

    fn move_pointer(
        &mut self,
        application: &mut visula::Application,
        position: PhysicalPosition<f64>,
    ) {
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
        let bytes = crate::serialization::save(&self.world, self.time).unwrap();
        let mut writer = std::fs::File::create(path.with_extension("neuronify")).unwrap();
        writer.write_all(&bytes).unwrap();
    }

    pub fn load_example(&mut self, example: &Example) {
        match example.spawn() {
            Ok(world) => self.open_world(world, 0.0),
            Err(e) => log::error!("Failed to load example {}: {}", example.name, e),
        }
    }

    fn open_world(&mut self, world: hecs::World, time: f64) {
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

    fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let saved = crate::serialization::load_from_bytes(bytes)?;
        self.open_world(saved.world, saved.time);
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

        self.renderer.update(
            application,
            &self.world,
            self.theme,
            self.editor.connection_being_drawn(),
        );
        self.frame_clock.tick();
    }

    fn render(&mut self, data: &mut RenderData) {
        self.renderer.render(data);
    }

    fn render_shadow(&mut self, data: &mut ShadowRenderData) {
        self.renderer.render_shadow(data);
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
        if gui::corner_buttons(context, self.theme, &mut self.edit_enabled) {
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
            gui::settings_window(
                context,
                self.frame_clock.fps(),
                &mut self.iterations,
                &mut self.theme,
            );
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
