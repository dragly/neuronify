use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::elements::Element;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Tool {
    #[default]
    Select,
    Place(Element),
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
    Annotations,
}

impl ToolCategory {
    pub fn label(&self) -> &str {
        match self {
            ToolCategory::Interaction => "Interaction",
            ToolCategory::Neurons => "Neurons",
            ToolCategory::Connections => "Connections",
            ToolCategory::Annotations => "Annotations",
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
                (Tool::Place(Element::ExcitatoryNeuron), "Excitatory Neuron"),
                (Tool::Place(Element::InhibitoryNeuron), "Inhibitory Neuron"),
                (Tool::Place(Element::AdaptiveNeuron), "Adaptive Neuron"),
                (Tool::Place(Element::BurstNeuron), "Burst Neuron"),
                (Tool::Place(Element::CurrentSource), "Current Source"),
                (Tool::Place(Element::TouchSensor), "Touch Sensor"),
                (
                    Tool::Place(Element::RegularSpikeGenerator),
                    "Spike Generator",
                ),
                (Tool::Place(Element::PoissonGenerator), "Poisson Generator"),
                (Tool::Voltmeter, "Voltmeter"),
            ],
            ToolCategory::Connections => vec![
                (Tool::StaticConnection, "Static Connection"),
                (Tool::Axon, "Axon"),
            ],
            ToolCategory::Annotations => vec![(Tool::Place(Element::Note), "Note")],
        }
    }
}

pub const TOOL_CATEGORIES: [ToolCategory; 4] = [
    ToolCategory::Interaction,
    ToolCategory::Neurons,
    ToolCategory::Connections,
    ToolCategory::Annotations,
];

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
