use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::epaint::Mesh;
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use glam::Vec3;

use crate::elements::Element;
use crate::rendering::colors::{
    adaptation_gauge_color, burst_gauge_color, orange, terminal_color, yellow,
};
use crate::rendering::shapes::hand_outline;
use crate::rendering::Theme;
use crate::tools::Tool;

use super::style::Chrome;

const GRID: f32 = 24.0;
const GLYPH_GRID: f32 = 32.0;
const STROKE: f32 = 1.8;
const SPHERE_STEPS: usize = 40;
const CREAM: Color32 = Color32::from_rgb(255, 244, 214);
const MORPHOLOGY_GRID: Vec2 = vec2(64.0, 32.0);
const SOMA_SHARE: f32 = 0.62;
const PYRAMIDAL_SOMA: (f32, f32) = (16.0, 19.0);
const PYRAMIDAL: [&[(f32, f32)]; 4] = [
    &[(16.0, 19.0), (16.0, 13.0), (15.5, 8.0), (12.0, 3.5)],
    &[(15.5, 8.0), (20.0, 3.5)],
    &[(16.0, 19.0), (11.0, 23.5), (7.0, 28.0)],
    &[(16.0, 19.0), (21.0, 24.0), (24.5, 28.5)],
];
const STAR_SOMA: (f32, f32) = (16.0, 16.0);
const STAR_ARMS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Pencil,
    Grid,
    Back,
    Menu,
    More,
    Check,
    Folder,
    Save,
    Share,
    Trash,
    Undo,
    Redo,
    Close,
    Play,
    Pause,
    Playback,
    Plus,
    Minus,
    Eye,
    Stimulate,
    Erase,
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
        Icon::Back => {
            painter.add(Shape::line(
                vec![
                    at(vec2(14.5, 5.5)),
                    at(vec2(8.0, 12.0)),
                    at(vec2(14.5, 18.5)),
                ],
                stroke,
            ));
        }
        Icon::Menu => {
            for y in [7.0, 12.0, 17.0] {
                painter.line_segment([at(vec2(4.5, y)), at(vec2(19.5, y))], stroke);
            }
        }
        Icon::More => {
            for x in [5.5, 12.0, 18.5] {
                painter.circle_filled(at(vec2(x, 12.0)), 1.7 * scale, color);
            }
        }
        Icon::Check => {
            painter.add(Shape::line(
                vec![
                    at(vec2(5.0, 12.5)),
                    at(vec2(10.0, 17.5)),
                    at(vec2(19.0, 7.0)),
                ],
                stroke,
            ));
        }
        Icon::Folder => outline(&[
            vec2(3.5, 6.5),
            vec2(9.2, 6.5),
            vec2(11.2, 8.5),
            vec2(20.5, 8.5),
            vec2(20.5, 18.5),
            vec2(3.5, 18.5),
        ]),
        Icon::Save => {
            painter.line_segment([at(vec2(12.0, 4.0)), at(vec2(12.0, 15.0))], stroke);
            painter.add(Shape::line(
                vec![
                    at(vec2(7.5, 10.5)),
                    at(vec2(12.0, 15.0)),
                    at(vec2(16.5, 10.5)),
                ],
                stroke,
            ));
            painter.line_segment([at(vec2(5.0, 19.5)), at(vec2(19.0, 19.5))], stroke);
        }
        Icon::Share => {
            painter.line_segment([at(vec2(12.0, 3.5)), at(vec2(12.0, 14.5))], stroke);
            painter.add(Shape::line(
                vec![at(vec2(8.0, 7.5)), at(vec2(12.0, 3.5)), at(vec2(16.0, 7.5))],
                stroke,
            ));
            painter.add(Shape::line(
                vec![
                    at(vec2(9.0, 9.5)),
                    at(vec2(6.0, 9.5)),
                    at(vec2(6.0, 20.5)),
                    at(vec2(18.0, 20.5)),
                    at(vec2(18.0, 9.5)),
                    at(vec2(15.0, 9.5)),
                ],
                stroke,
            ));
        }
        Icon::Trash => {
            painter.line_segment([at(vec2(4.5, 7.0)), at(vec2(19.5, 7.0))], stroke);
            painter.add(Shape::line(
                vec![
                    at(vec2(9.5, 7.0)),
                    at(vec2(9.5, 4.5)),
                    at(vec2(14.5, 4.5)),
                    at(vec2(14.5, 7.0)),
                ],
                stroke,
            ));
            painter.add(Shape::line(
                vec![
                    at(vec2(6.5, 7.0)),
                    at(vec2(7.4, 19.8)),
                    at(vec2(16.6, 19.8)),
                    at(vec2(17.5, 7.0)),
                ],
                stroke,
            ));
            for x in [10.0, 14.0] {
                painter.line_segment([at(vec2(x, 10.8)), at(vec2(x, 16.5))], stroke);
            }
        }
        Icon::Undo | Icon::Redo => {
            let side = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let point = |x: f32, y: f32| at(vec2(12.0 + side * x, y));
            let mut hook = vec![point(-3.0, 18.5)];
            hook.extend(arc(
                point(2.0, 14.0),
                4.5 * scale,
                FRAC_PI_2,
                FRAC_PI_2 - side * PI,
            ));
            hook.push(point(-7.5, 9.5));
            painter.add(Shape::line(hook, stroke));
            painter.add(Shape::line(
                vec![point(-3.5, 5.5), point(-7.5, 9.5), point(-3.5, 13.5)],
                stroke,
            ));
        }
        Icon::Close => {
            painter.line_segment([at(vec2(6.5, 6.5)), at(vec2(17.5, 17.5))], stroke);
            painter.line_segment([at(vec2(17.5, 6.5)), at(vec2(6.5, 17.5))], stroke);
        }
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![
                    at(vec2(8.0, 5.6)),
                    at(vec2(18.4, 12.0)),
                    at(vec2(8.0, 18.4)),
                ],
                color,
                stroke,
            ));
        }
        Icon::Pause => {
            for left in [7.0, 13.6] {
                painter.rect_filled(
                    Rect::from_min_size(at(vec2(left, 5.5)), vec2(3.4, 13.0) * scale),
                    1.0 * scale,
                    color,
                );
            }
        }
        Icon::Playback => {
            painter.add(Shape::convex_polygon(
                vec![
                    at(vec2(3.5, 6.5)),
                    at(vec2(11.5, 12.0)),
                    at(vec2(3.5, 17.5)),
                ],
                color,
                stroke,
            ));
            for left in [13.8, 17.8] {
                painter.rect_filled(
                    Rect::from_min_size(at(vec2(left, 6.5)), vec2(2.6, 11.0) * scale),
                    0.8 * scale,
                    color,
                );
            }
        }
        Icon::Plus => {
            painter.line_segment([at(vec2(12.0, 5.0)), at(vec2(12.0, 19.0))], stroke);
            painter.line_segment([at(vec2(5.0, 12.0)), at(vec2(19.0, 12.0))], stroke);
        }
        Icon::Minus => {
            painter.line_segment([at(vec2(5.0, 12.0)), at(vec2(19.0, 12.0))], stroke);
        }
        Icon::Eye => {
            let steps = 16;
            let lid = |sign: f32| {
                (0..=steps).map(move |step| {
                    let t = step as f32 / steps as f32;
                    vec2(3.0 + 18.0 * t, 12.0 + sign * 6.8 * (PI * t).sin())
                })
            };
            let points: Vec<Vec2> = lid(-1.0).chain(lid(1.0).rev().skip(1)).collect();
            outline(&points);
            painter.circle_stroke(at(vec2(12.0, 12.0)), 3.0 * scale, stroke);
        }
        Icon::Stimulate => outline(&[
            vec2(13.5, 3.2),
            vec2(5.8, 13.4),
            vec2(11.5, 13.4),
            vec2(10.5, 20.8),
            vec2(18.2, 10.6),
            vec2(12.5, 10.6),
        ]),
        Icon::Erase => {
            outline(&[
                vec2(4.6, 15.2),
                vec2(13.8, 6.0),
                vec2(19.2, 11.4),
                vec2(11.4, 19.2),
                vec2(8.6, 19.2),
            ]);
            painter.line_segment([at(vec2(9.2, 10.6)), at(vec2(14.6, 16.0))], stroke);
            painter.line_segment([at(vec2(11.4, 19.6)), at(vec2(19.5, 19.6))], stroke);
        }
    }
}

fn linear(color: Vec3) -> Color32 {
    egui::Rgba::from_rgb(color.x, color.y, color.z).into()
}

fn shade(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, _] = color.to_array().map(|channel| channel as f32 * factor);
    Color32::from_rgb(r as u8, g as u8, b as u8)
}

fn sphere(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let rim = shade(color, 0.78);
    painter.circle_filled(center, radius, rim);
    let mut mesh = Mesh::default();
    mesh.colored_vertex(
        center + vec2(-0.32, -0.36) * radius,
        color.lerp_to_gamma(Color32::WHITE, 0.4),
    );
    for step in 0..=SPHERE_STEPS {
        let angle = TAU * step as f32 / SPHERE_STEPS as f32;
        mesh.colored_vertex(center + Vec2::angled(angle) * (radius - 0.6), rim);
    }
    for step in 0..SPHERE_STEPS as u32 {
        mesh.add_triangle(0, step + 1, step + 2);
    }
    painter.add(Shape::mesh(mesh));
}

fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    (0..=SPHERE_STEPS)
        .map(|step| {
            let angle = from + (to - from) * step as f32 / SPHERE_STEPS as f32;
            center + Vec2::angled(angle) * radius
        })
        .collect()
}

pub fn paint_morphology(painter: &Painter, compartments: bool, rect: Rect, theme: Theme) {
    let excitatory = linear(theme.palette().excitatory);
    let scale = (rect.width() / MORPHOLOGY_GRID.x).min(rect.height() / MORPHOLOGY_GRID.y);
    let origin = rect.center() - MORPHOLOGY_GRID / 2.0 * scale;
    let at = |(x, y): (f32, f32)| origin + vec2(x, y) * scale;
    if !compartments {
        let [start, bend, end] = [(19.5, 14.0), (32.0, 3.5), (43.5, 12.5)].map(at);
        let curve: Vec<Pos2> = (0..=SPHERE_STEPS)
            .map(|step| {
                let t = step as f32 / SPHERE_STEPS as f32;
                let u = 1.0 - t;
                pos2(
                    u * u * start.x + 2.0 * u * t * bend.x + t * t * end.x,
                    u * u * start.y + 2.0 * u * t * bend.y + t * t * end.y,
                )
            })
            .collect();
        painter.add(Shape::line(curve, Stroke::new(3.0 * scale, excitatory)));
        painter.circle_filled(end, 3.0 * scale, linear(terminal_color()));
        for soma in [(11.0, 17.0), (53.0, 17.0)] {
            sphere(painter, at(soma), 7.5 * scale, excitatory);
        }
        return;
    }
    let beads = |points: &[(f32, f32)]| {
        let points: Vec<Pos2> = points.iter().copied().map(at).collect();
        beads(painter, &points, scale, excitatory);
    };
    for (x, y) in [(12.0, 18.0), (52.0, 18.0)] {
        beads(&[
            (x, y),
            (x, y - 6.0),
            (x - 1.0, y - 11.0),
            (x - 4.0, y - 15.0),
        ]);
        beads(&[(x - 1.0, y - 11.0), (x + 3.0, y - 15.0)]);
        beads(&[(x, y), (x - 5.0, y + 5.0), (x - 8.0, y + 10.0)]);
        beads(&[(x, y), (x + 1.0, y + 6.0), (x + 3.0, y + 11.0)]);
    }
    beads(&[
        (12.0, 18.0),
        (18.0, 20.0),
        (24.0, 21.5),
        (30.0, 22.5),
        (36.0, 23.0),
        (42.0, 23.0),
        (47.0, 23.0),
    ]);
    for soma in [(12.0, 18.0), (52.0, 18.0)] {
        sphere(painter, at(soma), 4.5 * scale, excitatory);
    }
}

fn beads(painter: &Painter, points: &[Pos2], scale: f32, color: Color32) {
    painter.add(Shape::line(
        points.to_vec(),
        Stroke::new(1.2 * scale, color),
    ));
    for &point in &points[1..] {
        painter.circle_filled(point, 1.5 * scale, color);
    }
}

pub fn paint_tool(
    painter: &Painter,
    tool: Tool,
    rect: Rect,
    theme: Theme,
    color: Color32,
    compartments: bool,
) {
    match tool {
        Tool::Place(element) if compartments && element.is_neuron() => {
            paint_compartmental(painter, element, rect, theme, color)
        }
        _ => paint_glyph(painter, tool, rect, theme, color),
    }
}

fn paint_compartmental(
    painter: &Painter,
    element: Element,
    rect: Rect,
    theme: Theme,
    color: Color32,
) {
    let palette = theme.palette();
    let scale = rect.width().min(rect.height()) / GLYPH_GRID;
    let origin = rect.center() - Vec2::splat(GLYPH_GRID / 2.0) * scale;
    let at = |(x, y): (f32, f32)| origin + vec2(x, y) * scale;
    let (soma, branches, dendrite) = if element == Element::InhibitoryNeuron {
        let (x, y) = STAR_SOMA;
        let arms: Vec<Vec<(f32, f32)>> = (0..STAR_ARMS)
            .map(|arm| {
                let angle = -FRAC_PI_2 + TAU * arm as f32 / STAR_ARMS as f32;
                let reach = |distance: f32, turn: f32| {
                    let direction = Vec2::angled(angle + turn) * distance;
                    (x + direction.x, y + direction.y)
                };
                vec![STAR_SOMA, reach(7.5, 0.0), reach(13.5, 0.25)]
            })
            .collect();
        (STAR_SOMA, arms, palette.inhibitory)
    } else {
        let branches = PYRAMIDAL.iter().map(|branch| branch.to_vec()).collect();
        (PYRAMIDAL_SOMA, branches, palette.excitatory)
    };
    for branch in branches {
        let points: Vec<Pos2> = branch.into_iter().map(at).collect();
        beads(painter, &points, scale, linear(dendrite));
    }
    let glyph = Rect::from_center_size(at(soma), Vec2::splat(GLYPH_GRID * SOMA_SHARE * scale));
    paint_glyph(painter, Tool::Place(element), glyph, theme, color);
}

fn paint_glyph(painter: &Painter, tool: Tool, rect: Rect, theme: Theme, color: Color32) {
    let icon = match tool {
        Tool::View => Some(Icon::Eye),
        Tool::Edit => Some(Icon::Pencil),
        Tool::Stimulate => Some(Icon::Stimulate),
        Tool::Erase => Some(Icon::Erase),
        _ => None,
    };
    if let Some(icon) = icon {
        paint_icon(painter, icon, rect, color);
        return;
    }
    let chrome = Chrome::of(theme);
    let palette = theme.palette();
    let excitatory = linear(palette.excitatory);
    let scale = rect.width().min(rect.height()) / GLYPH_GRID;
    let origin = rect.center() - Vec2::splat(GLYPH_GRID / 2.0) * scale;
    let at = |x: f32, y: f32| origin + vec2(x, y) * scale;
    let stroke = |width: f32, color: Color32| Stroke::new(width * scale, color);
    match tool {
        Tool::Place(Element::ExcitatoryNeuron) => {
            sphere(painter, at(16.0, 16.0), 10.0 * scale, excitatory)
        }
        Tool::Place(Element::InhibitoryNeuron) => sphere(
            painter,
            at(16.0, 16.0),
            10.0 * scale,
            linear(palette.inhibitory),
        ),
        Tool::Place(Element::AdaptiveNeuron) => {
            sphere(painter, at(16.0, 16.0), 8.5 * scale, excitatory);
            painter.add(Shape::line(
                arc(at(16.0, 16.0), 12.5 * scale, -0.25 * TAU, 0.42 * TAU),
                stroke(2.4, linear(adaptation_gauge_color())),
            ));
        }
        Tool::Place(Element::BurstNeuron) => {
            sphere(painter, at(13.5, 16.0), 9.5 * scale, excitatory);
            painter.rect_filled(
                Rect::from_min_max(at(25.0, 8.0), at(28.5, 24.0)),
                1.75 * scale,
                linear(burst_gauge_color()),
            );
        }
        Tool::Place(Element::CurrentSource) => {
            painter.circle_filled(at(16.0, 16.0), 11.5 * scale, linear(yellow()));
            let arrow = stroke(2.4, CREAM);
            painter.line_segment([at(16.0, 22.5), at(16.0, 9.8)], arrow);
            painter.add(Shape::line(
                vec![at(11.8, 14.0), at(16.0, 9.8), at(20.2, 14.0)],
                arrow,
            ));
        }
        Tool::Place(Element::TouchSensor) => {
            painter.circle_filled(at(16.0, 16.0), 11.5 * scale, linear(orange()));
            let hand = hand_outline();
            let reach = hand
                .iter()
                .map(|point| point.abs().max_element())
                .fold(0.0, f32::max);
            let size = 7.0 * scale / reach;
            let points: Vec<Pos2> = hand
                .iter()
                .map(|point| at(16.0, 16.0) + vec2(point.x, -point.y) * size)
                .collect();
            let line = stroke(1.4, CREAM);
            for (index, &point) in points.iter().enumerate() {
                let next = points[(index + 1) % points.len()];
                painter.line_segment([point, next], line);
                painter.circle_filled(point, line.width / 2.0, CREAM);
            }
        }
        Tool::Place(Element::RegularSpikeGenerator) | Tool::Place(Element::PoissonGenerator) => {
            let generator = linear(orange());
            sphere(painter, at(16.0, 16.0), 7.0 * scale, generator);
            let ring = arc(at(16.0, 16.0), 12.0 * scale, 0.0, TAU);
            if tool == Tool::Place(Element::PoissonGenerator) {
                painter.extend(Shape::dashed_line(
                    &ring,
                    stroke(2.6, generator),
                    3.4 * scale,
                    2.6 * scale,
                ));
            } else {
                painter.add(Shape::closed_line(ring, stroke(2.6, generator)));
            }
        }
        Tool::Voltmeter => {
            painter.rect(
                Rect::from_min_max(at(4.0, 7.0), at(28.0, 25.0)),
                3.0 * scale,
                chrome.raised,
                stroke(1.5, chrome.subtext),
                StrokeKind::Inside,
            );
            painter.add(Shape::line(
                [
                    (7.0, 19.0),
                    (11.0, 19.0),
                    (13.0, 11.0),
                    (15.0, 22.5),
                    (17.0, 16.0),
                    (25.0, 16.0),
                ]
                .map(|(x, y)| at(x, y))
                .to_vec(),
                stroke(1.8, chrome.accent),
            ));
        }
        Tool::StaticConnection => {
            let curve: Vec<Pos2> = (0..=SPHERE_STEPS)
                .map(|step| {
                    let t = step as f32 / SPHERE_STEPS as f32;
                    let [start, bend, end] = [at(6.5, 24.5), at(9.5, 11.0), at(22.0, 9.5)];
                    let u = 1.0 - t;
                    pos2(
                        u * u * start.x + 2.0 * u * t * bend.x + t * t * end.x,
                        u * u * start.y + 2.0 * u * t * bend.y + t * t * end.y,
                    )
                })
                .collect();
            painter.add(Shape::line(curve, stroke(3.4, excitatory)));
            painter.circle_filled(at(24.0, 9.3), 3.6 * scale, linear(terminal_color()));
        }
        Tool::Axon => {
            let beads = [
                (5.5, 22.5),
                (10.5, 15.5),
                (16.0, 13.5),
                (21.5, 15.5),
                (26.5, 19.0),
            ]
            .map(|(x, y)| at(x, y));
            painter.add(Shape::line(beads.to_vec(), stroke(1.6, excitatory)));
            for bead in beads {
                painter.circle_filled(bead, 2.9 * scale, excitatory);
            }
        }
        Tool::Place(Element::Note) => {
            painter.rect(
                Rect::from_min_max(at(6.0, 6.0), at(26.0, 26.0)),
                3.0 * scale,
                chrome.panel,
                stroke(1.5, chrome.border),
                StrokeKind::Inside,
            );
            for (y, end) in [(12.0, 22.0), (16.0, 22.0), (20.0, 17.0)] {
                painter.line_segment([at(10.0, y), at(end, y)], stroke(1.6, chrome.subtext));
            }
        }
        Tool::View | Tool::Edit | Tool::Stimulate | Tool::Erase => {}
    }
}
