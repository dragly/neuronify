use egui::{vec2, Pos2};

use super::drawing::hand_outline;
use crate::gui::icons::{linear, Glyph, CREAM};
use crate::rendering::colors::orange;

pub fn icon(glyph: &Glyph) {
    glyph
        .painter
        .circle_filled(glyph.at(16.0, 16.0), 11.5 * glyph.scale, linear(orange()));
    let hand = hand_outline();
    let reach = hand
        .iter()
        .map(|point| point.abs().max_element())
        .fold(0.0, f32::max);
    let size = 7.0 * glyph.scale / reach;
    let points: Vec<Pos2> = hand
        .iter()
        .map(|point| glyph.at(16.0, 16.0) + vec2(point.x, -point.y) * size)
        .collect();
    let line = glyph.stroke(1.4, CREAM);
    for (index, &point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        glyph.painter.line_segment([point, next], line);
        glyph.painter.circle_filled(point, line.width / 2.0, CREAM);
    }
}
