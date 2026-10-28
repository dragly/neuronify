use crate::camera;
use crate::editor::{Editor, Pan};
use crate::frame_clock::FrameClock;
use crate::gui::{self, Gui, MenuChoice, Saving};
use crate::landing::{LandingChoice, LandingScreen};
use crate::rendering::{canvas_bounds, SceneRenderer, Theme};
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use visula::{winit::keyboard::ModifiersKeyState, CustomEvent, RenderData, ShadowRenderData};

use crate::input::{self, Keyboard, Mouse, TouchGesture, Touches};
use crate::simulation;
use crate::tools::Tool;

#[cfg(target_arch = "wasm32")]
const SAVED_NAME: &str = "circuit.neuronify";

pub struct Neuronify {
    edit_enabled: bool,
    pub landing: Option<LandingScreen>,
    editor: Editor,
    gui: Gui,
    theme: Theme,
    world: hecs::World,
    time: f64,
    running: bool,
    iterations: u32,
    mouse: Mouse,
    keyboard: Keyboard,
    touches: Touches,
    renderer: SceneRenderer,
    frame_clock: FrameClock,
    load_error: Option<String>,
    frame_scene_requested: bool,
    saving: Saving,
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

#[cfg(not(target_arch = "wasm32"))]
fn saving() -> Saving {
    Saving::Save
}

#[cfg(target_arch = "wasm32")]
fn saving() -> Saving {
    match (crate::web::can_share(SAVED_NAME), crate::web::is_ios()) {
        (false, _) => Saving::Save,
        (true, false) => Saving::SaveOrShare,
        (true, true) => Saving::Share,
    }
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

fn is_compact(application: &visula::Application) -> bool {
    gui::is_compact(application.egui_renderer.state.egui_ctx().content_rect())
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
            edit_enabled: false,
            landing: opened.is_none().then(|| LandingScreen::new(false)),
            editor: Editor::default(),
            gui: Gui::default(),
            theme,
            frame_scene_requested: opened.is_some(),
            world: opened.unwrap_or_default(),
            time: 0.0,
            running: true,
            iterations: 4,
            mouse: Mouse::default(),
            keyboard: Keyboard::default(),
            touches: Touches::default(),
            renderer,
            frame_clock: FrameClock::default(),
            load_error: None,
            saving: saving(),
        }
    }

    fn press_pointer(&mut self, application: &mut visula::Application, pressed: bool) {
        if self.mouse.press(pressed, input::now()) {
            self.tap(application);
        }
        self.handle_tool(application);
    }

    fn tap(&mut self, application: &visula::Application) {
        if self.edit_enabled {
            if self.editor.pressed_background() {
                self.editor.selection = None;
                if is_compact(application) {
                    self.edit_enabled = false;
                }
            }
        } else if let Some(position) = self.mouse.position {
            let position = camera::ground_position(application, position);
            self.editor.tap(&self.world, position);
        }
    }

    fn move_pointer(
        &mut self,
        application: &mut visula::Application,
        position: PhysicalPosition<f64>,
    ) {
        self.mouse
            .move_to(position, application.window.scale_factor());
        self.handle_tool(application);
    }

    fn sync_pointer_with_gui(&mut self, context: &egui::Context) {
        let held = context.input(|input| input.pointer.any_down() || input.any_touches());
        if !held {
            self.touches = Touches::default();
            self.mouse.cancel();
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

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) {
        let Some(path) = rfd::FileDialog::new().save_file() else {
            return;
        };
        let bytes = crate::serialization::save(&self.world, self.time).unwrap();
        std::fs::write(path.with_extension("neuronify"), bytes).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&self) {
        let bytes = crate::serialization::save(&self.world, self.time).unwrap();
        if let Err(error) = crate::web::download(&bytes, SAVED_NAME) {
            log::error!("Could not save: {error:?}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn share(&self) {
        self.save();
    }

    #[cfg(target_arch = "wasm32")]
    fn share(&self) {
        let bytes = crate::serialization::save(&self.world, self.time).unwrap();
        if let Err(error) = crate::web::share(&bytes, SAVED_NAME) {
            log::error!("Could not share: {error:?}");
        }
    }

    fn open_world(&mut self, world: hecs::World, time: f64) {
        self.world = world;
        self.time = time;
        self.edit_enabled = false;
        self.editor.release();
        self.editor.selection = None;
        self.landing = None;
        self.frame_scene_requested = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        let bytes = std::fs::read(path).unwrap();
        if let Err(error) = self.load_from_bytes(&bytes) {
            log::error!("{error}");
            self.load_error = Some(error);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open(&mut self) {
        if let Err(error) = crate::web::pick_file() {
            log::error!("Could not open: {error:?}");
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
        if let Some(loaded) = crate::web::take_loaded_file() {
            if let Err(error) = loaded.and_then(|bytes| self.load_from_bytes(&bytes)) {
                log::error!("{error}");
                self.load_error = Some(error);
            }
        }

        if let TouchGesture::Press(position) = self.touches.poll(input::now()) {
            self.move_pointer(application, position);
            self.press_pointer(application, true);
        }
        if !self.edit_enabled && is_compact(application) {
            if let Some(position) = self.mouse.take_hold(input::now()) {
                let position = camera::ground_position(application, position);
                self.edit_enabled = self.editor.hold(&self.world, position);
            }
        }
        if let Some(position) = self.editor.touch() {
            simulation::stimulate_nearby(&mut self.world, position);
        }
        if self.running {
            simulation::step_frame(&mut self.world, &mut self.time, self.iterations);
        } else {
            simulation::step_physics(&mut self.world, self.iterations);
        }

        if let Some(landing) = &mut self.landing {
            landing.update(self.iterations);
        }
        if self.frame_scene_requested {
            self.frame_scene_requested = false;
            let pixels_per_point = application
                .egui_renderer
                .state
                .egui_ctx()
                .pixels_per_point();
            let screen = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(
                    application.config.width as f32,
                    application.config.height as f32,
                ) / pixels_per_point,
            );
            let free = gui::canvas_area(screen, self.edit_enabled);
            let free = egui::Rect::from_min_max(
                (free.min.to_vec2() * pixels_per_point).to_pos2(),
                (free.max.to_vec2() * pixels_per_point).to_pos2(),
            );
            camera::frame(application, canvas_bounds(&self.world), free);
        }

        self.renderer.update(
            application,
            &self.world,
            self.theme,
            self.editor.connection_being_drawn(),
            self.editor.selection,
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
        let controls = gui::Controls {
            edit_enabled: &mut self.edit_enabled,
            editor: &mut self.editor,
            world: &mut self.world,
            theme: &mut self.theme,
            time: self.time,
            running: &mut self.running,
            iterations: &mut self.iterations,
            fps: self.frame_clock.fps(),
            saving: self.saving,
        };
        match self.gui.show(context, controls) {
            Some(MenuChoice::Examples) => self.landing = Some(LandingScreen::new(true)),
            Some(MenuChoice::Save) => self.save(),
            Some(MenuChoice::Share) => self.share(),
            Some(MenuChoice::Open) => self.open(),
            None => {}
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
                let wait = self.edit_enabled && self.editor.tool != Tool::Stimulate;
                match self.touches.update(touch, input::now(), wait) {
                    TouchGesture::Press(position) => {
                        self.move_pointer(application, position);
                        self.press_pointer(application, true);
                    }
                    TouchGesture::Drag(position) => self.move_pointer(application, position),
                    TouchGesture::Release => self.press_pointer(application, false),
                    TouchGesture::Tap(position) => {
                        self.move_pointer(application, position);
                        self.press_pointer(application, true);
                        self.press_pointer(application, false);
                    }
                    TouchGesture::Cancel => {
                        self.mouse.cancel();
                        self.handle_tool(application);
                    }
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
