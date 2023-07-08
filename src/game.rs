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

    fn hovered_cell(&self) -> Option<vec2<usize>> {
        if let Some(hovered_pos) = self.ctx.geng.window().cursor_position() {
            let hovered_pos = self.camera.screen_to_world(
                self.ctx.geng.window().size().map(|x| x as f32),
                hovered_pos.map(|x| x as f32),
            );
            let hovered_pos = hovered_pos.map(|x| (x + 0.5).floor() as i32);
            if Aabb2::ZERO
                .extend_positive(self.map.size().map(|x| x as i32))
                .contains(hovered_pos)
            {
                return Some(hovered_pos.map(|x| x as usize));
            }
        }
        None
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } if self.ctx.cli.editor => {
                if let Some(pos) = self.hovered_cell() {
                    self.map[pos] = MapCell::Wall;
                }
            }
            geng::Event::CursorMove { .. } if self.ctx.cli.editor => {
                if let Some(pos) = self.hovered_cell() {
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_button_pressed(geng::MouseButton::Left)
                    {
                        self.map[pos] = MapCell::Wall;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_button_pressed(geng::MouseButton::Right)
                    {
                        self.map[pos] = MapCell::Empty;
                    }
                }
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Right,
            } if self.ctx.cli.editor => {
                if let Some(pos) = self.hovered_cell() {
                    self.map[pos] = MapCell::Empty;
                }
            }
            geng::Event::KeyPress { key: geng::Key::S }
                if self
                    .ctx
                    .geng
                    .window()
                    .is_key_pressed(geng::Key::ControlLeft)
                    && self.ctx.cli.editor =>
            {
                self.map.save(run_dir().join("assets").join("map.txt"));
            }
            geng::Event::KeyPress {
                key: geng::Key::Space,
            } => {
                self.spawn_player();
            }
            _ => {}
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
        if self.ctx.cli.editor {
            if let Some(pos) = self.hovered_cell() {
                self.ctx.geng.draw2d().draw2d(
                    framebuffer,
                    &self.camera,
                    &draw2d::Quad::new(
                        Aabb2::point(pos.map(|x| x as f32)).extend_uniform(0.5),
                        colors.hovered,
                    ),
                );
            }
        }
    }
}
