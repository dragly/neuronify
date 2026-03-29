use glam::Vec3;
use hecs::Entity;

#[derive(Clone, Debug, PartialEq)]
pub enum GameTool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    Axon,
    MembraneSegment,
    MotorCilia,
    SpikeGenerator,
    PoissonGenerator,
    GlialCell,
    ActivitySensor,
    ChemicalSensor,
    TouchSensor,
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
