pub mod formats;

pub struct Beatmap {
    pub difficulties: Vec<Difficulty>,
}

#[derive(Default)]
pub struct Difficulty {
    pub version: i32,
    pub audio_bytes: Vec<u8>,
    pub general: GeneralSection,
    pub editor: EditorSection,
    pub metadata: MetadataSection,
    pub timing_points: Vec<TimingPoint>,
    pub hit_objects: Vec<HitObject>,
    pub difficulty: DifficultySection,
    pub colours: ColoursSection,
}

#[derive(Debug)]
pub enum GameMode {
    Osu,
    Taiko,
    CTB,
    Mania,
}

#[derive(Debug)]
pub struct GeneralSection {
    pub audio_filename: String,
    pub audio_lead_in: i32,
    pub preview_time: i32,
    pub countdown: bool,
    pub sample_set: String,
    pub stack_leniency: f32,
    pub game_mode: GameMode,
    pub letterbox_in_breaks: bool,
    pub widescreen_storyboard: bool,
    pub story_fire_in_front: bool,
    pub special_style: bool,
    pub epilepsy_warning: bool,
    pub use_skin_sprites: bool,
}

impl Default for GeneralSection {
    fn default() -> Self {
        GeneralSection {
            audio_filename: String::new(),
            audio_lead_in: 0,
            preview_time: 0,
            countdown: false,
            sample_set: String::new(),
            stack_leniency: 0.0,
            game_mode: GameMode::Osu,
            letterbox_in_breaks: false,
            widescreen_storyboard: false,
            story_fire_in_front: false,
            special_style: false,
            epilepsy_warning: false,
            use_skin_sprites: false,
        }
    }
}

pub struct EditorSection {
    pub bookmarks: Vec<i32>,
    pub distance_spacing: f32,
    pub beat_divisor: i32,
    pub grid_size: i32,
    pub timeline_zoom: f32,
}

impl Default for EditorSection {
    fn default() -> Self {
        EditorSection {
            bookmarks: Vec::new(),
            distance_spacing: 1.22,
            beat_divisor: 4,
            grid_size: 4,
            timeline_zoom: 1.0,
        }
    }
}

pub struct MetadataSection {
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: Vec<String>,
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
}

impl Default for MetadataSection {
    fn default() -> Self {
        MetadataSection {
            title: String::new(),
            title_unicode: String::new(),
            artist: String::new(),
            artist_unicode: String::new(),
            creator: String::new(),
            version: String::new(),
            source: String::new(),
            tags: Vec::new(),
            beatmap_id: 0,
            beatmap_set_id: 0,
        }
    }
}

#[derive(Default)]
pub struct DifficultySection {
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}

pub struct TimingPoint {
    pub offset: i32,
    pub ms_per_beat: f32,
    pub meter: i32,
    pub sample_set: String,
    pub sample_index: i32,
    pub volume: i32,
    pub inherited: bool,
    pub kiai_mode: bool,
}

#[derive(Clone)]
pub enum HitObject {
    HitCircle(HitCircle),
    Slider(Slider),
    Spinner(Spinner),
    HoldNote(HoldNote),
}

#[derive(Clone)]
pub struct HitCircle {
    pub x: i32,
    pub y: i32,
    pub new_combo: bool,
    pub color_skip: i32,
    pub time: i32,
    pub hitsound: i32,
    pub extras: HitObjectExtras,
}

#[derive(Clone)]
pub enum SliderType {
    Linear,
    Bezier,
    Perfect,
    Catmull,
}

#[derive(Clone)]
pub struct Slider {
    pub x: i32,
    pub y: i32,
    pub new_combo: bool,
    pub color_skip: i32,
    pub time: i32,
    pub slider_type: SliderType,
    pub curve_points: Vec<(i32, i32)>,
    pub repeat: i32,
    pub pixel_length: f32,
    pub edge_hitsounds: Vec<i32>,
    pub edge_additions: Vec<(i32, i32)>,
    pub hitsound: i32,
    pub extras: HitObjectExtras,
}

#[derive(Clone)]
pub struct Spinner {
    pub x: i32,
    pub y: i32,
    pub new_combo: bool,
    pub color_skip: i32,
    pub time: i32,
    pub hitsound: i32,
    pub end_time: i32,
    pub extras: HitObjectExtras,
}

#[derive(Clone)]
pub struct HoldNote {
    pub x: i32,
    pub y: i32,
    pub new_combo: bool,
    pub color_skip: i32,
    pub time: i32,
    pub hitsound: i32,
    pub end_time: i32,
    pub extras: HitObjectExtras,
}

#[derive(Clone)]
pub struct HitObjectExtras {
    pub sample_set: i32,
    pub addition_set: i32,
    pub custom_index: i32,
    pub sample_volume: i32,
    pub filename: String,
}

impl Default for HitObjectExtras {
    fn default() -> Self {
        HitObjectExtras {
            sample_set: 0,
            addition_set: 0,
            custom_index: 0,
            sample_volume: 0,
            filename: String::new(),
        }
    }
}

#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Colour(i32, i32, i32);

#[derive(Default)]
pub struct ColoursSection {
    pub colours: Vec<Colour>,
    pub slider_body: Colour,
    pub slider_track_override: Colour,
    pub slider_border: Colour,
}

enum Section {
    General(GeneralSection),
    Editor(EditorSection),
    Metadata(MetadataSection),
    TimingPoints(Vec<TimingPoint>),
    HitObjects(Vec<HitObject>),
    Difficulty(DifficultySection),
    Colours(ColoursSection),
    Events,
    None,
}
