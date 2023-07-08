use geng::prelude::*;

mod game;
mod id;
mod map;
mod snake;

use game::*;
use id::*;
use map::*;

#[derive(Deserialize)]
pub struct Colors {
    pub background: Rgba<f32>,
    pub wall: Rgba<f32>,
    pub player: Rgba<f32>,
    pub snake_head: Rgba<f32>,
    pub snake_tail: Rgba<f32>,
    pub snake: Vec<Rgba<f32>>,
    pub hovered: Rgba<f32>,
    pub snake_vision: Rgba<f32>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub tps: f64,
    pub cell_margin: f32,
    pub camera_margin: f32,
    pub snake_vision: usize,
    pub colors: Colors,
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
