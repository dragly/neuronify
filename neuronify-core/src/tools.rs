use glam::Vec3;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::elements::Element;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Tool {
    #[default]
    View,
    Edit,
    Place(Element),
    Voltmeter,
    SpikeDetector,
    RatePlot,
    Loudspeaker,
    StaticConnection,
    Axon,
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
            Tool::Place(Element::CurrentSource) => "Current source",
            Tool::Place(Element::AlternatingCurrentSource) => "Alternating current",
            Tool::Place(Element::TouchSensor) => "Touch sensor",
            Tool::Place(Element::RegularSpikeGenerator) => "Regular spikes",
            Tool::Place(Element::PoissonGenerator) => "Random spikes",
            Tool::Place(Element::VisualInput) => "Visual input",
            Tool::Place(Element::Note) => "Note",
            Tool::Voltmeter => "Voltmeter",
            Tool::SpikeDetector => "Spike detector",
            Tool::RatePlot => "Firing rate plot",
            Tool::Loudspeaker => "Loudspeaker",
            Tool::StaticConnection => "Connection",
            Tool::Axon => "Axon",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Tool::View => "Watch the circuit, press touch sensors and tap things to inspect them.",
            Tool::Edit => "Select things to change them, and drag them around.",
            Tool::Stimulate => "Press a neuron to make it fire. Hold to keep it firing.",
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
            Tool::Place(Element::CurrentSource) => {
                "Feeds a steady current into the neurons it connects to."
            }
            Tool::Place(Element::AlternatingCurrentSource) => {
                "Injects a current that swings up and down like a wave."
            }
            Tool::Place(Element::VisualInput) => {
                "Looks at a moving picture and fires when it matches its receptive field."
            }
            Tool::Loudspeaker => {
                "Press a neuron to hear a sound each time it fires. \
                 Connect more neurons to hear them too."
            }
            Tool::RatePlot => {
                "Press a neuron to plot how often it fires. \
                 Connect more neurons to plot their average."
            }
            Tool::SpikeDetector => {
                "Press a neuron to draw a tick each time it fires. \
                 Connect more neurons to give each a row."
            }
            Tool::Place(Element::TouchSensor) => "Fires a spike each time you press it.",
            Tool::Place(Element::RegularSpikeGenerator) => "Sends spikes at a steady beat.",
            Tool::Place(Element::PoissonGenerator) => {
                "Sends spikes at random moments, at a set average rate."
            }
            Tool::Place(Element::Note) => "Write a note on the canvas.",
            Tool::Voltmeter => "Press a neuron to plot its voltage over time.",
            Tool::StaticConnection => {
                "Drag from a neuron or an input to the neuron it should reach."
            }
            Tool::Axon => "Drag from a neuron to grow an axon to another neuron or its dendrites.",
        }
    }

    pub fn edits(self) -> bool {
        self != Tool::View
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolGroup {
    Neurons,
    Inputs,
    Connections,
    Meters,
}

impl ToolGroup {
    pub fn label(self) -> &'static str {
        match self {
            ToolGroup::Neurons => "Neurons",
            ToolGroup::Inputs => "Inputs",
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
            ToolGroup::Inputs => &[
                Tool::Place(Element::CurrentSource),
                Tool::Place(Element::AlternatingCurrentSource),
                Tool::Place(Element::TouchSensor),
                Tool::Place(Element::RegularSpikeGenerator),
                Tool::Place(Element::PoissonGenerator),
                Tool::Place(Element::VisualInput),
            ],
            ToolGroup::Connections => &[Tool::StaticConnection, Tool::Axon],
            ToolGroup::Meters => &[
                Tool::Voltmeter,
                Tool::SpikeDetector,
                Tool::RatePlot,
                Tool::Loudspeaker,
            ],
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
