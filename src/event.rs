//! Event Loop
//!
//! TODO: Describe event architecture

/// Defines engine level events
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Event<T> {
    Tick,
    Render,
    User(T)
}

// This structure can allow us to define scenes as a single flat function/closure
//
// ```rust
// fn main() {
//   let w = Window::new();
//   let r = Renderer::new(&w);
//   window.run(move |ev| {
//     match ev {
//       Event::Tick => {
//          // simulation code
//       },
//       Event::Render => {
//          // render code
//       }
//     }
//   })
// }
//
// ```
// This is pretty great but doesn't leave us
// with a full understanding of how to map
// inputs from their source.
//
// We should probably define a handful of 
// traits for engine supported event sources
// for instance a `From<KeyInput>` trait that
// the engine can call internally to create
// well-typed events
//
// For unit testing this solution is also 
// compelling because we can skip the event
// loop and directly synthesise events
// ```
// fn test() {
//   let scene = Scene::new(...);
//
//   scene.handle_event(Event::Tick);
//   scene.handle_event(Event::Tick);
//   // now check contents of scene after
//   // two physics ticks
// }
// ```
