use std::{sync::{Arc, Mutex}, fmt};

use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes}, event::WindowEvent,
};

use crate::{Rivik, Stage, bus::MessageBus, input::{Action, ActionHandler}};

pub struct RivikController<'a, A: Action> {
    pub(super) event_loop: &'a ActiveEventLoop,
    pub(super) rivik: &'a mut Rivik<A>,
}

impl<'a, A: Action> RivikController<'a, A> {

    pub fn input(&mut self) -> &mut ActionHandler<A> {
        &mut self.rivik.input
    }

    pub fn new_window(&self) -> Arc<Window> {
        let window = Arc::new(
            self.event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
        self.rivik
            .windows
            .write()
            .unwrap()
            .insert(window.id(), Arc::clone(&window));
        window
    }
    pub fn register_stage(&self, window: &Arc<Window>, stage: impl Stage<A> + 'static) {
        self.rivik
            .stages
            .write()
            .unwrap()
            .insert(window.id(), Arc::new(Mutex::new(stage)));
    }
}
