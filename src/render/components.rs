//! This is the ECS interface for rendering
//!
//! This module defines a number of components to control the rendering engine with
//! A 2d sprite is a special case of a single material mesh

// most of the contents of these structs are allowed to be opaque, especially mesh since it will
// mostly consist of handles into GPU memory

// TODO:
// - Create a builder API
//   - This API should be easy to make compatible with Tcl
// - Decide on a render architecture
//   - Support custom shaders if possible
// - Implement render system

/// Material definition associated with a mesh
/// We'll need some kind of handle for the user
/// to manipulate the textures with
struct Material {
    albedo: todo!(),
    normal: todo!(),
    metallic: todo!(),
    roughness: todo!(),
    ambient_occlusion: todo!(),
}

/// A single material mesh
struct Mesh {
    material: Material,
    pub mesh: wgpu::Buffer,
}

/// A list of meshes allowing multiple materials
struct Model {
    meshes: Vec<Mesh>,
}

/// A light source
///
/// The different types of light are disambiguated by the components they have
///     - Point
///         - pos
///         - color
///     - Directional
///         - dir
///         - color
///     - Spotlight
///         - dir
///         - pos
///         - color
///     - Ambient
///         - color
struct LightColor {
    r: f32,
    g: f32,
    b: f32,
}
