use glam::{Vec3, Vec4};
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::MouseScrollDelta;

use crate::input::midpoint;
use crate::rendering::Bounds;

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 200.0;
const DEFAULT_DISTANCE: f32 = 50.0;
const EMPTY_SCENE_DISTANCE: f32 = 2.0 * DEFAULT_DISTANCE;
const FIELD_OF_VIEW_DEGREES: f32 = 40.0;
const TILT_DEGREES: f32 = 6.7;
const FRAMING_MARGIN: f32 = 1.15;
const SIDE_FRAMING_MARGIN: f32 = 1.25;
const ZOOM_PER_SCROLL_LINE: f32 = 0.1;
const PIXELS_PER_SCROLL_LINE: f32 = 100.0;
const POINTS_PER_ZOOM_DOUBLING: f64 = 150.0;

pub fn set_up(application: &mut visula::Application) {
    let camera = &mut application.camera_controller;
    camera.enabled = false;
    camera.target_transform.center = Vec3::ZERO;
    let tilt = TILT_DEGREES.to_radians();
    camera.target_transform.forward = Vec3::new(tilt.sin(), -tilt.cos(), 0.0);
    camera.target_transform.distance = DEFAULT_DISTANCE;
    camera.current_transform = camera.target_transform.clone();
}

pub fn ground_position(application: &visula::Application, position: PhysicalPosition<f64>) -> Vec3 {
    position_at_height(application, position, 0.0)
}

pub fn ground_length(application: &visula::Application, pixels: f64) -> f32 {
    let center = PhysicalPosition::new(
        f64::from(application.config.width) / 2.0,
        f64::from(application.config.height) / 2.0,
    );
    let beside = PhysicalPosition::new(center.x + pixels, center.y);
    ground_position(application, center).distance(ground_position(application, beside))
}

pub fn position_at_height(
    application: &visula::Application,
    position: PhysicalPosition<f64>,
    height: f32,
) -> Vec3 {
    positions_at_height(application, height)(position)
}

pub fn positions_at_height(
    application: &visula::Application,
    height: f32,
) -> impl Fn(PhysicalPosition<f64>) -> Vec3 {
    let width = application.config.width as f32;
    let screen_height = application.config.height as f32;
    let camera = &application.camera_controller;
    let eye_from_clip = camera.projection_matrix(width / screen_height).inverse();
    let world_from_eye = camera.view_matrix().inverse();
    let ray_origin = camera.position();
    move |position| {
        let ndc_x = 2.0 * position.x as f32 / width - 1.0;
        let ndc_y = 1.0 - 2.0 * position.y as f32 / screen_height;
        let ray_eye = eye_from_clip * Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let ray_eye = Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
        let ray_world = (world_from_eye * ray_eye).truncate().normalize();
        let t = (height - ray_origin.y) / ray_world.y;
        ray_origin + t * ray_world
    }
}

pub fn screen_position(application: &visula::Application, position: Vec3) -> PhysicalPosition<f64> {
    let width = application.config.width as f32;
    let height = application.config.height as f32;
    let camera = &application.camera_controller;
    let clip =
        camera.projection_matrix(width / height) * camera.view_matrix() * position.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    PhysicalPosition::new(
        ((ndc.x + 1.0) / 2.0 * width) as f64,
        ((1.0 - ndc.y) / 2.0 * height) as f64,
    )
}

pub fn pan(application: &mut visula::Application, offset: Vec3) {
    let camera = &mut application.camera_controller;
    camera.target_transform.center -= offset;
    camera.current_transform.center = camera.target_transform.center;
}

pub fn slide(
    application: &mut visula::Application,
    from: PhysicalPosition<f64>,
    to: PhysicalPosition<f64>,
) {
    let offset = ground_position(application, to) - ground_position(application, from);
    pan(application, offset);
}

pub fn pinch(
    application: &mut visula::Application,
    previous: [PhysicalPosition<f64>; 2],
    current: [PhysicalPosition<f64>; 2],
) {
    let spread = |[a, b]: [PhysicalPosition<f64>; 2]| (a.x - b.x).hypot(a.y - b.y);
    let zoom = match (spread(previous), spread(current)) {
        (before, after) if before > 0.0 && after > 0.0 => (before / after) as f32,
        _ => 1.0,
    };
    zoom_and_pan(application, zoom, midpoint(previous), midpoint(current));
}

pub fn drag_zoom(
    application: &mut visula::Application,
    anchor: PhysicalPosition<f64>,
    previous: PhysicalPosition<f64>,
    current: PhysicalPosition<f64>,
) {
    let points = (current.y - previous.y) / application.window.scale_factor();
    let zoom = (-points / POINTS_PER_ZOOM_DOUBLING).exp2() as f32;
    zoom_and_pan(application, zoom, anchor, anchor);
}

fn zoom_and_pan(
    application: &mut visula::Application,
    zoom: f32,
    from: PhysicalPosition<f64>,
    to: PhysicalPosition<f64>,
) {
    let anchor = ground_position(application, from);
    let camera = &mut application.camera_controller;
    camera.current_transform.distance =
        (camera.current_transform.distance * zoom).clamp(MIN_DISTANCE, MAX_DISTANCE);
    camera.target_transform.distance = camera.current_transform.distance;
    let under_fingers = ground_position(application, to);
    let camera = &mut application.camera_controller;
    camera.current_transform.center += anchor - under_fingers;
    camera.target_transform.center = camera.current_transform.center;
}

pub fn scroll(application: &mut visula::Application, delta: &MouseScrollDelta) {
    let lines = match delta {
        MouseScrollDelta::LineDelta(_, y) => *y,
        MouseScrollDelta::PixelDelta(position) => position.y as f32 / PIXELS_PER_SCROLL_LINE,
    };
    let target = &mut application.camera_controller.target_transform;
    target.distance =
        (target.distance * (1.0 - lines * ZOOM_PER_SCROLL_LINE)).clamp(MIN_DISTANCE, MAX_DISTANCE);
}

pub fn frame(application: &mut visula::Application, bounds: Option<Bounds>, free: egui::Rect) {
    let screen_width = application.config.width as f32;
    let screen_height = application.config.height.max(1) as f32;
    let aspect_ratio = screen_width / screen_height;
    let (center, distance) = match bounds {
        Some(bounds) => {
            let height_on_screen = bounds.size().x / (free.height() / screen_height);
            let width_on_screen = bounds.size().z / (free.width() / screen_width);
            let tangent = (FIELD_OF_VIEW_DEGREES.to_radians() / 2.0).tan();
            let fit = (height_on_screen / tangent * FRAMING_MARGIN)
                .max(width_on_screen / (tangent * aspect_ratio) * SIDE_FRAMING_MARGIN)
                / 2.0;
            (bounds.center().with_y(0.0), fit)
        }
        None => (Vec3::ZERO, EMPTY_SCENE_DISTANCE),
    };
    let camera = &mut application.camera_controller;
    camera.target_transform.center = center;
    camera.target_transform.distance = distance.clamp(DEFAULT_DISTANCE, MAX_DISTANCE);
    camera.current_transform = camera.target_transform.clone();
    let middle = PhysicalPosition::new(free.center().x as f64, free.center().y as f64);
    let offset = center - ground_position(application, middle);
    let camera = &mut application.camera_controller;
    camera.target_transform.center += offset;
    camera.current_transform = camera.target_transform.clone();
}
