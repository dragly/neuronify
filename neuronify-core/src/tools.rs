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

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Stimulate => "Stimulate",
            Tool::Erase => "Erase",
            Tool::Place(Element::ExcitatoryNeuron) => "Excitatory neuron",
            Tool::Place(Element::InhibitoryNeuron) => "Inhibitory neuron",
            Tool::Place(Element::AdaptiveNeuron) => "Adaptive neuron",
            Tool::Place(Element::BurstNeuron) => "Burst neuron",
            Tool::Place(Element::CurrentSource) => "Current source",
            Tool::Place(Element::TouchSensor) => "Touch sensor",
            Tool::Place(Element::RegularSpikeGenerator) => "Spike generator",
            Tool::Place(Element::PoissonGenerator) => "Poisson generator",
            Tool::Place(Element::Note) => "Note",
            Tool::Voltmeter => "Voltmeter",
            Tool::StaticConnection => "Static connection",
            Tool::Axon => "Axon",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Tool::Select => "Select elements and drag them around.",
            Tool::Stimulate => "Touch neurons to make them fire.",
            Tool::Erase => "Erase elements and connections.",
            Tool::Place(Element::ExcitatoryNeuron) => {
                "Its spikes push the neurons it connects to towards threshold."
            }
            Tool::Place(Element::InhibitoryNeuron) => {
                "Its spikes pull the neurons it connects to away from threshold."
            }
            Tool::Place(Element::AdaptiveNeuron) => "Fires less often the longer it is driven.",
            Tool::Place(Element::BurstNeuron) => {
                "Fires a short burst of spikes when it reaches threshold."
            }
            Tool::Place(Element::CurrentSource) => {
                "Injects a steady current into the neurons it connects to."
            }
            Tool::Place(Element::TouchSensor) => "Fires while you touch or hold it.",
            Tool::Place(Element::RegularSpikeGenerator) => "Fires at a regular frequency.",
            Tool::Place(Element::PoissonGenerator) => "Fires at random around a mean rate.",
            Tool::Place(Element::Note) => "Text on the canvas.",
            Tool::Voltmeter => "Click a neuron or compartment to plot its voltage.",
            Tool::StaticConnection => "Drag from a neuron or source to the neuron it should drive.",
            Tool::Axon => {
                "Drag from a neuron to grow a cable of compartments to a neuron or its dendrites."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolGroup {
    Neurons,
    Inputs,
    Connections,
}

impl ToolGroup {
    pub fn label(self) -> &'static str {
        match self {
            ToolGroup::Neurons => "Neurons",
            ToolGroup::Inputs => "Inputs",
            ToolGroup::Connections => "Connect",
        }
    }

    pub fn tools(self) -> &'static [Tool] {
        match self {
            ToolGroup::Neurons => &[
                Tool::Place(Element::ExcitatoryNeuron),
                Tool::Place(Element::InhibitoryNeuron),
                Tool::Place(Element::AdaptiveNeuron),
                Tool::Place(Element::BurstNeuron),
            ],
            ToolGroup::Inputs => &[
                Tool::Place(Element::CurrentSource),
                Tool::Place(Element::TouchSensor),
                Tool::Place(Element::RegularSpikeGenerator),
                Tool::Place(Element::PoissonGenerator),
            ],
            ToolGroup::Connections => &[Tool::StaticConnection, Tool::Axon],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionTool {
    pub start: Vec3,
    pub end: Vec3,
    pub from: Entity,
}

#[derive(Clone, Debug)]
pub struct ConnectionOffer {
    pub from: Entity,
    pub since: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionHint {
    pub string: Vec<Vec3>,
    pub label: Vec3,
    pub shown: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StimulationTool {
    pub position: Vec3,
}
