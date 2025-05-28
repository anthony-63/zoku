use macroquad::prelude::*;

use crate::content::beatmap::{DifficultySection, HitCircle, HitObject, Slider, Spinner};

use super::music::MusicManager;

pub struct NoteSpawner {
    objs: Vec<HitObject>,
    index: usize,
    preemt: f32,
    fade_in: f32,
    cs: f32,
    render_queue: Vec<RenderableObject>,
    combo: usize,
    combo_colors: Vec<(f32, f32, f32)>,
}

#[derive(Debug, Clone)]
pub enum RenderableObject {
    Circle(RenderableCircle),
    Slider(RenderableSlider),
    Spinner(RenderableSpinner),
}

#[derive(Debug, Clone)]
pub struct RenderableCircle {
    combo_color: (f32, f32, f32),
    time: f32,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
pub struct RenderableSlider {
    combo_color: (f32, f32, f32),
    time: f32,
    x: f32,
    y: f32,
    curves: Vec<Vec2>,
    repeat: usize,
}

#[derive(Debug, Clone)]
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

        let fade_in = if difficulty.approach_rate == 5. {
            800.
        } else if difficulty.approach_rate < 5. {
            800. + 400. * (5. - difficulty.approach_rate) / 5.
        } else if difficulty.approach_rate > 5. {
            800. - 500. * (difficulty.approach_rate - 5.) / 5.
        } else {
            panic!("sometihgn went very very wrong!");
        };

        Self {
            objs: hit_objects,
            render_queue: Vec::new(),
            index: 0,
            preemt,
            fade_in,
            cs: difficulty.circle_size,
            combo: 0,
            combo_colors: vec![(0.0, 0.81, 0.82),
                                (0.13, 0.70, 0.67),
                                (0.25, 0.88, 0.82),
                                (0.0, 1.0, 1.0),
                                (0.0, 0.75, 1.0)]
        }
    }

    fn map_coords(p: Vec2, playfield: Rect) -> Vec2 {
        let scale = playfield.h / 384.;

        Vec2 {
            x: (p.x * scale) + playfield.x,
            y: (p.y * scale) + playfield.y,
        }
    }

    fn cs(&self, playfield: Rect) -> f32 {
        let scale = playfield.h / 384.;
        (109. - 9. * self.cs) / scale
    }

    pub fn update(&mut self, music: &MusicManager) {
        if self.index >= self.objs.len() {
            return;
        }

        let curr = &self.objs[self.index];
        let combo_color = self.combo_colors[self.combo % self.combo_colors.len()];
        match curr {
            HitObject::HitCircle(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                    self.render_queue
                        .push(RenderableObject::Circle(RenderableCircle {
                            combo_color,
                            time: obj.time as f32,
                            x: obj.x as f32,
                            y: obj.y as f32,
                        }));
                    if obj.new_combo {
                        self.combo = 0;
                    }
                }

            }
            HitObject::Slider(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                    self.render_queue
                        .push(RenderableObject::Slider(RenderableSlider {
                            combo_color,
                            time: obj.time as f32,
                            x: obj.x as f32,
                            y: obj.y as f32,
                            curves: obj.curve_points.iter().map(|i| Vec2 { x: i.0 as f32, y: i.1 as f32 }).collect(),
                            repeat: obj.repeat as usize,
                        }));
                    if obj.new_combo {
                        self.combo = 0;
                    }
                }

            }
            HitObject::Spinner(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                    self.render_queue
                        .push(RenderableObject::Spinner(RenderableSpinner {
                            time: obj.time as f32,
                        }));
                    if obj.new_combo {
                        self.combo = 0;
                    }
                }
            }
            _ => {}
        }

        self.render_queue = self.render_queue.clone().into_iter().filter(|o| {
            match o {
                RenderableObject::Circle(obj) => {
                    obj.time > music.time.as_millis() as f32
                }
                RenderableObject::Slider(obj) => {
                    obj.time > music.time.as_millis() as f32
                }
                RenderableObject::Spinner(obj) => {
                    obj.time > music.time.as_millis() as f32
                }
            }
        }).collect();
        self.render_queue.reverse();
    }

    pub fn render(&mut self, music: &MusicManager, playfield: Rect) {
        for o in self.render_queue.iter() {
            match o {
                RenderableObject::Circle(obj) => {
                    let coord = Self::map_coords(Vec2::new(obj.x, obj.y), playfield);
                    let note_start = obj.time - self.preemt;
                    let fade_in_end = note_start + self.fade_in;

                    let a = music.time.as_millis() as f32 / fade_in_end;

                    draw_circle_lines(
                        coord.x,
                        coord.y,
                        self.cs(playfield),
                        3.,
                        Color {
                            r: obj.combo_color.0,
                            g: obj.combo_color.1,
                            b: obj.combo_color.2,
                            a,
                        },
                    );
                }
                RenderableObject::Slider(obj) => {
                    let start_coord = Self::map_coords(Vec2::new(obj.x, obj.y), playfield);
                    let note_start = obj.time - self.preemt;
                    let fade_in_end = note_start + self.fade_in;

                    let a = music.time.as_millis() as f32 / fade_in_end;

                    draw_circle_lines(
                        start_coord.x,
                        start_coord.y,
                        self.cs(playfield),
                        3.,
                        Color {
                            r: obj.combo_color.0,
                            g: obj.combo_color.1,
                            b: obj.combo_color.2,
                            a,
                        },
                    );

                    let mut last = start_coord;

                    for curve in obj.curves.iter() {
                        let coord = Self::map_coords(Vec2::new(curve.x, curve.y), playfield);
                        draw_line(
                            last.x,
                            last.y,
                            coord.x,
                            coord.y,
                            3.,
                            Color {
                                r: 1.,
                                g: 1.,
                                b: 1.,
                                a,
                            },
                        );
                        last = coord;
                    }
                }
                RenderableObject::Spinner(obj) => {
                    draw_circle_lines(screen_width() / 2., screen_height() / 2., screen_width() / 6., 5., GREEN);
                    draw_circle(screen_width() / 2., screen_height() / 2., 5., GREEN);
                }
            }
        }
    }
}
