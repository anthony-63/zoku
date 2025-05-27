use music::MusicManager;

use crate::content::beatmap::Difficulty;

use macroquad::prelude::*;

mod music;

pub struct Game {
    music: MusicManager,
}

impl Game {
    pub fn new(difficulty: &Difficulty) -> Self {
        let music = MusicManager::new(&difficulty.audio_bytes);

        Self { music }
    }

    pub async fn play(&mut self) {
        self.music.play();

        loop {
            next_frame().await;
        }
    }
}
