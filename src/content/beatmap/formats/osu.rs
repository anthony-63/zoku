use crate::content::beatmap::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fmt::Display, io::Read};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parse,
    Syntax(String),
    Message(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Message(ref msg) => formatter.write_str(msg),
            Error::Syntax(ref reason) => write!(formatter, "Syntax error: {}", reason),
            Error::Parse => formatter.write_str("Parsing error"),
        }
    }
}

pub struct OsuParser {
    files: HashMap<String, Vec<u8>>,
}

pub struct ParseState<'a> {
    lines: std::iter::Filter<std::str::Lines<'a>, fn(&&str) -> bool>,
    current_line: Option<&'a str>,
}

impl<'a> ParseState<'a> {
    pub fn new(input: &'a str) -> Self {
        lazy_static! {
            static ref EMPTY_LINE: Regex = Regex::new(r"^\s*$").unwrap();
        }

        ParseState {
            lines: input.lines().filter(|l| !EMPTY_LINE.is_match(l)),
            current_line: None,
        }
    }
    pub fn get_current_line(&mut self) -> Option<&'a str> {
        if let Some(line) = self.current_line {
            Some(line)
        } else {
            self.read_next_line()
        }
    }

    pub fn read_next_line(&mut self) -> Option<&'a str> {
        let next_line = self.lines.next();
        self.current_line = next_line;

        next_line
    }
}

macro_rules! read_val {
    ($iter:ident, $func:expr) => {
        $iter.next().ok_or(Error::Parse).and_then($func)
    };
}

macro_rules! read_list {
    ($sep:expr, $iter:ident, $func:expr) => {
        $iter
            .next()
            .ok_or(Error::Parse)
            .and_then(|s| s.split($sep).map($func).collect())
    };
}

macro_rules! parse_into_struct {
	($sep:expr, $dest:ident, $line:expr; {$($field:ident: $f:expr),*}) => {
		{
			let mut iter = $line.split($sep).map(|s| s.trim());
			$dest {
				$($field: {
					$f(iter.next()
						.ok_or_else(|| Error::Syntax(format!(
							"Unable to read field {} into struct {}",
							stringify!($field),
							stringify!($dest)))
						)?)?
				}),*
			}
		}
	};
}

macro_rules! value_parser {
    ($v:expr, $fn:expr) => {
        $fn($v)
    };
    ($v:expr, $fn:expr, $sep:expr) => {
        $v.split($sep)
            .map(|s| $fn(s.trim()))
            .collect::<std::result::Result<Vec<_>, _>>()
    };
}

pub fn parse_kv_pair<'a>(state: &'a mut ParseState) -> Option<(&'a str, &'a str)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\S+)\s*:(\s*(\S.*)$)?").unwrap();
    }

    state
        .read_next_line()
        .and_then(|l| RE.captures(l))
        .and_then(|c| {
            c.get(1).and_then(|k| {
                c.get(3).map_or(Some((k.as_str(), "none")), |v| {
                    Some((k.as_str(), v.as_str()))
                })
            })
        })
}

macro_rules! parse_kv_section {
    (|$s_t:ty, $state:ident| {$($str:expr => $field:ident: $($f:expr),*;)*}) => {
        {
            let mut section: $s_t = Default::default();

            loop {
                match parse_kv_pair($state) {
                    $(
                    Some(($str, v)) => section.$field = value_parser!(v, $($f),*)?,
                    )*
                    _ => break,
                }
            }

            section
        }
    }
}

macro_rules! make_syntax_err {
    ($reason:expr) => {
        || Error::Syntax(String::from($reason))
    };
}

pub fn parse_num<T: std::str::FromStr>(n: &str) -> Result<T> {
    n.parse()
        .map_err(|_| Error::Syntax(String::from("Unable to parse number")))
}

pub fn parse_string(s: &str) -> Result<String> {
    Ok(String::from(s))
}

pub fn parse_bool(s: &str) -> Result<bool> {
    s.parse::<i32>()
        .map(|n| n != 0)
        .map_err(|_| Error::Syntax(String::from("Could not parse bool")))
}

pub fn parse_mode(s: &str) -> Result<GameMode> {
    match s {
        "0" => Ok(GameMode::Osu),
        "1" => Ok(GameMode::Taiko),
        "2" => Ok(GameMode::CTB),
        "3" => Ok(GameMode::Mania),
        _ => Err(Error::Syntax(String::from("Unable to parse gamemode"))),
    }
}

pub fn parse_colour(s: &str) -> Result<Colour> {
    let mut iter = s.split(",");
    Ok(Colour(
        read_val!(iter, parse_num)?,
        read_val!(iter, parse_num)?,
        read_val!(iter, parse_num)?,
    ))
}

pub fn parse_extras(s: &str) -> Result<HitObjectExtras> {
    Ok(parse_into_struct!(":", HitObjectExtras, s; {
        sample_set: parse_num,
        addition_set: parse_num,
        custom_index: parse_num,
        sample_volume: parse_num,
        filename: parse_string
    }))
}

pub fn parse_slider_type(s: &str) -> Result<SliderType> {
    match s {
        "L" => Ok(SliderType::Linear),
        "B" => Ok(SliderType::Bezier),
        "P" => Ok(SliderType::Perfect),
        "C" => Ok(SliderType::Catmull),
        _ => Err(Error::Syntax(String::from("Invalid slider type"))),
    }
}

pub fn parse_coord(s: &str) -> Result<(i32, i32)> {
    let mut iter = s.split(":");
    Ok((read_val!(iter, parse_num)?, read_val!(iter, parse_num)?))
}

fn parse_curve_points(s: &str) -> Result<(SliderType, Vec<(i32, i32)>)> {
    let mut iter = s.split("|");

    let slider_type = read_val!(iter, parse_slider_type)?;

    let points = iter.map(parse_coord).collect::<Result<Vec<(i32, i32)>>>()?;

    Ok((slider_type, points))
}

pub fn parse_hit_object(s: &str) -> Result<HitObject> {
    let mut iter = s.split(",");

    let x: i32 = read_val!(iter, parse_num)?;
    let y: i32 = read_val!(iter, parse_num)?;
    let time: i32 = read_val!(iter, parse_num)?;
    let obj_type: i32 = read_val!(iter, parse_num)?;

    let new_combo = obj_type & 4 != 0;
    let color_skip = (obj_type >> 4) & 7;

    let hitsound = read_val!(iter, parse_num)?;

    match obj_type & 139 {
        1 => Ok(HitObject::HitCircle(HitCircle {
            x,
            y,
            new_combo,
            color_skip,
            time,
            hitsound,

            extras: read_val!(iter, parse_extras).unwrap_or(Default::default()),
        })),

        2 => {
            let (slider_type, curve_points) = read_val!(iter, parse_curve_points)?;
            Ok(HitObject::Slider(Slider {
                x,
                y,
                new_combo,
                color_skip,
                time,
                hitsound,
                slider_type,
                curve_points,

                repeat: read_val!(iter, parse_num)?,
                pixel_length: read_val!(iter, parse_num)?,

                edge_hitsounds: read_list!("|", iter, parse_num).unwrap_or(Vec::new()),

                edge_additions: read_list!("|", iter, parse_coord).unwrap_or(Vec::new()),

                extras: read_val!(iter, parse_extras).unwrap_or(Default::default()),
            }))
        }

        8 => Ok(HitObject::Spinner(Spinner {
            x,
            y,
            time,
            new_combo,
            color_skip,
            hitsound,

            end_time: read_val!(iter, parse_num)?,

            extras: read_val!(iter, parse_extras).unwrap_or(Default::default()),
        })),

        128 => Ok(HitObject::HoldNote(HoldNote {
            x,
            y,
            time,
            new_combo,
            color_skip,
            hitsound,

            end_time: read_val!(iter, parse_num)?,

            extras: read_val!(iter, parse_extras).unwrap_or(Default::default()),
        })),

        _ => Err(Error::Syntax(String::from("Invalid hit object type"))),
    }
}

impl OsuParser {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn from_osz(path: String) -> Beatmap {
        let mut parser = Self::new();

        let file = std::fs::File::open(path.clone())
            .expect(&format!("Failed to open beatmap file {}", path));

        let mut archive = zip::ZipArchive::new(file).expect("Failed to parse osz archive");
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            let outpath = file.name();
            parser.files.insert(
                outpath.into(),
                file.bytes()
                    .collect::<std::result::Result<Vec<_>, std::io::Error>>()
                    .unwrap(),
            );
        }

        let mut difficulties: Vec<Difficulty> = vec![];

        for (file, _) in parser.files.clone() {
            if file.ends_with(".osu") {
                difficulties.push(parser.from_osu(file));
            }
        }

        Beatmap { difficulties }
    }

    pub fn parse_difficulty(input: &str) -> Result<Difficulty> {
        let mut state = ParseState::new(input);

        let version = Self::parse_version_string(&mut state)?;
        state.read_next_line();

        let mut map = Difficulty {
            version,
            ..Default::default()
        };

        loop {
            match Self::parse_section(&mut state)? {
                Section::General(s) => map.general = s,
                Section::Editor(s) => map.editor = s,
                Section::Metadata(s) => map.metadata = s,
                Section::TimingPoints(s) => map.timing_points = s,
                Section::HitObjects(s) => map.hit_objects = s,
                Section::Difficulty(s) => map.difficulty = s,
                Section::Colours(s) => map.colours = s,
                Section::Events => {}
                Section::None => break,
            }
        }

        Ok(map)
    }

    fn parse_section(state: &mut ParseState) -> Result<Section> {
        if let Some(header_line) = state.get_current_line() {
            lazy_static! {
                static ref HEADER_RE: Regex = Regex::new(r"^\[([^\[\]]*)\]\s*$").unwrap();
            }

            println!("Header: '{}'", header_line);

            let section_title = HEADER_RE
                .captures(header_line)
                .and_then(|c| c.get(1))
                .map(|c| c.as_str())
                .ok_or_else(make_syntax_err!("Malformed section header"))?;

            match section_title {
                "General" => Ok(Section::General(parse_kv_section! {
                    |GeneralSection, state| {
                        "AudioFilename" => audio_filename: parse_string;
                        "AudioLeadIn" => audio_lead_in: parse_num;
                        "PreviewTime" => preview_time: parse_num;
                        "Countdown" => countdown: parse_bool;
                        "SampleSet" => sample_set: parse_string;
                        "StackLeniency" => stack_leniency: parse_num;
                        "Mode" => game_mode: parse_mode;
                        "LetterboxInBreaks" => letterbox_in_breaks: parse_bool;
                        "WidescreenStoryboard" => widescreen_storyboard: parse_bool;
                        "StoryFireInFront" => story_fire_in_front: parse_bool;
                        "SpecialStyle" => special_style: parse_bool;
                        "EpilepsyWarning" => epilepsy_warning: parse_bool;
                    }
                })),

                "Editor" => Ok(Section::Editor(parse_kv_section! {
                    |EditorSection, state| {
                        "Bookmarks" => bookmarks: parse_num, ",";
                        "DistanceSpacing" => distance_spacing: parse_num;
                        "BeatDivisor" => beat_divisor: parse_num;
                        "GridSize" => grid_size: parse_num;
                        "TimelineZoom" => timeline_zoom: parse_num;
                    }
                })),

                "Metadata" => Ok(Section::Metadata(parse_kv_section! {
                    |MetadataSection, state| {
                        "Title" => title: parse_string;
                        "TitleUnicode" => title_unicode: parse_string;
                        "Artist" => artist: parse_string;
                        "ArtistUnicode" => artist_unicode: parse_string;
                        "Creator" => creator: parse_string;
                        "Version" => version: parse_string;
                        "Source" => source: parse_string;
                        "Tags" => tags: parse_string, " ";
                        "BeatmapID" => beatmap_id: parse_num;
                        "BeatmapSetID" => beatmap_set_id: parse_num;
                    }
                })),

                "Difficulty" => Ok(Section::Difficulty(parse_kv_section! {
                    |DifficultySection, state| {
                        "HPDrainRate" => hp_drain_rate: parse_num;
                        "CircleSize" => circle_size: parse_num;
                        "OverallDifficulty" => overall_difficulty: parse_num;
                        "ApproachRate" => approach_rate: parse_num;
                        "SliderMultiplier" => slider_multiplier: parse_num;
                        "SliderTickRate" => slider_tick_rate: parse_num;
                    }
                })),

                "Events" => {
                    // Just skipping this for now
                    Self::skip_section(state);
                    Ok(Section::Events)
                }

                "TimingPoints" => {
                    Self::parse_timing_points(state).map(|s| Section::TimingPoints(s))
                }

                "HitObjects" => Self::parse_hit_objects(state).map(|s| Section::HitObjects(s)),

                "Colours" => Self::parse_colours(state).map(|s| Section::Colours(s)),

                _ => Err(Error::Syntax(format!(
                    "Unknown section header {}",
                    section_title
                ))),
            }
        } else {
            Ok(Section::None)
        }
    }

    fn skip_section(state: &mut ParseState) {
        lazy_static! {
            static ref HEADER_RE: Regex = Regex::new(r"^\[([^\[\]]*)\]\s*$").unwrap();
        }

        loop {
            match state.read_next_line() {
                Some(l) if !HEADER_RE.is_match(l) => {}
                _ => break,
            }
        }
    }

    fn parse_version_string(state: &mut ParseState) -> Result<i32> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^osu file format v(\d+)$").unwrap();
        }

        state
            .get_current_line()
            .filter(|line| RE.is_match(line))
            .and_then(|line| line[17..].parse::<i32>().ok())
            .ok_or_else(make_syntax_err!("unable to parse version string"))
    }

    fn parse_timing_points(state: &mut ParseState) -> Result<Vec<TimingPoint>> {
        lazy_static! {
            static ref HEADER_RE: Regex = Regex::new(r"^\[([^\[\]]*)\]\s*$").unwrap();
        }

        let mut timing_points = Vec::with_capacity(100);
        loop {
            match state.read_next_line() {
                Some(l) if !HEADER_RE.is_match(l) => {
                    let timing_point = parse_into_struct!(",", TimingPoint, l; {
                        offset: parse_num,
                        ms_per_beat: parse_num,
                        meter: parse_num,
                        sample_set: parse_string,
                        sample_index: parse_num,
                        volume: parse_num,
                        inherited: parse_bool,
                        kiai_mode: parse_bool
                    });

                    timing_points.push(timing_point)
                }
                _ => break,
            };
        }

        Ok(timing_points)
    }

    fn parse_colours(state: &mut ParseState) -> Result<ColoursSection> {
        lazy_static! {
            static ref COLOR_RE: Regex = Regex::new(r"^Combo\d+$").unwrap();
        }

        let mut section: ColoursSection = Default::default();

        let mut colours = Vec::with_capacity(10);

        loop {
            match parse_kv_pair(state) {
                Some((k, v)) if COLOR_RE.is_match(k) => {
                    let n: i32 = parse_num(&k[5..])?;
                    colours.push((n, parse_colour(v)?));
                }

                Some(("SliderBody", v)) => section.slider_body = parse_colour(v)?,

                Some(("SliderTrackOverride", v)) => {
                    section.slider_track_override = parse_colour(v)?
                }

                Some(("SliderBorder", v)) => section.slider_border = parse_colour(v)?,

                Some(_) => return Err(Error::Syntax(String::from("Unknown key value"))),

                _ => break,
            }
        }

        colours.sort_unstable();
        section.colours = colours.into_iter().map(|(_, c)| c).collect();
        Ok(section)
    }

    fn parse_hit_objects(state: &mut ParseState) -> Result<Vec<HitObject>> {
        lazy_static! {
            static ref HEADER_RE: Regex = Regex::new(r"^\[([^\[\]]*)\]\s*$").unwrap();
        }

        let mut hit_objects = Vec::with_capacity(100);

        loop {
            match state.read_next_line() {
                Some(l) if !HEADER_RE.is_match(l) => {
                    hit_objects.push(parse_hit_object(l)?);
                }
                _ => break,
            }
        }

        Ok(hit_objects)
    }

    fn from_osu(&mut self, name: String) -> Difficulty {
        let mut diff = Self::parse_difficulty(
            String::from_utf8(self.files.get(&name).unwrap().clone())
                .unwrap()
                .as_str(),
        )
        .unwrap();

        diff.audio_bytes = self
            .files
            .get(&diff.general.audio_filename)
            .expect("Failed to retrieve audio from osz")
            .clone();

        println!(
            "Parsed '{}[{}]'",
            diff.metadata.title, diff.metadata.version
        );
        diff
    }
}
