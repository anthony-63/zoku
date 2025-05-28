use std::{io::Write, path::Path};

use content::{beatmap::formats::osu::OsuParser, skin::Skin};

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
    let map_path = std::env::args()
        .skip(1)
        .next()
        .expect("Expected map path as argument");
    let map = OsuParser::from_osz(map_path);
    println!(
        "Select a difficulty for: {}",
        map.difficulties[0].metadata.title
    );
    for (i, diff) in map.difficulties.iter().enumerate() {
        println!("{}. {}", i + 1, diff.metadata.version);
    }

    print!("> ");
    std::io::stdout().flush().unwrap();
    let mut inp = String::new();
    std::io::stdin()
        .read_line(&mut inp)
        .expect("Expected input");
    let selected: usize = inp.trim().parse().expect("Expected number input");

    if selected < 1 || selected > map.difficulties.len() {
        println!("Input not in correct range");
    }

    let skin = Skin::load(Path::new("skin/")).await;
    let mut game = Game::new(&skin, &map.difficulties[selected - 1]);
    game.play().await;
}
