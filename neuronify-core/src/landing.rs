use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use egui::epaint::{Mesh, TessellationOptions, Tessellator};
use egui::{
    pos2, vec2, Align, Color32, ColorImage, CursorIcon, FontId, Galley, Key, Layout, Painter, Pos2,
    Rect, RichText, ScrollArea, Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions,
    Ui, Vec2, WidgetInfo, WidgetType,
};
use glam::Vec3;
use hecs::World;

use crate::constants::*;
use crate::examples::{Example, EXAMPLE_CATEGORIES};
use crate::rendering::colors::terminal_color;
use crate::rendering::{
    circuit_bounds, collect_connections, collect_scene, collect_voltmeter_traces, Bounds,
    ConnectionData, CylinderData,
};
use crate::simulation;
use crate::tools::Tool;

const BACKGROUND: Color32 = Color32::from_rgb(30, 30, 46);
const SURFACE: Color32 = Color32::from_rgb(49, 50, 68);
const SURFACE_HOVERED: Color32 = Color32::from_rgb(69, 71, 90);
const ACCENT: Color32 = Color32::from_rgb(137, 180, 250);
const TEXT: Color32 = Color32::from_rgb(205, 214, 244);
const SUBTEXT: Color32 = Color32::from_rgb(166, 173, 200);

const MAX_CONTENT_WIDTH: f32 = 1100.0;
const PAGE_MARGIN: f32 = 16.0;
const MIN_CARD_WIDTH: f32 = 240.0;
const CARD_GAP: f32 = 16.0;
const CARD_PADDING: f32 = 10.0;
const CARD_RADIUS: u8 = 10;
const TITLE_GAP: f32 = 4.0;
const THUMBNAIL_ASPECT: f32 = 9.0 / 16.0;
const THUMBNAIL_RADIUS: u8 = 6;
const THUMBNAIL_PADDING: f32 = 10.0;
const THUMBNAIL_MIN_CANVAS_EXTENT: f32 = 20.0;
const CONNECTION_WIDTH_PER_STRENGTH: f32 = 0.3;
const MIN_LINE_WIDTH: f32 = 1.0;
const MIN_TRACE_POINT_SPACING: f32 = 1.0;

const WARM_UP_SPEED: u32 = 16;
const PREVIEW_FADE_IN_SECONDS: f64 = 0.25;
const MAX_LIVE_PREVIEW_ENTITIES: u32 = 1000;
const STEPS_UNTIL_VOLTMETERS_ARE_DRAWN: u32 = 2;
const STILL_DRAWING_MICROSECONDS_PER_FRAME: i64 = 2000;
const TRIANGLES_BETWEEN_CLOCK_CHECKS: usize = 256;

pub enum LandingChoice {
    Example { world: Box<World>, time: f64 },
    EmptyCanvas,
    BackToSimulation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PreviewMode {
    Live,
    Still,
}

fn preview_mode(world: &World) -> PreviewMode {
    if world.len() <= MAX_LIVE_PREVIEW_ENTITIES {
        PreviewMode::Live
    } else {
        PreviewMode::Still
    }
}

struct Preview {
    example: &'static Example,
    world: World,
    time: f64,
    bounds: Option<Bounds>,
    mode: PreviewMode,
    on_screen: bool,
    fade_in_started_at: Option<f64>,
    still_drawing: Option<StillDrawing>,
}

impl Preview {
    fn new(example: &'static Example) -> Option<Preview> {
        let mut world = example
            .spawn()
            .map_err(|e| log::error!("Could not load {}: {e}", example.name))
            .ok()?;
        let mode = preview_mode(&world);
        let mut time = 0.0;
        if mode == PreviewMode::Still {
            simulation::step_frame(&mut world, &mut time, STEPS_UNTIL_VOLTMETERS_ARE_DRAWN);
        }
        Some(Preview {
            example,
            bounds: circuit_bounds(&world),
            mode,
            world,
            time,
            on_screen: false,
            fade_in_started_at: None,
            still_drawing: None,
        })
    }

    fn warmed_up(&self) -> bool {
        self.time >= VOLTMETER_TIME_WINDOW
    }
}

struct StillDrawing {
    size_in_pixels: [usize; 2],
    texture: TextureHandle,
    unfinished: Option<UnfinishedDrawing>,
}

struct UnfinishedDrawing {
    mesh_in_pixels: Mesh,
    next_triangle: usize,
    premultiplied_pixels: Vec<[f32; 4]>,
}

#[derive(Clone, Copy)]
enum Card {
    EmptyCanvas,
    Preview(usize),
}

enum Picked {
    Card(usize, Card),
    BackToSimulation,
}

pub struct LandingScreen {
    previews_by_category: Vec<Vec<Preview>>,
    can_go_back: bool,
    still_drawing_deadline: Option<DateTime<Utc>>,
}

impl LandingScreen {
    pub fn new(can_go_back: bool) -> LandingScreen {
        let previews_by_category = EXAMPLE_CATEGORIES
            .iter()
            .map(|category| category.examples.iter().filter_map(Preview::new).collect())
            .collect();
        LandingScreen {
            previews_by_category,
            can_go_back,
            still_drawing_deadline: None,
        }
    }

    pub fn update(&mut self, iterations: u32) {
        for preview in self.previews_by_category.iter_mut().flatten() {
            if preview.mode == PreviewMode::Still || !preview.on_screen {
                continue;
            }
            let speed = if preview.warmed_up() {
                1
            } else {
                WARM_UP_SPEED
            };
            simulation::step_frame(&mut preview.world, &mut preview.time, iterations * speed);
        }
    }

    pub fn show(&mut self, context: &egui::Context) -> Option<LandingChoice> {
        let mut picked = None;
        self.still_drawing_deadline =
            Some(Utc::now() + Duration::microseconds(STILL_DRAWING_MICROSECONDS_PER_FRAME));
        #[allow(deprecated)]
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BACKGROUND))
            .show(context, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width =
                            (ui.available_width() - 2.0 * PAGE_MARGIN).min(MAX_CONTENT_WIDTH);
                        ui.horizontal_top(|ui| {
                            ui.add_space((ui.available_width() - width) / 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                picked = self.contents(ui, width);
                            });
                        });
                    });
            });
        if self.can_go_back && context.input(|input| input.key_pressed(Key::Escape)) {
            picked = Some(Picked::BackToSimulation);
        }
        match picked? {
            Picked::Card(_, Card::EmptyCanvas) => Some(LandingChoice::EmptyCanvas),
            Picked::Card(category, Card::Preview(index)) => {
                let preview = self.previews_by_category[category].swap_remove(index);
                Some(LandingChoice::Example {
                    world: Box::new(preview.world),
                    time: preview.time,
                })
            }
            Picked::BackToSimulation => Some(LandingChoice::BackToSimulation),
        }
    }

    fn contents(&mut self, ui: &mut Ui, width: f32) -> Option<Picked> {
        let mut picked = None;
        ui.add_space(40.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Neuronify").size(34.0).color(TEXT));
            if self.can_go_back {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let button = egui::Button::new(
                        RichText::new("Back to simulation").size(15.0).color(TEXT),
                    )
                    .fill(SURFACE)
                    .corner_radius(CARD_RADIUS)
                    .min_size(vec2(0.0, 36.0));
                    if ui.add(button).clicked() {
                        picked = Some(Picked::BackToSimulation);
                    }
                });
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Simulations of neurons and small circuits. \
                 Open an example below, or start from an empty canvas.",
            )
            .size(16.0)
            .color(SUBTEXT),
        );

        for (category_index, category) in EXAMPLE_CATEGORIES.iter().enumerate() {
            ui.add_space(32.0);
            ui.label(RichText::new(category.name).size(20.0).color(TEXT));
            ui.add_space(12.0);
            let mut cards = Vec::new();
            if category_index == 0 {
                cards.push(Card::EmptyCanvas);
            }
            cards.extend((0..self.previews_by_category[category_index].len()).map(Card::Preview));
            if let Some(card) = self.card_grid(ui, width, category_index, &cards) {
                picked = Some(Picked::Card(category_index, card));
            }
        }
        ui.add_space(40.0);
        picked
    }

    fn card_texts(&self, ui: &Ui, category: usize, card: Card, width: f32) -> [Arc<Galley>; 2] {
        let (name, description) = match card {
            Card::EmptyCanvas => (
                "Empty canvas",
                "Build your own circuit with the tools in the Elements window.",
            ),
            Card::Preview(index) => {
                let example = self.previews_by_category[category][index].example;
                (example.name, example.description)
            }
        };
        let painter = ui.painter();
        [
            painter.layout(name.to_owned(), FontId::proportional(16.0), TEXT, width),
            painter.layout(
                description.to_owned(),
                FontId::proportional(13.5),
                SUBTEXT,
                width,
            ),
        ]
    }

    fn card_grid(
        &mut self,
        ui: &mut Ui,
        width: f32,
        category: usize,
        cards: &[Card],
    ) -> Option<Card> {
        let columns = (((width + CARD_GAP) / (MIN_CARD_WIDTH + CARD_GAP)) as usize).max(1);
        let card_width = (width - CARD_GAP * (columns - 1) as f32) / columns as f32;
        let text_width = card_width - 2.0 * CARD_PADDING;
        let thumbnail_size = vec2(text_width, (text_width * THUMBNAIL_ASPECT).round());
        let mut clicked = None;
        for row in cards.chunks(columns) {
            let texts: Vec<[Arc<Galley>; 2]> = row
                .iter()
                .map(|&card| self.card_texts(ui, category, card, text_width))
                .collect();
            let tallest_text_in_row = texts
                .iter()
                .map(|[name, description]| name.size().y + TITLE_GAP + description.size().y)
                .fold(0.0, f32::max);
            let card_size = vec2(
                card_width,
                3.0 * CARD_PADDING + thumbnail_size.y + tallest_text_in_row,
            );
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                for (&card, [name, description]) in row.iter().zip(texts) {
                    let (rect, response) = ui.allocate_exact_size(card_size, Sense::click());
                    let on_screen = ui.is_rect_visible(rect);
                    if let Card::Preview(index) = card {
                        self.previews_by_category[category][index].on_screen = on_screen;
                    }
                    if !on_screen {
                        continue;
                    }
                    let highlighted = response.hovered() || response.has_focus();
                    let painter = ui.painter();
                    painter.rect(
                        rect,
                        CARD_RADIUS,
                        if highlighted {
                            SURFACE_HOVERED
                        } else {
                            SURFACE
                        },
                        if highlighted {
                            Stroke::new(1.5_f32, ACCENT)
                        } else {
                            Stroke::NONE
                        },
                        StrokeKind::Inside,
                    );
                    let thumbnail = Rect::from_min_size(
                        rect.min + vec2(CARD_PADDING, CARD_PADDING),
                        thumbnail_size,
                    );
                    painter.rect_filled(thumbnail, THUMBNAIL_RADIUS, BACKGROUND);
                    let thumbnail_painter = ui.painter_at(thumbnail);
                    match card {
                        Card::EmptyCanvas => paint_plus(&thumbnail_painter, thumbnail),
                        Card::Preview(index) => {
                            let preview = &mut self.previews_by_category[category][index];
                            match preview.mode {
                                PreviewMode::Live => {
                                    paint_live_preview(&thumbnail_painter, thumbnail, preview)
                                }
                                PreviewMode::Still => paint_still_preview(
                                    &thumbnail_painter,
                                    thumbnail,
                                    preview,
                                    &mut self.still_drawing_deadline,
                                ),
                            }
                        }
                    }
                    let name_position = pos2(
                        rect.left() + CARD_PADDING,
                        thumbnail.bottom() + CARD_PADDING,
                    );
                    let description_position = name_position + vec2(0.0, name.size().y + TITLE_GAP);
                    let label = name.text().to_owned();
                    painter.galley(name_position, name, TEXT);
                    painter.galley(description_position, description, SUBTEXT);
                    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &label));
                    if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        clicked = Some(card);
                    }
                }
            });
            ui.add_space(CARD_GAP);
        }
        clicked
    }
}

fn egui_color_from_linear(color: Vec3) -> Color32 {
    egui::Rgba::from_rgb(color.x, color.y, color.z).into()
}

fn rendered_line_width(strength: f32, scale: f32) -> f32 {
    (strength * CONNECTION_WIDTH_PER_STRENGTH * scale).max(MIN_LINE_WIDTH)
}

fn paint_live_preview(painter: &Painter, thumbnail: Rect, preview: &mut Preview) {
    let Some(bounds) = &preview.bounds else {
        return;
    };
    if !preview.warmed_up() {
        return;
    }
    let now = painter.ctx().input(|input| input.time);
    let fade_in_started_at = *preview.fade_in_started_at.get_or_insert(now);
    let opacity = ((now - fade_in_started_at) / PREVIEW_FADE_IN_SECONDS).min(1.0) as f32;
    if opacity > 0.0 {
        let mut painter = painter.clone();
        painter.multiply_opacity(opacity);
        painter.extend(top_down_shapes(
            thumbnail.shrink(THUMBNAIL_PADDING),
            &preview.world,
            bounds,
        ));
    }
}

fn paint_still_preview(
    painter: &Painter,
    thumbnail: Rect,
    preview: &mut Preview,
    still_drawing_deadline: &mut Option<DateTime<Utc>>,
) {
    let Some(bounds) = &preview.bounds else {
        return;
    };
    let pixels_per_point = painter.pixels_per_point();
    let pixels = (thumbnail.size() * pixels_per_point).round();
    let size_in_pixels = [pixels.x as usize, pixels.y as usize];
    let finished = preview.still_drawing.as_ref().is_some_and(|drawing| {
        drawing.size_in_pixels == size_in_pixels && drawing.unfinished.is_none()
    });
    if let Some(deadline) = (!finished).then(|| still_drawing_deadline.take()).flatten() {
        if preview
            .still_drawing
            .as_ref()
            .is_none_or(|drawing| drawing.size_in_pixels != size_in_pixels)
        {
            let area = Rect::from_min_size(Pos2::ZERO, thumbnail.size()).shrink(THUMBNAIL_PADDING);
            let unfinished = UnfinishedDrawing {
                mesh_in_pixels: tessellate_untextured_in_pixels(
                    top_down_shapes(area, &preview.world, bounds),
                    pixels_per_point,
                ),
                next_triangle: 0,
                premultiplied_pixels: vec![[0.0; 4]; size_in_pixels[0] * size_in_pixels[1]],
            };
            let texture = painter.ctx().load_texture(
                format!("{} preview", preview.example.name),
                unfinished.color_image(size_in_pixels),
                TextureOptions::LINEAR,
            );
            preview.still_drawing = Some(StillDrawing {
                size_in_pixels,
                texture,
                unfinished: Some(unfinished),
            });
        }
        if let Some(drawing) = &mut preview.still_drawing {
            if let Some(unfinished) = &mut drawing.unfinished {
                let done = unfinished.fill_triangles_until(drawing.size_in_pixels, deadline);
                drawing.texture.set(
                    unfinished.color_image(drawing.size_in_pixels),
                    TextureOptions::LINEAR,
                );
                if done {
                    drawing.unfinished = None;
                }
            }
        }
    }
    if let Some(drawing) = &preview.still_drawing {
        let whole_texture = Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0));
        painter.image(
            drawing.texture.id(),
            thumbnail,
            whole_texture,
            Color32::WHITE,
        );
    }
}

fn top_down_shapes(area: Rect, world: &World, bounds: &Bounds) -> Vec<Shape> {
    let size = bounds.size();
    let scale = (area.width() / size.z.max(THUMBNAIL_MIN_CANVAS_EXTENT))
        .min(area.height() / size.x.max(THUMBNAIL_MIN_CANVAS_EXTENT));
    let center = bounds.center();
    let to_screen = |p: Vec3| area.center() + vec2(p.z - center.z, center.x - p.x) * scale;
    let scene = collect_scene(world);
    let mut shapes: Vec<Shape> = scene
        .cylinders
        .iter()
        .map(|cylinder| cylinder_from_above(cylinder, scale, to_screen))
        .collect();

    for connection in collect_connections(world, &Tool::Select, &None) {
        let start = connection.position_a;
        let vector = connection.position_b - start;
        if vector.length() <= f32::EPSILON {
            continue;
        }
        let end_before_terminal =
            start + vector - connection.directional * vector.normalize() * NODE_RADIUS * 2.0;
        shapes.push(Shape::line_segment(
            [to_screen(start), to_screen(end_before_terminal)],
            Stroke::new(
                rendered_line_width(connection.strength, scale),
                egui_color_from_linear(connection.start_color),
            ),
        ));
        if connection.directional > 0.0 {
            shapes.push(Shape::circle_filled(
                to_screen(end_before_terminal),
                0.5 * NODE_RADIUS * scale,
                egui_color_from_linear(terminal_color()),
            ));
        }
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

fn tessellate_untextured_in_pixels(shapes: Vec<Shape>, pixels_per_point: f32) -> Mesh {
    let mut tessellator = Tessellator::new(
        pixels_per_point,
        TessellationOptions::default(),
        [1, 1],
        Vec::new(),
    );
    let mut mesh = Mesh::default();
    for shape in shapes {
        tessellator.tessellate_shape(shape, &mut mesh);
    }
    for vertex in &mut mesh.vertices {
        vertex.pos = (vertex.pos.to_vec2() * pixels_per_point).to_pos2();
    }
    mesh
}

impl UnfinishedDrawing {
    fn fill_triangles_until(
        &mut self,
        [width, height]: [usize; 2],
        deadline: DateTime<Utc>,
    ) -> bool {
        let mesh = &self.mesh_in_pixels;
        let triangles = mesh.indices.len() / 3;
        while self.next_triangle < triangles {
            let end = (self.next_triangle + TRIANGLES_BETWEEN_CLOCK_CHECKS).min(triangles);
            for triangle in self.next_triangle..end {
                let corners = [0, 1, 2].map(|corner| {
                    let vertex = &mesh.vertices[mesh.indices[3 * triangle + corner] as usize];
                    let color = vertex.color.to_array().map(|c| c as f32 / 255.0);
                    (vertex.pos.to_vec2(), color)
                });
                blend_triangle(&mut self.premultiplied_pixels, width, height, corners);
            }
            self.next_triangle = end;
            if Utc::now() >= deadline {
                break;
            }
        }
        self.next_triangle == triangles
    }

    fn color_image(&self, size: [usize; 2]) -> ColorImage {
        let pixels = self
            .premultiplied_pixels
            .iter()
            .map(|pixel| {
                let [r, g, b, a] = pixel.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
                Color32::from_rgba_premultiplied(r, g, b, a)
            })
            .collect();
        ColorImage::new(size, pixels)
    }
}

fn blend_triangle(
    premultiplied_pixels: &mut [[f32; 4]],
    width: usize,
    height: usize,
    corners: [(Vec2, [f32; 4]); 3],
) {
    let [(a, _), (b, _), (c, _)] = corners;
    let area = (b - a).x * (c - a).y - (c - a).x * (b - a).y;
    if area.abs() < f32::EPSILON {
        return;
    }
    let top = a.y.min(b.y).min(c.y);
    let bottom = a.y.max(b.y).max(c.y);
    let first_row = (top - 0.5).ceil().max(0.0) as usize;
    let end_row = ((bottom - 0.5).floor() + 1.0).clamp(0.0, height as f32) as usize;
    for row in first_row..end_row {
        let y = row as f32 + 0.5;
        let crossings = [(a, b), (b, c), (c, a)]
            .into_iter()
            .filter_map(|(p, q)| edge_crossing_at_row(p, q, y));
        let (left, right) = crossings.fold((f32::INFINITY, f32::NEG_INFINITY), |(l, r), x| {
            (l.min(x), r.max(x))
        });
        let first_column = (left - 0.5).ceil().max(0.0) as usize;
        let end_column = ((right - 0.5).floor() + 1.0).clamp(0.0, width as f32) as usize;
        for column in first_column..end_column {
            let point = vec2(column as f32 + 0.5, y);
            let weight_a = ((b - point).x * (c - point).y - (c - point).x * (b - point).y) / area;
            let weight_b = ((c - point).x * (a - point).y - (a - point).x * (c - point).y) / area;
            let weights = [weight_a, weight_b, 1.0 - weight_a - weight_b];
            let mut color = [0.0; 4];
            for (weight, (_, corner)) in weights.iter().zip(&corners) {
                for channel in 0..4 {
                    color[channel] += weight.clamp(0.0, 1.0) * corner[channel];
                }
            }
            let pixel = &mut premultiplied_pixels[row * width + column];
            for channel in 0..4 {
                pixel[channel] = color[channel] + pixel[channel] * (1.0 - color[3]);
            }
        }
    }
}

fn edge_crossing_at_row(p: Vec2, q: Vec2, y: f32) -> Option<f32> {
    let horizontal = p.y == q.y;
    let spans_row = p.y.min(q.y) <= y && y <= p.y.max(q.y);
    (!horizontal && spans_row).then(|| p.x + (y - p.y) * (q.x - p.x) / (q.y - p.y))
}

fn paint_plus(painter: &Painter, area: Rect) {
    let center = area.center();
    let arm = area.height() * 0.15;
    let stroke = Stroke::new(2.0_f32, SUBTEXT);
    painter.line_segment([center - vec2(arm, 0.0), center + vec2(arm, 0.0)], stroke);
    painter.line_segment([center - vec2(0.0, arm), center + vec2(0.0, arm)], stroke);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::rendering::colors;
    use egui::{Event, PointerButton, RawInput};

    fn run(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        run_sized(context, landing, events, vec2(1280.0, 800.0))
    }

    fn run_sized(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
        screen: Vec2,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        run_at(context, landing, events, screen, 0.0)
    }

    fn run_at(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
        screen: Vec2,
        time: f64,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut choice = None;
        let output = context.run_ui(input, |ui| choice = landing.show(ui.ctx()));
        (output, choice)
    }

    fn text_position(output: &egui::FullOutput, text: &str) -> Option<Pos2> {
        output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => Some(shape.pos),
                _ => None,
            })
    }

    #[test]
    fn clicking_a_card_opens_its_example() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let (output, _) = run(&context, &mut landing, vec![]);
        let position = text_position(&output, "Tutorial 1: Intro").unwrap() + vec2(4.0, 4.0);

        let click = |pressed| Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        run(
            &context,
            &mut landing,
            vec![Event::PointerMoved(position), click(true)],
        );
        let (_, choice) = run(&context, &mut landing, vec![click(false)]);

        let Some(LandingChoice::Example { world, .. }) = choice else {
            panic!("clicking the card did not open an example");
        };
        assert_eq!(world.query::<&LeakyNeuron>().iter().count(), 1);
    }

    #[test]
    fn escape_goes_back_only_when_there_is_something_to_go_back_to() {
        let escape = || {
            vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }]
        };
        let context = egui::Context::default();
        let (_, choice) = run(&context, &mut LandingScreen::new(false), escape());
        assert!(choice.is_none());
        let (_, choice) = run(&context, &mut LandingScreen::new(true), escape());
        assert!(matches!(choice, Some(LandingChoice::BackToSimulation)));
    }

    fn neuron_at(world: &mut World, position: Vec3) -> hecs::Entity {
        world.spawn((
            Position { position },
            LeakyNeuron::default(),
            LeakyDynamics::default(),
            LeakCurrent::default(),
            NeuronType::Excitatory,
        ))
    }

    #[test]
    fn small_examples_run_live_and_large_ones_are_still() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            assert_eq!(preview_mode(&world), PreviewMode::Live, "{}", example.name);
        }
        let mut world = World::new();
        for i in 0..=MAX_LIVE_PREVIEW_ENTITIES {
            neuron_at(&mut world, Vec3::new(i as f32, 0.0, 0.0));
        }
        assert_eq!(preview_mode(&world), PreviewMode::Still);
    }

    #[test]
    fn previews_out_of_view_do_not_run() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let window_with_room_for_one_row = vec2(1280.0, 400.0);
        run_sized(&context, &mut landing, vec![], window_with_room_for_one_row);
        landing.update(4);
        let first = &landing.previews_by_category[0][0];
        let last = landing.previews_by_category.last().unwrap().last().unwrap();
        assert!(first.on_screen && first.time > 0.0);
        assert!(!last.on_screen && last.time == 0.0);
    }

    fn unfinished_drawing(world: &World, size: Vec2, pixels_per_point: f32) -> UnfinishedDrawing {
        let pixels = size * pixels_per_point;
        let area = Rect::from_min_size(Pos2::ZERO, size).shrink(THUMBNAIL_PADDING);
        let shapes = top_down_shapes(area, world, &circuit_bounds(world).unwrap());
        UnfinishedDrawing {
            mesh_in_pixels: tessellate_untextured_in_pixels(shapes, pixels_per_point),
            next_triangle: 0,
            premultiplied_pixels: vec![[0.0; 4]; (pixels.x * pixels.y) as usize],
        }
    }

    #[test]
    fn still_preview_draws_the_world_where_a_live_one_would() {
        let mut world = World::new();
        neuron_at(&mut world, Vec3::new(0.0, 0.0, -10.0));
        neuron_at(&mut world, Vec3::new(0.0, 0.0, 10.0));
        let size = vec2(200.0, 120.0);
        let mut drawing = unfinished_drawing(&world, size, 2.0);
        let far_away = Utc::now() + Duration::days(1);
        assert!(drawing.fill_triangles_until([400, 240], far_away));
        let image = drawing.color_image([400, 240]);

        let area = Rect::from_min_size(Pos2::ZERO, size).shrink(THUMBNAIL_PADDING);
        let pixel =
            |point: Pos2| image.pixels[(point.y * 2.0) as usize * 400 + (point.x * 2.0) as usize];
        let resting = egui_color_from_linear(colors::blue());
        let bounds = circuit_bounds(&world).unwrap();
        for shape in top_down_shapes(area, &world, &bounds) {
            if let Shape::Circle(circle) = shape {
                assert_eq!(circle.fill, resting);
                let drawn = pixel(circle.center);
                for (a, b) in drawn.to_array().iter().zip(resting.to_array()) {
                    assert!(a.abs_diff(b) <= 2, "{drawn:?} is not {resting:?}");
                }
            }
        }
        assert_eq!(pixel(pos2(1.0, 1.0)), Color32::TRANSPARENT);
        assert_eq!(pixel(area.center()), Color32::TRANSPARENT);
    }

    #[test]
    fn still_preview_drawn_over_several_frames_matches_one_drawn_at_once() {
        let world = EXAMPLE_CATEGORIES
            .iter()
            .flat_map(|c| c.examples)
            .find(|e| e.name == "Lateral Inhibition 2")
            .unwrap()
            .spawn()
            .unwrap();
        let size = vec2(200.0, 120.0);
        let mut at_once = unfinished_drawing(&world, size, 1.0);
        assert!(at_once.fill_triangles_until([200, 120], Utc::now() + Duration::days(1)));

        let mut in_pieces = unfinished_drawing(&world, size, 1.0);
        let deadline_already_passed = Utc::now() - Duration::days(1);
        let mut frames = 0;
        while !in_pieces.fill_triangles_until([200, 120], deadline_already_passed) {
            frames += 1;
        }
        assert!(frames > 1);
        assert_eq!(in_pieces.premultiplied_pixels, at_once.premultiplied_pixels);
    }

    #[test]
    fn live_previews_are_hidden_while_warming_up_and_then_fade_in() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let screen = vec2(1280.0, 800.0);
        let neuron_opacities = |output: &egui::FullOutput| -> Vec<u8> {
            output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    Shape::Circle(circle) => Some(circle.fill.a()),
                    _ => None,
                })
                .collect()
        };
        let mut frame = 0;
        let mut next_frame = |landing: &mut LandingScreen| {
            landing.update(4);
            frame += 1;
            run_at(&context, landing, vec![], screen, frame as f64 / 60.0).0
        };

        let output = next_frame(&mut landing);
        assert!(neuron_opacities(&output).is_empty());
        while !landing.previews_by_category[0][0].warmed_up() {
            let output = next_frame(&mut landing);
            assert!(neuron_opacities(&output).is_empty());
        }
        let fade_frames = (PREVIEW_FADE_IN_SECONDS * 60.0) as usize;
        let mut output = next_frame(&mut landing);
        for _ in 0..fade_frames / 2 {
            output = next_frame(&mut landing);
        }
        let halfway_through_fade = neuron_opacities(&output);
        assert!(
            halfway_through_fade.iter().any(|&a| 0 < a && a < 255),
            "{halfway_through_fade:?}"
        );
        for _ in 0..fade_frames {
            output = next_frame(&mut landing);
        }
        let shown = neuron_opacities(&output);
        assert!(!shown.is_empty() && shown.iter().all(|&a| a == 255));
    }
}
