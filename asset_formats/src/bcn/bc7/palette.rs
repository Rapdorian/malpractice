use crate::bcn::util;
use glam::Vec4;
use num_traits::{abs_sub, CheckedSub, FromPrimitive, Num, Signed, ToPrimitive, Unsigned};
use std::ops::{Add, Div, Index, Sub};

pub type Color = Vec4;

/// A linearly interpolated color palette
pub struct Palette {
    start: Color,
    end: Color,
    step: Color,
}

impl Palette {
    pub fn new(start: Color, end: Color, steps: usize) -> Self {
        Self {
            start,
            end,
            step: (end - start) / steps as f32,
        }
    }

    pub fn generate(colors: &[Color], steps: usize) -> Self {
        // we already have code written for generating a color palette from an rgb byte array so we'll
        // use that for now
        let bytes: Vec<u8> = colors
            .iter()
            .map(|c| [(c.x * 255.) as u8, (c.y * 255.) as u8, (c.z * 255.) as u8])
            .flatten()
            .collect();
        let (min, max) = util::generate_palette3d(&bytes);
        Self::new(
            Color::new(min[0] as f32, min[1] as f32, min[2] as f32, 255.) / 255.,
            Color::new(max[0] as f32, max[1] as f32, max[2] as f32, 255.) / 255.,
            steps,
        )
    }

    pub fn raw(&self) -> ([u8; 3], [u8; 3]) {
        let min = self.start * 255.;
        let max = self.end * 255.;
        (
            [min.x as u8, min.y as u8, min.z as u8],
            [max.x as u8, max.y as u8, max.z as u8],
        )
    }

    pub fn map(&self, val: Color) -> usize {
        // figure out where along line val is
        let distance = val - self.start;
        let step = self.step;
        (distance / step).length() as usize
    }

    pub fn get(&self, idx: usize) -> Color {
        self.start + (self.step * idx as f32)
    }

    pub fn error(&self, val: Color) -> f32 {
        let idx = self.map(val);
        let col = self.get(idx);
        (col - val).length()
    }
}
