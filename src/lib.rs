//! This is going to be a root of a game engine I'm working on.

pub mod app;
pub mod bench;
pub mod bus;
pub mod components;
pub mod render;

/// This module handles processing raw input into game events
pub mod input {
    pub(crate) mod action;
    pub(crate) mod event;
    pub(crate) mod handler;

    pub use event::InputType;
    pub use handler::Action;
    pub use handler::ActionHandler;
}

pub mod assets {
    mod asset_manager;
    pub use asset_manager::*;
    pub(crate) mod fs;

    #[cfg(target_os = "android")]
    pub(crate) mod android;
    pub mod platform;
}

pub use app::*;