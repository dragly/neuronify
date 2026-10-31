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
    Amplitude,
    AlternatingFrequency,
    Frequency,
    Strength,
    Rate,
    MaximumCurrent,
    SynapseTimeConstant,
    Delay,
    HighestVoltage,
    LowestVoltage,
    TimeRange,
    Window,
    KernelWidth,
    HighestRate,
    LowestRate,
    Sound,
    Volume,
    Mute,
    Picture,
    PictureAngle,
    ReceptiveField,
    FieldAngle,
    FieldSideways,
    FieldUpDown,
    OffCenter,
    Sensitivity,
    ShowField,
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
            Setting::Amplitude => "Amplitude",
            Setting::AlternatingFrequency => "Frequency",
            Setting::Frequency => "Frequency",
            Setting::Strength => "Strength",
            Setting::Rate => "Rate",
            Setting::MaximumCurrent => "Maximum current",
            Setting::SynapseTimeConstant => "Time constant",
            Setting::Delay => "Delay",
            Setting::HighestVoltage => "Highest voltage",
            Setting::LowestVoltage => "Lowest voltage",
            Setting::TimeRange => "Time range",
            Setting::Window => "Window",
            Setting::KernelWidth => "Smoothing",
            Setting::HighestRate => "Highest rate",
            Setting::LowestRate => "Lowest rate",
            Setting::Sound => "Sound",
            Setting::Volume => "Volume",
            Setting::Mute => "Mute",
            Setting::Picture => "Picture",
            Setting::PictureAngle => "Picture angle",
            Setting::ReceptiveField => "Receptive field",
            Setting::FieldAngle => "Field angle",
            Setting::FieldSideways => "Field sideways",
            Setting::FieldUpDown => "Field up or down",
            Setting::OffCenter => "Off center",
            Setting::Sensitivity => "Sensitivity",
            Setting::ShowField => "Show field",
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
            Setting::Amplitude => "How far the current swings up and down.",
            Setting::AlternatingFrequency => "How many times a second the current swings.",
            Setting::Frequency => "How many spikes it sends each second.",
            Setting::Strength => "How strongly each spike at this ending affects the next neuron.",
            Setting::Rate => "How many spikes it sends each second on average.",
            Setting::MaximumCurrent => "How hard each spike pushes the neuron it reaches.",
            Setting::SynapseTimeConstant => "How long the push from each spike lasts.",
            Setting::Delay => "How long a spike takes to arrive.",
            Setting::HighestVoltage => "The voltage at the top of each plot.",
            Setting::LowestVoltage => "The voltage at the bottom of each plot.",
            Setting::TimeRange => "How far back in time the plot reaches.",
            Setting::Window => "Only spikes this recent count toward the rate.",
            Setting::KernelWidth => {
                "How long each spike counts. More gives a smoother but slower curve."
            }
            Setting::HighestRate => "The rate at the top of the plot.",
            Setting::LowestRate => "The rate at the bottom of the plot.",
            Setting::Sound => "What it plays each time a connected neuron fires.",
            Setting::Volume => "How loud it plays.",
            Setting::Mute => "Silences it without disconnecting it.",
            Setting::Picture => "What the visual input looks at.",
            Setting::PictureAngle => "Which way the picture is turned.",
            Setting::ReceptiveField => "The pattern of light that makes it fire.",
            Setting::FieldAngle => "Which way the stripes of the field are turned.",
            Setting::FieldSideways => "Where the field sits, from left to right.",
            Setting::FieldUpDown => "Where the field sits, from bottom to top.",
            Setting::OffCenter => "Fire for darkness in the center instead of light.",
            Setting::Sensitivity => "How often it fires when the picture matches the field perfectly.",
            Setting::ShowField => {
                "Tints the picture orange where light excites it and blue where light holds it back."
            }
        }
    }
}
