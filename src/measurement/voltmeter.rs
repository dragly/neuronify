#[derive(Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub struct VoltageMeasurement {
    pub voltage: f64,
    pub time: f64,
}

#[derive(Clone, Debug)]
pub struct Voltmeter {
    pub measurements: RollingWindow<VoltageMeasurement>,
}
