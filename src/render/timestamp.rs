use std::ops;

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct TimeStamp(f64);

impl TimeStamp {
    pub fn new(time: f64) -> Self {
        Self(time)
    }

    /// Move timestamp forward by `dt`
    pub fn tick(&mut self, dt: f32) {
        self.0 += dt as f64;
    }
}

impl ops::Deref for TimeStamp {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
