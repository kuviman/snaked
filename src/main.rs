use geng::prelude::*;

mod game;
mod id;
mod map;
mod snake;

use game::*;
use id::*;
use map::*;

#[derive(Deserialize)]
pub struct Weights {
    pub food: f64,
    pub reverse: f64,
    pub snake_speed_up: f64,
    pub snake_speed_down: f64,
    pub snake_split: f64,
}

#[derive(Deserialize)]
pub struct Colors {
    pub background: Rgba<f32>,
    pub wall: Rgba<f32>,
    pub player: Rgba<f32>,
    pub food: Rgba<f32>,
    pub snake_head: Rgba<f32>,
    pub snake_tail: Rgba<f32>,
    pub snake: Vec<Rgba<f32>>,
    pub hovered: Rgba<f32>,
    pub reverse: Rgba<f32>,
    pub snake_vision: Rgba<f32>,
    pub snake_speed_up: Rgba<f32>,
    pub snake_speed_down: Rgba<f32>,
    pub snake_split: Rgba<f32>,
}

#[derive(Deserialize)]
pub struct SnakeSpeedItemConfig {
    pub multiplier: f64,
    pub time: f64,
}

#[derive(Deserialize)]
pub struct ItemsConfig {
    pub snake_speed: SnakeSpeedItemConfig,
}

#[derive(Deserialize)]
pub struct Controls {
    pub use_item: Vec<geng::Key>,
    pub left: Vec<geng::Key>,
    pub right: Vec<geng::Key>,
    pub up: Vec<geng::Key>,
    pub down: Vec<geng::Key>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub start_snake_size: usize,
    pub items: ItemsConfig,
    pub snake_speed: f64,
    pub player_speed: f64,
    pub new_item_time: f64,
    pub cell_margin: f32,
    pub camera_margin: f32,
    pub snake_vision: usize,
    pub colors: Colors,
    pub controls: Controls,
    pub weights: Weights,
    pub food_value: usize,
    pub time_scale: f64,
    pub max_items: usize,
    pub snake_wake_up_time: f64,
    pub snake_reverse_speed: f64,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub map: String,
    pub config: Config,
}

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub cli: Rc<CliArgs>,
}

#[derive(clap::Parser)]
pub struct CliArgs {
    #[clap(long)]
    pub editor: bool,
}

fn main() {
    let cli: CliArgs = cli::parse();
    Geng::run("Snaked", |geng| async move {
        let assets: Assets = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let ctx = Context {
            geng: geng.clone(),
            assets: Rc::new(assets),
            cli: Rc::new(cli),
        };
        geng.run_state(Game::new(&ctx)).await;
    });
}
