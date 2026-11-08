use crate::channel::ChannelId;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::units::{
    Celsius, Hertz, MicrofaradsPerSquareCentimetre, Micrometres, Microsiemens,
    MilliamperesPerSquareCentimetre, Millivolts, Nanofarads, OhmCentimetres, OhmSquareCentimetres,
    SiemensPerSquareCentimetre, SquareMicrometres,
};

pub const MAXIMUM_SEGMENTS: usize = 1001;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Density {
    pub channel: ChannelId,
    pub conductance: SiemensPerSquareCentimetre,
    pub reversal: Millivolts,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Membrane {
    pub capacitance: MicrofaradsPerSquareCentimetre,
    pub axial_resistivity: OhmCentimetres,
    pub leak: SiemensPerSquareCentimetre,
    pub leak_reversal: Millivolts,
    pub channels: Vec<Density>,
}

impl Membrane {
    pub fn squid() -> Self {
        Membrane {
            capacitance: MicrofaradsPerSquareCentimetre(1.0),
            axial_resistivity: OhmCentimetres(35.4),
            leak: SiemensPerSquareCentimetre(0.0003),
            leak_reversal: Millivolts(-54.3),
            channels: vec![
                Density {
                    channel: ChannelId::SquidSodium,
                    conductance: SiemensPerSquareCentimetre(0.12),
                    reversal: Millivolts(50.0),
                },
                Density {
                    channel: ChannelId::SquidPotassium,
                    conductance: SiemensPerSquareCentimetre(0.036),
                    reversal: Millivolts(-77.0),
                },
            ],
        }
    }

    pub fn resting_at(mut self, voltage: Millivolts, celsius: Celsius) -> Self {
        let current: MilliamperesPerSquareCentimetre = self
            .channels
            .iter()
            .map(|density| {
                let open: f64 = density
                    .channel
                    .channel()
                    .gates
                    .iter()
                    .map(|gate| gate.steady_state(voltage, celsius).powi(gate.power))
                    .product();
                density.conductance * open * (voltage - density.reversal)
            })
            .sum();
        self.leak_reversal = voltage + current / self.leak;
        self
    }

    pub fn passive(
        resistance: OhmSquareCentimetres,
        reversal: Millivolts,
        axial_resistivity: OhmCentimetres,
    ) -> Self {
        Membrane {
            capacitance: MicrofaradsPerSquareCentimetre(1.0),
            axial_resistivity,
            leak: 1.0 / resistance,
            leak_reversal: reversal,
            channels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeMembrane {
    Point,
    Lumped {
        capacitance: Nanofarads,
        conductance: Microsiemens,
        reversal: Millivolts,
    },
    Patch {
        area: SquareMicrometres,
        membrane: Membrane,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cable {
    pub from: usize,
    pub to: usize,
    pub length: Micrometres,
    pub diameter: Micrometres,
    pub membrane: Membrane,
}

impl Cable {
    pub fn length_constant(&self, frequency: Hertz) -> Micrometres {
        let Membrane {
            capacitance,
            axial_resistivity,
            ..
        } = self.membrane;
        let product = 4.0 * PI * frequency.0 * axial_resistivity.0 * capacitance.0;
        Micrometres(1e5 * (self.diameter.0 / product).sqrt())
    }

    pub fn segments(&self, d_lambda: f64) -> usize {
        let pieces = self.length / (d_lambda * self.length_constant(Hertz(100.0)));
        (((pieces + 0.9) / 2.0) as usize).min(MAXIMUM_SEGMENTS / 2) * 2 + 1
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Morphology {
    pub nodes: Vec<NodeMembrane>,
    pub cables: Vec<Cable>,
}

impl Morphology {
    pub fn node(&mut self, membrane: NodeMembrane) -> usize {
        self.nodes.push(membrane);
        self.nodes.len() - 1
    }

    pub fn cable(
        &mut self,
        from: usize,
        to: usize,
        length: Micrometres,
        diameter: Micrometres,
        membrane: Membrane,
    ) -> usize {
        self.cables.push(Cable {
            from,
            to,
            length,
            diameter,
            membrane,
        });
        self.cables.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cables_split_into_an_odd_number_of_segments_by_d_lambda() {
        let cable = |length, diameter| Cable {
            from: 0,
            to: 1,
            length: Micrometres(length),
            diameter: Micrometres(diameter),
            membrane: Membrane::squid(),
        };
        assert_eq!(cable(100.0, 1.0).segments(0.1), 3);
        assert_eq!(cable(100.0, 0.25).segments(0.1), 5);
        assert_eq!(cable(1.0, 1.0).segments(0.1), 1);
        assert!((cable(100.0, 1.0).length_constant(Hertz(100.0)).0 - 474.1).abs() < 0.5);
    }

    #[test]
    fn no_cable_splits_into_more_than_the_largest_number_of_segments() {
        let cable = |length, diameter| Cable {
            from: 0,
            to: 1,
            length: Micrometres(length),
            diameter: Micrometres(diameter),
            membrane: Membrane::squid(),
        };
        for (length, diameter) in [
            (100.0, 0.0),
            (100.0, 1e-20),
            (1e15, 1.0),
            (f64::INFINITY, 1.0),
        ] {
            assert_eq!(cable(length, diameter).segments(0.1), MAXIMUM_SEGMENTS);
        }
        assert_eq!(cable(f64::NAN, 1.0).segments(0.1), 1);
    }
}
