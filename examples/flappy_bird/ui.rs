use glam::{Mat4, Vec3};
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;

pub struct UIElements {
    start: SpriteHandle,
    game_over: SpriteHandle,
    digits: [SpriteHandle; 10],
    panel: SpriteHandle,
}

impl UIElements {
    pub fn new(img: &SpriteHandle) -> Self {
        Self {
            start: img.sub_sprite([134., 1.], [29., 7.]),
            game_over: img.sub_sprite([135., 9.], [26., 15.]),
            digits: [
                img.sub_sprite([172., 1.], [5., 7.]),
                img.sub_sprite([178., 1.], [5., 7.]),
                img.sub_sprite([184., 1.], [5., 7.]),
                img.sub_sprite([190., 1.], [5., 7.]),
                img.sub_sprite([196., 1.], [5., 7.]),
                img.sub_sprite([172., 9.], [5., 7.]),
                img.sub_sprite([178., 9.], [5., 7.]),
                img.sub_sprite([184., 9.], [5., 7.]),
                img.sub_sprite([190., 9.], [5., 7.]),
                img.sub_sprite([196., 9.], [5., 7.]),
            ],
            panel: img.sub_sprite([169., 19.], [34., 19.]),
        }
    }

    pub fn draw_start(&self, rend: &mut GrrRender) {
        let ui_pos = Mat4::from_translation(Vec3::new(0.0, 20.0, 0.0)).into();
        rend.draw_sprite(&self.panel, ui_pos);
        rend.draw_sprite(&self.start, ui_pos);
    }

    pub fn draw_game_over(&self, rend: &mut GrrRender) {
        let ui_pos = Mat4::from_translation(Vec3::new(0.0, 20.0, 0.0)).into();
        rend.draw_sprite(&self.panel, ui_pos);
        rend.draw_sprite(&self.game_over, ui_pos);
    }

    pub fn draw_score(&self, score: usize, rend: &mut GrrRender) {
        let score_pos = Mat4::from_translation(Vec3::new(0.0, 90.0, 0.0)).into();
        let tens_pos = Mat4::from_translation(Vec3::new(-2.5, 90.0, 0.0)).into();
        let ones_pos = Mat4::from_translation(Vec3::new(2.5, 90.0, 0.0)).into();

        // display score
        let tens = (score / 10) % 10;
        let ones = (score) % 10;

        if tens != 0 {
            rend.draw_sprite(&self.digits[tens], tens_pos);
        }

        rend.draw_sprite(
            &self.digits[ones],
            if tens == 0 { score_pos } else { ones_pos },
        );
    }
}
