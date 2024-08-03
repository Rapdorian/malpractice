use std::sync::Arc;

use glam::Vec3;
use rivik::{
    components::Transform,
    input::{ActionHandler, InputType},
    render::Surface,
    Rivik, Stage,
};
use winit::{keyboard::KeyCode, window::Window};

struct App {
    surface: Surface,
    window: Arc<Window>,
    world: hecs::World,
    camera: Transform,
}

impl Stage<GameEvent> for App {
    fn render(&self, dt: f32) {
        self.surface
            .next_frame(dt)
            .clear_screen(0.5, 0.5, 0.5)
            .draw_sprites_from_world(&self.world, self.camera)
            .submit();
    }

    fn tick(&mut self, input: &mut ActionHandler<GameEvent>, step: f32) {
        for (_, (t, trans)) in self.world.query_mut::<(&mut f32, &mut Transform,)>() {
            trans.position.y = t.sin() * 200.0;
            trans.position.x = t.cos() * 200.0;
            *t += 0.4 * step;

        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
enum GameEvent {
    Foo,
    Bar,
    RemapBar,
}

fn main() {
    env_logger::init();
    Rivik::run(|mut rivik| {
        let window = rivik.new_window();
        let surface = Surface::new(&window);
        let mut world = hecs::World::new();
        world.spawn((
            surface.load_sprite("sprite.toml", "foo"),
            Transform::default().with_pos(60.0, 0.0, 0.0),
            0.0_f32,
            GameEvent::Foo,
        ));

        rivik.register_stage(
            &window,
            App {
                surface,
                window: Arc::clone(&window),
                world,
                camera: Transform::default()
                    .with_scale(0.003, 0.003, 0.003)
                    .with_pos(-0.2, -0.2, 0.0),
            },
        );
    });
}
