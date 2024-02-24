//! Engine configuration
use crate::utils::Rate;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub framerate: Rate,
    pub physics_step: Rate,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            framerate: Rate::from_hertz(60.0),
            physics_step: Rate::from_hertz(60.0),
        }
    }
}
