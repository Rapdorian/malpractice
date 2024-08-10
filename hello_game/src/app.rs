use std::mem;
use std::sync::Arc;

use glam::Vec3;
use rivik::{
    components::Transform,
    input::{ActionHandler, InputType},
    render::Surface,
    ActiveRivik, Rivik, Stage,
};
use winit::window::WindowAttributes;
use winit::{keyboard::KeyCode, window::Window};

struct App {
    world: hecs::World,
    camera: Transform,
    speed: f32,
    radius: f32,
    delay: f32,
}

impl Stage<GameEvent> for App {
    fn render(&self, surface: &Surface, dt: f32) {
        let Some(frame) = surface.next_frame(dt) else {
            return;
        };

        let _bench = rivik::bench::start("render-logic");
        frame
            .clear_screen(0.5, 0.5, 0.5)
            .draw_sprites_from_world(&self.world, self.camera)
            .submit();
    }

    fn tick(&mut self, input: &mut ActionHandler<GameEvent>, step: f32) {
        let _bench = rivik::bench::start("user-update");
        // pretend to run a long computation
        std::thread::sleep_ms(self.delay as u32);

        for (_, (t, trans)) in self.world.query_mut::<(&mut f32, &mut Transform)>() {
            trans.position.y = t.sin() * self.radius;
            trans.position.x = t.cos() * self.radius;
            *t += 0.4 * step * self.speed;
        }
    }

    fn ui(&mut self, ui: &egui::Context, input: &mut ActionHandler<GameEvent>) {
        let _bench = rivik::bench::start("UI");
        egui::Window::new("Properties").show(ui, |ui| {
            ui.add(egui::Slider::new(&mut self.speed, 0.0..=10.0).text("Speed"));
            ui.add(egui::Slider::new(&mut self.radius, 10.0..=300.0).text("Radius"));
            ui.add(egui::Slider::new(&mut self.delay, 0.0..=100.0).text("Delay"));
            ui.separator();
            rivik::bench::egui_report(ui);
        });
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum GameEvent {
    Foo,
    Bar,
    Report,
}

pub fn run(rivik: &mut ActiveRivik<GameEvent>) {
    rivik
        .input()
        .map(InputType::Key(KeyCode::Space), GameEvent::Foo);

    let mut stage = rivik.open(WindowAttributes::default());

    let mut world = hecs::World::new();

    /*world.spawn((
        stage.surface().load_sprite("sprite.toml", "foo"),
        Transform::default().with_pos(60.0, 0.0, 0.0),
        0.0_f32,
        GameEvent::Foo,
    ));*/

    stage.build(App {
        world,
        camera: Transform::default()
            .with_scale(0.003, 0.003, 0.003)
            .with_pos(-0.2, -0.2, 0.0),
        speed: 1.0,
        radius: 200.0,
        delay: 1.0,
    });
}
