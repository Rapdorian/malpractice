// Create a rendering context with WGPU
// some of these cane be made static to simplify

use super::{RenderState, Vertex};
use crate::assets;
use crate::assets::AssetManager;
use egui_winit::egui::{ClippedPrimitive, TexturesDelta};
use image::{EncodableLayout, GenericImageView};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};
use wgpu::{
    util::{DeviceExt, TextureDataOrder},
    BufferUsages, TextureUsages,
};

pub struct Surface {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub surface: Option<wgpu::Surface<'static>>,
    pub state: Mutex<RenderState<u32, mint::Vector3<f32>>>,
    pub format: wgpu::TextureFormat,
    pub ui_output: Option<(Vec<ClippedPrimitive>, TexturesDelta)>,
    pub dimensions: [u32; 2],
    pub win: Arc<winit::window::Window>,
}

pub struct Texture {
    pub(crate) label: String,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Texutre(\"{}\", ({},{})",
            self.label, self.width, self.height
        )
    }
}

pub struct Mesh {
    pub(crate) verts: wgpu::Buffer,
    #[allow(unused)]
    pub(crate) idx: wgpu::Buffer,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

unsafe impl<T> bytemuck::Pod for Rect<T> where T: bytemuck::Pod {}
unsafe impl<T> bytemuck::Zeroable for Rect<T> where T: bytemuck::Zeroable {}

#[derive(Serialize, Deserialize)]
pub struct SpriteMap {
    path: String,
    sprites: HashMap<String, Rect<u32>>,
}

#[derive(Debug)]
pub struct Sprite {
    pub(crate) texture: Arc<Texture>,
    pub(crate) rect: Rect<f32>,
}

type AssetCache<T> = Lazy<Mutex<HashMap<String, Weak<T>>>>;

fn lookup_asset<T>(cache: &AssetCache<T>, id: &str) -> Option<Arc<T>> {
    if let Some(asset) = cache.lock().unwrap().get(id) {
        if let Some(asset) = asset.upgrade() {
            log::info!("Found existing asset for {id}");
            return Some(asset);
        }
    }
    None
}

impl Surface {
    pub(crate) fn set_ui(
        &mut self,
        clipped_primitive: Vec<ClippedPrimitive>,
        textures_delta: TexturesDelta,
    ) {
        self.ui_output = Some((clipped_primitive, textures_delta));
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.surface
            .as_ref()
            .unwrap()
            .get_capabilities(&self.adapter)
            .formats[0]
    }

    pub fn resume(&mut self) {
        log::warn!("Recreating surface");
        self.surface = Some(self.instance.create_surface(Arc::clone(&self.win)).unwrap());
        self.reconfig();
    }

    pub fn suspend(&mut self) {
        self.surface.take();
    }

    pub fn sampler(&self) -> Arc<wgpu::Sampler> {
        static SAMPLER: OnceCell<Arc<wgpu::Sampler>> = OnceCell::new();
        SAMPLER
            .get_or_init(|| {
                Arc::new(self.device.create_sampler(&wgpu::SamplerDescriptor {
                    ..Default::default()
                }))
            })
            .clone()
    }

    pub fn load_sprite(&self, path: &str, fragment: &str) -> Sprite {
        let mut file = String::new();
        assets::platform::os_asset_manager()
            .open(path)
            .unwrap()
            .read_to_string(&mut file)
            .unwrap();
        let sprite_map: SpriteMap = toml::from_str(&file).unwrap();

        let atlas = self.load_texture(&sprite_map.path);
        let sprite_rect = sprite_map.sprites.get(fragment).unwrap();
        let uv_rect = Rect {
            x: sprite_rect.x as f32 / atlas.width as f32,
            y: sprite_rect.y as f32 / atlas.height as f32,
            width: sprite_rect.width as f32 / atlas.width as f32,
            height: sprite_rect.height as f32 / atlas.height as f32,
        };

        Sprite {
            texture: atlas,
            rect: uv_rect,
        }
    }

    /// Loads a mesh to the GPU
    ///
    /// Deduplicates based on provided label.
    /// **NOTE** If you do not change the label you will not be able
    /// to upload different data even if you change the vertex data.
    pub fn load_mesh(&self, label: &str, verts: &[Vertex], idx: &[u16]) -> Arc<Mesh> {
        // TODO:
        // - DB of loaded meshes
        // - Load obj mesh with tobj
        //      - Rework to be more flexible later
        // - Upload buffers
        log::info!("Loading mesh: {label}");
        static MESHES: AssetCache<Mesh> = Lazy::new(Default::default);

        if let Some(mesh) = lookup_asset(&MESHES, label) {
            return mesh;
        }

        let verts = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(verts),
                usage: BufferUsages::VERTEX,
            });

        let idx = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{label} indices")),
                contents: bytemuck::cast_slice(idx),
                usage: BufferUsages::INDEX,
            });
        let mesh = Arc::new(Mesh {
            verts,
            idx,
        });
        MESHES
            .lock()
            .unwrap()
            .insert(label.to_string(), Arc::downgrade(&mesh));
        mesh
    }

    pub fn reconfig(&mut self) {
        let Some(surface) = &self.surface else {
            log::warn!("Attempted to reconfigure destroyed surface");
            return;
        };
        let size = self.win.inner_size();
        let config = wgpu::SurfaceConfiguration {
            present_mode: wgpu::PresentMode::AutoVsync,
            ..surface
                .get_default_config(&self.adapter, size.width, size.height)
                .unwrap()
        };
        surface.configure(&self.device, &config);
        self.dimensions = [size.width, size.height];
    }

    /// Loads a texture from a file.
    ///
    /// This method will deduplicate successive loads from the same file
    pub fn load_texture(&self, path: &str) -> Arc<Texture> {
        log::info!("Loading texture: {path}");
        static TEXTURES: AssetCache<Texture> = Lazy::new(Default::default);

        if let Some(tex) = lookup_asset(&TEXTURES, path) {
            return tex;
        }

        //TODO: Call out to an asset manager that can load packed assets for the raw file data
        //TODO: Error handling and logging
        let file = assets::platform::os_asset_manager().read_bytes(path).unwrap();
        let img = image::load_from_memory(&file).unwrap();
        let bytes = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some(path),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            bytes.as_bytes(),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{path} view")),
            ..Default::default()
        });
        let tex = Arc::new(Texture {
            label: path.to_string(),
            texture,
            view,
            width: dimensions.0,
            height: dimensions.1,
        });

        TEXTURES
            .lock()
            .unwrap()
            .insert(path.to_string(), Arc::downgrade(&tex));
        tex
    }

    pub fn new(win: &Arc<winit::window::Window>) -> Surface {
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let surface = instance.create_surface(Arc::clone(win)).unwrap();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                })
                .await
                .unwrap();
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .unwrap();

            let size = win.inner_size();
            let config = wgpu::SurfaceConfiguration {
                present_mode: wgpu::PresentMode::AutoVsync,
                ..surface
                    .get_default_config(&adapter, size.width, size.height)
                    .unwrap()
            };
            surface.configure(&device, &config);
            let format = surface.get_capabilities(&adapter).formats[0];
            Surface {
                adapter,
                device,
                queue,
                surface: Some(surface),
                instance,
                state: Mutex::new(RenderState::new(0.0)),
                format,
                ui_output: None,
                dimensions: [size.width, size.height],
                win: win.clone(),
            }
        })
    }
}
