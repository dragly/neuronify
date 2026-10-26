use chrono::{DateTime, Utc};
use egui::epaint::{Mesh, TessellationOptions, Tessellator};
use egui::{vec2, Color32, ColorImage, Shape, Vec2};

const TRIANGLES_BETWEEN_CLOCK_CHECKS: usize = 256;

pub struct UnfinishedDrawing {
    pub mesh_in_pixels: Mesh,
    pub next_triangle: usize,
    pub premultiplied_pixels: Vec<[f32; 4]>,
}

pub fn tessellate_untextured_in_pixels(shapes: Vec<Shape>, pixels_per_point: f32) -> Mesh {
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
    pub fn fill_triangles_until(
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

    pub fn color_image(&self, size: [usize; 2]) -> ColorImage {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;
    use crate::examples::EXAMPLE_CATEGORIES;
    use crate::landing::preview::THUMBNAIL_PADDING;
    use crate::landing::top_down::{egui_color_from_linear, top_down_shapes};
    use crate::rendering::{circuit_bounds, colors, Theme};
    use chrono::Duration;
    use egui::{pos2, vec2, Pos2, Rect, Vec2};
    use glam::Vec3;
    use hecs::World;

    fn unfinished_drawing(world: &World, size: Vec2, pixels_per_point: f32) -> UnfinishedDrawing {
        let pixels = size * pixels_per_point;
        let area = Rect::from_min_size(Pos2::ZERO, size).shrink(THUMBNAIL_PADDING);
        let shapes = top_down_shapes(
            area,
            world,
            &circuit_bounds(world).unwrap(),
            &Theme::Dark.palette(),
        );
        UnfinishedDrawing {
            mesh_in_pixels: tessellate_untextured_in_pixels(shapes, pixels_per_point),
            next_triangle: 0,
            premultiplied_pixels: vec![[0.0; 4]; (pixels.x * pixels.y) as usize],
        }
    }

    #[test]
    fn still_preview_draws_the_world_where_a_live_one_would() {
        let mut world = World::new();
        Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(0.0, 0.0, -10.0));
        Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(0.0, 0.0, 10.0));
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
        for shape in top_down_shapes(area, &world, &bounds, &Theme::Dark.palette()) {
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
}
