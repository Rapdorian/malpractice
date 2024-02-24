//! This module should provide some basic types that are backend agnostic so we can add our own
//! rendering backends as submodules. Ideally we want this to be generic enough that we could implement
//! a console backend
//!
//! Ok, so now we need to be able to determine what this rendering API is responsible for.
//! - render is NOT responsible for handling scene graph
//! - render SHOULD be capable of handling 2d graphics
//! - render SHOULD be able to handle egui
//! - render should NOT have integration with egui
//!
//! At a high level there are a handful of objects we need to be able to render
//! - Meshes
//! - Textures
//! - Lights
//! - Sprites ( a subset of mesh + texture)
//! - Material
//!
//! We need to load the renderer with a list of objects and a camera and output surface and it should
//! be able to render.
//!
//! the renderer should be smart enough to do at least some culling

pub mod grr;
mod texture;

pub use texture::*;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use mint;
use mint::ColumnMatrix4;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vertex<T> {
    pub pos: mint::Vector3<T>,
    pub uv: mint::Vector2<T>,
    pub normal: mint::Vector3<T>,
}

unsafe impl<T> Zeroable for Vertex<T> where T: Zeroable {}

unsafe impl<T> Pod for Vertex<T> where T: Pod {}

pub struct Material<T> {
    pub diffuse: T,
}

pub struct Camera<T> {
    pub transform: ColumnMatrix4<T>,
}

pub enum Light<T> {
    Point {
        pos: mint::Vector3<T>,
        color: mint::Vector3<T>,
    },
    Directional {
        dir: mint::Vector3<T>,
        color: mint::Vector3<T>,
    },
}

pub trait Renderer<T> {
    type MeshHandle: GpuResource;
    type BufferHandle: GpuResource;
    type SpriteHandle: GpuResource;
    type Surface;

    fn upload_mesh(&mut self, verts: &[Vertex<T>], indices: &[u16]) -> Self::MeshHandle;
    fn upload_buffer(&mut self, data: &[u8]) -> Self::BufferHandle;

    /// Upload an image to the GPU
    fn upload_sprite(&mut self, tex: &Texture) -> Self::SpriteHandle;

    // need to include a transform here
    fn draw_mesh(&mut self, mesh: &Self::MeshHandle, material: Material<Self::BufferHandle>);
    fn draw_sprite(&mut self, sprite: &Self::SpriteHandle, transform: mint::ColumnMatrix4<T>);
    fn draw_light(&mut self, light: Light<T>);

    fn clear(&mut self, surface: &mut Self::Surface);
    fn render(&mut self, camera: Camera<T>, surface: &mut Self::Surface);
    fn finish(&mut self);
}

pub trait GpuResource {
    fn label<S: AsRef<str>>(&mut self, s: S);
    fn with_label<S: AsRef<str>>(mut self, s: S) -> Self
    where
        Self: Sized,
    {
        self.label(s);
        self
    }
}

// we'll need to initialize a window somehow
// We'll need a way to tie said window init into the input module and a window control module
// Using a static would work. but IDK if that's really what I want to do.
