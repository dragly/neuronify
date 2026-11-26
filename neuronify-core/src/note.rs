pub mod drawing;
pub mod gui;

use glam::Vec3;
use hecs::{Entity, EntityBuilder, World};
use serde::{Deserialize, Serialize};

use crate::components::{Deletable, Position};
use crate::serialization::bounds::{bounded_f32, FRAME_SIZES};

pub const NAME: &str = "Note";
pub const DESCRIPTION: &str = "Write a note on the canvas.";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Annotation {
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoteSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub example: String,
}

impl Default for NoteSize {
    fn default() -> Self {
        Self {
            width: 7.2,
            height: 4.8,
        }
    }
}

pub fn builder(position: Vec3) -> EntityBuilder {
    let mut note = EntityBuilder::new();
    note.add_bundle((
        Position { position },
        Annotation {
            text: "New note".to_string(),
        },
        NoteSize::default(),
        Deletable {},
    ));
    note
}

pub fn spawn(world: &mut World, position: Vec3) -> Entity {
    world.spawn(builder(position).build())
}

pub fn corners(center: Vec3, size: &NoteSize) -> [Vec3; 2] {
    let half_extent = Vec3::new(size.height * 0.5, 0.0, size.width * 0.5);
    [center - half_extent, center + half_extent]
}

pub fn bound(world: &mut World) {
    for size in world.query_mut::<&mut NoteSize>() {
        let default = NoteSize::default();
        size.width = bounded_f32(size.width, FRAME_SIZES, default.width);
        size.height = bounded_f32(size.height, FRAME_SIZES, default.height);
    }
}
