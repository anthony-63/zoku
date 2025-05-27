use std::collections::VecDeque;

use crate::content::beatmap::{DifficultySection, HitCircle, HitObject, Slider, Spinner};

use super::music::MusicManager;

pub struct NoteSpawner {
    objs: Vec<HitObject>,
    index: usize,
    preemt: f32,
    render_queue: VecDeque<RenderableObject>,
    despawn_count: usize,
}

pub enum RenderableObject {
    Circle(RenderableCircle),
    Slider(RenderableSlider),
    Spinner(RenderableSpinner),
}

pub struct RenderableCircle {
    time: f32,
}

pub struct RenderableSlider {
    time: f32,
}

pub struct RenderableSpinner {
    time: f32,
}

impl NoteSpawner {
    pub fn new(hit_objects: Vec<HitObject>, difficulty: &DifficultySection) -> Self {
        let preemt = if difficulty.approach_rate == 5. {
            1200.
        } else if difficulty.approach_rate < 5. {
            1200. + 600. * (5. - difficulty.approach_rate) / 5.
        } else if difficulty.approach_rate > 5. {
            1200. - 750. * (difficulty.approach_rate - 5.) / 5.
        } else {
            panic!("sometihgn went very very wrong!");
        };

        let mut render_queue = VecDeque::new();
        render_queue.make_contiguous();

        Self {
            objs: hit_objects,
            render_queue,
            index: 0,
            preemt,
            despawn_count: 0,
        }
    }

    pub fn update(&mut self, music: &MusicManager) {
        if self.index >= self.objs.len() {
            return;
        }

        let curr = &self.objs[self.index];
        match curr {
            HitObject::HitCircle(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                } else {
                    return;
                }
                println!("Spawn Circle");
                self.render_queue
                    .push_back(RenderableObject::Circle(RenderableCircle {
                        time: obj.time as f32,
                    }))
            }
            HitObject::Slider(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                } else {
                    return;
                }
                println!("Spawn Slider");
                self.render_queue
                    .push_back(RenderableObject::Slider(RenderableSlider {
                        time: obj.time as f32,
                    }))
            }
            HitObject::Spinner(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                } else {
                    return;
                }
                println!("Spawn Spinner");

                self.render_queue
                    .push_back(RenderableObject::Spinner(RenderableSpinner {
                        time: obj.time as f32,
                    }))
            }
            _ => {}
        }

        for o in self.render_queue.iter() {
            match o {
                RenderableObject::Circle(obj) => {
                    if obj.time <= music.time.as_millis() as f32 {
                        self.despawn_count += 1;
                        println!("Hit Circle");
                    }
                }
                RenderableObject::Slider(obj) => {
                    if obj.time <= music.time.as_millis() as f32 {
                        self.despawn_count += 1;
                        println!("Hit Slider");
                    }
                }
                RenderableObject::Spinner(obj) => {
                    if obj.time <= music.time.as_millis() as f32 {
                        self.despawn_count += 1;
                        println!("Hit Spinner");
                    }
                }
            }
        }

        self.render_queue.drain(0..self.despawn_count);
        self.despawn_count = 0;
    }

    pub fn render(&mut self) {}
}
