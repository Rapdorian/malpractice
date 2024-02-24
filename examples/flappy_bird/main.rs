use crate::menu::Menu;
use glam::{Mat4, Vec3};
use image::EncodableLayout;
use rivik::render::grr::GrrRender;
use rivik::render::{Camera, Renderer};
use rivik::Scene;
use std::error::Error;

pub mod background;
pub mod bird;
mod dead;
mod game;
mod menu;
pub mod ui;

pub const GRAVITY: f32 = -0.09;
pub const JUMP: f32 = 2.;
pub const SPEED: f32 = 0.5;

pub fn camera() -> Camera<f32> {
    Camera {
        transform: Mat4::from_scale(Vec3::new(0.01, 0.01, 1.0)).into(),
    }
}

/// TODO: Simple networking
/// - If no args start a server at a known port
/// - If ip in args connecct to that server as a client
fn main() {
    rivik::console::init();
    let w = rivik::window::Window::new("Flappy Bird");
    rivik::run(
        move |_ctx: &mut _| -> Result<Option<Box<dyn Scene>>, Box<dyn Error>> {
            let mut rend = GrrRender::new(&w)?;

            // load image
            let img = image::open("examples/1.png").unwrap().to_rgba8();
            let mut buf = Vec::new();
            // push header values
            buf.extend_from_slice(&(img.width() as u16).to_ne_bytes());
            buf.extend_from_slice(&(img.height() as u16).to_ne_bytes());
            buf.push(2);
            // add img bytes
            buf.extend_from_slice(img.as_bytes());
            let img = rend.upload_sprite(buf.as_slice().as_ref());

            Ok(Some(Box::new(Menu::new(img, rend))))
        },
    );
}
