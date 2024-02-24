use glam::{Mat4, Vec2, Vec2Swizzles, Vec3};
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;
use std::ops::Deref;

pub struct Bird {
    frames: [SpriteHandle; 3],
    frame: usize,
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Bird {
    pub fn new(img: &SpriteHandle) -> Self {
        Self {
            frames: [
                img.sub_sprite([35., 1.], [10., 8.]),
                img.sub_sprite([47., 1.], [10., 8.]),
                img.sub_sprite([59., 1.], [10., 8.]),
            ],
            frame: 0,
            pos: Default::default(),
            vel: Default::default(),
        }
    }

    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_vel(mut self, vel: Vec2) -> Self {
        self.vel = vel;
        self
    }

    pub fn draw(&self, rend: &mut GrrRender) {
        rend.draw_sprite(
            &self,
            Mat4::from_translation(Vec3::new(0.0, self.pos.y, 0.0)).into(),
        );
    }

    pub fn next_frame(&mut self) {
        if self.vel.y > 0.0 {
            self.frame = (self.frame + 1) % 3;
        } else {
            self.frame = 2;
        }
    }

    pub fn glide(&mut self) {
        self.frame = 2;
    }
}

impl Deref for Bird {
    type Target = SpriteHandle;

    fn deref(&self) -> &Self::Target {
        &self.frames[self.frame]
    }
}
