use crate::background::Background;
use crate::bird::Bird;
use crate::game::Game;
use crate::menu::Menu;
use crate::ui::UIElements;
use crate::{camera, SPEED};
use glam::Vec2;
use rivik::input::KeyState;
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;
use rivik::{EngineContext, Scene};
use std::error::Error;
use tracing::{info_span, Span};
use winit::keyboard::KeyCode;

pub struct Dead {
    atlas: SpriteHandle,
    ui: UIElements,
    bg: Background,
    bird: Bird,
    grr: GrrRender,
}

impl Dead {
    pub fn new(pos: Vec2, img: SpriteHandle, grr: GrrRender) -> Self {
        Self {
            ui: UIElements::new(&img),
            bg: Background::new(&img),
            bird: Bird::new(&img).with_pos(pos),
            atlas: img,
            grr,
        }
    }
}

impl Scene for Dead {
    fn update(
        mut self: Box<Self>,
        ctx: &mut EngineContext,
    ) -> Result<Box<dyn Scene>, Box<dyn Error>> {
        if ctx.keys.space == KeyState::Click {
            Ok(Box::new(Menu::new(self.atlas, self.grr)))
        } else {
            Ok(self)
        }
    }

    fn render(&mut self) {
        self.grr.clear(&mut ());
        self.bg.draw(self.bird.pos.x, &mut self.grr);
        self.ui.draw_game_over(&mut self.grr);
        self.bird.draw(&mut self.grr);
        self.grr.render(camera(), &mut ());
        self.grr.finish()
    }

    fn span(&self) -> Span {
        info_span!("Menu")
    }
}
