use crate::content::beatmap::TimingPoint;

use super::music::MusicManager;

pub struct TimingPointManager {
    pub current: TimingPoint,
    points: Vec<TimingPoint>,
    index: usize,
}

impl TimingPointManager {
    pub fn new(points: Vec<TimingPoint>) -> Self {
        Self {
            current: points[0].clone(),
            points,
            index: 0,
        }
    }

    pub fn update(&mut self, music: MusicManager) {
        if self.index >= self.points.len() {
            return
        }

        let curr = &self.points[self.index];
        if music.time.as_millis() as i32 >= curr.offset {
            self.current = curr.clone();
            self.index += 1;
        }
    }
}