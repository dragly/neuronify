use crate::channel::ChannelId;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Density {
    pub channel: ChannelId,
    pub conductance: f64,
    pub reversal: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Membrane {
    pub capacitance: f64,
    pub axial_resistivity: f64,
    pub leak: f64,
    pub leak_reversal: f64,
    pub channels: Vec<Density>,
}

impl Membrane {
    pub fn squid() -> Self {
        Membrane {
            capacitance: 1.0,
            axial_resistivity: 35.4,
            leak: 0.0003,
            leak_reversal: -54.3,
            channels: vec![
                Density {
                    channel: ChannelId::SquidSodium,
                    conductance: 0.12,
                    reversal: 50.0,
                },
                Density {
                    channel: ChannelId::SquidPotassium,
                    conductance: 0.036,
                    reversal: -77.0,
                },
            ],
        }
    }

    pub fn resting_at(mut self, voltage: f64, celsius: f64) -> Self {
        let current: f64 = self
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

    pub fn passive(resistance: f64, reversal: f64, axial_resistivity: f64) -> Self {
        Membrane {
            capacitance: 1.0,
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
        capacitance: f64,
        conductance: f64,
        reversal: f64,
    },
    Patch {
        area: f64,
        membrane: Membrane,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cable {
    pub from: usize,
    pub to: usize,
    pub length: f64,
    pub diameter: f64,
    pub membrane: Membrane,
}

impl Cable {
    pub fn length_constant(&self, frequency: f64) -> f64 {
        let Membrane {
            capacitance,
            axial_resistivity,
            ..
        } = self.membrane;
        1e5 * (self.diameter / (4.0 * PI * frequency * axial_resistivity * capacitance)).sqrt()
    }

    pub fn segments(&self, d_lambda: f64) -> usize {
        let pieces = self.length / (d_lambda * self.length_constant(100.0));
        ((pieces + 0.9) / 2.0) as usize * 2 + 1
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
        length: f64,
        diameter: f64,
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
            length,
            diameter,
            membrane: Membrane::squid(),
        };
        assert_eq!(cable(100.0, 1.0).segments(0.1), 3);
        assert_eq!(cable(100.0, 0.25).segments(0.1), 5);
        assert_eq!(cable(1.0, 1.0).segments(0.1), 1);
        assert!((cable(100.0, 1.0).length_constant(100.0) - 474.1).abs() < 0.5);
    }
}
