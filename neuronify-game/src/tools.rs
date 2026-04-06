use glam::Vec3;
use hecs::Entity;

#[derive(Clone, Debug, PartialEq)]
pub enum GameTool {
    Select,
    Axon,
    /// Build glial processes — connect glial cells to blood vessels or neurons.
    GlialProcess,
    Erase,
}

#[derive(Clone, Debug)]
pub struct ConnectionTool {
    pub start: Vec3,
    pub end: Vec3,
    pub from: Entity,
}

#[derive(Clone, Debug)]
pub struct PreviousCreation {
    pub entity: Entity,
}
