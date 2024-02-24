use crate::bcn::util::avg_error;
use bytemuck::{Pod, Zeroable};
use image::imageops::FilterType;
use image::{DynamicImage, EncodableLayout};
use log::{info, warn};

/// Bc4 Compression block
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Bc4Block {
    a: u8,
    b: u8,
    data: [u8; 6],
}

pub fn encode(img: &DynamicImage) -> Vec<Bc4Block> {
    let w = (img.width() / 4) * 4;
    let h = (img.height() / 4) * 4;
    let img = img.resize_exact(w, h, FilterType::Nearest);
    let mut blocks = vec![];
    for y in (0..h).step_by(4) {
        for x in (0..w).step_by((4)) {
            // grab cutout of this block
            let block = img.crop_imm(x, y, 4, 4).to_luma8();
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(block.as_bytes());
            let block = gen_block(bytes);
            blocks.push(block);
        }
    }
    blocks
}

/// Generate block
pub(super) fn gen_block(block: [u8; 16]) -> Bc4Block {
    // first get a palette
    let end_points = palette(&block);
    let pal = interpolate(end_points.0, end_points.1);
    let mut data = 0u64;
    for (px, c) in block.iter().enumerate() {
        let mut best_fit = (0, pal[0]);
        for (i, pal) in pal.iter().enumerate() {
            if pal.abs_diff(*c) < best_fit.1.abs_diff(*c) {
                best_fit = (i, *pal);
            }
        }
        data = data | ((best_fit.0 & 0x7) << (px * 3)) as u64;
    }
    let mut bytes = [0u8; 6];
    bytes.copy_from_slice(&data.to_ne_bytes()[0..6]);

    Bc4Block {
        a: end_points.0,
        b: end_points.1,
        data: bytes,
    }
}

/// Get Optimal palette for a set of colors
fn palette(colors: &[u8]) -> (u8, u8) {
    let pal = super::util::generate_palette(colors);
    (pal.1 as u8, pal.0 as u8)
}

/// Generate color palette for a block based on start and end colors
fn interpolate(a: u8, b: u8) -> [u8; 8] {
    fn pol(a: u8, b: u8, i: u8, max: u8) -> u8 {
        let a = a as u16;
        let b = b as u16;
        let i = i as u16;
        let max = max as u16;
        (((max - i) * a + i * b) / max) as u8
    }

    if a > b {
        [
            a,
            b,
            pol(a, b, 1, 7),
            pol(a, b, 2, 7),
            pol(a, b, 3, 7),
            pol(a, b, 4, 7),
            pol(a, b, 5, 7),
            pol(a, b, 6, 7),
        ]
    } else {
        [
            a,
            b,
            pol(a, b, 1, 5),
            pol(a, b, 2, 5),
            pol(a, b, 3, 5),
            pol(a, b, 4, 5),
            0,
            255,
        ]
    }
}
