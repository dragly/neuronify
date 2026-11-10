use neuronify_engine::units::{
    Celsius, MicrofaradsPerSquareCentimetre, Micrometres, Microsiemens, Milliseconds, Millivolts,
    OhmCentimetres, OhmSquareCentimetres, SiemensPerSquareCentimetre,
};
use neuronify_engine::{ChannelId, Density, Membrane};

use super::model::{DENDRITE_CAPACITANCE, MYELIN_CAPACITANCE, MYELIN_LEAK};

pub const CELSIUS: Celsius = Celsius(37.0);
pub const RESTING_POTENTIAL: Millivolts = Millivolts(-70.0);
pub const AXON_DIAMETER: Micrometres = Micrometres(0.4);
pub const ENDING_CONDUCTANCE: Microsiemens = Microsiemens(0.01);
pub const ENDING_DELAY: Milliseconds = Milliseconds(0.5);
pub const SHORTEST_DENDRITE_PULSE_INTERVAL: Milliseconds = Milliseconds(15.0);
const MEMBRANE_RESISTANCE: OhmSquareCentimetres = OhmSquareCentimetres(20_000.0);
const AXIAL_RESISTIVITY: OhmCentimetres = OhmCentimetres(150.0);
const DENDRITE_MEMBRANE_RESISTANCE: OhmSquareCentimetres = OhmSquareCentimetres(8_000.0);
const DENDRITE_AXIAL_RESISTIVITY: OhmCentimetres = OhmCentimetres(300.0);
const SODIUM_REVERSAL: Millivolts = Millivolts(60.0);
const POTASSIUM_REVERSAL: Millivolts = Millivolts(-90.0);
const HCN_REVERSAL: Millivolts = Millivolts(-45.0);
const AXON_SODIUM: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.2);
const AXON_POTASSIUM: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.06);
const DENDRITE_SODIUM: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.03);
const DENDRITE_POTASSIUM: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.009);
const DENDRITE_A_TYPE: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.02);
const DENDRITE_HCN: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.005);
const RANVIER_SODIUM: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(3.0);
const RANVIER_LEAK: SiemensPerSquareCentimetre = SiemensPerSquareCentimetre(0.02);

fn sodium(conductance: SiemensPerSquareCentimetre) -> Density {
    Density {
        channel: ChannelId::CorticalSodium,
        conductance,
        reversal: SODIUM_REVERSAL,
    }
}

fn potassium(conductance: SiemensPerSquareCentimetre) -> Density {
    Density {
        channel: ChannelId::CorticalPotassium,
        conductance,
        reversal: POTASSIUM_REVERSAL,
    }
}

pub fn axon_membrane() -> Membrane {
    Membrane {
        capacitance: MicrofaradsPerSquareCentimetre(1.0),
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
        capacitance: MicrofaradsPerSquareCentimetre(1.0),
        axial_resistivity: AXIAL_RESISTIVITY,
        leak: RANVIER_LEAK,
        leak_reversal: RESTING_POTENTIAL,
        channels: vec![sodium(RANVIER_SODIUM)],
    }
    .resting_at(RESTING_POTENTIAL, CELSIUS)
}
