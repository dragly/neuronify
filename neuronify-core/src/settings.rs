#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Threshold,
    RestingPotential,
    Capacitance,
    Current,
    Frequency,
    Strength,
    Rate,
}

impl Setting {
    pub fn label(self) -> &'static str {
        match self {
            Setting::Threshold => "Threshold",
            Setting::RestingPotential => "Resting potential",
            Setting::Capacitance => "Capacitance",
            Setting::Current => "Current",
            Setting::Frequency => "Frequency",
            Setting::Strength => "Strength",
            Setting::Rate => "Rate",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Setting::Threshold => "The neuron fires when its voltage climbs above this.",
            Setting::RestingPotential => "The voltage the neuron settles at when left alone.",
            Setting::Capacitance => {
                "How much charge it takes to change the voltage. More means slower changes."
            }
            Setting::Current => "How strongly it pushes the neurons it is connected to.",
            Setting::Frequency => "How many spikes it sends each second.",
            Setting::Strength => "How strongly each spike at this ending affects the next neuron.",
            Setting::Rate => "How many spikes it sends each second on average.",
        }
    }
}
