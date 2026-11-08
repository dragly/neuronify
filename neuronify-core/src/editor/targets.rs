use std::collections::HashMap;

use glam::{Vec2, Vec3};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::SELECTION_RANGE;
use crate::measurement::voltmeter::{Corner, Frame};
use crate::rendering::bounds::note_corners;
use crate::rendering::cylinders::sheath_radius;
use crate::rendering::{compartment_radii, element_radius};

use super::{meter_frames, nearest_within};

const OUTLINE_BISECTIONS: usize = 16;

#[derive(Clone, Copy)]
pub struct Piece {
    entity: Entity,
    position: Vec3,
    start: Vec3,
    end: Vec3,
    radius: f32,
}

impl Piece {
    fn gap(&self, point: Vec3) -> f32 {
        let along = self.end - self.start;
        let t = (point - self.start).dot(along) / along.length_squared().max(f32::MIN_POSITIVE);
        point.distance(self.start + t.clamp(0.0, 1.0) * along) - self.radius
    }
}

pub enum Shape {
    Pieces(Vec<Piece>),
    Rectangle([Vec3; 2]),
}

impl Shape {
    pub fn center(&self) -> Vec3 {
        match self {
            Shape::Pieces(pieces) => pieces[0].position,
            Shape::Rectangle([min, max]) => min.lerp(*max, 0.5),
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            Shape::Pieces(_) => self.thickness(),
            Shape::Rectangle([min, max]) => 0.5 * min.distance(*max),
        }
    }

    pub fn thickness(&self) -> f32 {
        match self {
            Shape::Pieces(pieces) => pieces.iter().map(|piece| piece.radius).fold(0.0, f32::max),
            Shape::Rectangle(_) => f32::INFINITY,
        }
    }

    pub fn outline(&self, gap: f32, directions: impl IntoIterator<Item = Vec3>) -> Vec<Vec3> {
        let center = self.center();
        let reach = self.extent() + gap;
        directions
            .into_iter()
            .map(|direction| {
                let (mut inside, mut outside) = (0.0, reach);
                for _ in 0..OUTLINE_BISECTIONS {
                    let middle = 0.5 * (inside + outside);
                    if self.gap(center + middle * direction) < gap {
                        inside = middle;
                    } else {
                        outside = middle;
                    }
                }
                center + 0.5 * (inside + outside) * direction
            })
            .collect()
    }

    fn gap(&self, point: Vec3) -> f32 {
        match self {
            Shape::Pieces(pieces) => pieces
                .iter()
                .map(|piece| piece.gap(point))
                .fold(f32::INFINITY, f32::min),
            Shape::Rectangle([min, max]) => {
                let offset = point - min.lerp(*max, 0.5);
                let half = 0.5 * (*max - *min);
                let beyond = Vec2::new(offset.x.abs() - half.x, offset.z.abs() - half.z);
                beyond.max(Vec2::ZERO).length() + beyond.max_element().min(0.0)
            }
        }
    }

    fn extent(&self) -> f32 {
        match self {
            Shape::Pieces(pieces) => pieces
                .iter()
                .map(|piece| piece.end.distance(piece.start) + piece.radius)
                .fold(0.0, f32::max),
            Shape::Rectangle([min, max]) => 0.5 * min.distance(*max),
        }
    }
}

pub struct Targets {
    frames: Vec<(Entity, Vec3, Frame)>,
    elements: Vec<(Entity, Vec3)>,
    pieces: Vec<Piece>,
    notes: Vec<(Entity, Vec3, NoteSize)>,
}

impl Targets {
    pub fn new(world: &World) -> Targets {
        let elements = world
            .query::<(Entity, &Position)>()
            .without::<&Annotation>()
            .iter()
            .map(|(entity, element)| (entity, element.position))
            .collect::<Vec<_>>();
        let frames = meter_frames(world);
        let radii = compartment_radii(world);
        let mut pieces: Vec<Piece> = elements
            .iter()
            .filter(|&&(entity, _)| frames.iter().all(|&(framed, _, _)| framed != entity))
            .map(|&(entity, position)| Piece {
                entity,
                position,
                start: position,
                end: position,
                radius: element_radius(world, entity, &radii),
            })
            .collect();
        pieces.extend(cable_halves(world, &radii));
        let notes = world
            .query::<(Entity, (&Position, &NoteSize))>()
            .with::<&Annotation>()
            .iter()
            .map(|(entity, (note, size))| (entity, note.position, size.clone()))
            .collect();
        Targets {
            frames,
            elements,
            pieces,
            notes,
        }
    }

    pub fn around(&self, center: Vec3, radius: f32) -> Targets {
        let gaps: Vec<f32> = self.pieces.iter().map(|piece| piece.gap(center)).collect();
        let nearest = gaps.iter().copied().fold(f32::INFINITY, f32::min);
        Targets {
            frames: self.frames.clone(),
            elements: self.elements.clone(),
            pieces: self
                .pieces
                .iter()
                .zip(gaps)
                .filter(|&(_, gap)| gap <= nearest + 2.0 * radius)
                .map(|(&piece, _)| piece)
                .collect(),
            notes: self.notes.clone(),
        }
    }

    pub fn shape(&self, entity: Entity) -> Option<Shape> {
        let framed = self
            .frames
            .iter()
            .find(|&&(framed, _, _)| framed == entity)
            .map(|(_, _, frame)| {
                [
                    frame.corner(Corner::BottomLeft),
                    frame.corner(Corner::TopRight),
                ]
            });
        let noted = || {
            self.notes
                .iter()
                .find(|&&(note, _, _)| note == entity)
                .map(|(_, center, size)| note_corners(*center, size))
        };
        if let Some(corners) = framed.or_else(noted) {
            return Some(Shape::Rectangle(corners));
        }
        let pieces: Vec<Piece> = self
            .pieces
            .iter()
            .filter(|piece| piece.entity == entity)
            .copied()
            .collect();
        (!pieces.is_empty()).then_some(Shape::Pieces(pieces))
    }

    pub fn at(&self, position: Vec3) -> Option<(Entity, Vec3)> {
        self.frame_at(position)
            .or_else(|| nearest_within(self.elements.iter().copied(), position, SELECTION_RANGE))
            .or_else(|| self.note_at(position))
    }

    pub fn near(&self, position: Vec3, reach: f32) -> Option<(Entity, Vec3)> {
        self.frame_at(position)
            .or_else(|| self.piece_near(position, reach, |_| true))
            .or_else(|| self.note_at(position))
    }

    pub fn piece_near(
        &self,
        position: Vec3,
        reach: f32,
        accepts: impl Fn(Entity) -> bool,
    ) -> Option<(Entity, Vec3)> {
        self.pieces
            .iter()
            .filter(|piece| accepts(piece.entity))
            .map(|piece| (piece, piece.gap(position)))
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .filter(|&(_, gap)| gap < reach)
            .map(|(piece, _)| (piece.entity, piece.position))
    }

    fn frame_at(&self, position: Vec3) -> Option<(Entity, Vec3)> {
        self.frames
            .iter()
            .find(|(_, _, frame)| frame.contains(position))
            .map(|&(meter, meter_position, _)| (meter, meter_position))
    }

    fn note_at(&self, position: Vec3) -> Option<(Entity, Vec3)> {
        self.notes
            .iter()
            .filter(|(_, note, size)| {
                let offset = position - *note;
                offset.x.abs() <= size.height * 0.5 && offset.z.abs() <= size.width * 0.5
            })
            .max_by_key(|(entity, _, _)| entity.id())
            .map(|&(entity, note, _)| (entity, note))
    }
}

fn cable_halves(world: &World, radii: &HashMap<Entity, f32>) -> Vec<Piece> {
    let mut halves = Vec::new();
    let sheath = sheath_radius(&NeuronDisplay::of(world));
    let cables = world
        .query::<(&Connection, Option<&Myelin>)>()
        .with::<&Cable>()
        .iter()
        .filter(|(cable, _)| !cable.directional)
        .map(|(cable, sheath)| (cable.from, cable.to, sheath.is_some()))
        .collect::<Vec<_>>();
    for (from, to, sheathed) in cables {
        let (Ok(start), Ok(end)) = (world.get::<&Position>(from), world.get::<&Position>(to))
        else {
            continue;
        };
        let middle = start.position.lerp(end.position, 0.5);
        for (entity, position) in [(from, start.position), (to, end.position)] {
            let Some(&radius) = radii.get(&entity) else {
                continue;
            };
            halves.push(Piece {
                entity,
                position,
                start: position,
                end: middle,
                radius: if sheathed { radius.max(sheath) } else { radius },
            });
        }
    }
    halves
}
