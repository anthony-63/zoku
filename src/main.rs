use content::beatmap::formats::osu::OsuParser;

use game::Game;
use macroquad::prelude::*;

mod content;
mod game;

fn window_conf() -> Conf {
    Conf {
        window_title: "zoku!".into(),
        fullscreen: false,
        window_resizable: false,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let map = OsuParser::from_osz("maps/yume.osz".into());
    let diff = &map.difficulties[0];

    let mut game = Game::new(diff);
    game.play().await;
}
