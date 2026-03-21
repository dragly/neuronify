use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Select,
    ExcitatoryNeuron,
    InhibitoryNeuron,
    CurrentSource,
    TouchSensor,
    RegularSpikeGenerator,
    PoissonGenerator,
    Voltmeter,
    StaticConnection,
    Axon,
    Erase,
    Stimulate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToolCategory {
    Interaction,
    Neurons,
    Connections,
}

impl ToolCategory {
    pub fn label(&self) -> &str {
        match self {
            ToolCategory::Interaction => "Interaction",
            ToolCategory::Neurons => "Neurons",
            ToolCategory::Connections => "Connections",
        }
    }
    pub fn tools(&self) -> Vec<(Tool, &str)> {
        match self {
            ToolCategory::Interaction => vec![
                (Tool::Select, "Select"),
                (Tool::Stimulate, "Stimulate"),
                (Tool::Erase, "Erase"),
            ],
            ToolCategory::Neurons => vec![
                (Tool::ExcitatoryNeuron, "Excitatory Neuron"),
                (Tool::InhibitoryNeuron, "Inhibitory Neuron"),
                (Tool::CurrentSource, "Current Source"),
                (Tool::TouchSensor, "Touch Sensor"),
                (Tool::RegularSpikeGenerator, "Spike Generator"),
                (Tool::PoissonGenerator, "Poisson Generator"),
                (Tool::Voltmeter, "Voltmeter"),
            ],
            ToolCategory::Connections => vec![
                (Tool::StaticConnection, "Static Connection"),
                (Tool::Axon, "Axon"),
            ],
        }
    }
}

pub const TOOL_CATEGORIES: [ToolCategory; 3] = [
    ToolCategory::Interaction,
    ToolCategory::Neurons,
    ToolCategory::Connections,
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreviousCreation {
    pub entity: Entity,
}

#[derive(Clone, Debug)]
pub struct ConnectionTool {
    pub start: Vec3,
    pub end: Vec3,
    pub from: Entity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StimulationTool {
    pub position: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
