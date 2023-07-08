use geng::prelude::*;

mod game;
mod id;
mod map;

use game::*;
use id::*;
use map::*;

#[derive(Deserialize)]
pub struct Colors {
    pub background: Rgba<f32>,
    pub wall: Rgba<f32>,
    pub player: Rgba<f32>,
    pub snake: Rgba<f32>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub margin: f32,
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
