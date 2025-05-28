use music::MusicManager;
use notes::NoteSpawner;
use timing::TimingPointManager;

use crate::content::beatmap::{Difficulty, MetadataSection};

use macroquad::{miniquad::window::set_window_size, prelude::*};

mod music;
mod notes;
mod timing;

pub struct Game {
    music: MusicManager,
    notes: NoteSpawner,
    timing: TimingPointManager,
    title: String,
    playfield: Rect,
}

impl Game {
    pub fn new(difficulty: &Difficulty) -> Self {
        let music = MusicManager::new(&difficulty.audio_bytes);
        let notes = NoteSpawner::new(difficulty.hit_objects.clone(), &difficulty.difficulty);
        let timing = TimingPointManager::new(difficulty.timing_points.clone());

        let h = screen_height() * 0.8;
        let w = h * (4. / 3.);

        let playfield = Rect {
            x: screen_width() / 2. - w / 2.,
            y: screen_height() / 2. - h / 2.,
            h,
            w,
        };
        Self {
            music,
            notes,
            playfield,
            timing,
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

            self.notes.update(&self.music, &self.timing);
            self.notes.render(&self.music, self.playfield);

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

            draw_rectangle_lines(
                self.playfield.x,
                self.playfield.y,
                self.playfield.w,
                self.playfield.h,
                3.,
                WHITE,
            );
            next_frame().await;
        }
    }
}
