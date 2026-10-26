use egui::{vec2, Color32, Pos2, Rect, Shape, Stroke};
use glam::Vec3;
use hecs::World;

use crate::rendering::{
    collect_scene, collect_voltmeter_traces, collect_voltmeter_wires, Bounds, ConnectionData,
    CylinderData, Palette,
};

const THUMBNAIL_MIN_CANVAS_EXTENT: f32 = 20.0;
const CONNECTION_WIDTH_PER_STRENGTH: f32 = 0.3;
const MIN_LINE_WIDTH: f32 = 1.0;
const MIN_TRACE_POINT_SPACING: f32 = 1.0;

pub fn egui_color_from_linear(color: Vec3) -> Color32 {
    egui::Rgba::from_rgb(color.x, color.y, color.z).into()
}

fn rendered_line_width(strength: f32, scale: f32) -> f32 {
    (strength * CONNECTION_WIDTH_PER_STRENGTH * scale).max(MIN_LINE_WIDTH)
}

pub fn top_down_shapes(
    area: Rect,
    world: &World,
    bounds: &Bounds,
    palette: &Palette,
) -> Vec<Shape> {
    let size = bounds.size();
    let scale = (area.width() / size.z.max(THUMBNAIL_MIN_CANVAS_EXTENT))
        .min(area.height() / size.x.max(THUMBNAIL_MIN_CANVAS_EXTENT));
    let center = bounds.center();
    let to_screen = |p: Vec3| area.center() + vec2(p.z - center.z, center.x - p.x) * scale;
    let scene = collect_scene(world, palette);
    let mut shapes: Vec<Shape> = scene
        .cylinders
        .iter()
        .map(|cylinder| cylinder_from_above(cylinder, scale, to_screen))
        .collect();
    for ring in &scene.rings {
        shapes.push(Shape::circle_stroke(
            to_screen(ring.position),
            ring.major_radius * scale,
            Stroke::new(
                2.0 * ring.minor_radius * scale,
                egui_color_from_linear(ring.color),
            ),
        ));
    }

    for wire in collect_voltmeter_wires(world, palette) {
        shapes.push(Shape::line_segment(
            [to_screen(wire.position_a), to_screen(wire.position_b)],
            Stroke::new(
                rendered_line_width(wire.strength, scale),
                egui_color_from_linear(wire.start_color),
            ),
        ));
    }

    shapes.extend(joined_voltmeter_traces(
        collect_voltmeter_traces(world),
        scale,
        to_screen,
    ));

    for sphere in scene.spheres {
        shapes.push(Shape::circle_filled(
            to_screen(sphere.position),
            sphere.radius * scale,
            egui_color_from_linear(sphere.color),
        ));
    }
    shapes
}

fn cylinder_from_above(
    cylinder: &CylinderData,
    scale: f32,
    to_screen: impl Fn(Vec3) -> Pos2,
) -> Shape {
    let color = egui_color_from_linear(cylinder.color);
    let start = to_screen(cylinder.start);
    let end = to_screen(cylinder.end);
    let along = end - start;
    if along.length() <= f32::EPSILON {
        let radius = cylinder.start_radius.max(cylinder.end_radius);
        return Shape::circle_filled(start, radius * scale, color);
    }
    let across = along.normalized().rot90() * scale;
    Shape::convex_polygon(
        vec![
            start + across * cylinder.start_radius,
            end + across * cylinder.end_radius,
            end - across * cylinder.end_radius,
            start - across * cylinder.start_radius,
        ],
        color,
        Stroke::NONE,
    )
}

fn joined_voltmeter_traces(
    segments: Vec<ConnectionData>,
    scale: f32,
    to_screen: impl Fn(Vec3) -> Pos2,
) -> Vec<Shape> {
    let mut paths = Vec::new();
    let mut path: Vec<Pos2> = Vec::new();
    let mut path_stroke = Stroke::NONE;
    let mut path_end = None;
    for segment in segments {
        let stroke = Stroke::new(
            rendered_line_width(segment.strength, scale),
            egui_color_from_linear(segment.start_color),
        );
        if path_end != Some(segment.position_a) || stroke != path_stroke {
            finish_path(&mut paths, &mut path, path_stroke);
            path.push(to_screen(segment.position_a));
            path_stroke = stroke;
        }
        let point = to_screen(segment.position_b);
        if path
            .last()
            .is_some_and(|last| last.distance(point) >= MIN_TRACE_POINT_SPACING)
        {
            path.push(point);
        }
        path_end = Some(segment.position_b);
    }
    finish_path(&mut paths, &mut path, path_stroke);
    paths
}

fn finish_path(paths: &mut Vec<Shape>, path: &mut Vec<Pos2>, stroke: Stroke) {
    if path.len() > 1 {
        paths.push(Shape::line(std::mem::take(path), stroke));
    }
    path.clear();
}
