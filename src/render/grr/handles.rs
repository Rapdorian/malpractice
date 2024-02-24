use crate::render::GpuResource;
use glam::{Mat4, Vec2, Vec3};
use grr::{
    BaseFormat, Buffer, Device, Extent, Format, FormatLayout, Image, ImageType, ImageView,
    MemoryLayout, Offset, SubresourceLayers,
};
use mint::Vector2;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct BufferHandle {
    buffer: Buffer,
    grr: Rc<Device>,
}

impl BufferHandle {
    pub fn new(device: Rc<Device>, buffer: Buffer) -> Self {
        Self {
            buffer,
            grr: device,
        }
    }

    pub fn buffer(&self) -> Buffer {
        self.buffer
    }
}

impl GpuResource for BufferHandle {
    fn label<S: AsRef<str>>(&mut self, s: S) {
        unsafe {
            self.grr.object_name(self.buffer, s.as_ref());
        }
    }
}

#[derive(Clone)]
pub struct SpriteHandle {
    img: Image,
    view: ImageView,
    grr: Rc<Device>,
    transform: Mat4,
    pub(super) uv_min: Vec2,
    pub(super) uv_extent: Vec2,
    pub(super) uv_scroll: Vec2,
    width: f32,
    height: f32,
    sub_dim: Vec2,
}

impl SpriteHandle {
    pub fn new(width: u32, height: u32, data: &[u8], grr: Rc<Device>) -> Self {
        unsafe {
            let (img, view) = grr
                .create_image_and_view(
                    ImageType::D2 {
                        width,
                        height,
                        layers: 1,
                        samples: 1,
                    },
                    Format::R8G8B8A8_SRGB,
                    1,
                )
                .unwrap();

            // copy in data
            grr.copy_host_to_image(
                &data,
                img,
                grr::HostImageCopy {
                    host_layout: MemoryLayout {
                        base_format: BaseFormat::RGBA,
                        format_layout: FormatLayout::U8,
                        row_length: width,
                        image_height: height,
                        alignment: 4,
                    },
                    image_subresource: SubresourceLayers {
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: Offset { x: 0, y: 0, z: 0 },
                    image_extent: Extent {
                        width,
                        height,
                        depth: 1,
                    },
                },
            );

            Self {
                img,
                view,
                grr,
                transform: Mat4::from_scale(Vec3::new(
                    width as f32 / 100.0,
                    height as f32 / 100.0,
                    1.0,
                )),
                uv_min: Vec2::ZERO,
                uv_extent: Vec2::ONE,
                uv_scroll: Vec2::ZERO,
                width: width as f32,
                height: height as f32,
                sub_dim: Vec2::new(width as f32, height as f32),
            }
        }
    }

    /// Returns a sprite that uses a subset of this sprite
    ///
    /// `pos` and `extent` are measured in pixels
    pub fn sub_sprite(
        &self,
        pos: impl Into<Vector2<f32>>,
        extent: impl Into<Vector2<f32>>,
    ) -> Self {
        let dim = extent.into();
        // scale from pixel coord to 0..1
        let mut pos = Vec2::from(pos.into()) / Vec2::new(self.width, self.height);
        let extent = Vec2::from(dim) / Vec2::new(self.width, self.height);

        pos += self.uv_min;

        Self {
            img: self.img.clone(),
            view: self.view.clone(),
            grr: self.grr.clone(),
            transform: Mat4::from_scale(Vec3::new(dim.x / 2.0, dim.y / 2.0, 1.0)),
            uv_min: pos,
            uv_extent: extent,
            uv_scroll: Vec2::ZERO,
            width: self.width,
            height: self.height,
            sub_dim: Vec2::new(dim.x, dim.y),
        }
    }

    pub fn recenter(mut self, center: impl Into<Vector2<f32>>) -> Self {
        let center = Vec2::from(center.into()) * 1.0 / 10.0;
        self.transform =
            self.transform * Mat4::from_translation(Vec3::new(center.x, center.y, 0.0));
        self
    }

    pub fn set_scroll(&mut self, scroll: impl Into<Vector2<f32>>) {
        let scroll = scroll.into();
        self.uv_scroll = Vec2::new(scroll.x, scroll.y);
    }

    pub(super) fn view(&self) -> ImageView {
        self.view
    }
    pub(super) fn transform(&self) -> Mat4 {
        self.transform
    }
}

impl GpuResource for SpriteHandle {
    fn label<S: AsRef<str>>(&mut self, s: S) {
        unsafe {
            self.grr.object_name(self.img, s.as_ref());
            self.grr
                .object_name(self.view, &format!("{}-view", s.as_ref()));
        }
    }
}
