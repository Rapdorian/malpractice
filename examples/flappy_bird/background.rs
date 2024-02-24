use glam::{Mat4, Quat, Vec2, Vec3};
use mint::ColumnMatrix4;
use rivik::render::grr::{GrrRender, SpriteHandle};
use rivik::render::Renderer;

pub struct Background {
    main: Vec<SpriteHandle>,
    reflection: Vec<SpriteHandle>,
}

impl Background {
    pub fn new(img: &SpriteHandle) -> Self {
        Self {
            main: vec![
                img.sub_sprite([1., 28.], [40., 31.]).recenter([0.0, 13.0]),
                img.sub_sprite([42., 28.], [40., 31.]).recenter([0.0, 13.0]),
                img.sub_sprite([83., 28.], [40., 31.]).recenter([0.0, 13.0]),
                img.sub_sprite([124., 28.], [40., 31.])
                    .recenter([0.0, 13.0]),
                img.sub_sprite([1., 60.], [40., 14.]).recenter([0.0, 13.0]),
                img.sub_sprite([42., 60.], [40., 14.]).recenter([0.0, 13.0]),
                img.sub_sprite([83., 60.], [40., 14.]).recenter([0.0, 13.0]),
                img.sub_sprite([1., 20.], [40., 7.]).recenter([0.0, -3.0]),
            ],
            reflection: vec![
                img.sub_sprite([1., 83.], [40., 13.]).recenter([0.0, -21.0]),
                img.sub_sprite([42., 83.], [40., 13.])
                    .recenter([0.0, -21.0]),
                img.sub_sprite([83., 83.], [40., 13.])
                    .recenter([0.0, -21.0]),
                img.sub_sprite([124., 83.], [40., 13.])
                    .recenter([0.0, -21.0]),
                img.sub_sprite([1., 76.], [40., 7.]).recenter([0.0, -30.0]),
                img.sub_sprite([42., 76.], [40., 7.]).recenter([0.0, -30.0]),
                img.sub_sprite([83., 76.], [40., 7.]).recenter([0.0, -30.0]),
                img.sub_sprite([42., 19.], [40., 7.]).recenter([0.0, -10.0]),
            ],
        }
    }

    pub fn draw(&mut self, offset: f32, rend: &mut GrrRender) {
        let bg_len = self.main.len() as f32;
        for (i, (top, bottom)) in self
            .main
            .iter_mut()
            .zip(self.reflection.iter_mut())
            .enumerate()
        {
            // set parallax scroll value
            let scroll = Vec2::new(offset, 0.0) * (i as f32 / bg_len) * 0.003;
            top.set_scroll(scroll);
            bottom.set_scroll(scroll);
            let transform: ColumnMatrix4<f32> = Mat4::from_scale_rotation_translation(
                Vec3::new(5.0, 4.0, 1.0),
                Quat::IDENTITY,
                Vec3::new(0.0, -20.0, 0.0),
            )
            .into();

            rend.draw_sprite(top, transform);
            rend.draw_sprite(bottom, transform);
        }
    }
}
