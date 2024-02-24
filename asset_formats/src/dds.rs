//! DirectDraw Surface image format

use crate::dds::dx10::DxgiFormat::{DXGI_FORMAT_BC5_UNORM, DXGI_FORMAT_BC7_UNORM};
use crate::dds::dx10::{Dx10Header, DxgiFormat, ResourceDimension};
use crate::ImageFormat;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use clap::builder::styling::Color::Rgb;
use log::warn;

pub const DDS_MAGIC: u32 = 0x20534444;

#[derive(Debug)]
pub struct FullDdsHeader {
    pub magic: u32,
    pub header: DdsHeader,
    pub dx10_header: Option<Dx10Header>,
}

impl FullDdsHeader {
    pub fn new(width: u32, height: u32, depth: Option<u32>, format: ImageFormat) -> Self {
        FullDdsHeader {
            magic: DDS_MAGIC,
            header: DdsHeader::new(width, height, depth, format),
            dx10_header: if format.dx10() {
                Some(Dx10Header {
                    format: format.dxgi(),
                    dimension: ResourceDimension::Unknown,
                    flag: 0,
                    array_size: 0,
                    flags2: 0,
                })
            } else {
                None
            },
        }
    }
}

/// Base DDS Header
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct DdsHeader {
    /// Size of structure. This member must be set to 124.
    pub size: u32,
    /// Flags to indicate which members contain valid data.
    pub flags: DdsFlags,
    /// Surface height (in pixels).
    pub height: u32,
    /// Surface height (in pixels).
    pub width: u32,
    /// The pitch or number of bytes per scan line in an uncompressed texture; the total number of bytes in the top
    /// level texture for a compressed texture. For information about how to compute the pitch, see the DDS File Layout
    /// section of the [Programming Guide for DDS](https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide).
    pub pitch: u32,
    /// Depth of a volume texture (in pixels), otherwise unused.
    pub depth: u32,
    /// Number of mipmap levels, otherwise unused.
    pub mip_levels: u32,
    pub reserved: [u32; 11],
    pub spf: PixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
}

impl DdsHeader {
    pub fn new(mut width: u32, mut height: u32, depth: Option<u32>, format: ImageFormat) -> Self {
        let mut flags = DdsFlags::default();
        if depth.is_some() {
            flags = flags | DdsFlags::DEPTH;
        }

        if format.is_compressed() {
            width = (width / 4) * 4;
            height = (height / 4) * 4;
        }

        Self {
            size: 124,
            flags,
            height,
            width,
            pitch: format.pitch(width),
            depth: depth.unwrap_or(0),
            mip_levels: 0,
            reserved: [0; 11],
            spf: PixelFormat {
                size: 32,
                flags: format.dds_pixel_format_flags(),
                fourcc: format.fourcc(),
                rgb_bit_count: format.bits_per_pixel(),
                bit_mask: format.dds_bit_mask(),
            },
            caps: 0x1000,
            caps2: 0,
            caps3: 0,
            caps4: 0,
            reserved2: 0,
        }
    }
}
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct PixelFormat {
    /// Structure size; set to 32 (bytes).
    pub size: u32,
    pub flags: PixelFormatFlags,
    pub fourcc: u32,
    pub rgb_bit_count: u32,
    pub bit_mask: RgbaBitMask,
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct RgbaBitMask {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
}

bitflags! {
    #[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
    #[repr(C)]
    pub struct PixelFormatFlags: u32 {
        /// Texture contains alpha data; `RgbaBitMask.a` contains valid data.
        const ALPHA_PIXELS = 0x1;
        /// Used in some older DDS files for alpha channel only uncompressed data (dwRGBBitCount contains the alpha channel bitcount; dwABitMask contains valid data)
        const ALPHA = 0x2;
        /// Texture contains compressed RGB data; `PixelFormat.fourcc` contains valid data.
        const FOURCC = 0x4;
        /// Texture contains uncompressed RGB data; dwRGBBitCount and the RGB masks (dwRBitMask, dwGBitMask, dwBBitMask) contain valid data.
        const RGB = 0x40;
        /// Used in some older DDS files for YUV uncompressed data (dwRGBBitCount contains the YUV bit count; dwRBitMask contains the Y mask, dwGBitMask contains the U mask, dwBBitMask contains the V mask)
        const YUV = 0x200;
        /// Used in some older DDS files for single channel color uncompressed data (dwRGBBitCount contains the luminance channel bit count; dwRBitMask contains the channel mask). Can be combined with DDPF_ALPHAPIXELS for a two channel DDS file.
        const LUMINANCE = 0x20000;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Pod, Zeroable)]
    #[repr(C)]
    pub struct DdsFlags: u32 {
        /// Required in every .dds file
        const CAPS = 0x1;
        /// Required in every .dds file
        const HEIGHT = 0x2;
        /// Required in every .dds file
        const WIDTH = 0x4;
        /// Required when pitch is provided for an uncompressed texture.
        const PITCH = 0x8;
        /// Required in every .dds file
        const PIXEL_FORMAT = 0x1000;
        /// Required in a mipmapped texture.
        const MIPMAP_COUNT = 0x20000;
        /// Required when pitch is provided for a compressed texture.
        const LINEAR_SIZE = 0x80000;
        /// Required in a depth texture.
        const DEPTH = 0x800000;
    }
}

impl Default for crate::dds::DdsFlags {
    fn default() -> Self {
        Self::CAPS | Self::HEIGHT | Self::WIDTH | Self::PIXEL_FORMAT
    }
}

pub mod dx10 {
    use crate::ImageFormat;
    use bytemuck::{Pod, Zeroable};
    use log::warn;

    #[derive(Debug, Copy, Clone, Pod, Zeroable)]
    #[repr(C)]
    pub struct Dx10Header {
        pub format: DxgiFormat,
        pub dimension: ResourceDimension,
        pub flag: u32,

        /// The number of elements in the array.
        ///
        /// For a 2D texture that is also a cube-map texture, this number represents the number of cubes. This number is
        /// the same as the number in the NumCubes member of D3D10_TEXCUBE_ARRAY_SRV1 or D3D11_TEXCUBE_ARRAY_SRV). In
        /// this case, the DDS file contains arraySize*6 2D textures. For more information about this case, see the
        /// miscFlag description.
        ///
        /// For a 3D texture, you must set this number to 1.
        pub array_size: u32,
        pub flags2: u32,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone)]
    pub enum ResourceDimension {
        Unknown = 0,
        Buffer = 1,
        Texture1D = 2,
        Texture2D = 3,
        Texture3D = 4,
    }
    unsafe impl Zeroable for ResourceDimension {}
    unsafe impl Pod for ResourceDimension {}

    #[repr(u32)]
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    pub enum DxgiFormat {
        DXGI_FORMAT_UNKNOWN = 0,
        DXGI_FORMAT_R32G32B32A32_TYPELESS = 1,
        DXGI_FORMAT_R32G32B32A32_FLOAT = 2,
        DXGI_FORMAT_R32G32B32A32_UINT = 3,
        DXGI_FORMAT_R32G32B32A32_SINT = 4,
        DXGI_FORMAT_R32G32B32_TYPELESS = 5,
        DXGI_FORMAT_R32G32B32_FLOAT = 6,
        DXGI_FORMAT_R32G32B32_UINT = 7,
        DXGI_FORMAT_R32G32B32_SINT = 8,
        DXGI_FORMAT_R16G16B16A16_TYPELESS = 9,
        DXGI_FORMAT_R16G16B16A16_FLOAT = 10,
        DXGI_FORMAT_R16G16B16A16_UNORM = 11,
        DXGI_FORMAT_R16G16B16A16_UINT = 12,
        DXGI_FORMAT_R16G16B16A16_SNORM = 13,
        DXGI_FORMAT_R16G16B16A16_SINT = 14,
        DXGI_FORMAT_R32G32_TYPELESS = 15,
        DXGI_FORMAT_R32G32_FLOAT = 16,
        DXGI_FORMAT_R32G32_UINT = 17,
        DXGI_FORMAT_R32G32_SINT = 18,
        DXGI_FORMAT_R32G8X24_TYPELESS = 19,
        DXGI_FORMAT_D32_FLOAT_S8X24_UINT = 20,
        DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS = 21,
        DXGI_FORMAT_X32_TYPELESS_G8X24_UINT = 22,
        DXGI_FORMAT_R10G10B10A2_TYPELESS = 23,
        DXGI_FORMAT_R10G10B10A2_UNORM = 24,
        DXGI_FORMAT_R10G10B10A2_UINT = 25,
        DXGI_FORMAT_R11G11B10_FLOAT = 26,
        DXGI_FORMAT_R8G8B8A8_TYPELESS = 27,
        DXGI_FORMAT_R8G8B8A8_UNORM = 28,
        DXGI_FORMAT_R8G8B8A8_UNORM_SRGB = 29,
        DXGI_FORMAT_R8G8B8A8_UINT = 30,
        DXGI_FORMAT_R8G8B8A8_SNORM = 31,
        DXGI_FORMAT_R8G8B8A8_SINT = 32,
        DXGI_FORMAT_R16G16_TYPELESS = 33,
        DXGI_FORMAT_R16G16_FLOAT = 34,
        DXGI_FORMAT_R16G16_UNORM = 35,
        DXGI_FORMAT_R16G16_UINT = 36,
        DXGI_FORMAT_R16G16_SNORM = 37,
        DXGI_FORMAT_R16G16_SINT = 38,
        DXGI_FORMAT_R32_TYPELESS = 39,
        DXGI_FORMAT_D32_FLOAT = 40,
        DXGI_FORMAT_R32_FLOAT = 41,
        DXGI_FORMAT_R32_UINT = 42,
        DXGI_FORMAT_R32_SINT = 43,
        DXGI_FORMAT_R24G8_TYPELESS = 44,
        DXGI_FORMAT_D24_UNORM_S8_UINT = 45,
        DXGI_FORMAT_R24_UNORM_X8_TYPELESS = 46,
        DXGI_FORMAT_X24_TYPELESS_G8_UINT = 47,
        DXGI_FORMAT_R8G8_TYPELESS = 48,
        DXGI_FORMAT_R8G8_UNORM = 49,
        DXGI_FORMAT_R8G8_UINT = 50,
        DXGI_FORMAT_R8G8_SNORM = 51,
        DXGI_FORMAT_R8G8_SINT = 52,
        DXGI_FORMAT_R16_TYPELESS = 53,
        DXGI_FORMAT_R16_FLOAT = 54,
        DXGI_FORMAT_D16_UNORM = 55,
        DXGI_FORMAT_R16_UNORM = 56,
        DXGI_FORMAT_R16_UINT = 57,
        DXGI_FORMAT_R16_SNORM = 58,
        DXGI_FORMAT_R16_SINT = 59,
        DXGI_FORMAT_R8_TYPELESS = 60,
        DXGI_FORMAT_R8_UNORM = 61,
        DXGI_FORMAT_R8_UINT = 62,
        DXGI_FORMAT_R8_SNORM = 63,
        DXGI_FORMAT_R8_SINT = 64,
        DXGI_FORMAT_A8_UNORM = 65,
        DXGI_FORMAT_R1_UNORM = 66,
        DXGI_FORMAT_R9G9B9E5_SHAREDEXP = 67,
        DXGI_FORMAT_R8G8_B8G8_UNORM = 68,
        DXGI_FORMAT_G8R8_G8B8_UNORM = 69,
        DXGI_FORMAT_BC1_TYPELESS = 70,
        DXGI_FORMAT_BC1_UNORM = 71,
        DXGI_FORMAT_BC1_UNORM_SRGB = 72,
        DXGI_FORMAT_BC2_TYPELESS = 73,
        DXGI_FORMAT_BC2_UNORM = 74,
        DXGI_FORMAT_BC2_UNORM_SRGB = 75,
        DXGI_FORMAT_BC3_TYPELESS = 76,
        DXGI_FORMAT_BC3_UNORM = 77,
        DXGI_FORMAT_BC3_UNORM_SRGB = 78,
        DXGI_FORMAT_BC4_TYPELESS = 79,
        DXGI_FORMAT_BC4_UNORM = 80,
        DXGI_FORMAT_BC4_SNORM = 81,
        DXGI_FORMAT_BC5_TYPELESS = 82,
        DXGI_FORMAT_BC5_UNORM = 83,
        DXGI_FORMAT_BC5_SNORM = 84,
        DXGI_FORMAT_B5G6R5_UNORM = 85,
        DXGI_FORMAT_B5G5R5A1_UNORM = 86,
        DXGI_FORMAT_B8G8R8A8_UNORM = 87,
        DXGI_FORMAT_B8G8R8X8_UNORM = 88,
        DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM = 89,
        DXGI_FORMAT_B8G8R8A8_TYPELESS = 90,
        DXGI_FORMAT_B8G8R8A8_UNORM_SRGB = 91,
        DXGI_FORMAT_B8G8R8X8_TYPELESS = 92,
        DXGI_FORMAT_B8G8R8X8_UNORM_SRGB = 93,
        DXGI_FORMAT_BC6H_TYPELESS = 94,
        DXGI_FORMAT_BC6H_UF16 = 95,
        DXGI_FORMAT_BC6H_SF16 = 96,
        DXGI_FORMAT_BC7_TYPELESS = 97,
        DXGI_FORMAT_BC7_UNORM = 98,
        DXGI_FORMAT_BC7_UNORM_SRGB = 99,
        DXGI_FORMAT_AYUV = 100,
        DXGI_FORMAT_Y410 = 101,
        DXGI_FORMAT_Y416 = 102,
        DXGI_FORMAT_NV12 = 103,
        DXGI_FORMAT_P010 = 104,
        DXGI_FORMAT_P016 = 105,
        DXGI_FORMAT_420_OPAQUE = 106,
        DXGI_FORMAT_YUY2 = 107,
        DXGI_FORMAT_Y210 = 108,
        DXGI_FORMAT_Y216 = 109,
        DXGI_FORMAT_NV11 = 110,
        DXGI_FORMAT_AI44 = 111,
        DXGI_FORMAT_IA44 = 112,
        DXGI_FORMAT_P8 = 113,
        DXGI_FORMAT_A8P8 = 114,
        DXGI_FORMAT_B4G4R4A4_UNORM = 115,
        DXGI_FORMAT_P208 = 130,
        DXGI_FORMAT_V208 = 131,
        DXGI_FORMAT_V408 = 132,
        DXGI_FORMAT_SAMPLER_FEEDBACK_MIN_MIP_OPAQUE,
        DXGI_FORMAT_SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE,
        DXGI_FORMAT_FORCE_UINT = 0xffffffff,
    }

    unsafe impl Zeroable for DxgiFormat {}
    unsafe impl Pod for DxgiFormat {}
}

impl ImageFormat {
    pub fn pitch(&self, width: u32) -> u32 {
        if self.is_compressed() {
            1.max((width + 3) / 4) * self.block_size().unwrap()
        } else {
            (width * self.bits_per_pixel() + 7) / 8
        }
    }

    pub fn dds_pixel_format_flags(&self) -> PixelFormatFlags {
        match self {
            ImageFormat::Rgb8 | ImageFormat::Luma8 => PixelFormatFlags::RGB,
            ImageFormat::Rgba8 | ImageFormat::LumaAlpha8 => {
                PixelFormatFlags::RGB | PixelFormatFlags::ALPHA_PIXELS
            }
            _ => PixelFormatFlags::FOURCC,
        }
    }

    pub fn dds_bit_mask(&self) -> RgbaBitMask {
        match self {
            ImageFormat::Luma8 => RgbaBitMask {
                r: 0xFF,
                g: 0,
                b: 0,
                a: 0,
            },
            ImageFormat::LumaAlpha8 => RgbaBitMask {
                r: 0x00FF,
                g: 0,
                b: 0,
                a: 0xFF00,
            },
            ImageFormat::Rgb8 => RgbaBitMask {
                r: 0x0000FF,
                g: 0x00FF00,
                b: 0xFF0000,
                a: 0,
            },
            ImageFormat::Rgba8 => RgbaBitMask {
                r: 0x000000FF,
                g: 0x0000FF00,
                b: 0x00FF0000,
                a: 0xFF000000,
            },
            _ => RgbaBitMask {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
        }
    }

    pub fn dx10(&self) -> bool {
        match self {
            ImageFormat::Luma8_Bc4
            | ImageFormat::LumaAlpha8_Bc5
            | ImageFormat::Rg8_Bc5
            | ImageFormat::Rgb8_Bc7 => true,
            _ => false,
        }
    }

    pub fn dxgi(&self) -> DxgiFormat {
        match self {
            ImageFormat::Luma8_Bc4 => DxgiFormat::DXGI_FORMAT_BC4_UNORM,
            ImageFormat::LumaAlpha8_Bc5 => DxgiFormat::DXGI_FORMAT_BC5_UNORM,
            ImageFormat::Rg8_Bc5 => DxgiFormat::DXGI_FORMAT_BC5_UNORM,
            ImageFormat::Rgb8_Bc7 => DXGI_FORMAT_BC7_UNORM,
            _ => DxgiFormat::DXGI_FORMAT_UNKNOWN,
        }
    }

    pub fn fourcc(&self) -> u32 {
        match self {
            ImageFormat::Luma8_Bc4
            | ImageFormat::LumaAlpha8_Bc5
            | ImageFormat::Rg8_Bc5
            | ImageFormat::Rgb8_Bc7 => 0x30315844,
            _ => 0,
        }
    }
}
