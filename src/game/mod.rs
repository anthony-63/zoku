use music::MusicManager;
use notes::NoteSpawner;

use crate::content::beatmap::{Difficulty, MetadataSection};

use macroquad::prelude::*;

mod music;
mod notes;

pub struct Game {
    music: MusicManager,
    notes: NoteSpawner,
    title: String,
}

impl Game {
    pub fn new(difficulty: &Difficulty) -> Self {
        let music = MusicManager::new(&difficulty.audio_bytes);
        let notes = NoteSpawner::new(difficulty.hit_objects.clone(), &difficulty.difficulty);
        Self {
            music,
            notes,
            title: format!(
                "{}[{}]",
                difficulty.metadata.title.clone(),
                difficulty.metadata.version
            ),
        }
    }

    pub async fn play(&mut self) {
        self.music.play();

        loop {
            self.music.update();

            self.notes.update(&self.music);
            self.notes.render();

            draw_text(
                &format!("{}", self.title),
                10.,
                20.,
                23.,
                color_u8!(0xA7, 0xC7, 0xE7, 0xff),
            );

            draw_text(
                &format!("{}", self.music.time.as_secs_f64()),
                10.,
                40.,
                23.,
                color_u8!(0xFF, 0x74, 0x6C, 0xff),
            );
            next_frame().await;
        }
    }
}
