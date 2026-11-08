use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};

use crate::units::{Celsius, Kilohertz, Milliseconds, Millivolts, PerMillivolt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rate<T = Kilohertz> {
    Exp {
        rate: T,
        midpoint: Millivolts,
        scale: Millivolts,
    },
    Sigmoid {
        rate: T,
        midpoint: Millivolts,
        scale: Millivolts,
    },
    ExpLinear {
        rate: T,
        midpoint: Millivolts,
        scale: Millivolts,
    },
}

impl<T: Copy + Mul<f64, Output = T> + Div<f64, Output = T>> Rate<T> {
    pub fn at(&self, voltage: Millivolts) -> T {
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

#[derive(Clone, Copy, Debug)]
pub enum Kinetics {
    Rates {
        alpha: Rate,
        beta: Rate,
        steady: Option<Rate<f64>>,
    },
    Formula(fn(Millivolts, Celsius) -> (f64, Milliseconds)),
}

#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub power: i32,
    pub kinetics: Kinetics,
}

impl Gate {
    pub fn steady_state(&self, voltage: Millivolts, celsius: Celsius) -> f64 {
        match self.kinetics {
            Kinetics::Rates {
                steady: Some(steady),
                ..
            } => steady.at(voltage),
            Kinetics::Rates { alpha, beta, .. } => {
                let (alpha, beta) = (alpha.at(voltage), beta.at(voltage));
                alpha / (alpha + beta)
            }
            Kinetics::Formula(formula) => formula(voltage, celsius).0,
        }
    }

    pub fn rate(&self, voltage: Millivolts, celsius: Celsius, factor: f64) -> Kilohertz {
        match self.kinetics {
            Kinetics::Rates { alpha, beta, .. } => {
                alpha.at(voltage) * factor + beta.at(voltage) * factor
            }
            Kinetics::Formula(formula) => 1.0 / formula(voltage, celsius).1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub gates: Vec<Gate>,
    pub reference_celsius: Celsius,
    pub q10: f64,
}

impl Channel {
    pub fn rate_factor(&self, celsius: Celsius) -> f64 {
        self.q10
            .powf((celsius - self.reference_celsius) / Celsius(10.0))
    }
}

const SLOW_REFERENCE: Celsius = Celsius(12.6);
const CORTICAL_REFERENCE: Celsius = Celsius(23.0);
const CORTICAL_Q10: f64 = 2.3;
const FARADAY: f64 = 96_480.0;
const GAS_CONSTANT: f64 = 8.315;
const ZERO_CELSIUS: f64 = 273.16;

fn per_millivolt(celsius: Celsius) -> PerMillivolt {
    PerMillivolt(1e-3 * FARADAY / (GAS_CONSTANT * (ZERO_CELSIUS + celsius.0)))
}

fn a_type_activation(voltage: Millivolts, celsius: Celsius) -> (f64, Milliseconds) {
    let charge = -1.5 - 1.0 / (1.0 + ((voltage + Millivolts(40.0)) / Millivolts(5.0)).exp());
    let exponent = charge * (voltage - Millivolts(11.0)) * per_millivolt(celsius);
    let alpha = exponent.exp();
    let beta = (0.55 * exponent).exp();
    let speed = 5.0f64.powf((celsius - Celsius(24.0)) / Celsius(10.0));
    let rate = Kilohertz(speed * 0.05 * (1.0 + alpha));
    let time_constant = (beta / rate).max(Milliseconds(0.1));
    (1.0 / (1.0 + alpha), time_constant)
}

fn a_type_inactivation(voltage: Millivolts, celsius: Celsius) -> (f64, Milliseconds) {
    let alpha = (3.0 * (voltage + Millivolts(56.0)) * per_millivolt(celsius)).exp();
    let time_constant = Milliseconds(0.26 * (voltage + Millivolts(50.0)).0).max(Milliseconds(2.0));
    (1.0 / (1.0 + alpha), time_constant)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ChannelId {
    SquidSodium,
    SquidPotassium,
    SlowSodium,
    SlowPotassium,
    CorticalSodium,
    CorticalPotassium,
    ATypePotassium,
    Hcn,
}

impl ChannelId {
    pub const ALL: [ChannelId; 8] = [
        ChannelId::SquidSodium,
        ChannelId::SquidPotassium,
        ChannelId::SlowSodium,
        ChannelId::SlowPotassium,
        ChannelId::CorticalSodium,
        ChannelId::CorticalPotassium,
        ChannelId::ATypePotassium,
        ChannelId::Hcn,
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
                        kinetics: Kinetics::Rates {
                            alpha: Rate::ExpLinear {
                                rate: Kilohertz(1.0),
                                midpoint: Millivolts(-40.0),
                                scale: Millivolts(10.0),
                            },
                            beta: Rate::Exp {
                                rate: Kilohertz(4.0),
                                midpoint: Millivolts(-65.0),
                                scale: Millivolts(-18.0),
                            },
                            steady: None,
                        },
                    },
                    Gate {
                        power: 1,
                        kinetics: Kinetics::Rates {
                            alpha: Rate::Exp {
                                rate: Kilohertz(0.07),
                                midpoint: Millivolts(-65.0),
                                scale: Millivolts(-20.0),
                            },
                            beta: Rate::Sigmoid {
                                rate: Kilohertz(1.0),
                                midpoint: Millivolts(-35.0),
                                scale: Millivolts(10.0),
                            },
                            steady: None,
                        },
                    },
                ],
                reference_celsius: Celsius(6.3),
                q10: 3.0,
            },
            ChannelId::SquidPotassium => Channel {
                gates: vec![Gate {
                    power: 4,
                    kinetics: Kinetics::Rates {
                        alpha: Rate::ExpLinear {
                            rate: Kilohertz(0.1),
                            midpoint: Millivolts(-55.0),
                            scale: Millivolts(10.0),
                        },
                        beta: Rate::Exp {
                            rate: Kilohertz(0.125),
                            midpoint: Millivolts(-65.0),
                            scale: Millivolts(-80.0),
                        },
                        steady: None,
                    },
                }],
                reference_celsius: Celsius(6.3),
                q10: 3.0,
            },
            ChannelId::CorticalSodium => Channel {
                gates: vec![
                    Gate {
                        power: 3,
                        kinetics: Kinetics::Rates {
                            alpha: Rate::ExpLinear {
                                rate: Kilohertz(1.638),
                                midpoint: Millivolts(-30.0),
                                scale: Millivolts(9.0),
                            },
                            beta: Rate::ExpLinear {
                                rate: Kilohertz(1.116),
                                midpoint: Millivolts(-30.0),
                                scale: Millivolts(-9.0),
                            },
                            steady: None,
                        },
                    },
                    Gate {
                        power: 1,
                        kinetics: Kinetics::Rates {
                            alpha: Rate::ExpLinear {
                                rate: Kilohertz(0.12),
                                midpoint: Millivolts(-45.0),
                                scale: Millivolts(5.0),
                            },
                            beta: Rate::ExpLinear {
                                rate: Kilohertz(0.0455),
                                midpoint: Millivolts(-70.0),
                                scale: Millivolts(-5.0),
                            },
                            steady: Some(Rate::Sigmoid {
                                rate: 1.0,
                                midpoint: Millivolts(-60.0),
                                scale: Millivolts(-6.2),
                            }),
                        },
                    },
                ],
                reference_celsius: CORTICAL_REFERENCE,
                q10: CORTICAL_Q10,
            },
            ChannelId::CorticalPotassium => Channel {
                gates: vec![Gate {
                    power: 1,
                    kinetics: Kinetics::Rates {
                        alpha: Rate::ExpLinear {
                            rate: Kilohertz(0.18),
                            midpoint: Millivolts(25.0),
                            scale: Millivolts(9.0),
                        },
                        beta: Rate::ExpLinear {
                            rate: Kilohertz(0.018),
                            midpoint: Millivolts(25.0),
                            scale: Millivolts(-9.0),
                        },
                        steady: None,
                    },
                }],
                reference_celsius: CORTICAL_REFERENCE,
                q10: CORTICAL_Q10,
            },
            ChannelId::ATypePotassium => Channel {
                gates: vec![
                    Gate {
                        power: 1,
                        kinetics: Kinetics::Formula(a_type_activation),
                    },
                    Gate {
                        power: 1,
                        kinetics: Kinetics::Formula(a_type_inactivation),
                    },
                ],
                reference_celsius: Celsius(24.0),
                q10: 1.0,
            },
            ChannelId::Hcn => Channel {
                gates: vec![Gate {
                    power: 1,
                    kinetics: Kinetics::Rates {
                        alpha: Rate::ExpLinear {
                            rate: Kilohertz(0.076517),
                            midpoint: Millivolts(-154.9),
                            scale: Millivolts(-11.9),
                        },
                        beta: Rate::Exp {
                            rate: Kilohertz(0.193),
                            midpoint: Millivolts(0.0),
                            scale: Millivolts(33.1),
                        },
                        steady: None,
                    },
                }],
                reference_celsius: Celsius(34.0),
                q10: 3.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rates(gate: &Gate, voltage: f64) -> (f64, f64) {
        let voltage = Millivolts(voltage);
        match gate.kinetics {
            Kinetics::Rates { alpha, beta, .. } => (alpha.at(voltage).0, beta.at(voltage).0),
            Kinetics::Formula(_) => panic!("a formula gate has no rates"),
        }
    }

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
                let (gate_alpha, gate_beta) = rates(gate, v);
                assert!((gate_alpha - alpha).abs() < 1e-12 * alpha.abs().max(1.0));
                assert!((gate_beta - beta).abs() < 1e-12 * beta.abs().max(1.0));
            }
        }
    }

    fn trap(v: f64, midpoint: f64, rate: f64, scale: f64) -> f64 {
        rate * (v - midpoint) / (1.0 - (-(v - midpoint) / scale).exp())
    }

    #[test]
    fn cortical_rates_match_mainen_and_sejnowski() {
        let sodium = ChannelId::CorticalSodium.channel();
        let potassium = ChannelId::CorticalPotassium.channel();
        for v in [-90.0, -65.0, -52.3, -20.0, 10.0, 35.0] {
            let shifted = v - 5.0;
            let expected = [
                (
                    trap(shifted, -35.0, 0.182, 9.0),
                    trap(-shifted, 35.0, 0.124, 9.0),
                    None,
                ),
                (
                    trap(shifted, -50.0, 0.024, 5.0),
                    trap(-shifted, 75.0, 0.0091, 5.0),
                    Some(1.0 / (1.0 + ((shifted + 65.0) / 6.2).exp())),
                ),
                (trap(v, 25.0, 0.02, 9.0), trap(-v, -25.0, 0.002, 9.0), None),
            ];
            let gates = sodium.gates.iter().chain(&potassium.gates);
            for (gate, (alpha, beta, steady)) in gates.zip(expected) {
                let (gate_alpha, gate_beta) = rates(gate, v);
                assert!((gate_alpha - alpha).abs() < 1e-12 * alpha.abs().max(1.0));
                assert!((gate_beta - beta).abs() < 1e-12 * beta.abs().max(1.0));
                let steady = steady.unwrap_or(alpha / (alpha + beta));
                assert!((gate.steady_state(Millivolts(v), Celsius(37.0)) - steady).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn hcn_rates_match_kole_hallermann_and_stuart() {
        let channel = ChannelId::Hcn.channel();
        for v in [-120.0f64, -90.0, -70.0, -45.0, 0.0, 30.0] {
            let alpha = 0.00643 * (v + 154.9) / (((v + 154.9) / 11.9).exp() - 1.0);
            let beta = 0.193 * (v / 33.1).exp();
            let (gate_alpha, gate_beta) = rates(&channel.gates[0], v);
            assert!((gate_alpha - alpha).abs() < 1e-6 * alpha.abs().max(1e-3));
            assert!((gate_beta - beta).abs() < 1e-12 * beta.abs().max(1.0));
        }
    }

    #[test]
    fn the_a_type_current_opens_fast_and_inactivates_slowly_around_its_midpoints() {
        let channel = ChannelId::ATypePotassium.channel();
        let [activation, inactivation] = &channel.gates[..] else {
            panic!("two gates")
        };
        for celsius in [Celsius(24.0), Celsius(37.0)] {
            let half = |gate: &Gate, voltage| gate.steady_state(Millivolts(voltage), celsius) - 0.5;
            assert!(half(activation, 11.0).abs() < 1e-12);
            assert!(half(inactivation, -56.0).abs() < 1e-12);
        }
        let time_constant = |gate: &Gate, voltage: f64, celsius: f64| {
            (1.0 / gate.rate(Millivolts(voltage), Celsius(celsius), 1.0)).0
        };
        assert!((time_constant(inactivation, -70.0, 37.0) - 2.0).abs() < 1e-12);
        assert!((time_constant(inactivation, 0.0, 37.0) - 13.0).abs() < 1e-9);
        assert_eq!(
            time_constant(inactivation, 0.0, 24.0),
            time_constant(inactivation, 0.0, 37.0)
        );
        let (cool, warm) = (
            time_constant(activation, -30.0, 24.0),
            time_constant(activation, -30.0, 34.0),
        );
        assert!(
            (4.5..5.0).contains(&(cool / warm)),
            "{cool} against {warm} ms"
        );
        assert!(time_constant(activation, 40.0, 37.0) >= 0.1);
    }

    #[test]
    fn exp_linear_rate_is_smooth_at_its_midpoint() {
        let rate = Rate::ExpLinear {
            rate: Kilohertz(1.0),
            midpoint: Millivolts(-40.0),
            scale: Millivolts(10.0),
        };
        let at = |voltage| rate.at(Millivolts(voltage)).0;
        assert!((at(-40.0) - 1.0).abs() < 1e-9);
        assert!((at(-40.0 + 1e-4) - at(-40.0 - 1e-4)).abs() < 1e-4);
    }
}
