use glam::{Mat4, Vec3, Vec4};
use hecs::Entity;
use std::collections::{HashMap, HashSet};
use visula::{
    text::{tessellate_text_wrapped, TessellatedGeometry},
    Expression, PolygonDelegate, PolygonVertex, Polygons,
};

use crate::components::{Annotation, NoteSize, Position};

/// DejaVu Sans 2.37, cut down to Latin, Greek, punctuation, super- and subscripts,
/// arrows and math operators with
/// `pyftsubset DejaVuSans.ttf --unicodes=U+0020-007E,U+00A0-017F,U+0370-03FF,U+2000-209F,U+2190-22FF
/// --no-hinting --layout-features=''`.
/// Other characters are left out of the note.
pub const NOTE_FONT_DATA: &[u8] = include_bytes!("../../assets/DejaVuSans.ttf");

pub const NOTE_TEXT_HEIGHT: f32 = 0.8;
pub const NOTE_MARGIN: f32 = 0.4;
pub const NOTE_BACKGROUND_Y: f32 = 0.01;
pub const NOTE_TEXT_Y: f32 = 0.02;

/// The geometry of one note around its own center, rebuilt only when its text or size changes.
struct NoteGeometry {
    text: String,
    width: f32,
    height: f32,
    half_extent: [f32; 2],
    background: TessellatedGeometry,
    glyphs: TessellatedGeometry,
}

/// Draws every note with two pipelines, one for the backgrounds and one for the text.
/// Moving a note only uploads new vertices.
pub struct NoteRenderer {
    backgrounds: Polygons,
    text: Polygons,
    notes: HashMap<Entity, NoteGeometry>,
    uploaded: Vec<(Entity, Vec3)>,
}

impl NoteRenderer {
    pub fn new(application: &visula::Application) -> Self {
        let polygons = |color: Vec4, y: f32| {
            Polygons::new(
                &application.rendering_descriptor(),
                &PolygonDelegate {
                    color: color.into(),
                    transform: Expression::from(note_plane(y)),
                },
                &[],
                &[],
            )
            .unwrap()
        };
        Self {
            backgrounds: polygons(Vec4::new(0.95, 0.95, 0.85, 1.0), NOTE_BACKGROUND_Y),
            text: polygons(Vec4::new(0.1, 0.1, 0.15, 1.0), NOTE_TEXT_Y),
            notes: HashMap::new(),
            uploaded: Vec::new(),
        }
    }

    pub fn update(&mut self, application: &visula::Application, world: &hecs::World) {
        let mut changed = false;
        let mut placed = Vec::new();
        for (entity, (annotation, position, size)) in
            world.query::<(&Annotation, &Position, &NoteSize)>().iter()
        {
            let current = self.notes.get(&entity).is_some_and(|note| {
                note.text == annotation.text
                    && note.width == size.width
                    && note.height == size.height
            });
            if !current {
                self.notes
                    .insert(entity, build_note_geometry(&annotation.text, size));
                changed = true;
            }
            placed.push((entity, position.position));
        }
        // Later notes are drawn on top of earlier ones
        placed.sort_by_key(|(entity, _)| entity.id());
        let present: HashSet<Entity> = placed.iter().map(|(entity, _)| *entity).collect();
        self.notes.retain(|entity, _| present.contains(entity));

        if !changed && placed == self.uploaded {
            return;
        }
        let (backgrounds, text) = layout(&self.notes, &placed);
        self.backgrounds.update_geometry(
            &application.device,
            &application.queue,
            &backgrounds.vertices,
            &backgrounds.indices,
        );
        self.text.update_geometry(
            &application.device,
            &application.queue,
            &text.vertices,
            &text.indices,
        );
        self.uploaded = placed;
    }

    pub fn render(&self, data: &mut visula::RenderData) {
        use visula::Renderable;
        self.backgrounds.render(data);
        self.text.render(data);
    }
}

/// Places every note's geometry at its position, in drawing order,
/// returning the backgrounds and the text.
fn layout(
    notes: &HashMap<Entity, NoteGeometry>,
    placed: &[(Entity, Vec3)],
) -> (TessellatedGeometry, TessellatedGeometry) {
    let rectangles: Vec<Rectangle> = placed
        .iter()
        .map(|(entity, position)| Rectangle::around(*position, notes[entity].half_extent))
        .collect();
    let mut backgrounds = TessellatedGeometry::default();
    let mut text = TessellatedGeometry::default();
    for (index, (entity, position)) in placed.iter().enumerate() {
        let note = &notes[entity];
        // The notes are flat, so a note on top hides whatever lies under it
        let covers: Vec<Rectangle> = rectangles[index + 1..]
            .iter()
            .filter(|cover| cover.overlaps(&rectangles[index]))
            .copied()
            .collect();
        backgrounds.merge(&uncovered(translated(&note.background, *position), &covers));
        text.merge(&uncovered(translated(&note.glyphs, *position), &covers));
    }
    (backgrounds, text)
}

fn build_note_geometry(text: &str, size: &NoteSize) -> NoteGeometry {
    let width = size.width.max(0.5);
    let height = size.height.max(0.5);
    let inner_width = (width - 2.0 * NOTE_MARGIN).max(0.5);
    let glyphs = if text.is_empty() {
        TessellatedGeometry::default()
    } else {
        match tessellate_text_wrapped(NOTE_FONT_DATA, text, NOTE_TEXT_HEIGHT, inner_width) {
            // Lines start at the top left and go down, so shift the block to center it
            Ok(wrapped) => shifted(
                &wrapped.geometry,
                -wrapped.width * 0.5,
                wrapped.height * 0.5,
            ),
            Err(e) => {
                log::error!("Failed to tessellate note text: {e:?}");
                TessellatedGeometry::default()
            }
        }
    };
    NoteGeometry {
        text: text.to_string(),
        width: size.width,
        height: size.height,
        half_extent: [width * 0.5, height * 0.5],
        background: rectangle(width, height),
        glyphs,
    }
}

/// Maps the polygons' local (x, y, 0) onto the ground at height `y`.
/// Local x goes to world +Z (screen right) and local y to world +X (screen up),
/// matching how `convert_position` rotates the legacy QML scene.
fn note_plane(y: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, y, 0.0, 1.0),
    )
}

/// Moves geometry around a note's center to the note's position on the ground.
fn translated(geometry: &TessellatedGeometry, position: Vec3) -> TessellatedGeometry {
    shifted(geometry, position.z, position.x)
}

fn shifted(geometry: &TessellatedGeometry, dx: f32, dy: f32) -> TessellatedGeometry {
    TessellatedGeometry {
        vertices: geometry
            .vertices
            .iter()
            .map(|v| PolygonVertex {
                position: [v.position[0] + dx, v.position[1] + dy],
            })
            .collect(),
        indices: geometry.indices.clone(),
    }
}

/// An axis-aligned rectangle in the plane of the notes.
#[derive(Clone, Copy)]
struct Rectangle {
    min: [f32; 2],
    max: [f32; 2],
}

impl Rectangle {
    fn around(position: Vec3, half_extent: [f32; 2]) -> Self {
        let [x, y] = [position.z, position.x];
        Self {
            min: [x - half_extent[0], y - half_extent[1]],
            max: [x + half_extent[0], y + half_extent[1]],
        }
    }

    fn bounding(points: &[[f32; 2]]) -> Self {
        points.iter().fold(
            Self {
                min: [f32::MAX; 2],
                max: [f32::MIN; 2],
            },
            |bounds, point| Self {
                min: [bounds.min[0].min(point[0]), bounds.min[1].min(point[1])],
                max: [bounds.max[0].max(point[0]), bounds.max[1].max(point[1])],
            },
        )
    }

    fn overlaps(&self, other: &Rectangle) -> bool {
        (0..2).all(|axis| self.min[axis] < other.max[axis] && other.min[axis] < self.max[axis])
    }

    fn contains(&self, other: &Rectangle) -> bool {
        (0..2).all(|axis| self.min[axis] <= other.min[axis] && other.max[axis] <= self.max[axis])
    }
}

/// Removes the parts of the triangles that lie under any of the covers.
fn uncovered(geometry: TessellatedGeometry, covers: &[Rectangle]) -> TessellatedGeometry {
    if covers.is_empty() {
        return geometry;
    }
    let mut result = TessellatedGeometry::default();
    for triangle in geometry.indices.as_chunks::<3>().0 {
        let mut pieces = vec![triangle
            .iter()
            .map(|&index| geometry.vertices[index as usize].position)
            .collect::<Vec<_>>()];
        for cover in covers {
            pieces = pieces
                .iter()
                .flat_map(|piece| outside(piece, cover))
                .collect();
        }
        for piece in pieces {
            let first = result.vertices.len() as u32;
            result
                .vertices
                .extend(piece.iter().map(|&position| PolygonVertex { position }));
            for corner in 1..piece.len() as u32 - 1 {
                result
                    .indices
                    .extend([first, first + corner, first + corner + 1]);
            }
        }
    }
    result
}

/// Splits a convex polygon into the convex pieces that lie outside the cover.
fn outside(polygon: &[[f32; 2]], cover: &Rectangle) -> Vec<Vec<[f32; 2]>> {
    let bounds = Rectangle::bounding(polygon);
    if !bounds.overlaps(cover) {
        return vec![polygon.to_vec()];
    }
    if cover.contains(&bounds) {
        return vec![];
    }
    // The outside of the cover as four regions that don't overlap:
    // left and right of it, and below and above it in between
    let regions: [&[(usize, f32, bool)]; 4] = [
        &[(0, cover.min[0], true)],
        &[(0, cover.max[0], false)],
        &[
            (0, cover.min[0], false),
            (0, cover.max[0], true),
            (1, cover.min[1], true),
        ],
        &[
            (0, cover.min[0], false),
            (0, cover.max[0], true),
            (1, cover.max[1], false),
        ],
    ];
    regions
        .iter()
        .map(|half_planes| {
            half_planes
                .iter()
                .fold(polygon.to_vec(), |piece, &(axis, bound, below)| {
                    clip(&piece, axis, bound, below)
                })
        })
        .filter(|piece| piece.len() >= 3)
        .collect()
}

/// Keeps the part of a convex polygon at or below `bound` along `axis`, or at or above it.
fn clip(polygon: &[[f32; 2]], axis: usize, bound: f32, below: bool) -> Vec<[f32; 2]> {
    let inside = |point: &[f32; 2]| {
        if below {
            point[axis] <= bound
        } else {
            point[axis] >= bound
        }
    };
    let mut result = Vec::new();
    for (index, current) in polygon.iter().enumerate() {
        let previous = &polygon[(index + polygon.len() - 1) % polygon.len()];
        if inside(current) != inside(previous) {
            let t = (bound - previous[axis]) / (current[axis] - previous[axis]);
            result.push([
                previous[0] + t * (current[0] - previous[0]),
                previous[1] + t * (current[1] - previous[1]),
            ]);
        }
        if inside(current) {
            result.push(*current);
        }
    }
    result
}

fn rectangle(width: f32, height: f32) -> TessellatedGeometry {
    let (u, v) = (width * 0.5, height * 0.5);
    TessellatedGeometry {
        vertices: [[-u, -v], [u, -v], [u, v], [-u, v]]
            .into_iter()
            .map(|position| PolygonVertex { position })
            .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tessellates_short_text() {
        let wrapped =
            tessellate_text_wrapped(NOTE_FONT_DATA, "Hello", NOTE_TEXT_HEIGHT, 100.0).unwrap();
        assert!(!wrapped.geometry.vertices.is_empty());
        assert!(!wrapped.geometry.indices.is_empty());
        assert!(wrapped.width > 0.0);
        assert!(wrapped.height > 0.0);
    }

    #[test]
    fn wraps_long_text_into_multiple_lines() {
        let text =
            "This is a long note that should wrap onto several lines when given a narrow width";
        let narrow = tessellate_text_wrapped(NOTE_FONT_DATA, text, NOTE_TEXT_HEIGHT, 4.0).unwrap();
        let wide = tessellate_text_wrapped(NOTE_FONT_DATA, text, NOTE_TEXT_HEIGHT, 100.0).unwrap();
        assert!(
            narrow.height > wide.height,
            "narrow width should produce more lines (taller): narrow={} wide={}",
            narrow.height,
            wide.height
        );
    }

    fn area(geometry: &TessellatedGeometry) -> f32 {
        geometry
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle.map(|i| geometry.vertices[i as usize].position);
                ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.0
            })
            .sum()
    }

    #[test]
    fn hides_what_a_note_on_top_covers() {
        let corner = Rectangle {
            min: [0.0, 0.0],
            max: [3.0, 3.0],
        };
        let visible = uncovered(rectangle(2.0, 2.0), &[corner]);
        assert!((area(&visible) - 3.0).abs() < 1e-5);
        assert!(visible
            .vertices
            .iter()
            .all(|v| v.position[0] <= 1e-6 || v.position[1] <= 1e-6));

        let middle = Rectangle {
            min: [-0.5, -0.5],
            max: [0.5, 0.5],
        };
        let visible = uncovered(rectangle(2.0, 2.0), &[middle]);
        assert!((area(&visible) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn centers_text_in_the_note() {
        let note = build_note_geometry("Hello there", &NoteSize::default());
        let xs = note.glyphs.vertices.iter().map(|v| v.position[0]);
        let (min, max) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!((min + max).abs() < 0.1, "min={min} max={max}");
    }
}
