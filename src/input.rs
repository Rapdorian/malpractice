//! This module handles processing raw input into game events

pub(crate) mod action;
pub(crate) mod handler;
pub(crate) mod event;

pub use handler::ActionHandler;
pub use handler::Action;
pub use event::InputType;
