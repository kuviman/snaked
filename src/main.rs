use geng::prelude::*;

struct Game {
    geng: Geng,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        Self { geng: geng.clone() }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

fn main() {
    Geng::run("Snaked", |geng| async move {
        geng.run_state(Game::new(&geng)).await;
    });
}
