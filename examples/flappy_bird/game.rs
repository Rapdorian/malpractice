use crate::background::Background;
use crate::bird::Bird;
use crate::dead::Dead;
use crate::ui::UIElements;
use crate::{camera, GRAVITY, JUMP, SPEED};
use glam::{Vec2, Vec3};
use grr::CullMode::Back;
use rivik::input::{KeyState, Keys};
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;
use rivik::{EngineContext, Scene};
use std::error::Error;
use tracing::{info_span, Span};
use winit::keyboard::KeyCode;
use winit::keyboard::NamedKey::JunjaMode;

pub struct Game {
    rend: GrrRender,
    img: SpriteHandle,
    ui: UIElements,
    bird: Bird,
    bg: Background,
    score: f32,
    t: usize,
}

impl Game {
    pub fn new(offset: f32, img: SpriteHandle, grr: GrrRender) -> Self {
        Self {
            rend: grr,
            ui: UIElements::new(&img),
            bird: Bird::new(&img)
                .with_pos(Vec2::new(offset, 0.0))
                .with_vel(Vec2::new(0.0, JUMP)),
            bg: Background::new(&img),
            score: 0.0,
            img,
            t: 0,
        }
    }
}

impl Scene for Game {
    fn update(
        mut self: Box<Self>,
        ctx: &mut EngineContext,
    ) -> Result<Box<dyn Scene>, Box<dyn Error>> {
        self.bird.vel.y += GRAVITY;

        match ctx.keys {
            Keys {
                space: KeyState::Click,
                l_shift: KeyState::Pressed,
                ..
            }
            | Keys {
                space: KeyState::Click,
                l_shift: KeyState::Click,
                ..
            } => self.bird.vel.y = JUMP * 2.0,
            Keys {
                space: KeyState::Click,
                ..
            } => self.bird.vel.y = JUMP,

            _ => {}
        }

        self.t += 1;
        if self.t >= 10 {
            self.t = 0;
            self.bird.next_frame();
        }
        self.bird.pos.y += self.bird.vel.y;
        self.bird.pos.x += SPEED;
        self.score += SPEED / 50.0;

        // check if dead
        if self.bird.pos.y <= -6.0 {
            Ok(Box::new(Dead::new(self.bird.pos, self.img, self.rend)))
        } else {
            Ok(self)
        }
    }

    fn render(&mut self) {
        self.rend.clear(&mut ());
        self.bg.draw(self.bird.pos.x, &mut self.rend);
        self.bird.draw(&mut self.rend);
        self.ui.draw_score(self.score as usize, &mut self.rend);
        self.rend.render(camera(), &mut ());
        self.rend.finish();
    }

    fn span(&self) -> Span {
        info_span!("Game")
    }
}
