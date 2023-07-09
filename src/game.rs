use super::*;

struct SnakeSpeedModifier {
    time_left: f64,
    multiplier: f64,
}

#[derive(Debug)]
pub struct Results {
    time: f64,
    win: bool,
}

pub struct Game {
    id_gen: IdGen,
    time: f64,
    ctx: Context,
    map: Map,
    ai_state: HashMap<Id, snake::AiState>,
    camera: Camera2d,
    player_id: Option<Id>,
    held_item: Option<Item>,
    next_snake_move: HashMap<Id, f64>,
    next_player_move: f64,
    next_item: f64,
    snake_grow: HashMap<Id, usize>,
    snake_speed_modifier: HashMap<Id, SnakeSpeedModifier>,
    results: Option<Results>,
    alternate_move: usize,
}

impl Game {
    pub fn new(ctx: &Context) -> Self {
        let mut map = Map::parse(&ctx.assets.map);
        let (pos, _) = map
            .iter()
            .filter(|(_, cell)| matches!(cell, MapCell::Empty))
            .choose(&mut thread_rng())
            .unwrap();
        let mut id_gen = IdGen::new();
        let snake_id = id_gen.gen();
        map[pos] = MapCell::SnakePart {
            snake_id,
            segment_index: 0,
        };

        Self {
            id_gen,
            ctx: ctx.clone(),
            camera: Camera2d {
                center: map.size().map(|x| x as f32) / 2.0,
                rotation: Angle::ZERO,
                fov: map.size().y as f32 + ctx.assets.config.camera_margin * 2.0,
            },
            map,
            ai_state: HashMap::new(),
            next_snake_move: {
                let mut res = HashMap::new();
                res.insert(snake_id, ctx.assets.config.snake_wake_up_time);
                res
            },
            next_player_move: 0.0,
            next_item: 0.0,
            held_item: None,
            player_id: None,
            snake_speed_modifier: default(),
            results: None,
            snake_grow: {
                let mut res = HashMap::new();
                res.insert(snake_id, ctx.assets.config.start_snake_size - 1);
                res
            },
            alternate_move: 0,
            time: 0.0,
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
        let num_items = self
            .map
            .iter()
            .filter(|(_, cell)| matches!(cell, MapCell::Item(_)))
            .count();
        if num_items >= self.ctx.assets.config.max_items {
            return;
        }
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
                (weights.snake_split, Item::SnakeSplit),
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

    fn snake_ids(&self) -> HashSet<Id> {
        self.map
            .iter()
            .filter_map(|(_, cell)| {
                if let MapCell::SnakePart { snake_id, .. } = cell {
                    Some(*snake_id)
                } else {
                    None
                }
            })
            .collect()
    }

    fn use_item(&mut self, snake_id: Option<Id>, item: Item) {
        // TODO: instead of rng closest to player maybe"????
        let snake_id = snake_id.or(self.snake_ids().into_iter().choose(&mut thread_rng()));
        let Some(id) = snake_id else { return };
        match item {
            Item::Food => {
                *self.snake_grow.entry(id).or_default() += self.ctx.assets.config.food_value - 1;
            }
            Item::Reverse => {
                let head_idx = match self.map[snake::head(id, &self.map)] {
                    MapCell::SnakePart {
                        snake_id,
                        segment_index,
                    } if snake_id == id => segment_index,
                    _ => unreachable!(),
                };
                for (_pos, cell) in self.map.iter_mut() {
                    match *cell {
                        MapCell::SnakePart {
                            snake_id,
                            segment_index,
                        } if snake_id == id => {
                            *cell = MapCell::SnakePart {
                                snake_id,
                                segment_index: head_idx - segment_index,
                            }
                        }
                        _ => {}
                    }
                }
                self.ai_state.remove(&id);
            }
            Item::SnakeSpeedUp => {
                self.snake_speed_modifier.insert(
                    id,
                    SnakeSpeedModifier {
                        time_left: self.ctx.assets.config.items.snake_speed.time,
                        multiplier: self.ctx.assets.config.items.snake_speed.multiplier,
                    },
                );
            }
            Item::SnakeSpeedDown => {
                self.snake_speed_modifier.insert(
                    id,
                    SnakeSpeedModifier {
                        time_left: self.ctx.assets.config.items.snake_speed.time,
                        multiplier: 1.0 / self.ctx.assets.config.items.snake_speed.multiplier,
                    },
                );
            }
            Item::SnakeSplit => {
                let mut min_max: HashMap<Id, (u32, u32)> = HashMap::new();
                for (_pos, cell) in self.map.iter() {
                    if let MapCell::SnakePart {
                        snake_id,
                        segment_index,
                    } = *cell
                    {
                        let cur = min_max
                            .entry(snake_id)
                            .or_insert((segment_index, segment_index));
                        cur.0 = cur.0.min(segment_index);
                        cur.1 = cur.1.max(segment_index);
                    }
                }
                let new_snake_ids: HashMap<Id, Id> = min_max
                    .keys()
                    .copied()
                    .map(|id| (id, self.id_gen.gen()))
                    .collect();
                self.next_snake_move.insert(
                    new_snake_ids[&id],
                    self.ctx.assets.config.snake_wake_up_time,
                );
                for (_pos, cell) in self.map.iter_mut() {
                    if let MapCell::SnakePart {
                        snake_id,
                        segment_index,
                    } = cell
                    {
                        if *snake_id == id
                            && *segment_index < (min_max[snake_id].1 + min_max[snake_id].0) / 2
                        {
                            *snake_id = new_snake_ids[snake_id];
                        }
                    }
                }
            }
        }
    }

    fn results(&self) -> Results {
        Results {
            time: self.time,
            win: self.map.iter().any(|(_, cell)| {
                if let MapCell::Player(id) = *cell {
                    Some(id) == self.player_id
                } else {
                    false
                }
            }),
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time * self.ctx.assets.config.time_scale;
        self.time += delta_time;
        if !self.ctx.cli.editor && self.results.is_none() {
            if self.player_id.is_none() {
                self.player_id = Some(self.spawn_player());
            } else {
                if !self.map.iter().any(|(_, cell)| {
                    if let MapCell::Player(id) = *cell {
                        Some(id) == self.player_id
                    } else {
                        false
                    }
                }) {
                    self.results = Some(self.results());
                }
            }
        }

        for id in self.snake_ids() {
            if let Some(modifier) = self.snake_speed_modifier.get_mut(&id) {
                modifier.time_left -= delta_time;
                if modifier.time_left < 0.0 {
                    self.snake_speed_modifier.remove(&id);
                }
            }
            let next_move = self.next_snake_move.entry(id).or_default();
            *next_move -= delta_time;
            if *next_move < 0.0 {
                *next_move = 1.0
                    / self.ctx.assets.config.snake_speed
                    / self
                        .snake_speed_modifier
                        .get(&id)
                        .map_or(1.0, |modifier| modifier.multiplier);

                let snake_grow = self.snake_grow.entry(id).or_default();
                match snake::go_ai(
                    id,
                    &self.ctx.assets.config,
                    &mut self.map,
                    self.ai_state.entry(id).or_default(),
                    *snake_grow == 0,
                ) {
                    Ok(Some(item)) => {
                        self.use_item(Some(id), item);
                        self.spawn_item();
                    }
                    Ok(None) => {}
                    Err(()) => {
                        for (_, cell) in self.map.iter_mut() {
                            if let MapCell::SnakePart { snake_id, .. } = *cell {
                                if snake_id == id {
                                    *cell = MapCell::Empty;
                                }
                            }
                        }
                    }
                }
                let snake_grow = self.snake_grow.entry(id).or_default();
                if *snake_grow > 0 {
                    *snake_grow -= 1;
                }
            }
        }

        self.next_item -= delta_time;
        if self.next_item < 0.0 {
            self.next_item = self.ctx.assets.config.new_item_time;
            self.spawn_item();
        }
        self.next_player_move -= delta_time;
        if self.next_player_move < 0.0 {
            let mut dir = Vec::new();
            if self
                .ctx
                .assets
                .config
                .controls
                .left
                .iter()
                .any(|&key| self.ctx.geng.window().is_key_pressed(key))
            {
                dir.push(vec2(-1, 0));
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
                dir.push(vec2(1, 0));
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
                dir.push(vec2(0, 1));
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
                dir.push(vec2(0, -1));
            }
            if !dir.is_empty() {
                let dir = dir[self.alternate_move % dir.len()];
                self.move_player(dir);
            }
            self.alternate_move += 1;
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key }
                if self.ctx.assets.config.controls.use_item.contains(&key) =>
            {
                if let Some(item) = self.held_item.take() {
                    self.use_item(None, item);
                }
            }
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
        let item_color = |item: &Item| match item {
            Item::Food => colors.food,
            Item::Reverse => colors.reverse,
            Item::SnakeSpeedUp => colors.snake_speed_up,
            Item::SnakeSpeedDown => colors.snake_speed_down,
            Item::SnakeSplit => colors.snake_split,
        };
        let snake_ends: HashMap<Id, (vec2<usize>, vec2<usize>)> = self
            .snake_ids()
            .into_iter()
            .map(|id| (id, (snake::head(id, &self.map), snake::tail(id, &self.map))))
            .collect();
        for (pos, cell) in self.map.iter() {
            // if snake_ends.values().any(|&(head, _)| {
            //     self.map.distance(pos, head) <= self.ctx.assets.config.snake_vision
            // }) {
            //     self.ctx.geng.draw2d().draw2d(
            //         framebuffer,
            //         &self.camera,
            //         &draw2d::Quad::new(
            //             Aabb2::point(pos.map(|x| x as f32)).extend_uniform(0.5),
            //             colors.snake_vision,
            //         ),
            //     );
            // }

            let color = match *cell {
                MapCell::Empty => continue,
                MapCell::Wall => colors.wall,
                MapCell::Player(_id) => colors.player,
                MapCell::SnakePart {
                    snake_id,
                    segment_index: idx,
                } => {
                    if pos == snake_ends[&snake_id].0 {
                        colors.snake_head
                    } else if pos == snake_ends[&snake_id].1 {
                        colors.snake_tail
                    } else {
                        colors.snake[(match self.map[snake_ends[&snake_id].0] {
                            MapCell::SnakePart { segment_index, .. } => segment_index,
                            _ => unreachable!(),
                        } - idx) as usize
                            % colors.snake.len()]
                    }
                }
                MapCell::Item(ref item) => item_color(item),
            };
            let mut aabb = Aabb2::point(pos.map(|x| x as f32))
                .extend_uniform(0.5 - self.ctx.assets.config.cell_margin);
            let need_extend = |next: vec2<usize>| match (cell, &self.map[next]) {
                (MapCell::Wall, MapCell::Wall) => true,
                (
                    &MapCell::SnakePart {
                        snake_id: prev_id,
                        segment_index: prev,
                    },
                    &MapCell::SnakePart {
                        snake_id: next_id,
                        segment_index: next,
                    },
                ) if prev_id == next_id => prev + 1 == next || next + 1 == prev,
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
