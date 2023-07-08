use super::*;

pub struct Game {
    id_gen: IdGen,
    ctx: Context,
    map: Map,
    camera: Camera2d,
}

impl Game {
    pub fn new(ctx: &Context) -> Self {
        let map = Map::parse(&ctx.assets.map);
        Self {
            id_gen: id::IdGen::new(),
            ctx: ctx.clone(),
            camera: Camera2d {
                center: map.size().map(|x| x as f32) / 2.0,
                rotation: Angle::ZERO,
                fov: map.size().y as f32 + ctx.assets.config.margin * 2.0,
            },
            map,
        }
    }

    pub fn spawn_player(&mut self) -> Id {
        let id = self.id_gen.gen();
        let (pos, _) = self
            .map
            .iter()
            .filter(|(_, cell)| matches!(cell, MapCell::Empty))
            .choose(&mut thread_rng())
            .unwrap();
        self.map[pos] = MapCell::Player(id);
        id
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyPress {
            key: geng::Key::Space,
        } = event
        {
            self.spawn_player();
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let colors = &self.ctx.assets.config.colors;
        ugli::clear(framebuffer, Some(colors.background), None, None);
        for (pos, cell) in self.map.iter() {
            let color = match cell {
                MapCell::Empty => continue,
                MapCell::Wall => colors.wall,
                MapCell::Player(_) => colors.player,
                MapCell::SnakePart(_) => colors.snake,
            };
            self.ctx.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Quad::new(
                    Aabb2::point(pos.map(|x| x as f32)).extend_uniform(0.5),
                    color,
                ),
            );
        }
    }
}
