use hello_game::app;
use rivik::Rivik;

fn main() {
    env_logger::init();
    Rivik::run(|rivik| {
        app::run(rivik);
    });
}
