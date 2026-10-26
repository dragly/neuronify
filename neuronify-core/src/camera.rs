use glam::{Vec3, Vec4};
use visula::winit::dpi::PhysicalPosition;
use visula::winit::event::MouseScrollDelta;

use crate::rendering::Bounds;

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 200.0;
const DEFAULT_DISTANCE: f32 = 50.0;
const FIELD_OF_VIEW_DEGREES: f32 = 40.0;
const FRAMING_MARGIN: f32 = 1.15;
const ZOOM_PER_SCROLL_LINE: f32 = 0.1;
const PIXELS_PER_SCROLL_LINE: f32 = 100.0;

pub fn set_up(application: &mut visula::Application) {
    let camera = &mut application.camera_controller;
    camera.enabled = false;
    camera.target_transform.center = Vec3::ZERO;
    camera.target_transform.forward = Vec3::new(0.3, -1.0, 0.0).normalize();
    camera.target_transform.distance = DEFAULT_DISTANCE;
    camera.current_transform = camera.target_transform.clone();
}

pub fn ground_position(application: &visula::Application, position: PhysicalPosition<f64>) -> Vec3 {
    let ndc_x = 2.0 * position.x as f32 / application.config.width as f32 - 1.0;
    let ndc_y = 1.0 - 2.0 * position.y as f32 / application.config.height as f32;
    let ray_clip = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let aspect_ratio = application.config.width as f32 / application.config.height as f32;
    let camera = &application.camera_controller;
    let ray_eye = camera.projection_matrix(aspect_ratio).inverse() * ray_clip;
    let ray_eye = Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
    let ray_world = (camera.view_matrix().inverse() * ray_eye)
        .truncate()
        .normalize();
    let ray_origin = camera.position();
    let t = -ray_origin.y / ray_world.y;
    ray_origin + t * ray_world
}

pub fn pan(application: &mut visula::Application, offset: Vec3) {
    let camera = &mut application.camera_controller;
    camera.target_transform.center -= offset;
    camera.current_transform.center = camera.target_transform.center;
}

pub fn pinch(
    application: &mut visula::Application,
    previous: [PhysicalPosition<f64>; 2],
    current: [PhysicalPosition<f64>; 2],
) {
    let spread = |[a, b]: [PhysicalPosition<f64>; 2]| (a.x - b.x).hypot(a.y - b.y);
    let midpoint = |[a, b]: [PhysicalPosition<f64>; 2]| {
        PhysicalPosition::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
    };
    let anchor = ground_position(application, midpoint(previous));
    let camera = &mut application.camera_controller;
    if spread(previous) > 0.0 && spread(current) > 0.0 {
        let zoom = (spread(previous) / spread(current)) as f32;
        camera.current_transform.distance =
            (camera.current_transform.distance * zoom).clamp(MIN_DISTANCE, MAX_DISTANCE);
        camera.target_transform.distance = camera.current_transform.distance;
    }
    let under_fingers = ground_position(application, midpoint(current));
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

pub fn frame(application: &mut visula::Application, bounds: Option<Bounds>) {
    let aspect_ratio = application.config.width as f32 / application.config.height.max(1) as f32;
    let (center, distance) = match bounds {
        Some(bounds) => {
            let height_on_screen = bounds.size().x;
            let width_on_screen = bounds.size().z;
            let tangent = (FIELD_OF_VIEW_DEGREES.to_radians() / 2.0).tan();
            let fit = (height_on_screen / tangent).max(width_on_screen / (tangent * aspect_ratio))
                / 2.0
                * FRAMING_MARGIN;
            (bounds.center().with_y(0.0), fit)
        }
        None => (Vec3::ZERO, DEFAULT_DISTANCE),
    };
    let camera = &mut application.camera_controller;
    camera.target_transform.center = center;
    camera.target_transform.distance = distance.clamp(DEFAULT_DISTANCE, MAX_DISTANCE);
    camera.current_transform = camera.target_transform.clone();
}
