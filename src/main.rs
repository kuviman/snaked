use geng::prelude::*;

struct Id(u64);

#[derive(Default)]
enum Cell {
    #[default]
    Empty,
    Wall,
    Player(Id),
    SnakePart(u32),
}

struct Map {
    cells: Vec<Vec<Cell>>,
}

impl Map {
    pub fn iter(&self) -> impl Iterator<Item = (vec2<usize>, &Cell)> + '_ {
        self.cells.iter().enumerate().flat_map(|(x, row)| {
            row.iter()
                .enumerate()
                .map(move |(y, cell)| (vec2(x, y), cell))
        })
    }
    pub fn size(&self) -> vec2<usize> {
        vec2(self.cells.len(), self.cells[0].len())
    }
    pub fn parse(s: &str) -> Self {
        Self {
            cells: {
                let mut cells: Vec<Vec<Cell>> = vec![];
                for (y, line) in s.lines().enumerate() {
                    for (x, c) in line.chars().enumerate() {
                        let cell = match c {
                            ' ' => Cell::Empty,
                            '#' => Cell::Wall,
                            _ => {
                                if let Some(x) = c.to_digit(10) {
                                    Cell::SnakePart(x)
                                } else {
                                    panic!("Unexpected character {c:?}");
                                }
                            }
                        };
                        cells.resize_with(cells.len().max(x + 1), default);
                        let row = &mut cells[x];
                        row.resize_with(row.len().max(y + 1), default);
                        row[y] = cell;
                    }
                }
                let height = cells.iter().map(|row| row.len()).max().unwrap();
                for row in &mut cells {
                    row.resize_with(height, default);
                    row.reverse();
                }
                cells
            },
        }
    }
}

struct Game {
    ctx: Context,
    map: Map,
    camera: Camera2d,
}

impl Game {
    pub fn new(ctx: &Context) -> Self {
        let map = Map::parse(&ctx.assets.map);
        Self {
            ctx: ctx.clone(),
            camera: Camera2d {
                center: map.size().map(|x| x as f32) / 2.0,
                rotation: Angle::ZERO,
                fov: map.size().y as f32 + ctx.assets.config.margin * 2.0,
            },
            map,
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let colors = &self.ctx.assets.config.colors;
        ugli::clear(framebuffer, Some(colors.background), None, None);
        for (pos, cell) in self.map.iter() {
            let color = match cell {
                Cell::Empty => continue,
                Cell::Wall => colors.wall,
                Cell::Player(_) => colors.player,
                Cell::SnakePart(_) => colors.snake,
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

#[derive(Deserialize)]
struct Colors {
    background: Rgba<f32>,
    wall: Rgba<f32>,
    player: Rgba<f32>,
    snake: Rgba<f32>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
struct Config {
    margin: f32,
    colors: Colors,
}

#[derive(geng::asset::Load)]
struct Assets {
    map: String,
    config: Config,
}

#[derive(Clone)]
struct Context {
    geng: Geng,
    assets: Rc<Assets>,
}

fn main() {
    Geng::run("Snaked", |geng| async move {
        let assets: Assets = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let ctx = Context {
            geng: geng.clone(),
            assets: Rc::new(assets),
        };
        geng.run_state(Game::new(&ctx)).await;
    });
}
