use super::*;

struct SnakeSpeedModifier {
    time_left: f64,
    multiplier: f64,
}

pub struct Game {
    id_gen: IdGen,
    ctx: Context,
    map: Map,
    ai_state: snake::AiState,
    camera: Camera2d,
    player_id: Option<Id>,
    held_item: Option<Item>,
    next_snake_move: f64,
    next_player_move: f64,
    next_item: f64,
    snake_speed_modifier: Option<SnakeSpeedModifier>,
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
                fov: map.size().y as f32 + ctx.assets.config.camera_margin * 2.0,
            },
            map,
            ai_state: snake::AiState::new(),
            next_snake_move: 0.0,
            next_player_move: 0.0,
            next_item: 0.0,
            held_item: None,
            player_id: None,
            snake_speed_modifier: None,
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

    pub fn spawn_item(&mut self) {
        let (pos, _) = self
            .map
            .iter()
            .filter(|(_, cell)| matches!(cell, MapCell::Empty))
            .choose(&mut thread_rng())
            .unwrap();
        let weights = &self.ctx.assets.config.weights;
        self.map[pos] = MapCell::Item(
            [
                (weights.food, Item::Food),
                (weights.reverse, Item::Reverse),
                (weights.snake_speed_up, Item::SnakeSpeedUp),
                (weights.snake_speed_down, Item::SnakeSpeedDown),
            ]
            .choose_weighted(&mut thread_rng(), |&(weight, _)| weight)
            .unwrap()
            .1
            .clone(),
        );
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

    fn move_player(&mut self, dir: vec2<isize>) {
        if self.next_player_move > 0.0 {
            return;
        }
        self.next_player_move = 1.0 / self.ctx.assets.config.player_speed;
        let player_pos = self.map.iter().find(|(_, cell)| {
            if let MapCell::Player(id) = cell {
                Some(*id) == self.player_id
            } else {
                false
            }
        });
        if let Some((pos, _)) = player_pos {
            let new_pos = self.map.add_dir(pos, dir);
            match &self.map[new_pos] {
                MapCell::Empty => {}
                MapCell::Item(item) => {
                    if matches!(item, Item::Food) {
                        return;
                    }
                    if self.held_item.is_some() {
                        return;
                    }
                    self.held_item = Some(item.clone());
                }
                _ => return,
            }
            let cell = mem::take(&mut self.map[pos]);
            self.map[new_pos] = cell;
        }
    }

    fn use_item(&mut self, item: Item) {
        match item {
            Item::Food => {}
            Item::Reverse => {
                let head_idx = match self.map[snake::head(&self.map)] {
                    MapCell::SnakePart(idx) => idx,
                    _ => unreachable!(),
                };
                for (_pos, cell) in self.map.iter_mut() {
                    match *cell {
                        MapCell::SnakePart(idx) => *cell = MapCell::SnakePart(head_idx - idx),
                        _ => {}
                    }
                }
                self.ai_state = snake::AiState::new();
            }
            Item::SnakeSpeedUp => {
                self.snake_speed_modifier = Some(SnakeSpeedModifier {
                    time_left: self.ctx.assets.config.items.snake_speed.time,
                    multiplier: self.ctx.assets.config.items.snake_speed.multiplier,
                });
            }
            Item::SnakeSpeedDown => {
                self.snake_speed_modifier = Some(SnakeSpeedModifier {
                    time_left: self.ctx.assets.config.items.snake_speed.time,
                    multiplier: 1.0 / self.ctx.assets.config.items.snake_speed.multiplier,
                });
            }
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        if self.player_id.is_none() {
            self.player_id = Some(self.spawn_player());
        }
        if let Some(modifier) = &mut self.snake_speed_modifier {
            modifier.time_left -= delta_time;
            if modifier.time_left < 0.0 {
                self.snake_speed_modifier = None;
            }
        }
        self.next_snake_move -= delta_time;
        if self.next_snake_move < 0.0 {
            self.next_snake_move = 1.0
                / self.ctx.assets.config.snake_speed
                / self
                    .snake_speed_modifier
                    .as_ref()
                    .map_or(1.0, |modifier| modifier.multiplier);
            if let Some(item) =
                snake::go_ai(&self.ctx.assets.config, &mut self.map, &mut self.ai_state)
            {
                self.use_item(item);
            }
        }
        self.next_item -= delta_time;
        if self.next_item < 0.0 {
            self.next_item = self.ctx.assets.config.new_item_time;
            self.spawn_item();
        }
        self.next_player_move -= delta_time;
        if self.next_player_move < 0.0 {
            let mut dir = None;
            if self
                .ctx
                .assets
                .config
                .controls
                .left
                .iter()
                .any(|&key| self.ctx.geng.window().is_key_pressed(key))
            {
                dir = Some(vec2(-1, 0));
            }
            if self
                .ctx
                .assets
                .config
                .controls
                .right
                .iter()
                .any(|&key| self.ctx.geng.window().is_key_pressed(key))
            {
                dir = Some(vec2(1, 0));
            }
            if self
                .ctx
                .assets
                .config
                .controls
                .up
                .iter()
                .any(|&key| self.ctx.geng.window().is_key_pressed(key))
            {
                dir = Some(vec2(0, 1));
            }
            if self
                .ctx
                .assets
                .config
                .controls
                .down
                .iter()
                .any(|&key| self.ctx.geng.window().is_key_pressed(key))
            {
                dir = Some(vec2(0, -1));
            }
            if let Some(dir) = dir {
                self.move_player(dir);
            }
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key }
                if self.ctx.assets.config.controls.use_item.contains(&key) =>
            {
                if let Some(item) = self.held_item.take() {
                    self.use_item(item);
                }
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } if self.ctx.cli.editor => {
                if let Some(pos) = self.hovered_cell() {
                    self.map[pos] = MapCell::Wall;
                }
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Middle,
            } if self.ctx.cli.editor => {
                if let Some(pos) = self.hovered_cell() {
                    let _ = snake::go_to(&mut self.map, pos);
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
            geng::Event::KeyPress { key: geng::Key::R } if self.ctx.cli.editor => {
                self.player_id = Some(self.spawn_player());
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
            } if self.ctx.cli.editor => {
                self.spawn_item();
            }
            geng::Event::KeyPress { key }
                if self.ctx.assets.config.controls.left.contains(&key) =>
            {
                self.move_player(vec2(-1, 0));
            }
            geng::Event::KeyPress { key }
                if self.ctx.assets.config.controls.right.contains(&key) =>
            {
                self.move_player(vec2(1, 0));
            }
            geng::Event::KeyPress { key } if self.ctx.assets.config.controls.up.contains(&key) => {
                self.move_player(vec2(0, 1));
            }
            geng::Event::KeyPress { key }
                if self.ctx.assets.config.controls.down.contains(&key) =>
            {
                self.move_player(vec2(0, -1));
            }
            _ => {}
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let colors = &self.ctx.assets.config.colors;
        ugli::clear(framebuffer, Some(colors.background), None, None);
        let snake_head = snake::head(&self.map);
        let snake_head_idx = match self.map[snake_head] {
            MapCell::SnakePart(idx) => idx,
            _ => unreachable!(),
        };
        let snake_tail = snake::tail(&self.map);
        let snake_tail_idx = match self.map[snake_tail] {
            MapCell::SnakePart(idx) => idx,
            _ => unreachable!(),
        };
        let item_color = |item: &Item| match item {
            Item::Food => colors.food,
            Item::Reverse => colors.reverse,
            Item::SnakeSpeedUp => colors.snake_speed_up,
            Item::SnakeSpeedDown => colors.snake_speed_down,
        };
        for (pos, cell) in self.map.iter() {
            if self.map.distance(pos, snake_head) <= self.ctx.assets.config.snake_vision {
                self.ctx.geng.draw2d().draw2d(
                    framebuffer,
                    &self.camera,
                    &draw2d::Quad::new(
                        Aabb2::point(pos.map(|x| x as f32)).extend_uniform(0.5),
                        colors.snake_vision,
                    ),
                );
            }

            let color = match *cell {
                MapCell::Empty => continue,
                MapCell::Wall => colors.wall,
                MapCell::Player(_id) => colors.player,
                MapCell::SnakePart(idx) => {
                    if idx == snake_head_idx {
                        colors.snake_head
                    } else if idx == snake_tail_idx {
                        colors.snake_tail
                    } else {
                        colors.snake[(snake_head_idx - idx) as usize % colors.snake.len()]
                    }
                }
                MapCell::Item(ref item) => item_color(item),
            };
            let mut aabb = Aabb2::point(pos.map(|x| x as f32))
                .extend_uniform(0.5 - self.ctx.assets.config.cell_margin);
            let need_extend = |next: vec2<usize>| match (cell, &self.map[next]) {
                (MapCell::Wall, MapCell::Wall) => true,
                (&MapCell::SnakePart(prev), &MapCell::SnakePart(next)) => {
                    prev + 1 == next || next + 1 == prev
                }
                _ => false,
            };
            if need_extend(self.map.add_dir(pos, vec2(-1, 0))) {
                aabb = aabb.extend_left(self.ctx.assets.config.cell_margin);
            }
            if need_extend(self.map.add_dir(pos, vec2(1, 0))) {
                aabb = aabb.extend_right(self.ctx.assets.config.cell_margin);
            }
            if need_extend(self.map.add_dir(pos, vec2(0, -1))) {
                aabb = aabb.extend_down(self.ctx.assets.config.cell_margin);
            }
            if need_extend(self.map.add_dir(pos, vec2(0, 1))) {
                aabb = aabb.extend_up(self.ctx.assets.config.cell_margin);
            }
            self.ctx.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Quad::new(aabb, color),
            );
        }
        if let Some(item) = &self.held_item {
            self.ctx.geng.draw2d().draw2d(
                framebuffer,
                &geng::PixelPerfectCamera,
                &draw2d::Quad::new(
                    Aabb2::ZERO.extend_positive(vec2::splat(32.0)),
                    item_color(item),
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
