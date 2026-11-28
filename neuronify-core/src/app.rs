use std::cell::RefCell;
use std::collections::HashMap;

use crate::audio::{schedule, Speakers};
use crate::camera;
use crate::components::{Milliseconds, NeuronType, Position};
use crate::constants::{DEFAULT_SPEED, PHYSICS_STEPS_PER_SECOND, SIMULATION_STEPS_PER_SECOND};
use crate::editor::{Editor, Pan, Targets};
use crate::feedback::{self, Feedback, Pointer};
use crate::fire_hint::FireHint;
use crate::frame_clock::{FrameClock, StepClock};
use crate::gui::{self, FileError, Gui, MenuChoice, Saving};
use crate::inertia::Inertia;
use crate::landing::about::{AboutChoice, AboutScreen};
use crate::landing::{LandingChoice, LandingScreen};
use crate::library::{Autosave, Library, UNTITLED};
use crate::measurement::loudspeaker::{take_heard, Loudspeaker};
use crate::measurement::Meter;
use crate::note::Link;
use crate::rendering::{canvas_bounds, EditorMarks, SceneRenderer, Theme};
use crate::serialization::{Meta, SavedWorld};
use glam::Vec3;
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use visula::{CustomEvent, RenderData, ShadowRenderData};

use crate::input::{self, Mouse, TouchGesture, Touches};
use crate::simulation;
use crate::tools::{ConnectionHint, Tool};
use hecs::Entity;

#[cfg(target_arch = "wasm32")]
const SAVED_NAME: &str = "circuit.neuronify";
const HITBOX_TILE: f32 = 32.0;
const FIRE_HINT_LIFT: f32 = 18.0;

pub struct Neuronify {
    tapped_background: bool,
    pub landing: Option<LandingScreen>,
    about: Option<AboutScreen>,
    version: &'static str,
    editor: Editor,
    gui: Gui,
    theme: Theme,
    world: hecs::World,
    engine: simulation::Engine,
    time: Milliseconds,
    running: bool,
    speed: f64,
    mouse: Mouse,
    hover: Option<PhysicalPosition<f64>>,
    touch_input: bool,
    touches: Touches,
    inertia: Inertia,
    caught: bool,
    fire_hint: FireHint,
    renderer: SceneRenderer,
    frame_clock: FrameClock,
    simulation_clock: StepClock,
    physics_clock: StepClock,
    file_error: Option<FileError>,
    origin: Option<Origin>,
    frame_scene_requested: bool,
    saving: Saving,
    speakers: Speakers,
    library: Library,
    autosave: Autosave,
    hitboxes: bool,
    touch_reach: f64,
    feedback: Feedback,
}

#[cfg(target_arch = "wasm32")]
fn debug_view() -> Option<String> {
    crate::web::debug_view()
}

#[cfg(not(target_arch = "wasm32"))]
fn debug_view() -> Option<String> {
    std::env::var("NEURONIFY_DEBUG").ok()
}

#[cfg(target_arch = "wasm32")]
fn sharing() -> bool {
    crate::web::sharing()
}

#[cfg(not(target_arch = "wasm32"))]
fn sharing() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn file_stem(path: &std::path::Path) -> String {
    path.file_stem().map_or(UNTITLED.to_owned(), |stem| {
        stem.to_string_lossy().into_owned()
    })
}

#[cfg(not(target_arch = "wasm32"))]
type Origin = std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
type Origin = String;

#[cfg(not(target_arch = "wasm32"))]
fn suggested_file_name(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            character if character.is_control() => '-',
            character => character,
        })
        .collect();
    format!("{stem}.neuronify")
}

fn details(bytes: &[u8], file_name: &str) -> (String, String) {
    match crate::serialization::meta(bytes) {
        Some(meta) if !meta.name.trim().is_empty() => (meta.name, meta.description),
        Some(meta) => (file_name.to_owned(), meta.description),
        None => (file_name.to_owned(), String::new()),
    }
}

struct ReadFile {
    name: String,
    description: String,
    saved: SavedWorld,
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file(path: &std::path::Path) -> Result<ReadFile, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let saved = crate::serialization::load_from_bytes(&bytes)?;
    let (name, description) = details(&bytes, &file_stem(path));
    Ok(ReadFile {
        name,
        description,
        saved,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn file_from_command_line() -> Option<Result<(Origin, ReadFile), String>> {
    let path = std::path::PathBuf::from(std::env::args_os().nth(1)?);
    Some(read_file(&path).map(|file| (path, file)))
}

#[cfg(target_arch = "wasm32")]
fn file_from_command_line() -> Option<Result<(Origin, ReadFile), String>> {
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
            } | WindowEvent::KeyboardInput {
                event: visula::winit::event::KeyEvent {
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } | WindowEvent::Touch(visula::winit::event::Touch {
                phase: TouchPhase::Started,
                ..
            }),
            ..
        }
    )
}

impl Neuronify {
    pub fn new(application: &mut visula::Application, version: &'static str) -> Neuronify {
        #[cfg(not(target_arch = "wasm32"))]
        crate::window::name(&application.window);
        camera::set_up(application);

        let theme = match application.window.theme() {
            Some(visula::winit::window::Theme::Light) => Theme::Light,
            _ => Theme::Dark,
        };
        let renderer = SceneRenderer::new(application, theme)
            .expect("the shaders and font built into Neuronify should compile");

        let opened = file_from_command_line();
        let mut neuronify = Neuronify {
            tapped_background: false,
            landing: (!matches!(opened, Some(Ok(_)))).then(|| LandingScreen::new(false)),
            about: None,
            version,
            editor: Editor::default(),
            gui: Gui::default(),
            theme,
            frame_scene_requested: false,
            world: hecs::World::new(),
            engine: simulation::Engine::default(),
            time: Milliseconds::ZERO,
            running: true,
            speed: DEFAULT_SPEED,
            mouse: Mouse::default(),
            hover: None,
            touch_input: false,
            touches: Touches::default(),
            inertia: Inertia::default(),
            caught: false,
            fire_hint: FireHint::default(),
            renderer,
            frame_clock: FrameClock::default(),
            simulation_clock: StepClock::default(),
            physics_clock: StepClock::default(),
            file_error: None,
            origin: None,
            saving: saving(),
            speakers: Speakers::default(),
            library: Library::open(),
            autosave: Autosave::copy_of(UNTITLED, ""),
            hitboxes: debug_view().as_deref() == Some("hitboxes"),
            touch_reach: input::TOUCH_REACH,
            feedback: Feedback::default(),
        };
        match opened {
            Some(Ok((origin, file))) => neuronify.open_saved(origin, file),
            Some(Err(error)) => {
                log::error!("{error}");
                neuronify.file_error = Some(FileError::load(error));
            }
            None => {}
        }
        neuronify
    }

    fn press_pointer(&mut self, application: &mut visula::Application, pressed: bool) {
        if self.mouse.press(pressed, input::now()) && !self.caught {
            self.tap(application);
        }
        self.handle_tool(application);
        if !pressed {
            let scale_factor = application.window.scale_factor();
            self.inertia.release(input::now(), scale_factor);
            self.caught = false;
        }
    }

    fn answer_tap(&mut self, application: &visula::Application) -> bool {
        let Some(position) = self.mouse.position else {
            return false;
        };
        let position = camera::ground_position(application, position);
        let Some((tapped, _)) = Targets::new(&self.world).at(position) else {
            return false;
        };
        self.fire_hint
            .tap(&self.world, tapped, self.editor.tool, input::now())
    }

    fn catch_glide(&mut self, application: &visula::Application) {
        self.caught = self.inertia.press(application.window.scale_factor());
    }

    fn editing(&self) -> bool {
        self.editor.tool.edits()
    }

    fn tap(&mut self, application: &visula::Application) {
        let answered = self.answer_tap(application);
        if self.editing() {
            self.editor.snap_back(&mut self.world);
            if self.editor.tap_while_editing() {
                self.tapped_background = true;
            } else if !answered {
                self.editor.offer_connection(&self.world, input::now());
            }
        } else if let Some(position) = self.mouse.position {
            let position = camera::ground_position(application, position);
            if self.editor.touch().is_none()
                && self.editor.reverse_picture_at(&mut self.world, position)
            {
                feedback::tick();
            } else if self.editor.touch().is_none() && self.editor.empty_at(&self.world, position) {
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
            if !self.open_beside_origin(&file) {
                self.show_landing();
            }
            return;
        };
        match example.spawn() {
            Ok(world) => {
                let autosave = Autosave::copy_of(example.name, example.description);
                self.open_world(world, Milliseconds::ZERO, DEFAULT_SPEED, autosave)
            }
            Err(error) => self.file_error = Some(FileError::load(error)),
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
        if self.editor.tool != Tool::Erase {
            return None;
        }
        let finger = self
            .mouse
            .position
            .filter(|_| self.touch_input && self.mouse.left_down);
        let hover = self
            .hover
            .filter(|&hover| !gui::takes_press(application, hover));
        let pointer = finger.or(hover)?;
        Some(camera::ground_position(application, pointer))
    }

    fn paint_hitboxes(&self, application: &visula::Application, context: &egui::Context) {
        let targets = Targets::new(&self.world);
        let to_ground = camera::ground_under(application, context);
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

    fn paint_fire_hint(&self, application: &visula::Application, context: &egui::Context) {
        let Some((tapped, shown)) = self.fire_hint.answer(&self.world, input::now()) else {
            return;
        };
        let Ok(position) = self.world.get::<&Position>(tapped).map(|at| at.position) else {
            return;
        };
        let center = camera::screen_position(application, position);
        let pixels_per_point = context.pixels_per_point();
        let tip = egui::pos2(
            center.x as f32 / pixels_per_point,
            center.y as f32 / pixels_per_point - FIRE_HINT_LIFT,
        );
        gui::fire_hint(context, self.theme, tip, shown);
    }

    fn connection_hint(&mut self) -> Option<ConnectionHint> {
        let hint = self.editor.connection_hint(&self.world, input::now());
        hint.filter(|_| self.editing())
    }

    fn pointer_feedback(&mut self) -> (&mut Feedback, Pointer<'_>) {
        let pointer = Pointer {
            editor: &self.editor,
            world: &self.world,
            mouse: &self.mouse,
            hover: self.hover,
            touch_input: self.touch_input,
        };
        (&mut self.feedback, pointer)
    }

    fn handle_tool(&mut self, application: &mut visula::Application) {
        self.use_tool(application);
        let (feedback, pointer) = self.pointer_feedback();
        feedback.follow_ending(&pointer);
    }

    fn use_tool(&mut self, application: &mut visula::Application) {
        if !self.mouse.left_down {
            if let Some(target) = self.editor.lift(&mut self.world) {
                self.feedback.attached(target, input::now());
            }
            return;
        }
        if self.mouse.moved_while_pressed() {
            self.editor.move_held();
        }
        let Some(screen_position) = self.mouse.position else {
            return;
        };
        let position = camera::ground_position(application, screen_position);
        let requested_pan = if self.caught {
            self.editor.pan(position)
        } else if self.editing() {
            self.editor.edit(&mut self.world, position)
        } else {
            self.editor.view(&mut self.world, position)
        };
        if let Some(Pan(offset)) = requested_pan {
            camera::pan(application, offset);
        }
        if self.editor.pressed_background() {
            self.inertia.track(input::now(), screen_position);
        }
    }

    fn saved(&mut self) -> Result<Vec<u8>, String> {
        let meta = self.autosave.meta(&self.library, self.version);
        self.saved_as(&meta)
    }

    fn saved_as(&mut self, meta: &Meta) -> Result<Vec<u8>, String> {
        self.engine.store(&mut self.world);
        crate::serialization::save(&self.world, self.time, self.speed, meta)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self) {
        let meta = self.autosave.meta(&self.library, self.version);
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Neuronify", &["neuronify"])
            .set_file_name(suggested_file_name(&meta.name))
            .save_file()
        else {
            return;
        };
        let meta = Meta {
            name: file_stem(&path),
            ..meta
        };
        let written = self.saved_as(&meta).and_then(|bytes| {
            std::fs::write(&path, bytes)
                .map_err(|error| format!("Could not write {}: {error}", path.display()))
        });
        if let Err(error) = written {
            log::error!("{error}");
            self.file_error = Some(FileError::save(error));
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&mut self) {
        match self.saved() {
            Ok(bytes) => {
                if let Err(error) = crate::web::download(&bytes, SAVED_NAME) {
                    log::error!("Could not save: {error:?}");
                }
            }
            Err(error) => self.file_error = Some(FileError::save(error)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn share(&mut self) {
        self.save();
    }

    #[cfg(target_arch = "wasm32")]
    fn share(&mut self) {
        match self.saved() {
            Ok(bytes) => {
                if let Err(error) = crate::web::share(&bytes, SAVED_NAME) {
                    log::error!("Could not share: {error:?}");
                }
            }
            Err(error) => self.file_error = Some(FileError::save(error)),
        }
    }

    fn note_edits(&mut self) {
        let edited = self.editor.take_edited();
        let settings_changed =
            self.editor.take_settings_changed() && self.autosave.is_in(&self.library);
        if edited || settings_changed {
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

    fn open_world(
        &mut self,
        world: hecs::World,
        time: Milliseconds,
        speed: f64,
        autosave: Autosave,
    ) {
        self.save_edits();
        self.autosave = autosave;
        self.file_error = None;
        self.origin = None;
        self.world = world;
        self.engine = simulation::Engine::starting_at(time);
        self.time = time;
        self.speed = speed;
        self.simulation_clock = StepClock::default();
        self.physics_clock = StepClock::default();
        self.running = true;
        self.editor.tool = Tool::View;
        self.editor.forget_world();
        self.gui.forget_world();
        self.fire_hint.forget_world();
        self.feedback = Feedback::default();
        self.landing = None;
        self.frame_scene_requested = true;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        let opened = std::fs::read(&path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))
            .and_then(|bytes| self.open_file(&file_stem(&path), &bytes, true));
        match opened {
            Ok(()) => self.origin = Some(path),
            Err(error) => {
                log::error!("{error}");
                self.file_error = Some(FileError::load(error));
            }
        }
    }

    fn open_saved(&mut self, origin: Origin, file: ReadFile) {
        self.open_world(
            file.saved.world,
            file.saved.time,
            file.saved.speed,
            Autosave::copy_of(&file.name, &file.description),
        );
        self.origin = Some(origin);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_beside_origin(&mut self, file: &str) -> bool {
        let Some(path) = crate::legacy::sibling_file(file)
            .zip(self.origin.as_ref().and_then(|origin| origin.parent()))
            .map(|(name, folder)| folder.join(name))
            .filter(|path| path.is_file())
        else {
            return false;
        };
        match read_file(&path) {
            Ok(file) => self.open_saved(path, file),
            Err(error) => self.file_error = Some(FileError::load(error)),
        }
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn open_beside_origin(&mut self, file: &str) -> bool {
        match (crate::legacy::sibling_file(file), &self.origin) {
            (Some(name), Some(origin)) => crate::web::fetch_beside(origin, name),
            _ => false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open(&mut self) {
        if let Err(error) = crate::web::pick_file() {
            log::error!("Could not open: {error:?}");
        }
    }

    fn open_file(&mut self, file_name: &str, bytes: &[u8], import: bool) -> Result<(), String> {
        let saved = crate::serialization::load_from_bytes(bytes)?;
        let (name, description) = details(bytes, file_name);
        let autosave = if import {
            let meta = Meta {
                name,
                description,
                app: self.version.to_owned(),
            };
            let bytes = crate::serialization::save(&saved.world, saved.time, saved.speed, &meta)?;
            let id = self.library.create(&meta.name, &meta.description, bytes);
            Autosave::saved(&self.library, id)
        } else {
            Autosave::copy_of(&name, &description)
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
        let elapsed = self.frame_clock.tick();
        let simulation_steps = self
            .simulation_clock
            .steps(elapsed, self.speed * SIMULATION_STEPS_PER_SECOND);
        let physics_steps = self.physics_clock.steps(elapsed, PHYSICS_STEPS_PER_SECOND);

        #[cfg(target_arch = "wasm32")]
        if let Some(loaded) = crate::web::take_loaded_file() {
            let opened = loaded.and_then(|file| {
                self.open_file(&file.name, &file.bytes, file.picked)?;
                self.origin = file.url;
                Ok(())
            });
            if let Err(error) = opened {
                log::error!("{error}");
                self.file_error = Some(FileError::load(error));
            }
        }
        #[cfg(target_arch = "wasm32")]
        if crate::web::take_missing_link() {
            self.show_landing();
        }
        #[cfg(target_arch = "wasm32")]
        crate::web::hide_loading_screen_when_ready();
        let reach = self.touch_reach * application.window.scale_factor();
        self.editor.touch_reach = camera::ground_length(application, reach);
        self.library.poll();
        if let Some(problem) = self.library.take_problem() {
            log::error!("{problem}");
            self.file_error = Some(FileError::save(problem));
        }
        self.note_edits();
        if self.autosave.is_due(input::now()) {
            self.save_edits();
        }

        if self.tapped_background && !self.touches.may_double_tap(input::now()) {
            self.tapped_background = false;
            self.editor.selection = None;
            if self.editing() {
                self.editor.tool = Tool::View;
            }
        }
        self.editor.sync_selection(&mut self.world);
        if let TouchGesture::Press(position) = self.touches.poll(input::now()) {
            self.move_pointer(application, position);
            self.press_pointer(application, true);
        }
        let holds = !self.caught && (!self.editing() || matches!(self.editor.tool, Tool::Place(_)));
        let now = input::now();
        let mut grabbed = false;
        if holds {
            if let Some(position) = self.mouse.take_hold(now) {
                let position = camera::ground_position(application, position);
                let select = !self.editing();
                grabbed = self.editor.hold(&self.world, position, select);
            }
        }
        let (feedback, pointer) = self.pointer_feedback();
        feedback.follow_hold(&pointer, application, holds, grabbed, now);
        if let Some(target) = self.editor.dwell(&mut self.world, now) {
            self.feedback.attached(target, now);
        }
        let (feedback, pointer) = self.pointer_feedback();
        feedback.follow_dwell(&pointer, now);
        for position in self.editor.take_presses() {
            simulation::fire_nearby(&mut self.world, &mut self.engine, position);
        }
        let shown = self.landing.is_none() && self.about.is_none() && !sharing();
        if self.running && shown {
            let start = self.time;
            simulation::step_simulation(
                &mut self.world,
                &mut self.engine,
                &mut self.time,
                simulation_steps,
                self.editor.touch(),
            );
            let plays = schedule(take_heard(&mut self.world), start, self.time, elapsed);
            self.speakers.play(&plays);
        }
        if shown {
            simulation::step_physics(&mut self.world, physics_steps, self.editor.drawn());
        }

        if let Some(landing) = &mut self.landing {
            landing.update(elapsed, physics_steps);
        }
        if !shown {
            self.inertia.stop();
        }
        if let Some((from, to)) = self.inertia.step(input::now()) {
            camera::slide(application, from, to);
        }
        if self.frame_scene_requested {
            self.frame_scene_requested = false;
            self.inertia.stop();
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

        self.fire_hint.update(&self.world, input::now());
        self.feedback
            .land_placed(self.editor.take_placed_neuron(), self.theme);
        let (feedback, pointer) = self.pointer_feedback();
        feedback.dangle_ghost(&pointer, application, elapsed);
        feedback.prepare_loupe(&pointer, application);
        let candidate = feedback
            .candidate(&pointer)
            .filter(|&candidate| Some(candidate) != pointer.editor.selection);
        let cable = self.editor.cable_being_drawn(&self.world);
        let wire = self.editor.connection_being_drawn(&self.world);
        let hint = self.connection_hint();
        let (dangling, drawn_connection) = match (cable, wire, hint) {
            (Some((from, start, end)), _, _) => (Some((from, vec![start, end])), Vec::new()),
            (None, Some((start, end)), _) => (None, vec![start, end]),
            (None, None, Some(hint)) if Meter::of(&self.world, hint.from).is_none() => {
                (Some((hint.from, hint.string)), Vec::new())
            }
            (None, None, hint) => (None, hint.map(|hint| hint.string).unwrap_or_default()),
        };
        self.renderer.update(
            application,
            &self.world,
            self.time,
            self.theme,
            EditorMarks {
                drawn_connection: &drawn_connection,
                waiting: self
                    .editor
                    .axon_waiting(&self.world)
                    .map(|(_, start, end)| (start, end)),
                dangling: dangling
                    .as_ref()
                    .map(|(from, points)| (*from, points.as_slice())),
                axon: self.editor.axon_being_grown(&self.world),
                ending: self.editor.new_ending(&self.world),
                bead: self.editor.bead(&self.world),
                dendrite: self.editor.dendrite_being_grown(),
                ghost: self.feedback.ghost(),
                landing_ghosts: self.feedback.landing_ghosts(),
                selection: self.editor.selection,
                candidate,
                eraser: self.eraser(application),
                pulled: self.editor.pulled_tip(),
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
        #[cfg(target_arch = "wasm32")]
        crate::web::settle_keyboard(context);
        context.set_theme(match self.theme {
            Theme::Dark => egui::Theme::Dark,
            Theme::Light => egui::Theme::Light,
        });
        self.sync_pointer_with_gui(context);
        if let Some(about) = &mut self.about {
            if let Some(AboutChoice::Back) = about.show(context, self.theme) {
                self.about = None;
            }
            return;
        }
        if let Some(landing) = &mut self.landing {
            match landing.show(context, self.theme, &mut self.library) {
                Some(LandingChoice::Example {
                    name,
                    description,
                    world,
                }) => self.open_world(
                    *world,
                    Milliseconds::ZERO,
                    DEFAULT_SPEED,
                    Autosave::copy_of(name, description),
                ),
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
                    let autosave = Autosave::copy_of(UNTITLED, "");
                    self.open_world(
                        hecs::World::new(),
                        Milliseconds::ZERO,
                        DEFAULT_SPEED,
                        autosave,
                    )
                }
                Some(LandingChoice::BackToSimulation) => self.landing = None,
                Some(LandingChoice::About) => self.about = Some(AboutScreen::new(self.version)),
                None => {}
            }
            return;
        }
        gui::file_error(context, &mut self.file_error);
        let (feedback, pointer) = self.pointer_feedback();
        let drawing = feedback.shows_loupe(&pointer);
        let speed = self.speed;
        let stimulate_pulse = self.fire_hint.pulse(self.editor.tool, input::now());
        let controls = gui::Controls {
            editor: &mut self.editor,
            world: &mut self.world,
            theme: &mut self.theme,
            time: self.time,
            running: &mut self.running,
            speed: &mut self.speed,
            fps: self.frame_clock.fps(),
            saving: self.saving,
            stimulate_pulse,
        };
        let chosen = match drawing {
            true => None,
            false => self.gui.show(context, controls),
        };
        if self.speed != speed {
            self.editor.mark_settings_changed();
        }
        match chosen {
            Some(MenuChoice::Examples) => self.show_landing(),
            Some(MenuChoice::About) => self.about = Some(AboutScreen::new(self.version)),
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
            let neuron = self.world.satisfies::<&NeuronType>(hint.from);
            gui::connection_hint(context, self.theme, tip, hint.shown, neuron);
        }
        self.paint_fire_hint(application, context);
        if self.hitboxes {
            gui::reach_slider(context, &mut self.touch_reach);
            self.paint_hitboxes(application, context);
        }
        let theme = self.theme;
        let (feedback, pointer) = self.pointer_feedback();
        feedback.paint(&pointer, application, context, theme);
    }

    fn handle_event(&mut self, application: &mut visula::Application, event: &Event<CustomEvent>) {
        if is_press(event) {
            let needed = self
                .world
                .query::<(Entity, &Loudspeaker)>()
                .iter()
                .next()
                .is_some();
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
        if (self.landing.is_some() || self.about.is_some()) && is_press_scroll_or_touch(event) {
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
                let pressed = *state == ElementState::Pressed;
                if pressed {
                    self.catch_glide(application);
                }
                self.press_pointer(application, pressed);
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
                    if gui::takes_press(application, touch.location) {
                        self.touches.forget_tap();
                        return;
                    }
                    let position = camera::ground_position(application, touch.location);
                    if !self.editor.empty_at(&self.world, position) {
                        self.touches.forget_tap();
                    }
                    self.catch_glide(application);
                }
                let wait = !self.caught
                    && !matches!(self.editor.tool, Tool::View | Tool::Edit | Tool::Stimulate);
                let scale_factor = application.window.scale_factor();
                let gesture = self.touches.update(touch, input::now(), wait, scale_factor);
                if self.caught {
                    self.touches.forget_tap();
                }
                match gesture {
                    TouchGesture::Press(position) => {
                        self.move_pointer(application, position);
                        self.press_pointer(application, true);
                    }
                    TouchGesture::Drag(position) => self.move_pointer(application, position),
                    TouchGesture::Stroke { start, current } => {
                        self.move_pointer(application, start);
                        self.press_pointer(application, true);
                        self.move_pointer(application, current);
                    }
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
                        camera::pinch(application, previous, current);
                        self.inertia.track(input::now(), input::midpoint(current));
                    }
                    TouchGesture::PinchEnded => {
                        let scale_factor = application.window.scale_factor();
                        self.inertia.release(input::now(), scale_factor);
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
