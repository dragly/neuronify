use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rate {
    Exp {
        rate: f64,
        midpoint: f64,
        scale: f64,
    },
    Sigmoid {
        rate: f64,
        midpoint: f64,
        scale: f64,
    },
    ExpLinear {
        rate: f64,
        midpoint: f64,
        scale: f64,
    },
}

impl Rate {
    pub fn at(&self, voltage: f64) -> f64 {
        match *self {
            Rate::Exp {
                rate,
                midpoint,
                scale,
            } => rate * ((voltage - midpoint) / scale).exp(),
            Rate::Sigmoid {
                rate,
                midpoint,
                scale,
            } => rate / (1.0 + ((midpoint - voltage) / scale).exp()),
            Rate::ExpLinear {
                rate,
                midpoint,
                scale,
            } => {
                let x = (voltage - midpoint) / scale;
                if x.abs() < 1e-6 {
                    rate * (1.0 + x / 2.0)
                } else {
                    rate * x / (1.0 - (-x).exp())
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gate {
    pub power: i32,
    pub alpha: Rate,
    pub beta: Rate,
}

impl Gate {
    pub fn steady_state(&self, voltage: f64) -> f64 {
        let (alpha, beta) = (self.alpha.at(voltage), self.beta.at(voltage));
        alpha / (alpha + beta)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    pub gates: Vec<Gate>,
    pub reference_celsius: f64,
    pub q10: f64,
}

impl Channel {
    pub fn rate_factor(&self, celsius: f64) -> f64 {
        self.q10.powf((celsius - self.reference_celsius) / 10.0)
    }
}

const SLOW_REFERENCE: f64 = 12.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ChannelId {
    SquidSodium,
    SquidPotassium,
    SlowSodium,
    SlowPotassium,
}

impl ChannelId {
    pub const ALL: [ChannelId; 4] = [
        ChannelId::SquidSodium,
        ChannelId::SquidPotassium,
        ChannelId::SlowSodium,
        ChannelId::SlowPotassium,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn channel(self) -> Channel {
        match self {
            ChannelId::SlowSodium => Channel {
                reference_celsius: SLOW_REFERENCE,
                ..ChannelId::SquidSodium.channel()
            },
            ChannelId::SlowPotassium => Channel {
                reference_celsius: SLOW_REFERENCE,
                ..ChannelId::SquidPotassium.channel()
            },
            ChannelId::SquidSodium => Channel {
                gates: vec![
                    Gate {
                        power: 3,
                        alpha: Rate::ExpLinear {
                            rate: 1.0,
                            midpoint: -40.0,
                            scale: 10.0,
                        },
                        beta: Rate::Exp {
                            rate: 4.0,
                            midpoint: -65.0,
                            scale: -18.0,
                        },
                    },
                    Gate {
                        power: 1,
                        alpha: Rate::Exp {
                            rate: 0.07,
                            midpoint: -65.0,
                            scale: -20.0,
                        },
                        beta: Rate::Sigmoid {
                            rate: 1.0,
                            midpoint: -35.0,
                            scale: 10.0,
                        },
                    },
                ],
                reference_celsius: 6.3,
                q10: 3.0,
            },
            ChannelId::SquidPotassium => Channel {
                gates: vec![Gate {
                    power: 4,
                    alpha: Rate::ExpLinear {
                        rate: 0.1,
                        midpoint: -55.0,
                        scale: 10.0,
                    },
                    beta: Rate::Exp {
                        rate: 0.125,
                        midpoint: -65.0,
                        scale: -80.0,
                    },
                }],
                reference_celsius: 6.3,
                q10: 3.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp_linear(x: f64, y: f64) -> f64 {
        x / ((x / y).exp() - 1.0)
    }

    #[test]
    fn squid_rates_match_the_hodgkin_huxley_formulas() {
        let sodium = ChannelId::SquidSodium.channel();
        let potassium = ChannelId::SquidPotassium.channel();
        for v in [-90.0, -65.0, -52.3, -20.0, 10.0, 35.0] {
            let expected = [
                (
                    0.1 * exp_linear(-(v + 40.0), 10.0),
                    4.0 * (-(v + 65.0) / 18.0).exp(),
                ),
                (
                    0.07 * (-(v + 65.0) / 20.0).exp(),
                    1.0 / ((-(v + 35.0) / 10.0).exp() + 1.0),
                ),
                (
                    0.01 * exp_linear(-(v + 55.0), 10.0),
                    0.125 * (-(v + 65.0) / 80.0).exp(),
                ),
            ];
            let gates = sodium.gates.iter().chain(&potassium.gates);
            for (gate, (alpha, beta)) in gates.zip(expected) {
                assert!((gate.alpha.at(v) - alpha).abs() < 1e-12 * alpha.abs().max(1.0));
                assert!((gate.beta.at(v) - beta).abs() < 1e-12 * beta.abs().max(1.0));
            }
        }
    }

    #[test]
    fn exp_linear_rate_is_smooth_at_its_midpoint() {
        let rate = Rate::ExpLinear {
            rate: 1.0,
            midpoint: -40.0,
            scale: 10.0,
        };
        assert!((rate.at(-40.0) - 1.0).abs() < 1e-9);
        assert!((rate.at(-40.0 + 1e-4) - rate.at(-40.0 - 1e-4)).abs() < 1e-4);
    }
}
