use id3::TagLike;
use pyo3::buffer;
use tauri::Manager;

use base64::{engine::general_purpose, Engine as _};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default;
use std::fmt::Debug;
use std::fs::File;
use std::path::{self, Path};

use bwavfile::WaveReader;
use cue_sheet::parser;

use symphonia::core::codecs::CodecType;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{StandardTagKey, Tag, Value, Visual};
use symphonia::core::probe::Hint;

fn picture2base64(mime_type: &str, data: &[u8]) -> String {
    let data = general_purpose::STANDARD.encode(data);
    format!("data:{};base64,{}", mime_type, data)
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct Chapter {
    pub timestamp: f64, // Start Timestamp in seconds
    pub title: String,
    pub description: Option<String>, // Optional description of the chapter
}

/// AudioMetadata scheme
///
/// This also matches with the front-end JS/TS side
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct AudioMetadata {
    pub name: String,                      // Name of the audio file
    pub artist: Option<String>,            // Primary artist or short description of the audio file
    pub contributors: Option<Vec<String>>, // Additional contributing artist or information
    pub thumbnail: Option<String>,         // Path to the thumbnail image or Base64 encoded string
    pub path: String,                      // Path to the audio file
    pub duration: f64,                     // Duration of the audio file in seconds
    pub size: u64,                         // Size of the audio file in bytes
    pub sample_rate: u32,                  // Sample rate of the audio file
    pub channels: u32,                     // Number of audio channels
    pub bit_depth: u32,                    // Bit depth of the audio file

    pub chapters: Option<Vec<Chapter>>, // Optional chapters in the audio file

    // Additional metadata fields - stores unmatched tags as key-value pairs
    // Need to cast to string for now...
    pub extras: HashMap<String, String>,
}

impl AudioMetadata {
    /// Convert the struct to a generic hashmap for easy serializing/logging
    pub fn to_hashmap(&self) -> Result<HashMap<String, serde_json::Value>, serde_json::Error> {
        struct_to_hashmap(self)
    }

    /// Try to add known tags from the provided list to the metadata
    /// Unmatched tags will be added into `extras`
    pub fn try_add_known_tags(&mut self, tags: &[Tag]) {
        for tag in tags {
            match tag.std_key {
                Some(StandardTagKey::TrackTitle) => {
                    self.name = tag.value.to_string();
                }
                Some(StandardTagKey::Artist) => {
                    self.artist = Some(tag.value.to_string());
                }
                Some(StandardTagKey::AlbumArtist) => {
                    if let Some(ref mut contributors) = self.contributors {
                        contributors.push(tag.value.to_string());
                    } else {
                        self.contributors = Some(vec![tag.value.to_string()]);
                    }
                    self.contributors = self.contributors.as_ref().map(|c| {
                        c.iter()
                            .cloned()
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter()
                            .collect()
                    });
                }
                _ => {
                    // Add unmatched tags to extras
                    let key = if let Some(std_key) = &tag.std_key {
                        format!("{:?}", std_key)
                    } else {
                        tag.key.clone()
                    };
                    self.extras.insert(key, tag.value.to_string());
                }
            }

            if !tag.is_known() {
                self.extras.insert(tag.key.clone(), tag.value.to_string());
            }
        }
    }
}

impl Default for AudioMetadata {
    fn default() -> Self {
        AudioMetadata {
            name: String::new(),
            artist: None,
            contributors: None,
            thumbnail: None,
            path: String::new(),
            duration: 0.0,
            size: 0,
            sample_rate: 0,
            channels: 0,
            bit_depth: 0,

            chapters: None,

            extras: HashMap::new(),
        }
    }
}

/// Generic function to convert any serializable struct to HashMap (must have impl `serde::Serialize`)
pub fn struct_to_hashmap<T: serde::Serialize>(
    input: &T,
) -> Result<HashMap<String, serde_json::Value>, serde_json::Error> {
    let json_value = serde_json::to_value(input)?;
    if let serde_json::Value::Object(map) = json_value {
        Ok(map.into_iter().collect())
    } else {
        Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Input is not a struct",
        )))
    }
}

/// Symphonia is not great at extracting all cue points from PCM / Wave format, use this function to try with bwavfile
///
/// Returns `Some` chapters if found, or `None` if either (1) cannot parse (might not be valid wav format) or (2) no cue points are found
fn try_extract_wav_chapters(path: &Path) -> Option<Vec<Chapter>> {
    let mut f = WaveReader::open(path).ok()?;
    let cue_points = f.cue_points();
    let format = f.format();
    let chapters = match cue_points {
        Ok(cue_points) => {
            let format = format.unwrap(); // This unwrap should be ok, we already match for cue_points
            let mut chapters = Vec::new();
            let cue_points_len = cue_points.len();
            let width = cue_points_len.to_string().len();
            for (i, cue) in cue_points.iter().enumerate() {
                chapters.push(Chapter {
                    timestamp: cue.frame as f64 / format.sample_rate as f64,
                    title: cue.label.clone().unwrap_or(format!(
                        "Chapter #{:0width$}",
                        i + 1,
                        width = width
                    )),
                    description: cue.note.clone(),
                });
            }
            Some(chapters)
        }
        Err(_e) => {
            // log::warn!("Failed to extract cue points from {:?}: {:?}", path, _e);
            None
        }
    };

    chapters
}

fn cuesheet2chapters(cuesheet: &str) -> Option<Vec<Chapter>> {
    // Just in case some weird cuesheet heading is encountered
    // only need data starting from FILE to extract cues/markers
    let track_data = cuesheet
        .lines()
        .skip_while(|line| !line.trim().starts_with("FILE"))
        .collect::<Vec<_>>()
        .join("\n");

    let cuedata = parser::parse_cue(&track_data);
    if cuedata.is_err() {
        log::warn!("Failed to parse cuesheet: {:?}", cuedata.err());
        return None;
    }

    let commands = cuedata.unwrap();

    let mut chapters = Vec::new();
    let mut current_title = String::new();

    for command in commands {
        match command {
            parser::Command::Title(title) => {
                current_title = title.clone();
            }
            parser::Command::Index(index, timestamp) => {
                if index == 1 {
                    let start_sec = timestamp.total_seconds();
                    chapters.push(Chapter {
                        timestamp: start_sec,
                        title: current_title.clone(),
                        description: None,
                    });
                }
            }
            _ => {}
        }
    }

    if chapters.is_empty() {
        None
    } else {
        Some(chapters)
    }
}

fn chapters_from_cues(cues: &[symphonia::core::formats::Cue]) -> Option<Vec<Chapter>> {
    for cue in cues {
        let tags = cue.tags.clone();
        if let Some(chapters) = chapters_from_tags(&tags) {
            return Some(chapters);
        }

        // let cp = cue.points.clone();
        // https://docs.rs/symphonia-core/latest/symphonia_core/formats/struct.CuePoint.html
        // Handle parsing the cuepoints here
        // Though, it is quite rare to see chapter markers embedded here
    }

    None
}

fn chapters_from_tags(tags: &[Tag]) -> Option<Vec<Chapter>> {
    let mut chapters = Vec::with_capacity(1);
    for tag in tags {
        match tag.value {
            Value::String(ref s) => {
                if tag.key.to_lowercase().contains("cuesheet") {
                    if let Some(cues) = cuesheet2chapters(s) {
                        chapters.extend(cues);
                    }
                }
            }
            _ => {}
        }
    }
    if chapters.is_empty() {
        None
    } else {
        Some(chapters)
    }
}

/// Default thumbnail: files might have cover/folder . jpg/jpeg/png
///
/// Use those as the default in case no visuals can be extracted
fn local_thumbnails(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let mut default_thumbnail = None;

    let cover_name = vec!["cover", "folder"];
    let cover_extensions = vec!["jpg", "jpeg", "png"];

    for entry in parent.read_dir().ok()? {
        let entry = entry.ok()?;
        let entry = entry.path();
        let file_stem = entry.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !cover_extensions.contains(&ext) {
            continue; // Only check for cover/folder images
        }

        // Check if the file name matches cover or folder
        if cover_name.iter().any(|name| file_stem.contains(name)) {
            let mime_type = match ext {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                _ => "application/octet-stream",
            };

            match std::fs::read(entry) {
                Ok(data) => {
                    default_thumbnail = Some(format!(
                        "data:{};base64,{}",
                        mime_type,
                        general_purpose::STANDARD.encode(data)
                    ));
                    break;
                }
                Err(_e) => {}
            }
        }
    }

    default_thumbnail
}

/// Extract thumbnail from audio file's visuals
///
/// Fallback to `fallback` (typically `local_thumbnails()` if no visuals are found)
fn extract_thumbnail(visuals: &[Visual], default: Option<String>) -> Option<String> {
    if visuals.is_empty() {
        return default;
    }

    let mut area: Vec<u64> = vec![];
    let mut is_square: Vec<bool> = vec![];
    for (_i, vis) in visuals.iter().enumerate() {
        if let Some(dimensions) = &vis.dimensions {
            area.push((dimensions.width * dimensions.height) as u64);
            is_square.push(dimensions.width == dimensions.height);
        } else {
            area.push(0);
            is_square.push(false);
        }
    }

    // First try to find the square with highest resolution. Only if no square found, use the highest res.
    let mut indices: Vec<usize> = (0..visuals.len()).collect();
    indices.sort_by(|&a, &b| {
        // First prioritize square aspect ratio
        let square_cmp = is_square[b].cmp(&is_square[a]);
        if square_cmp != std::cmp::Ordering::Equal {
            return square_cmp;
        }
        // If both are square or both are not square, sort by area (descending)
        area[b].cmp(&area[a])
    });

    let idx = indices[0]; // Best index

    let visual = &visuals[idx];
    let data = &visual.data;
    if !data.is_empty() {
        return Some(picture2base64(&visual.media_type, data));
    }

    default
}

/// Match any CodecType value from 0x100 upto 0x125 as PCM/Wave
fn is_pcm_wave(codec: CodecType) -> bool {
    // Extract the u32 value from CodecType by parsing its hex string representation
    let codec_str = format!("{}", codec);
    if let Some(hex_str) = codec_str.strip_prefix("0x") {
        if let Ok(codec_value) = u32::from_str_radix(hex_str, 16) {
            return (0x100..=0x125).contains(&codec_value);
        }
    }
    false
}

/// Sometimes, Symphonia fails to read ID3 tags written to WAV by Mp3Tag
///
/// In that case, read the ID3 tags using the id3 crate directly.
fn try_fill_id3_tags(metadata: &mut AudioMetadata, path: &Path) {
    let tag = id3::Tag::read_from_path(path).ok();
    if tag.is_none() {
        return; // No ID3 tag found
    }

    let tag = tag.unwrap();

    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let title = tag.title().unwrap_or(file_stem);

    if metadata.name.is_empty() || metadata.name == file_stem {
        metadata.name = title.to_string();
    }
    if metadata.artist.is_none() {
        if let Some(artist) = tag.artist() {
            metadata.artist = Some(artist.to_string());
        }
    }
    if metadata.contributors.is_none() {
        if let Some(album_artist) = tag.album_artist() {
            metadata.contributors = Some(vec![album_artist.to_string()]);
        }
    }
    if metadata.thumbnail.is_none() {
        if let Some(picture) = tag.pictures().next() {
            let mime_type = picture.mime_type.as_str();
            metadata.thumbnail = Some(picture2base64(mime_type, &picture.data));
        }
    }
}

/// Uses Symphonia to parse audio metadata from a file.
pub fn parse_metadata(path: &Path) -> Result<Option<AudioMetadata>, String> {
    let mut result_metadata = AudioMetadata::default();
    result_metadata.path = path.to_string_lossy().to_string();

    // Get file size
    result_metadata.size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();

    // Stem of path as default name
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    result_metadata.name = stem;

    let source = Box::new(File::open(path).map_err(|e| e.to_string())?);
    let probe = symphonia::default::get_probe();
    let stream = MediaSourceStream::new(source, Default::default());
    let hint = Hint::new();

    let probed = probe
        .format(&hint, stream, &Default::default(), &Default::default())
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;

    // Find the first audio track with a decodeable codec.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .expect("no supported audio tracks");
    // Though, prioritize the default track
    let default_track = format.default_track().unwrap_or(track);
    let default_codec = default_track.codec_params.codec.clone();

    result_metadata.sample_rate = default_track.codec_params.sample_rate.unwrap_or(0);
    result_metadata.channels = track.codec_params.channels.map_or(0, |c| c.count() as u32);
    result_metadata.bit_depth = default_track.codec_params.bits_per_sample.unwrap_or(0);
    result_metadata.duration = track
        .codec_params
        .n_frames
        .map_or(0.0, |f| f as f64 / result_metadata.sample_rate as f64);

    let mut metadata = format.metadata();
    let metadata = metadata.skip_to_latest();
    if let Some(rev) = metadata {
        let tags = rev.tags();
        result_metadata.thumbnail = extract_thumbnail(rev.visuals(), local_thumbnails(path));
        result_metadata.chapters = chapters_from_tags(tags);
        result_metadata.try_add_known_tags(tags);
    } else {
        // log::warn!("No extra metadata found in file: {:?}", path);
    }

    // Most of the time, markers and cuesheet are located in metadata tags
    // In the off chance that they are not found, we can look for cues in the format
    let cues = format.cues();
    let chapters_from_cues = chapters_from_cues(cues);
    if result_metadata.chapters.is_none() && chapters_from_cues.is_some() {
        result_metadata.chapters = chapters_from_cues;
    }

    // Just in case Symphonia fails to extract some essential metadata, try to read the ID3 tags directly
    try_fill_id3_tags(&mut result_metadata, path);

    // If we still don't have chapter, and that this file is PCM/Wave based,
    // try to use bwavfile since Symphonia might not be able to extract chapters from it properly
    if result_metadata.chapters.is_none() && is_pcm_wave(default_codec) {
        result_metadata.chapters = try_extract_wav_chapters(path);
    }

    Ok(Some(result_metadata))
}

#[tauri::command]
pub async fn get_media_metadata(path: &Path) -> Result<Option<AudioMetadata>, String> {
    parse_metadata(path)
}
