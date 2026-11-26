use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::components::{Nanoamperes, Seconds, Setting};
use crate::constants::STIMULATION_CURRENT;
use crate::elements::Element;
use crate::input::DWELL;
use crate::measurement::Meter;
use crate::stimuli::Stimulus;

const METER_TOOLS: [Tool; Meter::ALL.len()] = {
    let mut tools = [Tool::View; Meter::ALL.len()];
    let mut index = 0;
    while index < tools.len() {
        tools[index] = Tool::Meter(Meter::ALL[index]);
        index += 1;
    }
    tools
};

const STIMULUS_TOOLS: [Tool; Stimulus::ALL.len()] = {
    let mut tools = [Tool::View; Stimulus::ALL.len()];
    let mut index = 0;
    while index < tools.len() {
        tools[index] = Tool::Place(Element::Stimulus(Stimulus::ALL[index]));
        index += 1;
    }
    tools
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Tool {
    #[default]
    View,
    Edit,
    Place(Element),
    Meter(Meter),
    StaticConnection,
    Axon,
    MyelinatedAxon,
    Dendrite,
    Erase,
    Stimulate,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::View => "View",
            Tool::Edit => "Edit",
            Tool::Stimulate => "Stimulate",
            Tool::Erase => "Erase",
            Tool::Place(Element::ExcitatoryNeuron) => "Excitatory neuron",
            Tool::Place(Element::InhibitoryNeuron) => "Inhibitory neuron",
            Tool::Place(Element::AdaptiveNeuron) => "Adaptive neuron",
            Tool::Place(Element::BurstNeuron) => "Bursting neuron",
            Tool::Place(Element::Stimulus(stimulus)) => stimulus.about().name,
            Tool::Place(Element::Note) => "Note",
            Tool::Meter(meter) => meter.about().name,
            Tool::StaticConnection => "Connection",
            Tool::Axon => "Axon",
            Tool::MyelinatedAxon => "Myelinated axon",
            Tool::Dendrite => "Dendrite",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Tool::View => "Watch the circuit, press touch sensors and tap things to inspect them.",
            Tool::Edit => "Select things to change them, and drag them around.",
            Tool::Stimulate => {
                "Press a neuron to make it fire. Hold to keep it firing. \
                 Click again to set the current."
            }
            Tool::Erase => "Drag over things and connections to erase them.",
            Tool::Place(Element::ExcitatoryNeuron) => {
                "Blue. Its spikes push the neurons it connects to toward firing."
            }
            Tool::Place(Element::InhibitoryNeuron) => {
                "Red. Its spikes pull the neurons it connects to away from firing."
            }
            Tool::Place(Element::AdaptiveNeuron) => {
                "Fires less and less while it is driven, and recovers when left alone."
            }
            Tool::Place(Element::BurstNeuron) => {
                "Once it fires after a rest, it keeps firing for a short burst."
            }
            Tool::Place(Element::Stimulus(stimulus)) => stimulus.about().description,
            Tool::Meter(meter) => meter.about().description,
            Tool::Place(Element::Note) => "Write a note on the canvas.",
            Tool::StaticConnection => {
                "Drag from a neuron or an input to the neuron it should reach."
            }
            Tool::Axon => "Drag from a neuron to grow an axon to another neuron or its dendrites.",
            Tool::MyelinatedAxon => {
                "Like an axon, but wrapped in myelin. Spikes jump from gap to gap and travel faster."
            }
            Tool::Dendrite => {
                "Drag from a neuron or one of its dendrites to grow a branch that axons can end on."
            }
        }
    }

    pub fn edits(self) -> bool {
        self != Tool::View
    }

    pub fn meter(self) -> Option<Meter> {
        match self {
            Tool::Meter(meter) => Some(meter),
            _ => None,
        }
    }

    pub fn connects(self) -> bool {
        match self {
            Tool::StaticConnection | Tool::Axon | Tool::MyelinatedAxon => true,
            Tool::Place(element) => element.is_neuron() || element.is_stimulus(),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum ConnectionMode {
    One,
    #[default]
    Chain,
    FanOut,
}

impl Setting for ConnectionMode {}

impl ConnectionMode {
    pub fn dwell(self) -> Seconds {
        match self {
            ConnectionMode::Chain => 2.0 * DWELL,
            ConnectionMode::One | ConnectionMode::FanOut => DWELL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolGroup {
    Neurons,
    Stimuli,
    Connections,
    Meters,
}

impl ToolGroup {
    pub fn label(self) -> &'static str {
        match self {
            ToolGroup::Neurons => "Neurons",
            ToolGroup::Stimuli => "Inputs",
            ToolGroup::Connections => "Connect",
            ToolGroup::Meters => "Measure",
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
            ToolGroup::Stimuli => &STIMULUS_TOOLS,
            ToolGroup::Connections => &[
                Tool::StaticConnection,
                Tool::Axon,
                Tool::MyelinatedAxon,
                Tool::Dendrite,
            ],
            ToolGroup::Meters => &METER_TOOLS,
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
    pub since: Seconds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionHint {
    pub from: Entity,
    pub string: Vec<Vec3>,
    pub label: Vec3,
    pub shown: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StimulationTool {
    pub position: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stimulation {
    pub current: Nanoamperes,
}

impl Default for Stimulation {
    fn default() -> Self {
        Stimulation {
            current: STIMULATION_CURRENT,
        }
    }
}
