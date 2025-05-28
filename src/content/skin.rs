use std::path::Path;

use macroquad::prelude::*;

#[derive(Clone)]
pub struct Skin {
    pub approach_circle: Texture2D,
    pub hit_circle: Texture2D,
    pub hit_circle_overlay: Texture2D,
    pub slider_start_circle: Texture2D,
    pub slider_start_circle_overlay: Texture2D,

    pub font: Font,
}

impl Skin {
    pub async fn load(path: &Path) -> Self {
        let approach_circle_path = path.join("approachcircle.png");
        let hit_circle_path = path.join("hitcircle.png");
        let hit_circle_overlay_path = path.join("hitcircleoverlay.png");
        let slider_start_circle_path = path.join("sliderstartcircle.png");
        let slider_start_circle_overlay_path = path.join("sliderstartcircleoverlay.png");
        let font_path = path.join("font.ttf");

        Self {
            font: load_ttf_font(font_path.to_str().unwrap()).await.unwrap(),
            approach_circle: load_texture(approach_circle_path.to_str().unwrap()).await.unwrap(),
            hit_circle: load_texture(hit_circle_path.to_str().unwrap()).await.unwrap(),
            hit_circle_overlay: load_texture(hit_circle_overlay_path.to_str().unwrap()).await.unwrap(),
            slider_start_circle: load_texture(slider_start_circle_path.to_str().unwrap()).await.unwrap(),
            slider_start_circle_overlay: load_texture(slider_start_circle_overlay_path.to_str().unwrap()).await.unwrap(),
        }
    }
}