use std::{
    io::Cursor,
    time::{Duration, SystemTime},
};

use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    sound::static_sound::StaticSoundData,
};

use super::mods::Mods;

pub struct MusicManager {
    audio_manager: AudioManager,
    music_data: StaticSoundData,
    start_stamp: SystemTime,
    speed: f64,
    pub time: Duration,
}

impl MusicManager {
    pub fn new(audio_data: &Vec<u8>) -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Failed to create audio backend.");
        let music = StaticSoundData::from_cursor(Cursor::new(audio_data.clone()))
            .expect("Failed to create sound data for music.");

        Self {
            audio_manager: manager,
            music_data: music,
            speed: 1.0,
            start_stamp: SystemTime::now(),
            time: Duration::from_secs(0),
        }
    }

    pub fn play(&mut self, mods: &Mods) {
        let mut music = self.music_data.clone();
        if mods.dt {
            self.speed = 1.5;
            music = music.playback_rate(self.speed);
        }
        self.audio_manager.play(music).unwrap();
        self.start_stamp = SystemTime::now();
    }

    pub fn update(&mut self) {
        self.time = self.start_stamp.elapsed().unwrap().mul_f64(self.speed);
    }
}
