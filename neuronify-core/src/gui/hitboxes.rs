use egui::{
    pos2, Align2, Area, Color32, Context, Frame, Id, LayerId, Mesh, Pos2, Shape, Slider, Stroke,
    Vec2,
};
use hecs::Entity;

const CELL: f32 = 4.0;
const BISECTIONS: usize = 4;
const FILL_OPACITY: f32 = 0.3;
const EDGE_WIDTH: f32 = 1.5;
const MAX_REACH: f64 = 66.0;
const SLIDER_OFFSET: f32 = 76.0;
const COLORS: [Color32; 6] = [
    Color32::from_rgb(30, 102, 245),
    Color32::from_rgb(64, 160, 43),
    Color32::from_rgb(254, 100, 11),
    Color32::from_rgb(136, 57, 239),
    Color32::from_rgb(23, 146, 153),
    Color32::from_rgb(210, 15, 57),
];

fn color(entity: Entity) -> Color32 {
    COLORS[entity.id() as usize % COLORS.len()]
}

fn sample(column: usize, row: usize) -> Pos2 {
    pos2((column as f32 + 0.5) * CELL, (row as f32 + 0.5) * CELL)
}

fn boundary(
    owner: &impl Fn(Pos2) -> Option<Entity>,
    mut inside: Pos2,
    mut outside: Pos2,
    side: Option<Entity>,
) -> Pos2 {
    for _ in 0..BISECTIONS {
        let middle = inside.lerp(outside, 0.5);
        if owner(middle) == side {
            inside = middle;
        } else {
            outside = middle;
        }
    }
    inside.lerp(outside, 0.5)
}

fn fill(mesh: &mut Mesh, outline: &[Pos2], entity: Option<Entity>) {
    let Some(entity) = entity else {
        return;
    };
    let first = mesh.vertices.len() as u32;
    let shade = color(entity).gamma_multiply(FILL_OPACITY);
    for &point in outline {
        mesh.colored_vertex(point, shade);
    }
    for corner in 1..outline.len() as u32 - 1 {
        mesh.add_triangle(first, first + corner, first + corner + 1);
    }
}

fn edge(edges: &mut Vec<Shape>, from: Pos2, to: Pos2, entity: Option<Entity>) {
    if let Some(entity) = entity {
        edges.push(Shape::line_segment(
            [from, to],
            Stroke::new(EDGE_WIDTH, color(entity)),
        ));
    }
}

fn square(
    points: [Pos2; 4],
    owners: [Option<Entity>; 4],
    crossings: [Option<Pos2>; 4],
    mesh: &mut Mesh,
    edges: &mut Vec<Shape>,
) {
    let on_side = |side: usize| {
        let side = side % 4;
        crossings[side].unwrap_or_else(|| points[side].lerp(points[(side + 1) % 4], 0.5))
    };
    let piece = |from: usize, to: usize| {
        let mut outline = vec![on_side(from)];
        outline.extend((from + 1..=to).map(|corner| points[corner % 4]));
        outline.push(on_side(to));
        outline
    };
    let crossed: Vec<usize> = (0..4).filter(|&side| crossings[side].is_some()).collect();
    match crossed[..] {
        [] => fill(mesh, &points, owners[0]),
        [first, second] => {
            fill(mesh, &piece(first, second), owners[second]);
            fill(mesh, &piece(second, first + 4), owners[first]);
            let owner = owners[first].or(owners[second]);
            edge(edges, on_side(first), on_side(second), owner);
        }
        _ => {
            let junction = crossed
                .iter()
                .fold(Vec2::ZERO, |sum, &side| sum + on_side(side).to_vec2())
                / crossed.len() as f32;
            let junction = junction.to_pos2();
            for corner in 0..4 {
                let outline = [
                    points[corner],
                    on_side(corner),
                    junction,
                    on_side(corner + 3),
                ];
                fill(mesh, &outline, owners[corner]);
            }
            for &side in &crossed {
                let owner = owners[side].or(owners[(side + 1) % 4]);
                edge(edges, on_side(side), junction, owner);
            }
        }
    }
}

pub fn paint_hitboxes(context: &Context, owner: impl Fn(Pos2) -> Option<Entity>) {
    let screen = context.content_rect();
    let columns = (screen.width() / CELL).ceil() as usize;
    let rows = (screen.height() / CELL).ceil() as usize;
    let owners: Vec<Option<Entity>> = (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (column, row)))
        .map(|(column, row)| owner(sample(column, row)))
        .collect();
    let at = |(column, row): (usize, usize)| owners[row * columns + column];
    let crossing = |from: (usize, usize), to: (usize, usize)| {
        let side = at(from);
        (side != at(to)).then(|| {
            let [from, to] = [from, to].map(|(column, row)| sample(column, row));
            boundary(&owner, from, to, side)
        })
    };
    let across: Vec<Option<Pos2>> = (0..rows)
        .flat_map(|row| (1..columns).map(move |column| (column, row)))
        .map(|(column, row)| crossing((column - 1, row), (column, row)))
        .collect();
    let down: Vec<Option<Pos2>> = (1..rows)
        .flat_map(|row| (0..columns).map(move |column| (column, row)))
        .map(|(column, row)| crossing((column, row - 1), (column, row)))
        .collect();
    let mut mesh = Mesh::default();
    let mut edges = Vec::new();
    for row in 0..rows.saturating_sub(1) {
        for column in 0..columns.saturating_sub(1) {
            let corners = [(0, 0), (1, 0), (1, 1), (0, 1)].map(|(x, y)| (column + x, row + y));
            let crossings = [
                across[row * (columns - 1) + column],
                down[row * columns + column + 1],
                across[(row + 1) * (columns - 1) + column],
                down[row * columns + column],
            ];
            square(
                corners.map(|(x, y)| sample(x, y)),
                corners.map(at),
                crossings,
                &mut mesh,
                &mut edges,
            );
        }
    }
    let painter = context.layer_painter(LayerId::background());
    painter.add(mesh);
    painter.extend(edges);
}

pub fn reach_slider(context: &Context, reach: &mut f64) {
    Area::new(Id::new("hitbox reach"))
        .anchor(Align2::CENTER_TOP, [0.0, SLIDER_OFFSET])
        .show(context, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.add(
                    Slider::new(reach, 0.0..=MAX_REACH)
                        .suffix(" pt")
                        .text("Reach"),
                );
            });
        });
}
