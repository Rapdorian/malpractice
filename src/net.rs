//! Networking layer
//!
//! I'm thinking a good flow for this layer is to create a API definition struct something like
//! ```
//! pub enum Packet {
//!     Jump{id: u32},
//!     SetPos{id: u32, pos: (f32, f32)},
//! }
//! ```
//! Then we can feed that into the event_loop as a user event type (With some implementation defined additional info) and
//! then we'll need a way to feed these events into the update() method of a scene
//!
//! __NOTE:__ On further thought different scenes will probably want different packet types. I am not sure if those should
//! be implemented separately or not
//!
//! Maybe we shouldn't piggyback off the event_loop. We could probably feed to scene with it's own packet type. I'm still
//! uncertain if we should be handling these events in `update()` or as a callback API. I'd rather not have to store flags
//! in the scene. So maybe the engine context should have a iterator of network events.
//!
//! The engine context is a global type. So we can't cleanly shovel a per scene packet type into it. We could handle
//! packet parsing in the scene itself. Having an iterator of something implementing `Deserialize`. Or we could have
//! application global packet types. I also don't really want to add a generic parameter to `EngineContext`
//! So at that point all the engine provides is collecting packets into a vec and serialization/sending packets. Which is
//! really not much. certainly an improvemnt over raw sockets.
