use crate::background::Background;
use crate::bird::Bird;
use crate::game::Game;
use crate::ui::UIElements;
use crate::{camera, SPEED};
use rivik::input::KeyState;
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;
use rivik::{EngineContext, Scene};
use std::error::Error;
use tracing::{info_span, Span};
use winit::keyboard::KeyCode;

pub struct Menu {
    atlas: SpriteHandle,
    ui: UIElements,
    bg: Background,
    bird: Bird,
    grr: GrrRender,
}

impl Menu {
    pub fn new(img: SpriteHandle, grr: GrrRender) -> Self {
        Self {
            ui: UIElements::new(&img),
            bg: Background::new(&img),
            bird: Bird::new(&img),
            atlas: img,
            grr,
        }
    }
}

impl Scene for Menu {
    fn update(
        mut self: Box<Self>,
        ctx: &mut EngineContext,
    ) -> Result<Box<dyn Scene>, Box<dyn Error>> {
        self.bird.pos.x += SPEED;
        if let KeyState::Click = ctx.keys.space {
            Ok(Box::new(Game::new(self.bird.pos.x, self.atlas, self.grr)))
        } else {
            Ok(self)
        }
    }

    fn render(&mut self) {
        self.grr.clear(&mut ());
        self.bg.draw(self.bird.pos.x, &mut self.grr);
        self.ui.draw_start(&mut self.grr);
        self.bird.draw(&mut self.grr);
        self.grr.render(camera(), &mut ());
        self.grr.finish()
    }

    fn span(&self) -> Span {
        info_span!("Menu")
    }
}
