use crate::render::grr::{shaders, BufferHandle, GrrError, SpriteHandle};
use crate::window::Window;
use grr::{DebugSource, Device, Pipeline, Sampler, VertexArray, VertexFormat};
use mint::ColumnMatrix4;
use raw_gl_context::{GlConfig, GlContext, Profile};
use std::rc::Rc;
use tracing::instrument;

pub struct GrrRender {
    pub(super) grr: Rc<grr::Device>,
    pub(super) ctx: GlContext,
    pub(super) vertex_array: VertexArray,
    pub(super) pipeline: Pipeline,
    pub(super) draw_list: Vec<BufferHandle>,
    pub(super) sprite_list: Vec<(SpriteHandle, BufferHandle)>,
    pub(super) sprite_pipeline: Pipeline,
    pub(super) sprite_vertex_array: VertexArray,
    pub(super) sampler: Sampler,
}

impl GrrRender {
    /// Initialize a Grr-backed renderer
    ///
    /// TODO: This needs to be split out into better pieces
    #[instrument(err, name = "new grr")]
    pub fn new(window: &Window) -> Result<Self, GrrError> {
        let ctx = window
            .with_winit(|window| {
                GlContext::create(
                    window,
                    GlConfig {
                        version: (4, 5),
                        profile: Profile::Core,
                        red_bits: 8,
                        blue_bits: 8,
                        green_bits: 8,
                        alpha_bits: 0,
                        depth_bits: 0,
                        stencil_bits: 0,
                        samples: None,
                        srgb: true,
                        double_buffer: true,
                        vsync: true,
                    },
                )
            })
            .ok_or(GrrError::WindowNotInitialized)?
            .unwrap();
        ctx.make_current();
        let grr = unsafe {
            grr::Device::new(
                |symbol| ctx.get_proc_address(symbol) as *const _,
                grr::Debug::Enable {
                    callback: |report, _, _, _, msg| {
                        println!("{:?}: {:?}", report, msg);
                    },
                    flags: grr::DebugReport::FULL,
                },
            )
        };
        unsafe {
            grr.begin_debug_marker(
                DebugSource::ThirdParty,
                0,
                "Initialize rivik rendering system",
            );
        }

        let pipeline = unsafe {
            let vs = grr
                .create_shader(
                    grr::ShaderStage::Vertex,
                    shaders::VERTEX_SRC.as_bytes(),
                    grr::ShaderFlags::VERBOSE,
                )
                .unwrap();
            let fs = grr
                .create_shader(
                    grr::ShaderStage::Fragment,
                    shaders::FRAGMENT_SRC.as_bytes(),
                    grr::ShaderFlags::VERBOSE,
                )
                .unwrap();
            grr.create_graphics_pipeline(
                grr::VertexPipelineDesc {
                    vertex_shader: vs,
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: Some(fs),
                },
                grr::PipelineFlags::VERBOSE,
            )
            .unwrap()
        };

        let sprite_pipeline = unsafe {
            let vs = grr
                .create_shader(
                    grr::ShaderStage::Vertex,
                    shaders::SPRITE_VERTEX_SRC.as_bytes(),
                    grr::ShaderFlags::VERBOSE,
                )
                .unwrap();
            let fs = grr
                .create_shader(
                    grr::ShaderStage::Fragment,
                    shaders::SPRITE_FRAGMENT_SRC.as_bytes(),
                    grr::ShaderFlags::VERBOSE,
                )
                .unwrap();
            grr.create_graphics_pipeline(
                grr::VertexPipelineDesc {
                    vertex_shader: vs,
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: Some(fs),
                },
                grr::PipelineFlags::VERBOSE,
            )
            .unwrap()
        };

        /* Initialize render pipeline data */
        let vertex_array = unsafe {
            grr.create_vertex_array(&[
                grr::VertexAttributeDesc {
                    location: 0,
                    binding: 0,
                    format: VertexFormat::Xyz32Float,
                    offset: 0,
                },
                grr::VertexAttributeDesc {
                    location: 1,
                    binding: 0,
                    format: VertexFormat::Xy32Float,
                    offset: (3 * 4) as _,
                },
                grr::VertexAttributeDesc {
                    location: 2,
                    binding: 0,
                    format: VertexFormat::Xyz32Float,
                    offset: ((3 + 2) * 4) as _,
                },
            ])
            .unwrap()
        };

        let sprite_vertex_array = unsafe { grr.create_vertex_array(&[]).unwrap() };
        let sampler = unsafe {
            grr.create_sampler(grr::SamplerDesc {
                min_filter: grr::Filter::Nearest,
                mag_filter: grr::Filter::Nearest,
                mip_map: None,
                address: (
                    grr::SamplerAddress::ClampEdge,
                    grr::SamplerAddress::ClampEdge,
                    grr::SamplerAddress::ClampEdge,
                ),
                lod_bias: 0.0,
                lod: 0.0..10.0,
                compare: None,
                border_color: [0.0, 0.0, 0.0, 1.0],
            })
            .unwrap()
        };

        unsafe {
            grr.end_debug_marker();
        }

        Ok(Self {
            grr: Rc::new(grr),
            ctx,
            vertex_array,
            pipeline,
            draw_list: Vec::new(),
            sprite_list: Vec::new(),
            sprite_pipeline,
            sprite_vertex_array,
            sampler,
        })
    }
}
