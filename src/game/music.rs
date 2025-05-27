use std::{
    io::Cursor,
    sync::WaitTimeoutResult,
    time::{Duration, SystemTime},
};

use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    clock::{ClockHandle, ClockSpeed},
    sound::static_sound::StaticSoundData,
};

pub struct MusicManager {
    audio_manager: AudioManager,
    music_data: StaticSoundData,
    start_stamp: SystemTime,
    pub time: Duration,
    pub playing: bool,
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
            start_stamp: SystemTime::now(),
            time: Duration::from_secs(0),
            playing: true,
        }
    }

    pub fn play(&mut self) {
        self.audio_manager.play(self.music_data.clone()).unwrap();
        self.start_stamp = SystemTime::now();
    }

    pub fn update(&mut self) {
        self.time = self.start_stamp.elapsed().unwrap();
    }
}
