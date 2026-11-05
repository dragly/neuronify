use std::collections::HashMap;

use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::SELECTION_RANGE;
use crate::measurement::voltmeter::Frame;
use crate::rendering::cylinders::SHEATH_RADIUS;
use crate::rendering::{compartment_radii, element_radius};

use super::{meter_frames, nearest_within};

#[derive(Clone, Copy)]
struct Piece {
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

pub struct Targets {
    frames: Vec<(Entity, Vec3, Frame)>,
    elements: Vec<(Entity, Vec3)>,
    pieces: Vec<Piece>,
    notes: Vec<(Entity, Vec3, NoteSize)>,
}

impl Targets {
    pub fn new(world: &World) -> Targets {
        let elements = world
            .query::<&Position>()
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
            .query::<(&Position, &NoteSize)>()
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

    pub fn at(&self, position: Vec3) -> Option<(Entity, Vec3)> {
        self.frame_at(position)
            .or_else(|| nearest_within(self.elements.iter().copied(), position, SELECTION_RANGE))
            .or_else(|| self.note_at(position))
    }

    pub fn near(&self, position: Vec3, reach: f32) -> Option<(Entity, Vec3)> {
        self.frame_at(position)
            .or_else(|| self.piece_near(position, reach))
            .or_else(|| self.note_at(position))
    }

    fn piece_near(&self, position: Vec3, reach: f32) -> Option<(Entity, Vec3)> {
        self.pieces
            .iter()
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
    let cables = world
        .query::<(&Connection, Option<&Myelin>)>()
        .with::<&CompartmentCurrent>()
        .iter()
        .filter(|(_, (cable, _))| !cable.directional)
        .map(|(_, (cable, sheath))| (cable.from, cable.to, sheath.is_some()))
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
                radius: if sheathed {
                    radius.max(SHEATH_RADIUS)
                } else {
                    radius
                },
            });
        }
    }
    halves
}
