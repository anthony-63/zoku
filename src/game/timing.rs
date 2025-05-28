use crate::content::beatmap::TimingPoint;

use super::music::MusicManager;

pub struct TimingPointManager {
    pub current_uninherited: TimingPoint,
    pub current_inherited: Option<TimingPoint>,
    points: Vec<TimingPoint>,
    index: usize,
}

impl TimingPointManager {
    pub fn new(points: Vec<TimingPoint>) -> Self {
        let first_uninherited = points.iter()
            .find(|p| p.ms_per_beat > 0.0)
            .unwrap_or(&points[0])
            .clone();

        Self {
            current_uninherited: first_uninherited,
            current_inherited: None,
            points,
            index: 0,
        }
    }

    pub fn update(&mut self, music: &MusicManager) {
        if self.index >= self.points.len() {
            return;
        }

        let curr = &self.points[self.index];
        if music.time.as_millis() as i32 >= curr.offset {
            if curr.ms_per_beat > 0.0 {
                self.current_uninherited = curr.clone();
                self.current_inherited = None;
            } else {
                self.current_inherited = Some(curr.clone());
            }
            self.index += 1;
        }
    }

    pub fn velocity_multiplier(&self) -> f32 {
        if let Some(inherited) = &self.current_inherited {
            -inherited.ms_per_beat / 100.0
        } else {
            1.0
        }
    }

    pub fn beat_length(&self) -> f32 {
        self.current_uninherited.ms_per_beat
    }

    pub fn bpm(&self) -> f32 {
        60000.0 / self.current_uninherited.ms_per_beat
    }
}