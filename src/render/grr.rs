//! implementation of rendering backend using grr

mod context;
mod error;
mod handles;
mod shaders;

use crate::render::{
    Camera, GpuResource, Light, Material, Renderer, Texture, TextureFormat, Vertex,
};
use bytemuck::{Pod, Zeroable};
pub use context::*;
pub use error::GrrError;
use glam::{Mat4, Vec2, Vec2Swizzles, Vec4};
use grr::{
    BaseFormat, BlendChannel, BlendFactor, BlendOp, Buffer, BufferRange, ColorBlend,
    ColorBlendAttachment, DebugSource, Device, Extent, Format, FormatLayout, ImageType, InputRate,
    MemoryFlags, MemoryLayout, Offset, Pipeline, Primitive, SubresourceLayers, VertexArray,
    VertexBufferView, VertexFormat,
};
pub use handles::*;
use mint::ColumnMatrix4;
use std::mem;
use tracing::{info, instrument, warn};

// TODO: Bitmap sprite rendering
// TODO: SDF vector rendering
// TODO: Font rendering with layout?

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
struct SpriteUniform {
    transform: Mat4,
    uv: Vec4,
    uv_extent: Vec4,
    uv_scroll: Vec4,
}

impl Renderer<f32> for GrrRender {
    type MeshHandle = Self::BufferHandle;
    type BufferHandle = BufferHandle;
    type SpriteHandle = SpriteHandle;
    type Surface = ();

    fn upload_mesh(&mut self, verts: &[Vertex<f32>], indices: &[u16]) -> Self::MeshHandle {
        let bytes = bytemuck::cast_slice(verts);
        self.upload_buffer(bytes)
    }

    fn upload_buffer(&mut self, data: &[u8]) -> Self::BufferHandle {
        // TODO: Optimize flags
        BufferHandle::new(
            self.grr.clone(),
            unsafe { self.grr.create_buffer_from_host(data, MemoryFlags::all()) }.unwrap(),
        )
    }

    /// TODO: Support sub-sprites for sprite atlas
    #[instrument(skip(self))]
    fn upload_sprite(&mut self, data: &Texture) -> Self::SpriteHandle {
        if data.format != TextureFormat::Rgba {
            warn!(
                "Texture is wrong format found {:?} expected {:?}",
                data.format,
                TextureFormat::Rgba
            );
        }

        SpriteHandle::new(
            data.width as u32,
            data.height as u32,
            data,
            self.grr.clone(),
        )
    }

    fn draw_mesh(&mut self, mesh: &Self::MeshHandle, material: Material<Self::BufferHandle>) {
        self.draw_list.push(mesh.clone());
    }

    fn draw_sprite(&mut self, sprite: &Self::SpriteHandle, transform: ColumnMatrix4<f32>) {
        let uniform = SpriteUniform {
            transform: glam::Mat4::from(transform) * sprite.transform(),
            uv: sprite.uv_min.xyxy(),
            uv_extent: sprite.uv_extent.xyxy(),
            uv_scroll: sprite.uv_scroll.xyxy(),
        };

        let bytes = bytemuck::bytes_of(&uniform);
        let buffer = self.upload_buffer(bytes);
        self.sprite_list.push((sprite.clone(), buffer));
    }

    fn draw_light(&mut self, light: Light<f32>) {
        todo!()
    }

    fn clear(&mut self, surface: &mut Self::Surface) {
        // TODO: Support additional surfaces
        unsafe {
            self.grr.clear_attachment(
                grr::Framebuffer::DEFAULT,
                grr::ClearAttachment::ColorFloat(0, [0.0, 0.0, 0.0, 1.0]),
            );
        }
    }

    /// I am not sure exactly how I want to structure this part of the API.
    /// It will largely come down to how much control I want to leave the user wrt how a frame is architected.
    /// At the moment I don't really support custom shaders so there is no point in providing too much control to the user.
    ///
    /// At the moment though I'll still be calling render multiple times per frame because 2d assets will not be drawn
    /// with the same camera settings.
    ///
    /// It is also plausible that I'll want to call this an render to a texture for use in something else. example: mirrors
    ///
    /// There is also some consideration to be had wrt 3d lighting pipeline. Should we be doing forward/deferred/forward+ rendering.
    /// If we do anything besides a boring forward renderer we'll have to be clever to not do more than we need in this render method
    ///
    /// If we did a forward+ renderer we might want to find a way to cache the light lists be cause it is likely we'll be
    /// rendering any given frame multiple times. We might even want to depart from clearing the scene everytime we call draw()
    /// and specify a method for clearing the scene.
    fn render(&mut self, camera: Camera<f32>, surface: &mut Self::Surface) {
        unsafe {
            self.grr
                .begin_debug_marker(DebugSource::ThirdParty, 0, "Rendering Scene");

            // create camera buffer
            let camera = glam::Mat4::from(camera.transform);
            let camera = bytemuck::bytes_of(&camera);
            let camera = self.upload_buffer(camera);

            // Draw sprites
            self.grr.bind_pipeline(self.sprite_pipeline);
            self.grr.bind_vertex_array(self.sprite_vertex_array);
            self.grr.bind_color_blend_state(&ColorBlend {
                attachments: vec![ColorBlendAttachment {
                    blend_enable: true,
                    color: BlendChannel {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        blend_op: BlendOp::Add,
                    },
                    alpha: BlendChannel {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::DstAlpha,
                        blend_op: BlendOp::Add,
                    },
                }],
            });

            // TODO: Transforms on sprites

            let mut sprites = Vec::new();
            mem::swap(&mut sprites, &mut self.sprite_list);
            for (sprite, uniform) in sprites {
                self.grr.bind_image_views(3, &[sprite.view()]);
                self.grr.bind_samplers(3, &[self.sampler]);
                self.grr.bind_uniform_buffers(
                    0,
                    &[BufferRange {
                        buffer: uniform.buffer(),
                        offset: 0,
                        size: mem::size_of::<SpriteUniform>(),
                    }],
                );
                self.grr.bind_uniform_buffers(
                    1,
                    &[BufferRange {
                        buffer: camera.buffer(),
                        offset: 0,
                        size: mem::size_of::<Mat4>(),
                    }],
                );

                self.grr.draw(Primitive::Triangles, 0..6, 0..1);
                // delete transform
                self.grr.delete_buffer(uniform.buffer());
            }

            /*
            // PER MATERIAL
            self.grr.bind_pipeline(self.pipeline);
            self.grr.bind_vertex_array(self.vertex_array);

            // PER OBJECT
            for obj in &self.draw_list {
                self.grr.bind_vertex_buffers(
                    self.vertex_array,
                    0,
                    &[VertexBufferView {
                        buffer: obj.buffer(),
                        offset: 0,
                        stride: mem::size_of::<Vertex<f32>>() as _,
                        input_rate: InputRate::Vertex,
                    }],
                );
                self.grr.draw(grr::Primitive::Triangles, 0..3, 0..1);
            }
            */
            self.draw_list.clear();

            self.grr.end_debug_marker();
        }
    }

    fn finish(&mut self) {
        self.ctx.swap_buffers();
    }
}
