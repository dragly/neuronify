use crate::channel::ChannelId;

const MIN_VOLTAGE: f64 = -100.0;
const MAX_VOLTAGE: f64 = 100.0;
const STEP: f64 = 0.1;

#[derive(Clone, Copy, Default)]
pub(crate) struct Lookup {
    index: usize,
    fraction: f64,
}

impl Lookup {
    pub(crate) fn new(voltage: f64) -> Self {
        let last = ((MAX_VOLTAGE - MIN_VOLTAGE) / STEP) as usize;
        let x = ((voltage - MIN_VOLTAGE) / STEP).clamp(0.0, last as f64 - 1e-9);
        let index = x as usize;
        Lookup {
            index,
            fraction: x - index as f64,
        }
    }
}

pub(crate) struct ChannelTable {
    powers: Vec<i32>,
    values: Vec<f64>,
}

impl ChannelTable {
    pub(crate) fn advance(&self, states: &mut [f64], at: &Lookup) -> f64 {
        let stride = 2 * self.powers.len();
        let rows = &self.values[at.index * stride..(at.index + 2) * stride];
        let (row, next) = rows.split_at(stride);
        let mut open = 1.0;
        for (gate, state) in states.iter_mut().enumerate() {
            let steady = row[2 * gate] + at.fraction * (next[2 * gate] - row[2 * gate]);
            let decay = row[2 * gate + 1] + at.fraction * (next[2 * gate + 1] - row[2 * gate + 1]);
            *state = steady + (*state - steady) * decay;
            open *= power(*state, self.powers[gate]);
        }
        open
    }
}

fn power(state: f64, power: i32) -> f64 {
    match power {
        1 => state,
        2 => state * state,
        3 => state * state * state,
        4 => {
            let squared = state * state;
            squared * squared
        }
        power => state.powi(power),
    }
}

pub struct Tables {
    dt: f64,
    celsius: f64,
    channels: Vec<ChannelTable>,
}

impl Tables {
    pub fn new(dt: f64, celsius: f64) -> Self {
        let points = ((MAX_VOLTAGE - MIN_VOLTAGE) / STEP) as usize + 1;
        let channels = ChannelId::ALL
            .iter()
            .map(|id| {
                let channel = id.channel();
                let factor = channel.rate_factor(celsius);
                let mut values = Vec::with_capacity(points * 2 * channel.gates.len());
                for i in 0..points {
                    let voltage = MIN_VOLTAGE + i as f64 * STEP;
                    for gate in &channel.gates {
                        values.push(gate.steady_state(voltage, celsius));
                        values.push((-dt * gate.rate(voltage, celsius, factor)).exp());
                    }
                }
                ChannelTable {
                    powers: channel.gates.iter().map(|gate| gate.power).collect(),
                    values,
                }
            })
            .collect();
        Tables {
            dt,
            celsius,
            channels,
        }
    }

    pub fn dt(&self) -> f64 {
        self.dt
    }

    pub fn celsius(&self) -> f64 {
        self.celsius
    }

    pub(crate) fn channel(&self, channel: ChannelId) -> &ChannelTable {
        &self.channels[channel.index()]
    }
}
