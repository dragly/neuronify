#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Label,
    Type,
    RefractoryPeriod,
    Threshold,
    Reset,
    RestingPotential,
    Resistance,
    Capacitance,
    TimeConstant,
    Adaptation,
    AdaptationTimeConstant,
    Current,
    Frequency,
    Strength,
    Rate,
    MaximumCurrent,
    SynapseTimeConstant,
    Delay,
    HighestVoltage,
    LowestVoltage,
}

impl Setting {
    pub fn label(self) -> &'static str {
        match self {
            Setting::Label => "Label",
            Setting::Type => "Type",
            Setting::RefractoryPeriod => "Refractory period",
            Setting::Threshold => "Threshold",
            Setting::Reset => "Reset",
            Setting::RestingPotential => "Resting potential",
            Setting::Resistance => "Resistance",
            Setting::Capacitance => "Capacitance",
            Setting::TimeConstant => "Time constant",
            Setting::Adaptation => "Adaptation",
            Setting::AdaptationTimeConstant => "Recovery time",
            Setting::Current => "Current",
            Setting::Frequency => "Frequency",
            Setting::Strength => "Strength",
            Setting::Rate => "Rate",
            Setting::MaximumCurrent => "Maximum current",
            Setting::SynapseTimeConstant => "Time constant",
            Setting::Delay => "Delay",
            Setting::HighestVoltage => "Highest voltage",
            Setting::LowestVoltage => "Lowest voltage",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Setting::Label => "A name shown above the neuron.",
            Setting::Type => {
                "Excitatory spikes push the neurons they reach toward firing. \
                 Inhibitory spikes hold them back."
            }
            Setting::RefractoryPeriod => "How long the neuron stays silent after each spike.",
            Setting::Threshold => "The neuron fires when its voltage climbs above this.",
            Setting::Reset => "The voltage the neuron drops to right after it fires.",
            Setting::RestingPotential => "The voltage the neuron settles at when left alone.",
            Setting::Resistance => {
                "How hard it is for charge to leak out. More means inputs add up for longer."
            }
            Setting::Capacitance => {
                "How much charge it takes to change the voltage. More means slower changes."
            }
            Setting::TimeConstant => {
                "Resistance times capacitance: roughly how long the voltage takes to settle back."
            }
            Setting::Adaptation => "How much harder it gets to fire after each spike.",
            Setting::AdaptationTimeConstant => {
                "How long the neuron takes to recover from adapting."
            }
            Setting::Current => "How strongly it pushes the neurons it is connected to.",
            Setting::Frequency => "How many spikes it sends each second.",
            Setting::Strength => "How strongly each spike at this ending affects the next neuron.",
            Setting::Rate => "How many spikes it sends each second on average.",
            Setting::MaximumCurrent => "How hard each spike pushes the neuron it reaches.",
            Setting::SynapseTimeConstant => "How long the push from each spike lasts.",
            Setting::Delay => "How long a spike takes to arrive.",
            Setting::HighestVoltage => "The voltage at the top of each plot.",
            Setting::LowestVoltage => "The voltage at the bottom of each plot.",
        }
    }
}
