use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub pos: mint::Vector3<f32>,
    pub uv: mint::Vector2<f32>,
    pub normal: mint::Vector3<f32>,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

unsafe impl Zeroable for Vertex {}

unsafe impl Pod for Vertex {}

/// Flexible method of creating a vertex
#[macro_export]
macro_rules! vertex {
    ($x:expr, $y:expr, $z:expr; $u:expr, $v:expr; $nx:expr, $ny:expr, $nz:expr) => {
        $crate::render::Vertex {
            pos: mint::Vector3{
                x: $x,
                y: $y,
                z: $z,
            },
            uv: mint::Vector2{
                x: $u,
                y: $v,
            },
            normal: mint::Vector3{
                x: $nx,
                y: $ny,
                z: $nz,
            }
        }
    };
    ($x:expr, $y:expr, $z:expr; $u:expr, $v:expr) => {
        vertex!($x, $y, $z; $u, $v; 0.0, 0.0, 0.0)
    };
    ($x:expr, $y:expr, $z:expr) => {
        vertex!($x, $y, $z; 0.0, 0.0; 0.0, 0.0, 0.0)
    };
    ($x:expr, $y:expr) => {
        vertex!($x, $y, 0.0; 0.0, 0.0; 0.0, 0.0, 0.0)
    };
}
