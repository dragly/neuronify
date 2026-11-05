use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::TAU;

use crate::audio::{schedule, Speakers};
use crate::camera;
use crate::components::Link;
use crate::constants::{DEFAULT_SPEED, PHYSICS_STEPS_PER_SECOND, SIMULATION_STEPS_PER_SECOND};
use crate::editor::{Editor, Pan, Targets};
use crate::frame_clock::{FrameClock, StepClock};
use crate::gui::{self, Gui, HoldRing, MenuChoice, Saving};
use crate::landing::{LandingChoice, LandingScreen};
use crate::library::{Autosave, Library, UNTITLED};
use crate::measurement::loudspeaker::{take_heard, Loudspeaker};
use crate::rendering::ghost::{self, GHOST_HEIGHT};
use crate::rendering::{canvas_bounds, EditorMarks, Ghost, SceneRenderer, Theme};
use glam::Vec3;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use visula::{winit::keyboard::ModifiersKeyState, CustomEvent, RenderData, ShadowRenderData};

use crate::input::{self, Keyboard, Mouse, TouchGesture, Touches};
use crate::simulation;
use crate::tools::{ConnectionHint, Tool};

#[cfg(target_arch = "wasm32")]
const SAVED_NAME: &str = "circuit.neuronify";
const HITBOX_TILE: f32 = 32.0;
const RING_POINTS: usize = 96;

pub struct Neuronify {
    tapped_background: bool,
    pub landing: Option<LandingScreen>,
    editor: Editor,
    gui: Gui,
    theme: Theme,
    world: hecs::World,
    engine: simulation::Engine,
    time: f64,
    running: bool,
    speed: f64,
    mouse: Mouse,
    hover: Option<PhysicalPosition<f64>>,
    touch_input: bool,
    keyboard: Keyboard,
    touches: Touches,
    ghost: Option<Ghost>,
    renderer: SceneRenderer,
    frame_clock: FrameClock,
    simulation_clock: StepClock,
    physics_clock: StepClock,
    load_error: Option<String>,
    frame_scene_requested: bool,
    saving: Saving,
    speakers: Speakers,
    library: Library,
    autosave: Autosave,
    hitboxes: bool,
    touch_reach: f64,
    hold_ring: Option<HoldRing>,
}

fn ground_under(
    application: &visula::Application,
    context: &egui::Context,
) -> impl Fn(egui::Pos2) -> Vec3 {
    let ground = camera::positions_at_height(application, 0.0);
    let pixels_per_point = context.pixels_per_point();
    move |point| {
        let pixel = |coordinate: f32| f64::from(coordinate * pixels_per_point);
        ground(PhysicalPosition::new(pixel(point.x), pixel(point.y)))
    }
}

#[cfg(target_arch = "wasm32")]
fn debug_view() -> Option<String> {
    crate::web::debug_view()
}

#[cfg(not(target_arch = "wasm32"))]
fn debug_view() -> Option<String> {
    std::env::var("NEURONIFY_DEBUG").ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn file_stem(path: &std::path::Path) -> String {
    path.file_stem().map_or(UNTITLED.to_owned(), |stem| {
        stem.to_string_lossy().into_owned()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn world_from_command_line() -> Option<(String, hecs::World)> {
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
    Some((file_stem(std::path::Path::new(&path)), world))
}

#[cfg(target_arch = "wasm32")]
fn world_from_command_line() -> Option<(String, hecs::World)> {
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

fn is_press(event: &Event<CustomEvent>) -> bool {
    matches!(
        event,
        Event::WindowEvent {
            event: WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } | WindowEvent::Touch(visula::winit::event::Touch {
                phase: TouchPhase::Started,
                ..
            }),
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
        #[cfg(not(target_arch = "wasm32"))]
        crate::window::name(&application.window);
        camera::set_up(application);

        let theme = match application.window.theme() {
            Some(visula::winit::window::Theme::Light) => Theme::Light,
            _ => Theme::Dark,
        };
        let renderer = SceneRenderer::new(application, theme);

        let opened = world_from_command_line();
        let autosave = Autosave::copy_of(opened.as_ref().map_or(UNTITLED, |(name, _)| name));
        Neuronify {
            tapped_background: false,
            landing: opened.is_none().then(|| LandingScreen::new(false)),
            editor: Editor::default(),
            gui: Gui::default(),
            theme,
            frame_scene_requested: opened.is_some(),
            world: opened.map(|(_, world)| world).unwrap_or_default(),
            engine: simulation::Engine::default(),
            time: 0.0,
            running: true,
            speed: DEFAULT_SPEED,
            mouse: Mouse::default(),
            hover: None,
            touch_input: false,
            keyboard: Keyboard::default(),
            touches: Touches::default(),
            ghost: None,
            renderer,
            frame_clock: FrameClock::default(),
            simulation_clock: StepClock::default(),
            physics_clock: StepClock::default(),
            load_error: None,
            saving: saving(),
            speakers: Speakers::default(),
            library: Library::open(),
            autosave,
            hitboxes: debug_view().as_deref() == Some("hitboxes"),
            touch_reach: input::TOUCH_REACH,
            hold_ring: None,
        }
    }

    fn press_pointer(&mut self, application: &mut visula::Application, pressed: bool) {
        if self.mouse.press(pressed, input::now()) {
            self.tap(application);
        }
        self.handle_tool(application);
    }

    fn editing(&self) -> bool {
        self.editor.tool.edits()
    }

    fn tap(&mut self, application: &visula::Application) {
        if self.editing() {
            if self.editor.tap_while_editing() {
                self.tapped_background = true;
            } else {
                self.editor.offer_connection(input::now());
            }
        } else if let Some(position) = self.mouse.position {
            let position = camera::ground_position(application, position);
            if self.editor.touch().is_none() && self.editor.empty_at(&self.world, position) {
                self.tapped_background = true;
            } else {
                self.editor.tap(&self.world, position);
                self.follow_link();
            }
        }
    }

    fn follow_link(&mut self) {
        let Some(file) = self.editor.selection.and_then(|selected| {
            let link = self.world.get::<&Link>(selected).ok()?;
            Some(link.example.clone())
        }) else {
            return;
        };
        self.editor.selection = None;
        let Some(example) = crate::examples::find(&file) else {
            self.show_landing();
            return;
        };
        match example.spawn() {
            Ok(world) => {
                let autosave = Autosave::copy_of(example.name);
                self.open_world(world, 0.0, DEFAULT_SPEED, autosave)
            }
            Err(error) => self.load_error = Some(error),
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
            self.touches.forget_fingers();
            self.mouse.cancel();
            self.editor.release(&self.world);
        }
    }

    fn eraser(&self, application: &visula::Application) -> Option<Vec3> {
        let hover = self.hover.filter(|&hover| {
            self.editor.tool == Tool::Erase && !gui_takes_press(application, hover)
        })?;
        Some(camera::ground_position(application, hover))
    }

    fn dangle_ghost(&mut self, application: &visula::Application, seconds: f64) {
        let painting = self
            .editor
            .painting()
            .filter(|_| self.editing() && self.mouse.moved_while_pressed());
        let (Some(element), Some(pointer)) = (painting, self.mouse.position) else {
            self.ghost = None;
            return;
        };
        let offset = ghost::offset_points(self.touch_input) * application.window.scale_factor();
        let above = PhysicalPosition::new(pointer.x, pointer.y - offset);
        let anchor = camera::position_at_height(application, above, GHOST_HEIGHT);
        self.ghost
            .get_or_insert_with(|| Ghost::new(element, anchor))
            .follow(element, anchor, self.touch_input, seconds as f32);
    }

    fn paint_hitboxes(&self, application: &visula::Application, context: &egui::Context) {
        let targets = Targets::new(&self.world);
        let to_ground = ground_under(application, context);
        let tiles = RefCell::new(HashMap::new());
        gui::paint_hitboxes(context, |point| {
            let tile = (point.to_vec2() / HITBOX_TILE).floor();
            let mut tiles = tiles.borrow_mut();
            let nearby = tiles
                .entry([tile.x, tile.y].map(|index| index as i32))
                .or_insert_with(|| {
                    let corner = |x: f32, y: f32| {
                        to_ground(((tile + egui::vec2(x, y)) * HITBOX_TILE).to_pos2())
                    };
                    let center = corner(0.5, 0.5);
                    let radius = [
                        corner(0.0, 0.0),
                        corner(1.0, 0.0),
                        corner(0.0, 1.0),
                        corner(1.0, 1.0),
                    ]
                    .into_iter()
                    .map(|point| point.distance(center))
                    .fold(0.0, f32::max);
                    targets.around(center, radius)
                });
            self.editor
                .hold_target(nearby, to_ground(point))
                .map(|(entity, _)| entity)
        });
    }

    fn paint_hold_ring(&self, application: &visula::Application, context: &egui::Context) {
        let Some(ring) = self.hold_ring else {
            return;
        };
        let Some(look) = ring.look(input::now()) else {
            return;
        };
        let Some(shape) = Targets::new(&self.world).shape(ring.target()) else {
            return;
        };
        let pixels_per_point = context.pixels_per_point();
        let per_point = camera::ground_length(application, f64::from(pixels_per_point));
        let gap = per_point * look.gap(shape.thickness() / per_point);
        let to_ground = ground_under(application, context);
        let to_screen = |position: Vec3| {
            let pixel = camera::screen_position(application, position);
            egui::pos2(pixel.x as f32, pixel.y as f32) / pixels_per_point
        };
        let center = to_screen(shape.center());
        let directions: Vec<Vec3> = (0..RING_POINTS)
            .map(|index| {
                let angle = TAU * index as f32 / RING_POINTS as f32;
                let beside = center + egui::vec2(angle.sin(), -angle.cos());
                (to_ground(beside) - to_ground(center)).normalize_or_zero()
            })
            .collect();
        let [inner, outer] = [0.0, gap].map(|gap| {
            let outline = shape.outline(gap, directions.iter().copied());
            outline.into_iter().map(to_screen).collect::<Vec<_>>()
        });
        gui::paint_hold_ring(context, self.theme, &inner, &outer, &look);
    }

    fn connection_hint(&mut self) -> Option<ConnectionHint> {
        let hint = self.editor.connection_hint(&self.world, input::now());
        hint.filter(|_| self.editing())
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            self.editor.lift(&mut self.world);
            return;
        }
        if self.mouse.moved_while_pressed() {
            self.editor.move_held();
        }
        let Some(screen_position) = self.mouse.position else {
            return;
        };
        let position = camera::ground_position(application, screen_position);
        let requested_pan = if self.editing() {
            self.editor
                .edit(&mut self.world, position, self.keyboard.shift_down)
        } else {
            self.editor.view(&mut self.world, position)
        };
        if let Some(Pan(offset)) = requested_pan {
            camera::pan(application, offset);
        }
    }

    fn saved(&mut self) -> Result<Vec<u8>, String> {
        self.engine.store(&mut self.world);
        crate::serialization::save(&self.world, self.time, self.speed)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self) {
        let Some(path) = rfd::FileDialog::new().save_file() else {
            return;
        };
        let bytes = self.saved().unwrap();
        std::fs::write(path.with_extension("neuronify"), bytes).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&mut self) {
        let bytes = self.saved().unwrap();
        if let Err(error) = crate::web::download(&bytes, SAVED_NAME) {
            log::error!("Could not save: {error:?}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn share(&mut self) {
        self.save();
    }

    #[cfg(target_arch = "wasm32")]
    fn share(&mut self) {
        let bytes = self.saved().unwrap();
        if let Err(error) = crate::web::share(&bytes, SAVED_NAME) {
            log::error!("Could not share: {error:?}");
        }
    }

    fn note_edits(&mut self) {
        if self.editor.take_edited() {
            self.autosave.edited(input::now());
            #[cfg(target_arch = "wasm32")]
            self.back_up_edits();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn back_up_edits(&mut self) {
        match self.saved() {
            Ok(bytes) => self.autosave.back_up(&self.library, bytes),
            Err(error) => log::error!("{error}"),
        }
    }

    fn save_edits(&mut self) {
        self.note_edits();
        if !self.autosave.has_unsaved_edits() {
            return;
        }
        match self.saved() {
            Ok(bytes) => self.autosave.save(&mut self.library, bytes),
            Err(error) => log::error!("{error}"),
        }
    }

    fn show_landing(&mut self) {
        self.save_edits();
        self.landing = Some(LandingScreen::new(true));
    }

    fn open_world(&mut self, world: hecs::World, time: f64, speed: f64, autosave: Autosave) {
        self.save_edits();
        self.autosave = autosave;
        self.world = world;
        self.engine = simulation::Engine::starting_at(time);
        self.time = time;
        self.speed = speed;
        self.running = true;
        self.editor.tool = Tool::View;
        self.editor.forget_world();
        self.landing = None;
        self.frame_scene_requested = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        let bytes = std::fs::read(&path).unwrap();
        if let Err(error) = self.open_file(&file_stem(&path), &bytes, true) {
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

    fn open_file(&mut self, name: &str, bytes: &[u8], import: bool) -> Result<(), String> {
        let saved = crate::serialization::load_from_bytes(bytes)?;
        let autosave = if import {
            let bytes = crate::serialization::save(&saved.world, saved.time, saved.speed)?;
            let id = self.library.create(name, bytes);
            Autosave::saved(&self.library, id)
        } else {
            Autosave::copy_of(name)
        };
        self.open_world(saved.world, saved.time, saved.speed, autosave);
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
        let seconds = self.frame_clock.tick();
        let simulation_steps = self
            .simulation_clock
            .steps(seconds, self.speed * SIMULATION_STEPS_PER_SECOND);
        let physics_steps = self.physics_clock.steps(seconds, PHYSICS_STEPS_PER_SECOND);

        #[cfg(target_arch = "wasm32")]
        if let Some(loaded) = crate::web::take_loaded_file() {
            let opened =
                loaded.and_then(|file| self.open_file(&file.name, &file.bytes, file.picked));
            if let Err(error) = opened {
                log::error!("{error}");
                self.load_error = Some(error);
            }
        }
        #[cfg(target_arch = "wasm32")]
        crate::web::hide_loading_screen_when_ready();
        let reach = self.touch_reach * application.window.scale_factor();
        self.editor.touch_reach = camera::ground_length(application, reach);
        self.library.poll();
        self.note_edits();
        if self.autosave.is_due(input::now()) {
            self.save_edits();
        }

        if self.tapped_background && !self.touches.may_double_tap(input::now()) {
            self.tapped_background = false;
            self.editor.selection = None;
            if self.editing() && is_compact(application) {
                self.editor.tool = Tool::View;
            }
        }
        if let TouchGesture::Press(position) = self.touches.poll(input::now()) {
            self.move_pointer(application, position);
            self.press_pointer(application, true);
        }
        let holds = if self.editing() {
            matches!(self.editor.tool, Tool::Place(_))
        } else {
            is_compact(application)
        };
        let now = input::now();
        let mut grabbed = false;
        if holds {
            if let Some(position) = self.mouse.take_hold(now) {
                let position = camera::ground_position(application, position);
                let select = !self.editing();
                grabbed = self.editor.hold(&self.world, position, select);
            }
        }
        let pending = self.mouse.pending_hold(now).filter(|_| holds);
        let pending = pending.and_then(|(start, held_for)| {
            let position = camera::ground_position(application, start);
            let (target, _) = self.editor.hold_candidate(&self.world, position)?;
            Some((target, held_for))
        });
        self.hold_ring = HoldRing::next(self.hold_ring, pending, grabbed, now);
        for position in self.editor.take_presses() {
            simulation::fire_nearby(&mut self.world, &mut self.engine, position);
        }
        if self.running {
            let start = self.time;
            simulation::step_simulation(
                &mut self.world,
                &mut self.engine,
                &mut self.time,
                simulation_steps,
                self.editor.touch(),
            );
            let plays = schedule(take_heard(&mut self.world), start, self.time, seconds);
            self.speakers.play(&plays);
        }
        simulation::step_physics(&mut self.world, physics_steps);

        if let Some(landing) = &mut self.landing {
            landing.update(seconds, physics_steps);
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
            let free = gui::canvas_area(screen, self.gui.rail_shown());
            let free = egui::Rect::from_min_max(
                (free.min.to_vec2() * pixels_per_point).to_pos2(),
                (free.max.to_vec2() * pixels_per_point).to_pos2(),
            );
            camera::frame(application, canvas_bounds(&self.world), free);
        }

        self.dangle_ghost(application, seconds);
        let drawn_connection = match self.editor.connection_being_drawn(&self.world) {
            Some((start, end)) => vec![start, end],
            None => self
                .connection_hint()
                .map(|hint| hint.string)
                .unwrap_or_default(),
        };
        self.renderer.update(
            application,
            &self.world,
            self.time,
            self.theme,
            EditorMarks {
                drawn_connection: &drawn_connection,
                axon: self.editor.axon_being_grown(&self.world),
                ghost: self.ghost,
                selection: self.editor.selection,
                eraser: self.eraser(application),
            },
        );
    }

    fn render(&mut self, data: &mut RenderData) {
        self.renderer.render(data);
    }

    fn render_shadow(&mut self, data: &mut ShadowRenderData) {
        self.renderer.render_shadow(data);
    }

    fn gui(&mut self, application: &visula::Application, context: &egui::Context) {
        context.set_theme(match self.theme {
            Theme::Dark => egui::Theme::Dark,
            Theme::Light => egui::Theme::Light,
        });
        self.sync_pointer_with_gui(context);
        if let Some(landing) = &mut self.landing {
            match landing.show(context, self.theme, &mut self.library) {
                Some(LandingChoice::Example { name, world, time }) => {
                    self.open_world(*world, time, DEFAULT_SPEED, Autosave::copy_of(name))
                }
                Some(LandingChoice::Saved {
                    id,
                    world,
                    time,
                    speed,
                }) => {
                    let autosave = Autosave::saved(&self.library, id);
                    self.open_world(*world, time, speed, autosave)
                }
                Some(LandingChoice::NewSimulation) => {
                    let autosave = Autosave::copy_of(UNTITLED);
                    self.open_world(hecs::World::new(), 0.0, DEFAULT_SPEED, autosave)
                }
                Some(LandingChoice::BackToSimulation) => self.landing = None,
                None => {}
            }
            return;
        }
        if let Some(error) = &self.load_error {
            gui::load_error(context, error);
        }
        let controls = gui::Controls {
            editor: &mut self.editor,
            world: &mut self.world,
            theme: &mut self.theme,
            time: self.time,
            running: &mut self.running,
            speed: &mut self.speed,
            fps: self.frame_clock.fps(),
            saving: self.saving,
        };
        match self.gui.show(context, controls) {
            Some(MenuChoice::Examples) => self.show_landing(),
            Some(MenuChoice::Save) => self.save(),
            Some(MenuChoice::Share) => self.share(),
            Some(MenuChoice::Open) => self.open(),
            None => {}
        }
        if let Some(hint) = self.connection_hint() {
            let tip = camera::screen_position(application, hint.label);
            let pixels_per_point = context.pixels_per_point();
            let tip = egui::pos2(
                tip.x as f32 / pixels_per_point,
                tip.y as f32 / pixels_per_point,
            );
            gui::connection_hint(context, self.theme, tip, hint.shown);
        }
        if self.hitboxes {
            gui::reach_slider(context, &mut self.touch_reach);
            self.paint_hitboxes(application, context);
        }
        self.paint_hold_ring(application, context);
    }

    fn handle_event(&mut self, application: &mut visula::Application, event: &Event<CustomEvent>) {
        if is_press(event) {
            let needed = self.world.query::<&Loudspeaker>().iter().next().is_some();
            self.speakers.unlock(needed);
        }
        if matches!(
            event,
            Event::WindowEvent {
                event: WindowEvent::Occluded(true) | WindowEvent::CloseRequested,
                ..
            }
        ) {
            self.save_edits();
        }
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
                self.touch_input = false;
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
                self.hover = Some(*position);
                self.touch_input = false;
                self.move_pointer(application, *position);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                ..
            } => self.hover = None,
            Event::WindowEvent {
                event: WindowEvent::Touch(touch),
                ..
            } => {
                self.hover = None;
                self.touch_input = true;
                if touch.phase == TouchPhase::Started {
                    if gui_takes_press(application, touch.location) {
                        self.touches.forget_tap();
                        return;
                    }
                    let position = camera::ground_position(application, touch.location);
                    if !self.editor.empty_at(&self.world, position) {
                        self.touches.forget_tap();
                    }
                }
                let wait = !matches!(self.editor.tool, Tool::View | Tool::Edit | Tool::Stimulate);
                let scale_factor = application.window.scale_factor();
                match self.touches.update(touch, input::now(), wait, scale_factor) {
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
                        self.editor.release(&self.world);
                    }
                    TouchGesture::Pinch { previous, current } => {
                        camera::pinch(application, previous, current)
                    }
                    TouchGesture::Zoom {
                        anchor,
                        previous,
                        current,
                    } => {
                        self.tapped_background = false;
                        camera::drag_zoom(application, anchor, previous, current);
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
