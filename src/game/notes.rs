use macroquad::prelude::*;
use num_integer::binomial;

use crate::content::beatmap::{DifficultySection, HitObject, SliderType};

use super::{music::MusicManager, timing::TimingPointManager};

pub struct NoteSpawner {
    objs: Vec<HitObject>,
    index: usize,
    preemt: f32,
    fade_in: f32,
    cs: f32,
    slider_multiplier: f32,
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
    curve_type: SliderType,
    length: f32,
    repeat: usize,
}

#[derive(Debug, Clone)]
pub struct RenderableSpinner {
    end_time: f32,
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
            slider_multiplier: difficulty.slider_multiplier,
            cs: difficulty.circle_size,
            combo: 0,
            combo_colors: vec![
                (0.90, 0.94, 0.39),
                (0.78, 0.69, 0.99),
                (0.45, 0.82, 0.53),
                (0.39, 0.55, 0.92),
            ],
        }
    }

    fn map_coords(p: Vec2, playfield: Rect) -> Vec2 {
        let scale = playfield.h / 384.;

        Vec2 {
            x: (p.x * scale) + playfield.x,
            y: (p.y * scale) + playfield.y,
        }
    }

    fn slider_length(&self, pixel_length: f32, timing_manager: &TimingPointManager) -> f32 {
        let base_velocity = 100.0 * self.slider_multiplier;
        let velocity_multiplier = timing_manager.velocity_multiplier();
        let velocity = base_velocity * velocity_multiplier;
        let beat_length = timing_manager.beat_length();

        let length = (pixel_length / velocity) * beat_length;

        length
    }

    fn cs(&self, playfield: Rect) -> f32 {
        let scale = playfield.h / 384.;
        (108.0 - 8.0 * self.cs) * scale / 2.
    }

    fn alpha(&self, note_time: f32, current_time: f32) -> f32 {
        let note_start = note_time - self.preemt;
        let fade_in_end = note_start + self.fade_in;
        current_time / fade_in_end
    }

    fn color_with_alpha(&self, rgb: (f32, f32, f32), alpha: f32) -> Color {
        Color {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
            a: alpha,
        }
    }

    pub fn spawn(&mut self, music: &MusicManager, timing: &TimingPointManager) {
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
                    self.combo += 1;
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
                            length: self.slider_length(obj.pixel_length, &timing),
                            curves: obj
                                .curve_points
                                .iter()
                                .map(|i| Vec2 {
                                    x: i.0 as f32,
                                    y: i.1 as f32,
                                })
                                .collect(),
                            curve_type: obj.slider_type.clone(),
                            repeat: obj.repeat as usize,
                        }));
                    if obj.new_combo {
                        self.combo = 0;
                    }
                    self.combo += 1;
                }
            }
            HitObject::Spinner(obj) => {
                if obj.time as f32 - self.preemt <= music.time.as_millis() as f32 {
                    self.index += 1;
                    self.render_queue
                        .push(RenderableObject::Spinner(RenderableSpinner {
                            end_time: obj.end_time as f32,
                        }));
                    if obj.new_combo {
                        self.combo = 0;
                    }
                    self.combo += 1;
                }
            }
            _ => {}
        }
    }

    pub fn despawn(&mut self, music: &MusicManager) {
        let current_time = music.time.as_millis() as f32;
        self.render_queue.retain(|o| match o {
            RenderableObject::Circle(obj) => obj.time > current_time,
            RenderableObject::Slider(obj) => obj.time + obj.length > current_time,
            RenderableObject::Spinner(obj) => obj.end_time > current_time,
        });
    }

    pub fn update(&mut self, music: &MusicManager, timing: &TimingPointManager) {
        if self.index < self.objs.len() {
            self.spawn(music, timing);
        }
        self.despawn(music);
    }

    pub fn render_circle(&self, circle: &RenderableCircle, current_time: f32, playfield: Rect) {
        let coord = Self::map_coords(Vec2::new(circle.x, circle.y), playfield);
        let alpha = self.alpha(circle.time, current_time);

        draw_circle_lines(
            coord.x,
            coord.y,
            self.cs(playfield),
            3.,
            self.color_with_alpha(circle.combo_color, alpha),
        );
    }

    fn render_slider(&self, slider: &RenderableSlider, current_time: f32, playfield: Rect) {
        let alpha = self.alpha(slider.time, current_time);
        let color = self.color_with_alpha(slider.combo_color, alpha);
        let radius = self.cs(playfield);

        let start_pos = Self::map_coords(Vec2::new(slider.x, slider.y), playfield);
        draw_circle_lines(start_pos.x, start_pos.y, radius, 3.0, color);
        self.render_slider_body(slider, radius, alpha, playfield);
        // self.draw_slider_ticks(slider, start_pos, radius, alpha, playfield);
    }

    fn render_slider_body(
        &self,
        slider: &RenderableSlider,
        radius: f32,
        alpha: f32,
        playfield: Rect,
    ) {
        let segments = self.calculate_slider_segments(slider, playfield);
        let color = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: alpha,
        };

        for i in 0..segments.len() - 1 {
            let start = segments[i];
            let end = segments[i + 1];

            draw_line(start.x, start.y, end.x, end.y, 3.0, color);
            draw_circle_lines(start.x, start.y, radius, 3.0, color);
        }

        if let Some(last) = segments.last() {
            draw_circle_lines(last.x, last.y, radius, 3.0, color);
        }
    }

    fn calculate_slider_segments(&self, slider: &RenderableSlider, playfield: Rect) -> Vec<Vec2> {
        let mut segments = Vec::new();

        // Always include the starting point
        segments.push(Self::map_coords(Vec2::new(slider.x, slider.y), playfield));

        // Handle empty curves case (linear slider with just start point)
        if slider.curves.is_empty() {
            return segments;
        }

        match slider.curve_type {
            SliderType::Linear => calculate_linear_segments(slider, playfield, &mut segments),
            SliderType::Bezier => {
                let points = std::iter::once((slider.x, slider.y))
                    .chain(slider.curves.iter().map(|c| (c.x, c.y)))
                    .collect::<Vec<_>>();

                if points.len() >= 2 {
                    for t in 0..=20 {
                        let t = t as f32 / 20.0;
                        let point = calculate_bezier_point(t, &points);
                        segments.push(Self::map_coords(point, playfield));
                    }
                }
            }
            SliderType::Catmull => {
                // Need at least 2 points for Catmull-Rom
                if slider.curves.len() >= 1 {
                    let points = std::iter::once((slider.x, slider.y))
                        .chain(slider.curves.iter().map(|c| (c.x, c.y)))
                        .collect::<Vec<_>>();

                    for t in 0..=20 {
                        let t = t as f32 / 20.0 * (points.len() - 1) as f32;
                        let point = calculate_catmull_rom_point(t, &points);
                        segments.push(Self::map_coords(point, playfield));
                    }
                }
            }
            SliderType::Perfect => {
                match slider.curves.len() {
                    0 => {}
                    1 => calculate_linear_segments(slider, playfield, &mut segments),
                    _ => {
                        let p0 = (slider.x, slider.y);
                        let p1 = (slider.curves[0].x, slider.curves[0].y);
                        let p2 = (slider.curves[1].x, slider.curves[1].y);

                        if are_points_collinear(p0, p1, p2) {
                            let mut current_point = Vec2::new(slider.x, slider.y);
                            segments.clear();
                            segments.push(Self::map_coords(current_point, playfield));

                            for curve_point in &slider.curves {
                                for i in 1..=10 {
                                    let t = i as f32 / 10.0;
                                    let point = Vec2::new(
                                        lerp(current_point.x, curve_point.x, t),
                                        lerp(current_point.y, curve_point.y, t),
                                    );
                                    segments.push(Self::map_coords(point, playfield));
                                }
                                current_point = *curve_point;
                            }
                        } else if let Some((center, radius)) = circle_through_points(p0, p1, p2) {
                            if radius < 1000.0 && radius > 1.0 {
                                let angle0 = (p0.1 - center.1).atan2(p0.0 - center.0);
                                let angle2 = (p2.1 - center.1).atan2(p2.0 - center.0);
                                let angle1 = (p1.1 - center.1).atan2(p1.0 - center.0);

                                let normalize_angle = |mut angle: f32| {
                                    while angle < 0.0 {
                                        angle += 2.0 * std::f32::consts::PI;
                                    }
                                    while angle >= 2.0 * std::f32::consts::PI {
                                        angle -= 2.0 * std::f32::consts::PI;
                                    }
                                    angle
                                };

                                let angle0 = normalize_angle(angle0);
                                let angle1 = normalize_angle(angle1);
                                let angle2 = normalize_angle(angle2);

                                let (start_angle, end_angle) =
                                    determine_arc_direction(angle0, angle1, angle2);

                                let arc_length = (end_angle - start_angle).abs();
                                let num_segments =
                                    ((radius * arc_length / 10.0).ceil() as usize).max(20);

                                segments.clear();
                                for i in 0..=num_segments {
                                    let t = i as f32 / num_segments as f32;
                                    let angle = start_angle + t * (end_angle - start_angle);
                                    let point = Vec2::new(
                                        center.0 + radius * angle.cos(),
                                        center.1 + radius * angle.sin(),
                                    );
                                    segments.push(Self::map_coords(point, playfield));
                                }
                            } else {
                                calculate_linear_segments_through_points(
                                    slider,
                                    playfield,
                                    &mut segments,
                                );
                            }
                        } else {
                            calculate_linear_segments_through_points(
                                slider,
                                playfield,
                                &mut segments,
                            );
                        }
                    }
                }
            }
        }

        segments
    }

    fn render_spinner(&self, _spinner: &RenderableSpinner) {
        let center_x = screen_width() / 2.;
        let center_y = screen_height() / 2.;

        draw_circle_lines(center_x, center_y, screen_width() / 6., 5., GREEN);
        draw_circle(center_x, center_y, 5., GREEN);
    }

    pub fn render(&mut self, music: &MusicManager, playfield: Rect) {
        let current_time = music.time.as_millis() as f32;

        for o in self.render_queue.iter().rev() {
            match o {
                RenderableObject::Circle(obj) => self.render_circle(obj, current_time, playfield),
                RenderableObject::Slider(obj) => self.render_slider(obj, current_time, playfield),
                RenderableObject::Spinner(obj) => self.render_spinner(obj),
            }
        }
    }
}

fn calculate_bezier_point(t: f32, points: &[(f32, f32)]) -> Vec2 {
    let n = points.len() - 1;
    let mut x = 0.0;
    let mut y = 0.0;

    for i in 0..=n {
        let binomial = binomial(n, i) as f32;
        let term = binomial * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32);
        x += points[i].0 * term;
        y += points[i].1 * term;
    }

    Vec2::new(x, y)
}

fn calculate_linear_segments(slider: &RenderableSlider, playfield: Rect, segments: &mut Vec<Vec2>) {
    let mut last = segments[0];
    for curve in &slider.curves {
        for i in 0..=20 {
            let coord = NoteSpawner::map_coords(*curve, playfield);
            segments.push(Vec2 {
                x: lerp(last.x, coord.x, i as f32 / 20.),
                y: lerp(last.y, coord.y, i as f32 / 20.),
            });
        }
        last = *curve;
    }
}

fn calculate_catmull_rom_point(t: f32, points: &[(f32, f32)]) -> Vec2 {
    let n = points.len();
    let segment = t.floor() as usize;
    let t = t - segment as f32;

    if segment == 0 {
        Vec2::new(
            lerp(points[0].0, points[1].0, t),
            lerp(points[0].1, points[1].1, t),
        )
    } else if segment >= n - 1 {
        Vec2::new(
            lerp(points[n - 2].0, points[n - 1].0, t),
            lerp(points[n - 2].1, points[n - 1].1, t),
        )
    } else {
        let p0 = points[segment - 1];
        let p1 = points[segment];
        let p2 = points[segment + 1];
        let p3 = if segment + 2 < n {
            points[segment + 2]
        } else {
            p2
        };

        Vec2::new(
            catmull_rom(t, p0.0, p1.0, p2.0, p3.0),
            catmull_rom(t, p0.1, p1.1, p2.1, p3.1),
        )
    }
}

fn determine_arc_direction(angle0: f32, angle1: f32, angle2: f32) -> (f32, f32) {
    let clockwise_arc = if angle2 >= angle0 {
        angle2 - angle0
    } else {
        angle2 + 2.0 * std::f32::consts::PI - angle0
    };

    let angle1_in_cw = if angle1 >= angle0 {
        angle1 - angle0 <= clockwise_arc
    } else {
        (angle1 + 2.0 * std::f32::consts::PI - angle0) <= clockwise_arc
    };

    if angle1_in_cw {
        if angle2 >= angle0 {
            (angle0, angle2)
        } else {
            (angle0, angle2 + 2.0 * std::f32::consts::PI)
        }
    } else {
        if angle0 >= angle2 {
            (angle0, angle2)
        } else {
            (angle0, angle2 - 2.0 * std::f32::consts::PI)
        }
    }
}

fn calculate_linear_segments_through_points(
    slider: &RenderableSlider,
    playfield: Rect,
    segments: &mut Vec<Vec2>,
) {
    segments.clear();
    let mut current = Vec2::new(slider.x, slider.y);
    segments.push(NoteSpawner::map_coords(current, playfield));

    for curve_point in &slider.curves {
        for i in 1..=10 {
            let t = i as f32 / 10.0;
            let point = Vec2::new(
                lerp(current.x, curve_point.x, t),
                lerp(current.y, curve_point.y, t),
            );
            segments.push(NoteSpawner::map_coords(point, playfield));
        }
        current = *curve_point;
    }
}

fn circle_through_points(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
) -> Option<((f32, f32), f32)> {
    let mid1 = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
    let mid2 = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);

    let slope1 = if (p1.0 - p0.0).abs() > f32::EPSILON {
        -(p1.0 - p0.0) / (p1.1 - p0.1)
    } else {
        f32::INFINITY
    };

    let slope2 = if (p2.0 - p1.0).abs() > f32::EPSILON {
        -(p2.0 - p1.0) / (p2.1 - p1.1)
    } else {
        f32::INFINITY
    };

    if slope1.is_infinite() && slope2.is_infinite() {
        return None;
    }

    if (slope1 - slope2).abs() < f32::EPSILON {
        return None;
    }

    let center_x = if slope1.is_infinite() {
        mid1.0
    } else if slope2.is_infinite() {
        mid2.0
    } else {
        (slope1 * mid1.0 - slope2 * mid2.0 + mid2.1 - mid1.1) / (slope1 - slope2)
    };

    let center_y = if slope1.is_infinite() {
        slope2 * (center_x - mid2.0) + mid2.1
    } else {
        slope1 * (center_x - mid1.0) + mid1.1
    };

    let radius = ((center_x - p0.0).powi(2) + (center_y - p0.1).powi(2)).sqrt();
    Some(((center_x, center_y), radius))
}

fn are_points_collinear(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32)) -> bool {
    let area = (p0.0 * (p1.1 - p2.1) + p1.0 * (p2.1 - p0.1) + p2.0 * (p0.1 - p1.1)).abs();
    area < 1.0
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
fn catmull_rom(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t.powi(2)
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t.powi(3))
}
