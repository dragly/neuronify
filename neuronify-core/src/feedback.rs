use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use visula::winit::dpi::PhysicalPosition;

use crate::camera;
use crate::components::Seconds;
use crate::editor::{Editor, PlacedNeuron, Targets};
use crate::elements::Element;
use crate::gui::{
    self, AttachBurst, AttachRing, HoldRing, Look, LoupeCorner, LoupeFocus, LoupeView, Subject,
    LOUPE_DELAY, LOUPE_SNAP, WAITING,
};
use crate::input::{self, Mouse};
use crate::rendering::ghost::{self, GHOST_HEIGHT};
use crate::rendering::{Ghost, LandingGhost, Theme};
use crate::tools::Tool;

const RING_POINTS: usize = 96;

#[cfg(target_arch = "wasm32")]
pub fn tick() {
    crate::web::tick();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn tick() {}

#[cfg(target_arch = "wasm32")]
fn thump() {
    crate::web::thump();
}

#[cfg(not(target_arch = "wasm32"))]
fn thump() {}

#[cfg(target_arch = "wasm32")]
fn snap() {
    crate::web::snap();
}

#[cfg(not(target_arch = "wasm32"))]
fn snap() {}

pub struct Pointer<'a> {
    pub editor: &'a Editor,
    pub world: &'a World,
    pub mouse: &'a Mouse,
    pub hover: Option<PhysicalPosition<f64>>,
    pub touch_input: bool,
}

impl Pointer<'_> {
    fn hover_anchor(&self, application: &visula::Application) -> Option<(Element, Vec3)> {
        if self.touch_input || self.mouse.left_down {
            return None;
        }
        let Tool::Place(element) = self.editor.tool else {
            return None;
        };
        let hover = self
            .hover
            .filter(|&hover| element.is_neuron() && !gui::takes_press(application, hover))?;
        let ground = camera::ground_position(application, hover);
        if !self.editor.empty_at(self.world, ground) {
            return None;
        }
        let anchor = camera::position_at_height(application, hover, GHOST_HEIGHT);
        Some((element, anchor))
    }

    fn way_to_next(&self, application: &visula::Application) -> Option<f32> {
        let position = camera::ground_position(application, self.mouse.position?);
        let way = self.editor.way_to_next(self.world, position)?;
        Some(way.clamp(0.0, 1.0))
    }

    fn dwell_progress(&self, now: Seconds) -> Option<f32> {
        let dwell = self.editor.connection_mode(self.world).dwell();
        let held_for = self.editor.dwelling(self.world, now)?;
        Some(gui::dwell_progress(held_for, dwell))
    }
}

#[derive(Default)]
pub struct Feedback {
    ghost: Option<Ghost>,
    landing_ghosts: Vec<LandingGhost>,
    hold_ring: Option<HoldRing>,
    attach_ring: Option<AttachRing>,
    attach_burst: Option<AttachBurst>,
    loupe_snap: Option<Seconds>,
    loupe: Option<LoupeCorner>,
    loupe_focus: Option<(LoupeFocus, Seconds)>,
}

impl Feedback {
    pub fn ghost(&self) -> Option<Ghost> {
        self.ghost
    }

    pub fn landing_ghosts(&self) -> &[LandingGhost] {
        &self.landing_ghosts
    }

    pub fn follow_hold(
        &mut self,
        pointer: &Pointer,
        application: &visula::Application,
        holds: bool,
        grabbed: bool,
        now: Seconds,
    ) {
        if grabbed {
            tick();
        }
        let pending = pointer.mouse.pending_hold(now).filter(|_| holds);
        let pending = pending.and_then(|(start, held_for)| {
            let position = camera::ground_position(application, start);
            let (target, _) = pointer.editor.hold_candidate(pointer.world, position)?;
            Some((target, held_for))
        });
        self.hold_ring = HoldRing::next(self.hold_ring, pending, grabbed, now);
    }

    pub fn attached(&mut self, target: Entity, now: Seconds) {
        thump();
        self.attach_ring = None;
        self.loupe_snap = None;
        self.attach_burst = Some(AttachBurst { target, since: now });
    }

    pub fn follow_dwell(&mut self, pointer: &Pointer, now: Seconds) {
        self.attach_burst = self.attach_burst.filter(|burst| burst.look(now).is_some());
        let progress = pointer.dwell_progress(now);
        let ending = pointer.editor.ending();
        self.attach_ring = AttachRing::next(self.attach_ring, ending, progress, false, now);
    }

    pub fn follow_ending(&mut self, pointer: &Pointer) {
        let ending = pointer.editor.ending();
        let shown = self.attach_ring.and_then(AttachRing::shown);
        let entered = ending.is_some() && shown.is_none();
        let left = ending.is_none() && shown.is_some() && pointer.mouse.left_down;
        let now = input::now();
        if entered {
            tick();
        }
        if left {
            snap();
            self.loupe_snap = Some(now);
        }
        let progress = pointer.dwell_progress(now);
        self.attach_ring = AttachRing::next(self.attach_ring, ending, progress, entered, now);
    }

    pub fn land_placed(&mut self, placed: Option<PlacedNeuron>, theme: Theme) {
        let now = input::now();
        self.landing_ghosts
            .retain(|landing| landing.progress(now) < 1.0);
        let Some(PlacedNeuron { neuron, painted }) = placed else {
            return;
        };
        if painted {
            tick();
        }
        let from = self.ghost.and_then(|ghost| ghost.sphere(&theme.palette()));
        self.landing_ghosts
            .extend(from.map(|from| LandingGhost::new(neuron, from, now)));
    }

    fn landing_flash(&self) -> f32 {
        let now = input::now();
        let latest = self.landing_ghosts.last();
        latest.map_or(0.0, |landing| 1.0 - landing.progress(now))
    }

    pub fn dangle_ghost(
        &mut self,
        pointer: &Pointer,
        application: &visula::Application,
        elapsed: Seconds,
    ) {
        let editing = pointer.editor.tool.edits();
        let painting = pointer
            .editor
            .painting()
            .filter(|_| editing && pointer.mouse.moved_while_pressed());
        if painting.is_none() {
            if let Some((element, anchor)) = pointer.hover_anchor(application) {
                self.ghost
                    .get_or_insert_with(|| Ghost::new(element, anchor))
                    .follow(element, anchor, 1.0, false, elapsed.0 as f32);
                return;
            }
        }
        let (Some(element), Some(position)) = (painting, pointer.mouse.position) else {
            self.ghost = None;
            return;
        };
        let offset = ghost::offset_points(pointer.touch_input) * application.window.scale_factor();
        let above = PhysicalPosition::new(position.x, position.y - offset);
        let anchor = camera::position_at_height(application, above, GHOST_HEIGHT);
        let way = pointer.way_to_next(application).unwrap_or(0.0);
        self.ghost
            .get_or_insert_with(|| Ghost::new(element, anchor))
            .follow(element, anchor, way, pointer.touch_input, elapsed.0 as f32);
    }

    pub fn shows_loupe(&self, pointer: &Pointer) -> bool {
        let carrying = pointer.mouse.moved_while_pressed() || pointer.mouse.has_held();
        let aiming = carrying && pointer.editor.aims();
        pointer.touch_input && (aiming || self.candidate(pointer).is_some())
    }

    pub fn candidate(&self, pointer: &Pointer) -> Option<Entity> {
        let (_, held_for) = pointer.mouse.pending_hold(input::now())?;
        if !pointer.touch_input || held_for < LOUPE_DELAY {
            return None;
        }
        match self.hold_ring {
            Some(HoldRing::Holding { target, .. }) => Some(target),
            _ => pointer.editor.picked(),
        }
    }

    pub fn prepare_loupe(&self, pointer: &Pointer, application: &mut visula::Application) {
        if self.shows_loupe(pointer) {
            gui::prepare_loupe(application);
        }
    }

    pub fn paint(
        &mut self,
        pointer: &Pointer,
        application: &visula::Application,
        context: &egui::Context,
        theme: Theme,
    ) {
        self.paint_rings(pointer, application, context, theme);
        self.paint_loupe(pointer, application, context, theme);
        self.paint_cursor_ring(pointer, application, context, theme);
    }

    fn paint_rings(
        &self,
        pointer: &Pointer,
        application: &visula::Application,
        context: &egui::Context,
        theme: Theme,
    ) {
        let now = input::now();
        let hold = self
            .hold_ring
            .and_then(|ring| Some((ring.target(), ring.look(now)?)));
        let attach = self
            .attach_ring
            .and_then(|ring| Some((ring.target(), ring.look(now)?)));
        let burst = self
            .attach_burst
            .and_then(|burst| Some((burst.target, burst.look(now)?)));
        let waiting = pointer
            .editor
            .axon_waiting(pointer.world)
            .map(|(soma, _, _)| (soma, WAITING));
        let rings = hold.into_iter().chain(attach).chain(burst).chain(waiting);
        for (target, look) in rings {
            paint_ring(pointer.world, application, context, theme, target, look);
        }
    }

    fn loupe_rim(
        &self,
        pointer: &Pointer,
        application: &visula::Application,
    ) -> (Option<f32>, f32) {
        let now = input::now();
        match self.hold_ring {
            Some(HoldRing::Holding { held_for, .. }) => {
                let progress = (held_for - LOUPE_DELAY) / (input::HOLD - LOUPE_DELAY);
                return (Some(progress.clamp(0.0, 1.0) as f32), 0.0);
            }
            Some(ring @ HoldRing::Grabbed { .. }) => {
                let flash = ring.look(now).map_or(0.0, |look| look.opacity);
                return (None, flash);
            }
            _ => {}
        }
        let progress = pointer
            .dwell_progress(now)
            .or_else(|| pointer.way_to_next(application));
        let flash = self
            .attach_burst
            .and_then(|burst| burst.look(now))
            .map_or(0.0, |look| look.opacity);
        (progress, flash.max(self.landing_flash()))
    }

    fn loupe_subject(
        &self,
        pointer: &Pointer,
        application: &visula::Application,
        pixels_per_point: f32,
    ) -> Option<Subject> {
        let candidate = self.candidate(pointer);
        let subject = candidate.or_else(|| pointer.editor.picked())?;
        let shape = Targets::new(pointer.world).shape(subject)?;
        let per_point = camera::ground_length(application, f64::from(pixels_per_point));
        let center = camera::screen_position(application, shape.center());
        Some(Subject {
            center: egui::pos2(center.x as f32, center.y as f32) / pixels_per_point,
            size: shape.size() / per_point,
            lifted: candidate.is_none(),
        })
    }

    fn paint_loupe(
        &mut self,
        pointer: &Pointer,
        application: &visula::Application,
        context: &egui::Context,
        theme: Theme,
    ) {
        let finger = pointer.mouse.position.filter(|_| self.shows_loupe(pointer));
        let Some(finger) = finger else {
            self.loupe = None;
            self.loupe_focus = None;
            return;
        };
        let pixels_per_point = context.pixels_per_point();
        let finger = egui::pos2(finger.x as f32, finger.y as f32) / pixels_per_point;
        let subject = self.loupe_subject(pointer, application, pixels_per_point);
        let target = LoupeFocus::of(finger, subject);
        let now = input::now();
        let focus = match self.loupe_focus {
            Some((previous, at)) => target.eased(previous, (now - at).0 as f32),
            None => target,
        };
        self.loupe_focus = Some((focus, now));
        let size = egui::vec2(
            application.config.width as f32,
            application.config.height as f32,
        );
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size / pixels_per_point);
        let corner = LoupeCorner::next(self.loupe, finger, screen);
        self.loupe = Some(corner);
        let (progress, flash) = self.loupe_rim(pointer, application);
        let snap = self
            .loupe_snap
            .map(|since| ((now - since) / LOUPE_SNAP) as f32)
            .filter(|&snap| snap < 1.0);
        if snap.is_none() {
            self.loupe_snap = None;
        }
        let view = LoupeView {
            corner,
            finger,
            focus,
            screen,
            srgb_target: application.config.format.is_srgb(),
            attached: pointer.editor.ending().is_some(),
            progress,
            flash,
            snap,
        };
        gui::paint_loupe(context, theme, view);
    }

    fn paint_cursor_ring(
        &self,
        pointer: &Pointer,
        application: &visula::Application,
        context: &egui::Context,
        theme: Theme,
    ) {
        let position = pointer
            .mouse
            .position
            .filter(|_| !pointer.touch_input && pointer.mouse.moved_while_pressed());
        let (Some(position), Some(way)) = (position, pointer.way_to_next(application)) else {
            return;
        };
        let center = egui::pos2(position.x as f32, position.y as f32) / context.pixels_per_point();
        gui::paint_cursor_ring(context, theme, center, way, self.landing_flash());
    }
}

fn paint_ring(
    world: &World,
    application: &visula::Application,
    context: &egui::Context,
    theme: Theme,
    target: Entity,
    look: Look,
) {
    let Some(shape) = Targets::new(world).shape(target) else {
        return;
    };
    let pixels_per_point = context.pixels_per_point();
    let per_point = camera::ground_length(application, f64::from(pixels_per_point));
    let gap = per_point * look.gap(shape.thickness() / per_point);
    let to_ground = camera::ground_under(application, context);
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
    gui::paint_hold_ring(context, theme, &inner, &outer, &look);
}
