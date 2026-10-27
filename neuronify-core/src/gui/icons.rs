use egui::{vec2, Color32, Painter, Rect, Shape, Stroke, StrokeKind, Vec2};

const GRID: f32 = 24.0;
const STROKE: f32 = 1.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Pencil,
    Grid,
}

pub fn paint_icon(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let scale = rect.width().min(rect.height()) / GRID;
    let origin = rect.center() - Vec2::splat(GRID / 2.0) * scale;
    let at = |point: Vec2| origin + point * scale;
    let stroke = Stroke::new(STROKE * scale, color);
    let outline = |points: &[Vec2]| {
        painter.add(Shape::closed_line(
            points.iter().map(|&point| at(point)).collect(),
            stroke,
        ));
    };
    match icon {
        Icon::Pencil => {
            let tip = vec2(4.5, 19.5);
            let back = vec2(18.6, 5.4);
            let along = (back - tip).normalized();
            let across = along.rot90() * 2.5;
            let shoulder = tip + along * 4.2;
            let ferrule = back - along * 3.2;
            outline(&[
                tip,
                shoulder + across,
                back + across,
                back - across,
                shoulder - across,
            ]);
            painter.line_segment([at(ferrule + across), at(ferrule - across)], stroke);
        }
        Icon::Grid => {
            for corner in [
                vec2(4.0, 4.0),
                vec2(13.0, 4.0),
                vec2(4.0, 13.0),
                vec2(13.0, 13.0),
            ] {
                painter.rect_stroke(
                    Rect::from_min_size(at(corner), Vec2::splat(7.0 * scale)),
                    1.6 * scale,
                    stroke,
                    StrokeKind::Middle,
                );
            }
        }
    }
}
