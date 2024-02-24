use std::fmt::{write, Debug, Formatter};
use std::ops::Deref;

/// Typed byte buffer representing a texture
#[repr(C)]
pub struct Texture {
    pub width: u16,
    pub height: u16,
    pub format: TextureFormat,
    buffer: [u8],
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{{ width: {}, height: {} }}",
            self.format, self.width, self.height
        )
    }
}

impl AsRef<Texture> for [u8] {
    fn as_ref(&self) -> &Texture {
        const HEADER_LEN: usize = 1 + 2 + 2;
        let ptr = &self[HEADER_LEN..] as *const [u8];
        // SAFETY: We effectively added `HEADER_LEN` in the line above so ptr-HEADER_LEN is valid
        let ptr = unsafe { ptr.byte_sub(HEADER_LEN) } as *const Texture;
        unsafe { &*ptr }
    }
}

impl Deref for Texture {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

/// Supported texture formats
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    // unsigned formats
    /// one channel image with 8bpc
    Gray,

    /// Two channel image with 8bpc
    Rg,

    /// three channel image with 8bpc
    Rgb,

    /// four channel image with 8bpc
    Rgba,

    // float formats
    /// one channel image of 16 bit floats
    GrayF16,
    /// two channel image of 16 bit floats
    RgF16,
    /// three channel image of 16 bit floats
    RgbF16,
    /// four channel image of 16 bit floats
    RgbaF16,
}
