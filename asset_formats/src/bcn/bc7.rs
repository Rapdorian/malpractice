use crate::bcn::util;
use bilge::prelude::*;
use bytemuck::{Pod, Zeroable};
use image::imageops::FilterType;
use image::{DynamicImage, EncodableLayout};
use log::info;
use std::io::Read;

mod palette;
mod partitions;
pub use palette::*;
pub use partitions::*;

/// TODO:
/// We will want to eventually generalize block mode's well enough that we can just pass in 16
/// colors to a block and it will generate its self and be able to query it's average error
/// This way we can just create a block for each mode and see which one is best.
///
/// We will probably also want to generalize the partitions for the same reason. a partition will
/// additionally need to know the palette size to score itself. The partition is also what will be
/// generating the indices (but potentially not encoding them)
#[bitsize(128)]
#[derive(Copy, Clone, Pod, Zeroable, DebugBits)]
#[repr(C)]
pub struct Bc7Mode3Block {
    id: u4,
    partition: u6,
    red: [u7; 4],
    green: [u7; 4],
    blue: [u7; 4],
    p: [u1; 4],
    idx: u30,
}

pub fn encode(img: &DynamicImage) -> Vec<Bc7Mode3Block> {
    let w = (img.width() / 4) * 4;
    let h = (img.height() / 4) * 4;
    let img = img.resize_exact(w, h, FilterType::Nearest);
    let mut blocks: Vec<Bc7Mode3Block> = vec![];
    for y in (0..h).step_by(4) {
        for x in (0..w).step_by((4)) {
            // grab cutout of this block
            let block = img.crop_imm(x, y, 4, 4).to_rgb8();
            let block = block.as_bytes();

            let pixels: Vec<Color> = block
                .chunks(3)
                .map(|b| Color::new(b[0] as f32, b[1] as f32, b[2] as f32, 255.) / 255.)
                .collect();

            let mut partition = 0;
            let mut pal = gen_partition_palette(&PARTITIONS[partition], &pixels, 4);
            let mut map = map_partition(&PARTITIONS[partition], &pixels, &pal);
            for i in 1..PARTITIONS.len() {
                let new_pal = gen_partition_palette(&PARTITIONS[i], &pixels, 4);
                let new_map = map_partition(&PARTITIONS[i], &pixels, &new_pal);
                if new_map.1 < map.1 {
                    partition = i;
                    pal = new_pal;
                    map = new_map;
                }
            }
            let a_pal = pal[0].raw();
            let b_pal = pal[1].raw();

            let mut indices = vec![];
            // map each color
            for y in 0..4 {
                for p in 0..2 {
                    for x in p * 2..(p * 2) + 2 {
                        let idx = ((y * 4) + x) * 3;
                        let col = [block[idx], block[idx + 1], block[idx + 2]];
                        let pal = if p == 0 { a_pal } else { b_pal };
                        let mapped = index(col, pal.0, pal.1);
                        indices.push(mapped);
                    }
                }
            }

            // now we have a full array of indices
            // we need to try to map them to a 30 bit area
            let mut data = 0u32;
            for (i, idx) in indices.iter().enumerate() {
                let idx = *idx as u32 & 0x3; // only first 2 bits
                let idx = idx << (i * 2);
                data = data | idx;
            }
            blocks.push(Bc7Mode3Block::new(
                u4::new(0b1000),
                u6::extract_u8(partition as u8, 0),
                [
                    u7::extract_u8(a_pal.0[0], 1),
                    u7::extract_u8(a_pal.1[0], 1),
                    u7::extract_u8(b_pal.0[0], 1),
                    u7::extract_u8(b_pal.1[0], 1),
                ],
                [
                    u7::extract_u8(a_pal.0[1], 1),
                    u7::extract_u8(a_pal.1[1], 1),
                    u7::extract_u8(b_pal.0[1], 1),
                    u7::extract_u8(b_pal.1[1], 1),
                ],
                [
                    u7::extract_u8(a_pal.0[2], 1),
                    u7::extract_u8(a_pal.1[2], 1),
                    u7::extract_u8(b_pal.0[2], 1),
                    u7::extract_u8(b_pal.1[2], 1),
                ],
                [u1::new(0); 4],
                //u30::new(0x1D834000),
                u30::extract_u32(data, 0),
            ))
        }
    }
    blocks
}

// thoughts on actually encoding this shit.
// we'll need to generate
fn gen_pal(colors: &[u8]) -> ([u8; 3], [u8; 3]) {
    /*
    // generate with min/max
    let mut min = [255u8;3];
    let mut max = [0u8;3];
    for pixel in colors.chunks(3) {
        for i in 0..3 {
            if pixel[i] < min[i] {
                min[i] = pixel[i];
            }
            if pixel[i] > max[i] {
                max[i] = pixel[i];
            }
        }
    }

    let mut acc =  palette_error(colors, min, max);
    let mut min2 = min.clone();
    let mut max2 = max.clone();
    let mut acc2 = acc;
    // TODO: Smarter optimization algorithm
    for c in 0..3 {
        for i in -200..200 {
            for j in -200..200 {
                let min3 = (min[c] as i16 + i) as u8;
                let max3 = (max[c] as i16 + j) as u8;
                let mut min = min2.clone();
                min[c] = min3;
                let mut max = max2.clone();
                max[c] = max3;
                let acc =  palette_error(colors, min, max);
                if acc < acc2 {
                    min2 = min;
                    max2 = max;
                    acc2 = acc;
                }
            }
        }
    }

    // check accuracy
    info!("Pallete average error: {} reduced from {}", acc2, acc);*/

    // maybe try out fancy new linreg algorithm
    let (min, max) = util::generate_palette3d(&colors);
    //let lin_err = palette_error(colors, min, max);
    //info!("Linear regression average error: {} change: {},{}", lin_err, lin_err - acc, lin_err-acc2);

    (min, max)
}

fn palette_error(colors: &[u8], a: [u8; 3], b: [u8; 3]) -> i16 {
    let palette = palette(a, b);
    let mut error = 0;
    for color in colors.chunks(3) {
        let color = [color[0], color[1], color[2]];
        let pal = palette[index(color, a, b) as usize];
        error += cmp_color(color, pal);
    }
    error / 16
}

fn palette(a: [u8; 3], b: [u8; 3]) -> [[u8; 3]; 4] {
    let mut palette = [[0u8; 3]; 4];
    for i in 0..4 {
        for c in 0..3 {
            palette[i][c] = pol(a[c], b[c], i as u8, 5);
        }
    }
    palette
}

fn index(color: [u8; 3], a: [u8; 3], b: [u8; 3]) -> u8 {
    let palette = palette(a, b);
    let mut pal = 0;
    for (i, p) in palette.iter().enumerate() {
        if cmp_color(*p, color) < cmp_color(palette[pal as usize], color) {
            pal = i as u8;
        }
    }
    pal
}

fn cmp_color(a: [u8; 3], b: [u8; 3]) -> i16 {
    // convert to signed values
    let a = [a[0] as f32, a[1] as f32, a[2] as f32];
    let b = [b[0] as f32, b[1] as f32, b[2] as f32];

    let diff = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let m = (diff[0] * diff[0]) + (diff[1] * diff[1]) + (diff[2] * diff[2]);
    let m = (m as f32).sqrt() as i16;
    m
}

fn pol(a: u8, b: u8, i: u8, max: u8) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let i = i as u16;
    let max = max as u16;
    (((max - i) * a + i * b) / max) as u8
}
