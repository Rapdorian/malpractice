use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Rate {
    hz: f32,
}

impl Default for Rate {
    fn default() -> Self {
        Self { hz: 60.0 }
    }
}

impl Rate {
    pub fn from_hertz(hz: f32) -> Self {
        Self { hz }
    }

    pub fn from_delay(dt: f32) -> Self {
        Self { hz: 1.0 / dt }
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs(1).div_f32(self.hz)
    }
}
