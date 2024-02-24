use crate::bcn::bc4::{gen_block, Bc4Block};
use bytemuck::{Pod, Zeroable};
use image::imageops::FilterType;
use image::DynamicImage;
use std::io::Read;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Bc5Block {
    l: Bc4Block,
    a: Bc4Block,
}

pub fn encode_grayscale(img: &DynamicImage) -> Vec<Bc5Block> {
    let w = (img.width() / 4) * 4;
    let h = (img.height() / 4) * 4;
    let img = img.resize_exact(w, h, FilterType::Nearest);
    let mut blocks = vec![];
    for y in (0..h).step_by(4) {
        for x in (0..w).step_by((4)) {
            // grab cutout of this block
            let block = img.crop_imm(x, y, 4, 4).to_luma_alpha8();
            let mut gray_bytes = [0u8; 16];
            let mut alpha_bytes = [0u8; 16];
            for (i, pixel) in block.chunks(2).enumerate() {
                gray_bytes[i] = pixel[0];
                alpha_bytes[i] = pixel[1];
            }

            blocks.push(Bc5Block {
                l: gen_block(gray_bytes),
                a: gen_block(alpha_bytes),
            });
        }
    }
    blocks
}

pub fn encode_color(img: &DynamicImage) -> Vec<Bc5Block> {
    let w = (img.width() / 4) * 4;
    let h = (img.height() / 4) * 4;
    let img = img.resize_exact(w, h, FilterType::Nearest);
    let mut blocks = vec![];
    for y in (0..h).step_by(4) {
        for x in (0..w).step_by((4)) {
            // grab cutout of this block
            let block = img.crop_imm(x, y, 4, 4).to_rgb8();
            let mut red = [0u8; 16];
            let mut green = [0u8; 16];
            for (i, pixel) in block.chunks(3).enumerate() {
                red[i] = pixel[0];
                green[i] = pixel[1];
            }

            blocks.push(Bc5Block {
                l: gen_block(red),
                a: gen_block(green),
            });
        }
    }
    blocks
}
