use flac_codec::metadata::contiguous::Contiguous;
use flac_codec::metadata::cuesheet::TrackNonCDDA;
use flac_codec::metadata::fields::{ARTIST, TITLE};
use flac_codec::metadata::{self, Picture, VorbisComment};
use std::fs::File;
use std::io::{Cursor, Seek};
use std::path::Path;
use symphonia::core::meta::{StandardTagKey, Value, Visual};
use symphonia::core::probe::Hint;
use symphonia::core::{audio::AudioBufferRef, io::MediaSourceStream};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use symphonium::{ResampleQuality, SymphoniumLoader};

use super::metadata::{parse_metadata, AudioMetadata};
use super::tag_n_vis;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicTrackMeta {
    pub sample_rate: u32,
    pub channels: u32,
    pub bit_depth: u32,
    pub title: String,
}
impl Default for BasicTrackMeta {
    fn default() -> Self {
        BasicTrackMeta {
            sample_rate: 0,
            channels: 0,
            bit_depth: 0,
            title: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackData {
    pub meta: BasicTrackMeta,
    pub data: Vec<Vec<f32>>, // Audio data samples, separated by channel
    pub nchannels: usize,
}
impl TrackData {
    pub fn data_as_mono(&self) -> Vec<f32> {
        if self.nchannels == 1 {
            return self.data[0].clone();
        }

        let mut mono_data = Vec::new();
        // All channels should have the same length, this is just in case...
        let min_length = self
            .data
            .iter()
            .map(|channel| channel.len())
            .min()
            .unwrap_or(0);

        for i in 0..min_length {
            let sample: f64 = self.data.iter().map(|channel| channel[i] as f64).sum();
            mono_data.push((sample / self.data.len() as f64) as f32);
        }
        mono_data
    }
}

pub fn get_basic_track_meta(path: &Path) -> Result<Option<BasicTrackMeta>, String> {
    let metadata: Option<AudioMetadata> = parse_metadata(path)?;
    let metadata = metadata.ok_or_else(|| "Failed to parse audio metadata".to_string())?;

    Ok(Some(BasicTrackMeta {
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        bit_depth: metadata.bit_depth,
        title: metadata.name,
    }))
}

pub fn load_and_resample_audio(
    path: &Path,
    target_sample_rate: u32,
) -> Result<Option<TrackData>, String> {
    let metadata = get_basic_track_meta(path)?;
    let nchannels = metadata.as_ref().map_or(0, |m| m.channels);

    let mut loader = SymphoniumLoader::new();

    let channel_data = loader
        .load_f32(
            path,
            Some(target_sample_rate),
            ResampleQuality::High,
            Some(2 * 1024 * 1024 * 1024 as usize), // 2GB max buffer size before OOM
        )
        .unwrap();

    assert!(
        channel_data.data.len() == nchannels as usize,
        "Audio data channels do not match metadata"
    );

    Ok(Some(TrackData {
        meta: metadata.unwrap_or_default(),
        data: channel_data.data,
        nchannels: nchannels as usize,
    }))
}

pub fn add_metadata_to_inmem_flac(
    flac: &mut Cursor<Vec<u8>>,
    playlist: &super::multitrack_gen::Playlist,
    cue_data: &super::multitrack_gen::CueData,
    visual: &Option<Visual>,
) -> Result<(), String> {
    let cuesheets = cue_data.cuesheets.clone();
    let cuesheet_tag_names: Vec<String> = if cuesheets.len() == 1 {
        vec!["cuesheet".to_string()]
    } else {
        (0..cuesheets.len())
            .map(|i| format!("cuesheet{}", i + 1))
            .collect()
    };

    let mut rebuilt: Vec<u8> = vec![];

    // Take ownership of the original Cursor so we can pass a &mut Cursor to update_file
    let mut original = std::mem::replace(flac, Cursor::new(Vec::new()));

    let _ = flac_codec::metadata::update_file::<_, _, flac_codec::Error>(
        // pass a mutable reference to the taken cursor
        &mut original,
        // a closure to create a new file, if necessary
        || Ok(&mut rebuilt),
        // the closure that performs the metadata update
        |blocklist| {
            blocklist.update::<VorbisComment>(|vc| {
                for (i, cuesheet) in cuesheets.iter().enumerate() {
                    let tag_name = &cuesheet_tag_names[i];
                    vc.insert(tag_name, cuesheet);
                }

                let ignored_playlist_fields: HashSet<&str> =
                    ["AlbumCover", "RandSeed", "Tracks", "Pauses", "Content"]
                        .into_iter()
                        .collect();

                if let Ok(json) = serde_json::to_value(playlist) {
                    if let JsonValue::Object(map) = json {
                        for (key, value) in map.iter() {
                            if ignored_playlist_fields.contains(key.as_str()) {
                                continue;
                            }
                            if value.is_null() {
                                continue;
                            }

                            let value_str = match value {
                                JsonValue::String(s) => s.clone(),
                                JsonValue::Number(n) => n.to_string(),
                                JsonValue::Bool(b) => b.to_string(),
                                JsonValue::Array(_) | JsonValue::Object(_) => {
                                    serde_json::to_string(value).unwrap_or_default()
                                }
                                _ => continue,
                            };

                            if !value_str.is_empty() {
                                if key == "TrackTitle" {
                                    vc.insert("Title", value_str.clone());
                                }
                                vc.insert(key, value_str);
                            }
                        }
                    }
                }
            });

            if let Some(visual) = visual {
                let picture = tag_n_vis::visual_to_flac_picture(visual);
                blocklist.insert::<Picture>(picture);
            }
            Ok(())
        },
    );

    // Put back either the rebuilt data (if produced) or the original cursor
    if !rebuilt.is_empty() {
        *flac = Cursor::new(rebuilt);
    } else {
        *flac = original;
    }

    Ok(())
}
