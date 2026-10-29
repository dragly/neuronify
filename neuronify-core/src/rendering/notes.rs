use glam::{Mat4, Vec3, Vec4};
use hecs::Entity;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use visula::{
    text::{GlyphAtlas, TessellatedGeometry},
    Expression, GlyphDelegate, GlyphVertex, Glyphs, PolygonDelegate, PolygonVertex, Polygons,
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

const PREPARED_CHARACTERS: RangeInclusive<char> = ' '..='~';
const PREPARED_GLYPHS_PER_FRAME: usize = 2;

/// The geometry of one note around its own center, rebuilt only when its text or size changes.
struct NoteGeometry {
    text: String,
    width: f32,
    height: f32,
    half_extent: [f32; 2],
    background: TessellatedGeometry,
    glyphs: TessellatedGeometry<GlyphVertex>,
}

/// Draws every note with two pipelines, one for the backgrounds and one for the text.
/// Moving a note only uploads new vertices.
pub struct NoteRenderer {
    backgrounds: Polygons,
    text: Glyphs,
    atlas: GlyphAtlas,
    unprepared: RangeInclusive<char>,
    notes: HashMap<Entity, NoteGeometry>,
    uploaded: Vec<(Entity, Vec3)>,
}

impl NoteRenderer {
    pub fn new(application: &visula::Application) -> Self {
        Self {
            backgrounds: Polygons::new(
                &application.rendering_descriptor(),
                &PolygonDelegate {
                    color: Vec4::new(0.95, 0.95, 0.85, 1.0).into(),
                    transform: Expression::from(note_plane(NOTE_BACKGROUND_Y)),
                },
                &[],
                &[],
            )
            .unwrap(),
            text: Glyphs::new(
                &application.rendering_descriptor(),
                &GlyphDelegate {
                    color: Vec4::new(0.1, 0.1, 0.15, 1.0).into(),
                    transform: Expression::from(note_plane(NOTE_TEXT_Y)),
                },
            )
            .unwrap(),
            atlas: GlyphAtlas::new(NOTE_FONT_DATA).unwrap(),
            unprepared: PREPARED_CHARACTERS,
            notes: HashMap::new(),
            uploaded: Vec::new(),
        }
    }

    pub fn update(&mut self, application: &visula::Application, world: &hecs::World) {
        prepare_glyphs(&mut self.atlas, &mut self.unprepared);
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
                self.notes.insert(
                    entity,
                    build_note_geometry(&mut self.atlas, &annotation.text, size),
                );
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
        self.text
            .update_atlas(&application.device, &application.queue, &self.atlas);
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
) -> (TessellatedGeometry, TessellatedGeometry<GlyphVertex>) {
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

fn prepare_glyphs(atlas: &mut GlyphAtlas, unprepared: &mut RangeInclusive<char>) {
    for character in unprepared.take(PREPARED_GLYPHS_PER_FRAME) {
        if let Err(e) = atlas.prepare(character) {
            log::error!("Failed to prepare the glyph for {character:?}: {e:?}");
        }
    }
}

fn build_note_geometry(atlas: &mut GlyphAtlas, text: &str, size: &NoteSize) -> NoteGeometry {
    let width = size.width.max(0.5);
    let height = size.height.max(0.5);
    let inner_width = (width - 2.0 * NOTE_MARGIN).max(0.5);
    let glyphs = if text.is_empty() {
        TessellatedGeometry::default()
    } else {
        match atlas.lay_out_wrapped(text, NOTE_TEXT_HEIGHT, inner_width) {
            // Lines start at the top left and go down, so shift the block to center it
            Ok(wrapped) => shifted(
                &wrapped.geometry,
                -wrapped.width * 0.5,
                wrapped.height * 0.5,
            ),
            Err(e) => {
                log::error!("Failed to lay out note text: {e:?}");
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
fn translated<V: PlaneVertex>(
    geometry: &TessellatedGeometry<V>,
    position: Vec3,
) -> TessellatedGeometry<V> {
    shifted(geometry, position.z, position.x)
}

fn shifted<V: PlaneVertex>(
    geometry: &TessellatedGeometry<V>,
    dx: f32,
    dy: f32,
) -> TessellatedGeometry<V> {
    TessellatedGeometry {
        vertices: geometry.vertices.iter().map(|v| v.moved(dx, dy)).collect(),
        indices: geometry.indices.clone(),
    }
}

trait PlaneVertex: Copy {
    fn position(&self) -> [f32; 2];
    fn moved(self, dx: f32, dy: f32) -> Self;
    fn toward(self, other: Self, t: f32) -> Self;
}

impl PlaneVertex for PolygonVertex {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn moved(self, dx: f32, dy: f32) -> Self {
        PolygonVertex {
            position: [self.position[0] + dx, self.position[1] + dy],
        }
    }

    fn toward(self, other: Self, t: f32) -> Self {
        PolygonVertex {
            position: lerp(self.position, other.position, t),
        }
    }
}

impl PlaneVertex for GlyphVertex {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn moved(self, dx: f32, dy: f32) -> Self {
        GlyphVertex {
            position: [self.position[0] + dx, self.position[1] + dy],
            ..self
        }
    }

    fn toward(self, other: Self, t: f32) -> Self {
        GlyphVertex {
            position: lerp(self.position, other.position, t),
            texel: lerp(self.texel, other.texel, t),
        }
    }
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
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

    fn bounding(points: impl Iterator<Item = [f32; 2]>) -> Self {
        points.fold(
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
fn uncovered<V: PlaneVertex>(
    geometry: TessellatedGeometry<V>,
    covers: &[Rectangle],
) -> TessellatedGeometry<V> {
    if covers.is_empty() {
        return geometry;
    }
    let mut result = TessellatedGeometry::default();
    for triangle in geometry.indices.as_chunks::<3>().0 {
        let mut pieces = vec![triangle
            .iter()
            .map(|&index| geometry.vertices[index as usize])
            .collect::<Vec<_>>()];
        for cover in covers {
            pieces = pieces
                .iter()
                .flat_map(|piece| outside(piece, cover))
                .collect();
        }
        for piece in pieces {
            let first = result.vertices.len() as u32;
            let count = piece.len() as u32;
            result.vertices.extend(piece);
            for corner in 1..count - 1 {
                result
                    .indices
                    .extend([first, first + corner, first + corner + 1]);
            }
        }
    }
    result
}

/// Splits a convex polygon into the convex pieces that lie outside the cover.
fn outside<V: PlaneVertex>(polygon: &[V], cover: &Rectangle) -> Vec<Vec<V>> {
    let bounds = Rectangle::bounding(polygon.iter().map(V::position));
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
fn clip<V: PlaneVertex>(polygon: &[V], axis: usize, bound: f32, below: bool) -> Vec<V> {
    let inside = |vertex: &V| {
        let point = vertex.position();
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
            let [from, to] = [previous.position()[axis], current.position()[axis]];
            result.push(previous.toward(*current, (bound - from) / (to - from)));
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

    fn atlas() -> GlyphAtlas {
        GlyphAtlas::new(NOTE_FONT_DATA).unwrap()
    }

    #[test]
    fn prepares_every_printable_ascii_glyph_a_few_per_frame() {
        let mut atlas = atlas();
        let mut unprepared = PREPARED_CHARACTERS;
        let mut frames = 0;
        while !unprepared.is_empty() {
            prepare_glyphs(&mut atlas, &mut unprepared);
            frames += 1;
        }
        assert_eq!(frames, 48);
        let revision = atlas.revision();
        let ascii: String = PREPARED_CHARACTERS.collect();
        atlas
            .lay_out_wrapped(&ascii, NOTE_TEXT_HEIGHT, 100.0)
            .unwrap();
        assert_eq!(atlas.revision(), revision);
    }

    #[test]
    fn lays_out_short_text() {
        let wrapped = atlas()
            .lay_out_wrapped("Hello", NOTE_TEXT_HEIGHT, 100.0)
            .unwrap();
        assert!(!wrapped.geometry.vertices.is_empty());
        assert!(!wrapped.geometry.indices.is_empty());
        assert!(wrapped.width > 0.0);
        assert!(wrapped.height > 0.0);
    }

    #[test]
    fn wraps_long_text_into_multiple_lines() {
        let text =
            "This is a long note that should wrap onto several lines when given a narrow width";
        let mut atlas = atlas();
        let narrow = atlas.lay_out_wrapped(text, NOTE_TEXT_HEIGHT, 4.0).unwrap();
        let wide = atlas
            .lay_out_wrapped(text, NOTE_TEXT_HEIGHT, 100.0)
            .unwrap();
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
    fn hidden_text_keeps_its_texels_in_place() {
        let note = build_note_geometry(&mut atlas(), "H", &NoteSize::default());
        let [first, opposite] = [note.glyphs.vertices[0], note.glyphs.vertices[2]];
        let cover = Rectangle {
            min: [-0.1, -1.0],
            max: [0.1, 1.0],
        };
        let visible = uncovered(translated(&note.glyphs, Vec3::ZERO), &[cover]);
        assert!(!visible.indices.is_empty());
        for vertex in &visible.vertices {
            let x = vertex.position[0];
            assert!(x <= cover.min[0] + 1e-5 || x >= cover.max[0] - 1e-5);
            for axis in 0..2 {
                let t = (vertex.position[axis] - first.position[axis])
                    / (opposite.position[axis] - first.position[axis]);
                let expected = first.texel[axis] + t * (opposite.texel[axis] - first.texel[axis]);
                assert!((vertex.texel[axis] - expected).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn centers_text_in_the_note() {
        let note = build_note_geometry(&mut atlas(), "Hello there", &NoteSize::default());
        let xs = note.glyphs.vertices.iter().map(|v| v.position[0]);
        let (min, max) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!((min + max).abs() < 0.1, "min={min} max={max}");
    }
}
