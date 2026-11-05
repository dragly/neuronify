use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::SELECTION_RANGE;
use crate::measurement::voltmeter::Frame;

use super::{meter_frames, nearest_within};

pub struct Targets {
    frames: Vec<(Entity, Vec3, Frame)>,
    elements: Vec<(Entity, Vec3)>,
    notes: Vec<(Entity, Vec3, NoteSize)>,
}

impl Targets {
    pub fn new(world: &World) -> Targets {
        let elements = world
            .query::<&Position>()
            .without::<&Annotation>()
            .iter()
            .map(|(entity, element)| (entity, element.position))
            .collect();
        let notes = world
            .query::<(&Position, &NoteSize)>()
            .with::<&Annotation>()
            .iter()
            .map(|(entity, (note, size))| (entity, note.position, size.clone()))
            .collect();
        Targets {
            frames: meter_frames(world),
            elements,
            notes,
        }
    }

    pub fn at(&self, position: Vec3) -> Option<(Entity, Vec3)> {
        self.frame_at(position)
            .or_else(|| nearest_within(self.elements.iter().copied(), position, SELECTION_RANGE))
            .or_else(|| self.note_at(position))
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
