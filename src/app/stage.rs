use crate::input::{Action, ActionHandler};
use crate::render::Surface;
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};

pub trait Stage<A: Action> {
    fn render(&self, _surface: &Surface, _interp: f32) {}
    fn tick(&mut self, _input: &mut ActionHandler<A>, _step: f32) {}

    fn ui(&mut self, _egui: &egui_winit::egui::Context, _input: &mut ActionHandler<A>) {}
}

/// Track engine state that is per-stage
pub(in crate::app) struct EngineStage<A> {
    surface: Option<Surface>,
    window: Arc<Window>,
    user: Mutex<Box<dyn Stage<A>>>,
    egui: egui_winit::State,
}

impl<A: Action> EngineStage<A> {
    pub fn new(window: Arc<Window>, surf: Option<Surface>, stage: Box<dyn Stage<A>>) -> Self {
        let egui = {
            let egui = egui_winit::egui::Context::default();
            let id = egui.viewport_id();
            egui_winit::State::new(egui, id, &window, None, None, None)
        };
        Self {
            surface: surf,
            window,
            user: Mutex::new(stage),
            egui,
        }
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn resume(&mut self) {
        match &mut self.surface {
            None => self.surface = Some(Surface::new(&self.window)),
            Some(surf) => surf.resume(),
        }
    }

    pub fn resize(&mut self) {
        if let Some(surf) = self.surface.as_mut() {
            surf.reconfig();
        }
    }

    pub fn suspend(&mut self) {
        match &mut self.surface {
            None => {}
            Some(surf) => surf.suspend(),
        }
    }

    pub fn raw_window_event(&mut self, e: &WindowEvent) {
        let _ = self.egui.on_window_event(&self.window, e);
    }

    pub fn tick(&self, input: &mut ActionHandler<A>, step: f32) {
        self.user.lock().unwrap().tick(input, step);
    }
    pub fn render(&mut self, interp: f32, handler: &mut ActionHandler<A>) {
        let mut user = self.user.lock().unwrap();
        let input = self.egui.take_egui_input(&self.window);
        let output = self.egui.egui_ctx().run(input, |ctx| user.ui(ctx, handler));
        self.egui
            .handle_platform_output(&self.window, output.platform_output);
        let primitives = self
            .egui
            .egui_ctx()
            .tessellate(output.shapes, output.pixels_per_point);
        self.surface
            .as_mut()
            .unwrap()
            .set_ui(primitives, output.textures_delta);

        user.render(self.surface.as_ref().unwrap(), interp);
        self.window.request_redraw();
    }
}
