//! A gameworld that is backed by a HECS world
//! - Create an API for creating entities and querying the world
//! - Allow registering action functions with the world
//! - Allow registering constructors with the world
//! - Integrate a Tcl interpreter that can call registered functions

use hecs::{DynamicBundle, Entity};
pub struct World {
    world: hecs::World,
}

impl World {
    // spawns a new entity into the world
    // may also update any other datastructures the world has
    // like spatial bucketing
    // this would allow creating entities from a script like
    // ```Tcl
    // spawn [pos 0 0 0] [mesh "foo.mesh"] [player]
    // ```
    // This lowers the scope of what the registered constructors have to do.
    pub fn spawn(&mut self, c: impl DynamicBundle) -> Entity {
        // for now we won't do anything interesting here
        self.world.spawn(c)
    }
}

// Ok, so what we really want the renderer to do now is to provide a set of components that
// can be added to a world object and then we should be able to call a method in the renderer
// somewhere that will query the world for renderable things and draw them. I should also figure
// out what I want to do about potentially multiple cameras, we are also very much going to use
// wgpu as out rendering abstraction
