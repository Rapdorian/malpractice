use crate::bcn::bc4::Bc4Block;
use crate::bcn::bc5::Bc5Block;
use crate::bcn::bc7::Bc7Mode3Block;
use std::mem;

pub mod dds;
pub mod bcn {
    pub mod bc4;
    pub mod bc5;
    pub mod bc7;

    mod util;
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ImageFormat {
    Luma8,
    LumaAlpha8,
    #[default]
    Rgb8,
    Rgba8,
    Luma8_Bc4,
    LumaAlpha8_Bc5,
    Rg8_Bc5,
    //Luma16_Bc5,
    //Rgba8_Bc7,
    Rgb8_Bc7,
    //RgbF_Bc6,
    //RgbUF_Bc6,
}

impl ImageFormat {
    pub fn bits_per_pixel(&self) -> u32 {
        match self {
            ImageFormat::Luma8 => 8,
            ImageFormat::LumaAlpha8 => 8 * 2,
            ImageFormat::Rgb8 => 8 * 3,
            ImageFormat::Rgba8 => 8 * 4,
            _ => 0,
        }
    }

    pub fn block_size(&self) -> Option<u32> {
        match self {
            ImageFormat::Luma8_Bc4 => Some(mem::size_of::<Bc4Block>() as u32),
            ImageFormat::LumaAlpha8_Bc5 | ImageFormat::Rg8_Bc5 => {
                Some(mem::size_of::<Bc5Block>() as u32)
            }
            ImageFormat::Rgb8_Bc7 => Some(mem::size_of::<Bc7Mode3Block>() as u32),
            _ => None,
        }
    }

    pub fn is_compressed(&self) -> bool {
        match self {
            ImageFormat::LumaAlpha8_Bc5
            | ImageFormat::Luma8_Bc4
            | ImageFormat::Rg8_Bc5
            | ImageFormat::Rgb8_Bc7 => true,
            _ => false,
        }
    }
}

// TODO: Start on texture converter.
//      TODO: Should be able to open simple rgb(a) image and generate uncompressed dds texture
//      TODO: Limit texture channels
//      TODO: Generate and store mipmaps
//      TODO: Store cubemaps
// TODO: BC4 compressed texture generation
// TODO: BC5 compressed texture generation
// TODO: BC6h compressed texture generation
// TODO: BC7 compressed texture generation
