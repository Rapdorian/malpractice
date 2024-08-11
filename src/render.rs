// impl2
mod context;
mod state;
mod timestamp;
mod vertex;
//mod components;

pub use context::*;
use egui_wgpu::ScreenDescriptor;
use egui_winit::egui::{ClippedPrimitive, TexturesDelta};
use once_cell::sync::{Lazy, OnceCell};
use std::time::Instant;
use std::{
    collections::HashMap,
    mem,
    sync::{Arc, Mutex},
};
pub use vertex::*;
use wgpu::{
    util::DeviceExt,
    BufferUsages, StoreOp,
};

use crate::components::Transform;
pub use state::*;
pub use timestamp::*;
//pub use components::*;

pub struct Frame<'a> {
    surface: &'a Surface,
    output: wgpu::SurfaceTexture,
    output_view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
    format: wgpu::TextureFormat,
}

impl Surface {
    pub fn next_frame(&self, interp: f32) -> Option<Frame> {
        static FPS_TIMER: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(1.0));
        static PREV_FRAME: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

        // record this frame's time
        if let Some(prev) = *PREV_FRAME.lock().unwrap() {
            const ALPHA: f32 = 0.05;
            let t = prev.elapsed().as_nanos();
            let mut timer = FPS_TIMER.lock().unwrap();
            *timer = ((t as f32 - *timer) * ALPHA) + *timer;
            crate::bench::raw_section("avg time", *timer as u128);
        }

        let output = self.surface.as_ref()?.get_current_texture().unwrap();
        *PREV_FRAME.lock().unwrap() = Some(Instant::now());

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        self.state.lock().unwrap().set_timestamp(interp);

        // record the time for the start of this frame

        Some(Frame {
            surface: self,
            output,
            output_view: view,
            encoder,
            format: self.format,
        })
    }
}

impl<'a> Frame<'a> {
    /// This method should allow drawing a renderpass that is
    /// created by an external or internal user.
    pub fn draw(
        mut self,
        f: impl FnOnce(
            &wgpu::Device,
            &mut wgpu::CommandEncoder,
            &wgpu::TextureView,
            wgpu::TextureFormat,
            &Mutex<RenderState<u32, mint::Vector3<f32>>>,
        ),
    ) -> Frame<'a> {
        (f)(
            &self.surface.device,
            &mut self.encoder,
            &self.output_view,
            self.format,
            &self.surface.state,
        );
        self
    }

    /// Finish drawing this frame
    pub fn submit(mut self) {
        // submit will accept anything that implements IntoIter
        if let Some(ui) = &self.surface.ui_output {
            self = self.draw_egui(ui);
        }
        self.surface
            .queue
            .submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }

    pub fn draw_egui(self, ui: &(Vec<ClippedPrimitive>, TexturesDelta)) -> Self {
        let queue = &self.surface.queue;
        let desc = ScreenDescriptor {
            size_in_pixels: self.surface.dimensions.clone(),
            pixels_per_point: 1.0,
        };
        self.draw(|device, enc, out, format, _state| {
            static UI_RENDER: OnceCell<Mutex<egui_wgpu::Renderer>> = OnceCell::new();
            let mut ui_render = UI_RENDER
                .get_or_init(|| {
                    Mutex::new(egui_wgpu::Renderer::new(&device, format, None, 1, false))
                })
                .lock()
                .unwrap();
            for (id, img_delta) in &ui.1.set {
                ui_render.update_texture(device, queue, *id, img_delta);
            }
            ui_render.update_buffers(device, queue, enc, &ui.0, &desc);
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            ui_render.render(&mut rpass, &ui.0, &desc);
            mem::drop(rpass);
            for x in &ui.1.free {
                ui_render.free_texture(x);
            }
        })
    }

    pub fn draw_sprites<'w>(
        self,
        sprites: impl Iterator<Item = (u32, (&'w Sprite, &'w Transform))>,
        camera: Transform,
    ) -> Self {
        let _bench = crate::bench::start("sprite-pass");
        let sampler = self.surface.sampler();
        self.draw(|device, enc, out, format, state| {
            // create a sprite pipeline
            static PIPELINE: OnceCell<(
                wgpu::PipelineLayout,
                wgpu::RenderPipeline,
                wgpu::BindGroupLayout,
            )> = OnceCell::new();
            let (_pipeline_layout, pipeline, bg_layout) = PIPELINE.get_or_init(|| {
                log::info!("Initializing sprite pipeline");
                let shader = load_shader(
                    device,
                    r#"

                        struct VertexOutput {
                            @builtin(position) pos: vec4<f32>,
                            @location(0) uv: vec2<f32>,
                        }

                        @vertex
                        fn vs_main(
                                @location(0) mat_0: vec4<f32>,
                                @location(1) mat_1: vec4<f32>,
                                @location(2) mat_2: vec4<f32>,
                                @location(3) mat_3: vec4<f32>,
                                @location(4) uv_offset: vec2<f32>,
                                @location(5) uv_size: vec2<f32>,
                                @location(6) texture: u32,
                                @builtin(vertex_index) idx: u32
                            ) -> VertexOutput {

                            let a = i32(idx) & 1;
                            let b = ((i32(idx) & 2) >> 1);
                            let c = ((i32(idx) & 4) >> 2);

                            let x = f32(b + ((~a) & c));
                            let y = f32(a);
                            let pos = vec2<f32>(x,y);

                            let trans = mat4x4<f32>(
                                mat_0,
                                mat_1,
                                mat_2,
                                mat_3
                            );
                            
                            var out: VertexOutput;
                            out.pos = trans * vec4<f32>(pos.xy, 0.0, 1.0);
                            out.uv = (pos * uv_size) + uv_offset;
                            out.uv.y = 1.0-out.uv.y;
                            return out;
                        }

                        @group(0) @binding(0)
                        var atlas: texture_2d<f32>;
                        @group(0) @binding(1)
                        var samplr: sampler;

                        @fragment
                        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                            return textureSample(atlas, samplr, in.uv);
                        }
                    "#,
                );

                // create instance buffer desc
                let sprite_buffer = wgpu::VertexBufferLayout {
                    array_stride: 4 * 16 + 4 * 2 + 4 * 2 + 4,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Float32x4,
                        4 => Float32x2,
                        5 => Float32x2 ,
                        6 => Uint32,
                    ],
                };

                let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&bg_layout],
                        push_constant_ranges: &[],
                    });
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        compilation_options: Default::default(),
                        buffers: &[sprite_buffer],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(format.into())],
                    }),
                    multiview: None,
                    cache: None,
                });
                (pipeline_layout, pipeline, bg_layout)
            });

            // create instance buffers
            let mut textures = Vec::new();
            let mut texture_lookup = HashMap::new();
            for (id, (sprite, transform)) in sprites {
                let t_id = sprite.texture.texture.global_id();
                let idx = *texture_lookup.entry(t_id).or_insert_with(|| {
                    textures.push((Arc::clone(&sprite.texture), Vec::new()));
                    textures.len() - 1
                });
                let instance_buffer = &mut textures[idx].1;

                // write into instance buffer
                // Buffer format:
                // transform: mat4x4
                // uv_offset: vec2
                // uv_size: vec2
                // texture: u32
                // handle render state interpolation
                let mut transform = transform.clone();
                transform.position = {
                    let mut state = state.lock().unwrap();
                    if state.get(&id).is_none() {
                        state.store(
                            id,
                            mint::Vector3 {
                                x: transform.position.x,
                                y: transform.position.y,
                                z: transform.position.z,
                            },
                        );
                    }
                    let prev = glam::Vec3::from(state[id]);
                    let cur = glam::Vec3::from(transform.position);
                    let time = state.timestamp();
                    // write back into state
                    let lerped = prev.lerp(cur, time as f32);
                    state.store(id, lerped.into());
                    lerped.into()
                };

                let mat: mint::ColumnMatrix4<f32> = transform.into();
                let mat: glam::Mat4 = mat.into();

                // get sprite shape
                let width = sprite.texture.width as f32 * sprite.rect.width;
                let height = sprite.texture.height as f32 * sprite.rect.height;
                let sprite_mat = glam::Mat4::from_scale(glam::Vec3::new(width, height, 1.0));
                let camera: glam::Mat4 = camera.into();

                let mat = camera * mat * sprite_mat;

                instance_buffer.extend_from_slice(bytemuck::bytes_of(&mat));
                instance_buffer.extend_from_slice(bytemuck::bytes_of(&sprite.rect));
                instance_buffer.extend_from_slice(bytemuck::bytes_of(&(idx as u32)));
            }

            //render pass
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(pipeline);

            for (texture, buffer) in textures {
                let sprite_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &buffer,
                    usage: BufferUsages::VERTEX,
                });

                let atlas = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: bg_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });
                let sprite_count = buffer.len() as u32 / (4 * 16 + 4 * 4 + 4);
                rpass.set_bind_group(0, &atlas, &[]);
                rpass.set_vertex_buffer(0, sprite_buffer.slice(..));
                rpass.draw(0..6, 0..sprite_count);
            }
        })
    }

    pub fn draw_sprites_from_world(self, world: &hecs::World, camera: Transform) -> Self {
        self.draw_sprites(
            world
                .query::<(&Sprite, &Transform)>()
                .iter()
                .map(|(e, c)| (e.id(), c)),
            camera,
        )
    }

    pub fn draw_mesh(self, mesh: &Mesh) -> Frame<'a> {
        self.draw(|device, enc, out, format, _| {
            // declare static pipeline info
            static PIPELINE: OnceCell<(wgpu::PipelineLayout, wgpu::RenderPipeline)> =
                OnceCell::new();
            let (_pipeline_layout, pipeline) = PIPELINE.get_or_init(|| {
                log::info!("Initializing mesh pipeline");
                let shader = load_shader(
                    device,
                    r#"
                        @vertex
                        fn vs_main(
                            //@location(0) pos:  vec3<f32>,
                            //@location(1) uv:   vec2<f32>,
                            //@location(2) norm: vec3<f32>
                            @builtin(vertex_index) idx: u32
                            ) -> @builtin(position) vec4<f32> {
                            let x = f32(i32(idx) -1);
                            let y = f32(i32(idx & 1u) * 2 - 1);
                            return vec4<f32>(x,y, 0.0, 1.0);
                        }

                        @fragment
                        fn fs_main() -> @location(0) vec4<f32> {
                            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                        }

                    "#,
                );
                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        compilation_options: Default::default(),
                        buffers: &[Vertex::desc()],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(format.into())],
                    }),
                    multiview: None,
                    cache: None,
                });
                (pipeline_layout, pipeline)
            });
            // now render
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(pipeline);
            rpass.set_vertex_buffer(0, mesh.verts.slice(..));
            rpass.draw(0..3, 0..1);
        })
    }

    // we need a system that can be run on a hecs world that will fetch renderables and draw them
    // for now lets just do a simple clear screen routine.
    pub fn clear_screen(self, r: f32, g: f32, b: f32) -> Frame<'a> {
        self.draw(|_, enc, out, _, _| {
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        })
    }
}

fn load_shader(device: &wgpu::Device, shader: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader)),
    })
}
