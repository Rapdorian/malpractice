use crate::app::stage::EngineStage;
use crate::input::Action;
use crate::render::Surface;
use crate::{Rivik, Stage};
use std::sync::Arc;
use winit::window::Window;

pub struct StageBuilder<'a, A: Action> {
    pub(super) window: Arc<Window>,
    pub(super) rivik: &'a mut Rivik<A>,
    pub(super) surface: Option<Surface>,
}

impl<'a, A: Action> StageBuilder<'a, A> {
    pub fn surface(&mut self) -> &Surface {
        if self.surface.is_none() {
            self.surface = Some(Surface::new(&self.window));
        }
        self.surface.as_ref().unwrap()
    }

    pub fn build(self, stage: impl Stage<A> + 'static) {
        let stage = EngineStage::new(self.window, self.surface, Box::new(stage));
        self.rivik.stages.insert(stage.id(), stage);
    }
}
