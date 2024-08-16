
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollingWindow<T> {
    data: Vec<T>,
    max_capacity: usize,
}

impl<T> RollingWindow<T> {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() == self.max_capacity {
            self.data.remove(0);
        }
        self.data.push(value);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageMeasurement {
    pub voltage: f64,
    pub time: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoltageSeries {
    pub measurements: RollingWindow<VoltageMeasurement>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Voltmeter {}
