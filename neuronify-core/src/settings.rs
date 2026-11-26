#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Setting {
    pub label: &'static str,
    pub description: &'static str,
}

impl Setting {
    pub const LABEL: Setting = Setting {
        label: "Label",
        description: "A name shown above the neuron.",
    };

    pub const TYPE: Setting = Setting {
        label: "Type",
        description: "Excitatory spikes push the neurons they reach toward firing. \
                      Inhibitory spikes hold them back.",
    };

    pub const REFRACTORY_PERIOD: Setting = Setting {
        label: "Refractory period",
        description: "How long the neuron stays silent after each spike.",
    };

    pub const THRESHOLD: Setting = Setting {
        label: "Threshold",
        description: "The neuron fires when its voltage climbs above this.",
    };

    pub const SPIKE_THRESHOLD: Setting = Setting {
        label: "Spike threshold",
        description: "Its connections count a spike when the voltage climbs above this, \
                      and the next one only after it has fallen back below.",
    };

    pub const RESET: Setting = Setting {
        label: "Reset",
        description: "The voltage the neuron drops to right after it fires.",
    };

    pub const RESTING_POTENTIAL: Setting = Setting {
        label: "Resting potential",
        description: "The voltage the neuron settles at when left alone.",
    };

    pub const RESISTANCE: Setting = Setting {
        label: "Resistance",
        description: "How hard it is for charge to leak out. More means inputs add up for longer.",
    };

    pub const CAPACITANCE: Setting = Setting {
        label: "Capacitance",
        description: "How much charge it takes to change the voltage. More means slower changes.",
    };

    pub const TIME_CONSTANT: Setting = Setting {
        label: "Time constant",
        description:
            "Resistance times capacitance: roughly how long the voltage takes to settle back.",
    };

    pub const SOMA_DIAMETER: Setting = Setting {
        label: "Soma diameter",
        description: "How wide the cell body is. A bigger body takes more current to fire.",
    };

    pub const ADAPTATION: Setting = Setting {
        label: "Adaptation",
        description: "How much harder it gets to fire after each spike.",
    };

    pub const ADAPTATION_TIME_CONSTANT: Setting = Setting {
        label: "Recovery time",
        description: "How long the neuron takes to recover from adapting.",
    };

    pub const TOUCH_CURRENT: Setting = Setting {
        label: "Current",
        description: "How hard holding a neuron pushes it toward firing.",
    };

    pub const STRENGTH: Setting = Setting {
        label: "Strength",
        description: "How strongly each spike at this ending affects the next neuron.",
    };

    pub const MAXIMUM_CURRENT: Setting = Setting {
        label: "Maximum current",
        description: "How hard each spike pushes the neuron it reaches.",
    };

    pub const SYNAPSE_TIME_CONSTANT: Setting = Setting {
        label: "Time constant",
        description: "How long the push from each spike lasts.",
    };

    pub const DELAY: Setting = Setting {
        label: "Delay",
        description: "How long a spike takes to arrive.",
    };

    pub const CHANNELS: Setting = Setting {
        label: "Channels",
        description: "Squid uses the Hodgkin-Huxley channels from textbooks. \
                      Mammal uses channels measured in rat neurons at body temperature, \
                      with two more kinds in the dendrites.",
    };

    pub const TEMPERATURE: Setting = Setting {
        label: "Temperature",
        description: "Temperature affects how fast channels open and close. \
                      With higher temperatures, spikes are shorter and travel faster.",
    };
}
