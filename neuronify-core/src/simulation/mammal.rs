use neuronify_engine::{ChannelId, Density, Membrane};

use super::model::{DENDRITE_CAPACITANCE, MYELIN_CAPACITANCE, MYELIN_LEAK};

pub const CELSIUS: f64 = 37.0;
pub const RESTING_POTENTIAL: f64 = -70.0;
pub const AXON_DIAMETER: f64 = 0.4;
pub const ENDING_CONDUCTANCE: f64 = 0.01;
pub const SHORTEST_DENDRITE_PULSE_INTERVAL: f64 = 15.0;
const MEMBRANE_RESISTANCE: f64 = 20_000.0;
const AXIAL_RESISTIVITY: f64 = 150.0;
const DENDRITE_MEMBRANE_RESISTANCE: f64 = 8_000.0;
const DENDRITE_AXIAL_RESISTIVITY: f64 = 300.0;
const SODIUM_REVERSAL: f64 = 60.0;
const POTASSIUM_REVERSAL: f64 = -90.0;
const HCN_REVERSAL: f64 = -45.0;
const AXON_SODIUM: f64 = 0.2;
const AXON_POTASSIUM: f64 = 0.06;
const DENDRITE_SODIUM: f64 = 0.03;
const DENDRITE_POTASSIUM: f64 = 0.009;
const DENDRITE_A_TYPE: f64 = 0.02;
const DENDRITE_HCN: f64 = 0.005;
const RANVIER_SODIUM: f64 = 3.0;
const RANVIER_LEAK: f64 = 0.02;

fn sodium(conductance: f64) -> Density {
    Density {
        channel: ChannelId::CorticalSodium,
        conductance,
        reversal: SODIUM_REVERSAL,
    }
}

fn potassium(conductance: f64) -> Density {
    Density {
        channel: ChannelId::CorticalPotassium,
        conductance,
        reversal: POTASSIUM_REVERSAL,
    }
}

pub fn axon_membrane() -> Membrane {
    Membrane {
        capacitance: 1.0,
        axial_resistivity: AXIAL_RESISTIVITY,
        leak: 1.0 / MEMBRANE_RESISTANCE,
        leak_reversal: RESTING_POTENTIAL,
        channels: vec![sodium(AXON_SODIUM), potassium(AXON_POTASSIUM)],
    }
    .resting_at(RESTING_POTENTIAL, CELSIUS)
}

pub fn dendrite_membrane() -> Membrane {
    Membrane {
        capacitance: DENDRITE_CAPACITANCE,
        axial_resistivity: DENDRITE_AXIAL_RESISTIVITY,
        leak: 1.0 / DENDRITE_MEMBRANE_RESISTANCE,
        leak_reversal: RESTING_POTENTIAL,
        channels: vec![
            sodium(DENDRITE_SODIUM),
            potassium(DENDRITE_POTASSIUM),
            Density {
                channel: ChannelId::ATypePotassium,
                conductance: DENDRITE_A_TYPE,
                reversal: POTASSIUM_REVERSAL,
            },
            Density {
                channel: ChannelId::Hcn,
                conductance: DENDRITE_HCN,
                reversal: HCN_REVERSAL,
            },
        ],
    }
    .resting_at(RESTING_POTENTIAL, CELSIUS)
}

pub fn myelin_membrane() -> Membrane {
    Membrane {
        capacitance: MYELIN_CAPACITANCE,
        axial_resistivity: AXIAL_RESISTIVITY,
        leak: MYELIN_LEAK,
        leak_reversal: RESTING_POTENTIAL,
        channels: Vec::new(),
    }
}

pub fn ranvier_membrane() -> Membrane {
    Membrane {
        capacitance: 1.0,
        axial_resistivity: AXIAL_RESISTIVITY,
        leak: RANVIER_LEAK,
        leak_reversal: RESTING_POTENTIAL,
        channels: vec![sodium(RANVIER_SODIUM)],
    }
    .resting_at(RESTING_POTENTIAL, CELSIUS)
}
