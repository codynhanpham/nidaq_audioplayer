use tauri::utils::mime_type;
use tauri::Manager;

use audiotags::{Tag, Picture, MimeType};
use flac::metadata;
use cue_sheet::parser::{self, Command};

use std::fmt::Debug;
use std::{default, hint, time};
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};


use symphonia::core::formats::FormatReader;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

struct SimpleMetadata {
    sample_rate: u32,
    channels: usize,
    bit_depth: u32,
    duration: f64,
}

fn extract_basic_metadata(path: &Path) -> Result<SimpleMetadata, Box<dyn std::error::Error>> {
    let source = Box::new(File::open(path)?);
    let probe = symphonia::default::get_probe();
    let stream = MediaSourceStream::new(source, Default::default());
    let hint = Hint::new();

    let format = probe.format(&hint, stream, &Default::default(), &Default::default())?.format;

    let track = format.default_track().ok_or("No default track found")?;
    let sample_rate = track.codec_params.sample_rate.ok_or("Sample rate not found")?;
    let nchannels = track.codec_params.channels.map_or(0, |c| c.count());
    let bit_depth = track.codec_params.bits_per_sample.ok_or("Bit depth not found")?;
    let duration = track.codec_params.n_frames.map_or(0.0, |f| f as f64 / sample_rate as f64);

    Ok(SimpleMetadata {
        sample_rate,
        channels: nchannels,
        bit_depth,
        duration,
    })
}



fn format_mime_type(mime_type: &MimeType) -> String {
    match mime_type {
        MimeType::Png => "image/png".to_string(),
        MimeType::Jpeg => "image/jpeg".to_string(),
        MimeType::Tiff => "image/tiff".to_string(),
        MimeType::Bmp => "image/bmp".to_string(),
        MimeType::Gif => "image/gif".to_string(),
    }
}

fn picture2base64(picture: &Picture) -> String {
    let data = general_purpose::STANDARD.encode(picture.data);
    format!("data:{};base64,{}", format_mime_type(&picture.mime_type), data)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    pub timestamp: f64, // Start Timestamp in seconds
    pub title: String,
    pub description: Option<String>, // Optional description of the chapter
}

fn parse_cue_commands(commands: &Vec<Command>) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let mut current_title = String::new();

    for command in commands {
        match command {
            Command::Title(title) => {
                current_title = title.clone();
            }
            Command::Index(index, timestamp) => {
                if *index == 1 {
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

    chapters
}

fn extract_chapters(path: &Path) -> Option<Vec<Chapter>> {
    match metadata::get_vorbis_comment(path.to_str()?) {
        Ok(vorbis_comment) => {
            let mut cuesheet = vorbis_comment.comments.get("cuesheet");
            if cuesheet.is_none() {
                cuesheet = vorbis_comment.comments.get("CUESHEET");
            }
            // Add more variations here!

            // Finalize parsing from file
            if cuesheet.is_none() {
                return None
            }

            // Extract cuesheet data
            // Remove all string before "TRACK" as sometimes REM commands include invalid/unparse-able data
            let cuesheet = cuesheet.unwrap();
            let track_data = cuesheet.lines()
                .skip_while(|line| !line.trim().starts_with("FILE"))
                .collect::<Vec<_>>()
                .join("\n");

            let cuedata = parser::parse_cue(&track_data);
            if cuedata.is_err() {
                log::warn!("Failed to parse cuesheet: {:?}", cuedata);
                return None;
            }

            let chapters = parse_cue_commands(&cuedata.unwrap());
            Some(chapters)
        }
        Err(error) => {
            log::warn!("Failed to extract vorbis comments from {:?}: {:?}", path, error);
            None
        }
    }
}


/// Extract the first image in file using the flac api
fn extract_flac_picture(path: &Path) -> Option<String> {
    let tag = metaflac::Tag::read_from_path(path).ok()?;
    let picture = tag.pictures().next()?;
    let data = general_purpose::STANDARD.encode(&picture.data);
    let mime_type = match picture.mime_type.as_str() {
        "image/png" => "image/png",
        "image/jpeg" => "image/jpeg",
        "image/tiff" => "image/tiff",
        "image/bmp" => "image/bmp",
        "image/gif" => "image/gif",
        _ => "application/octet-stream", // Fallback for unknown types
    };
    Some(format!("data:{};base64,{}", mime_type, data))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioMetadata {
    pub name: String, // Name of the audio file
    pub artist: Option<String>, // Primary artist or short description of the audio file
    pub contributors: Option<Vec<String>>, // Additional contributing artist or information
    pub thumbnail: Option<String>, // Path to the thumbnail image or Base64 encoded string
    pub path: String, // Path to the audio file
    pub duration: f64, // Duration of the audio file in seconds
    pub size: u64, // Size of the audio file in bytes
    pub sample_rate: u32, // Sample rate of the audio file
    pub channels: u32, // Number of audio channels
    pub bit_depth: u32, // Bit depth of the audio file

    pub chapters: Option<Vec<Chapter>>, // Optional chapters in the audio file
}


#[tauri::command]
pub async fn get_media_metadata(path: &Path) -> Result<Option<AudioMetadata>, String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Ok(None);
    }

    let file_size = path.metadata()
        .map_err(|e| e.to_string())?
        .len();

    let clean_file_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let basic_metadata = extract_basic_metadata(path).unwrap_or(SimpleMetadata {
        sample_rate: 0,
        channels: 0,
        bit_depth: 0,
        duration: 0.0,
    });

    let mut metadata = AudioMetadata {
        name: clean_file_name.clone(),
        artist: None,
        contributors: None,
        thumbnail: None,
        path: path.to_string_lossy().into_owned(),
        duration: basic_metadata.duration,
        size: file_size,
        sample_rate: basic_metadata.sample_rate,
        channels: basic_metadata.channels as u32,
        bit_depth: basic_metadata.bit_depth as u32,
        chapters: None,
    };

    // If file is .wav, return as it does not contain metadata
    if path.extension().map(|s| s == "wav").unwrap_or(false) {
        // Check the parent folder for cover art
        if let Some(parent) = path.parent() {
            let cover_name = vec!["cover", "folder"];
            let cover_extensions = vec!["jpg", "jpeg", "png"];
            for name in cover_name {
                for ext in cover_extensions.clone() {
                    let cover_path = parent.join(format!("{}.{}", name, ext));
                    if cover_path.exists() {
                        let mime_type = match ext {
                            "jpg" | "jpeg" => "image/jpeg",
                            "png" => "image/png",
                            _ => "application/octet-stream",
                        };
                        match std::fs::read(cover_path) {
                            Ok(data) => {
                                metadata.thumbnail = Some(format!("data:{};base64,{}", mime_type, general_purpose::STANDARD.encode(data)));
                                break;
                            }
                            Err(e) => {
                                log::warn!("Failed to read cover art file: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
        return Ok(Some(metadata));
    }

    let tag = Tag::new().read_from_path(path)
        .map_err(|e| e);
    match tag {
        Ok(tag) => {
            // Successfully read the tag
            metadata.name = tag.title().unwrap_or(&clean_file_name).to_string();
            metadata.artist = tag.artist().map(|s| s.to_string());
            metadata.contributors = tag.album_artists()
                .map(|artists| artists.iter().map(|s| s.to_string()).collect());
            metadata.thumbnail = tag.album_cover().map(|p| picture2base64(&p));
            metadata.path = path.to_string_lossy().into_owned();
            metadata.duration = tag.duration().unwrap_or(0.0);
            metadata.size = file_size;
            metadata.sample_rate = basic_metadata.sample_rate;
            metadata.channels = basic_metadata.channels as u32;
            metadata.chapters = None;
            
            // If media is flac, run extract_chapters
            if path.extension().map(|s| s == "flac").unwrap_or(false) {
                metadata.chapters = extract_chapters(&path);
                if metadata.thumbnail.is_none() {
                    // If no cover is found, try to read using the flac api
                    metadata.thumbnail = extract_flac_picture(&path);
                }
            }

            Ok(Some(metadata))
        }
        Err(e) => Err({
            log::error!("{:?}", e);
            format!("{:?}", e)
        }),
    }
}