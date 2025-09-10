use tauri::ipc::Response;

use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::pauses;
use super::tag_n_vis;
use super::tracks;
use serde::{Deserialize, Serialize};

use flac_codec::encode::{FlacChannelWriter, Options};
use std::io::{Cursor, Seek};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct Track {
    pub File: String,
    pub Title: String,
    pub Alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct Pause {
    pub Signature: String,
    pub Title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[allow(non_snake_case)]
#[serde(untagged)]
pub enum ContentItem {
    Scalar(Option<String>),
    List(Vec<Option<String>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Playlist {
    pub AcoustidFingerprint: Option<String>,
    pub AcoustidId: Option<String>,
    pub Album: Option<String>,
    pub AlbumArtist: Option<String>,
    pub Arranger: Option<String>,
    pub Artist: Option<String>,
    pub Bpm: Option<u32>,
    pub Comment: Option<String>,
    pub Compilation: Option<String>,
    pub Composer: Option<String>,
    pub Conductor: Option<String>,
    pub ContentGroup: Option<String>,
    pub Copyright: Option<String>,
    pub Date: Option<String>,
    pub Description: Option<String>,
    pub DiscNumber: Option<u32>,
    pub DiscSubtitle: Option<String>,
    pub DiscTotal: Option<u32>,
    pub EncodedBy: Option<String>,
    pub Encoder: Option<String>,
    pub EncoderSettings: Option<String>,
    pub EncodingDate: Option<String>,
    pub Engineer: Option<String>,
    pub Ensemble: Option<String>,
    pub Genre: Option<String>,
    pub IdentAsin: Option<String>,
    pub IdentBarcode: Option<String>,
    pub IdentCatalogNumber: Option<String>,
    pub IdentEanUpn: Option<String>,
    pub IdentIsrc: Option<String>,
    pub IdentPn: Option<String>,
    pub IdentPodcast: Option<String>,
    pub IdentUpc: Option<String>,
    pub Label: Option<String>,
    pub Language: Option<String>,
    pub License: Option<String>,
    pub Lyricist: Option<String>,
    pub Lyrics: Option<String>,
    pub MediaFormat: Option<String>,
    pub MixDj: Option<String>,
    pub MixEngineer: Option<String>,
    pub Mood: Option<String>,
    pub MovementName: Option<String>,
    pub MovementNumber: Option<u32>,
    pub MusicBrainzAlbumArtistId: Option<String>,
    pub MusicBrainzAlbumId: Option<String>,
    pub MusicBrainzArtistId: Option<String>,
    pub MusicBrainzDiscId: Option<String>,
    pub MusicBrainzGenreId: Option<String>,
    pub MusicBrainzLabelId: Option<String>,
    pub MusicBrainzOriginalAlbumId: Option<String>,
    pub MusicBrainzOriginalArtistId: Option<String>,
    pub MusicBrainzRecordingId: Option<String>,
    pub MusicBrainzReleaseGroupId: Option<String>,
    pub MusicBrainzReleaseStatus: Option<String>,
    pub MusicBrainzReleaseTrackId: Option<String>,
    pub MusicBrainzReleaseType: Option<String>,
    pub MusicBrainzTrackId: Option<String>,
    pub MusicBrainzWorkId: Option<String>,
    pub Opus: Option<String>,
    pub OriginalAlbum: Option<String>,
    pub OriginalArtist: Option<String>,
    pub OriginalDate: Option<String>,
    pub OriginalFile: Option<String>,
    pub OriginalWriter: Option<String>,
    pub Owner: Option<String>,
    pub Part: Option<u32>,
    pub PartTotal: Option<u32>,
    pub Performer: Option<String>,
    pub Podcast: Option<String>,
    pub PodcastCategory: Option<String>,
    pub PodcastDescription: Option<String>,
    pub PodcastKeywords: Option<String>,
    pub Producer: Option<String>,
    pub PurchaseDate: Option<String>,
    pub Rating: Option<u32>,
    pub ReleaseCountry: Option<String>,
    pub ReleaseDate: Option<String>,
    pub Remixer: Option<String>,
    pub ReplayGainAlbumGain: Option<String>,
    pub ReplayGainAlbumPeak: Option<String>,
    pub ReplayGainTrackGain: Option<String>,
    pub ReplayGainTrackPeak: Option<String>,
    pub Script: Option<String>,
    pub SortAlbum: Option<String>,
    pub SortAlbumArtist: Option<String>,
    pub SortArtist: Option<String>,
    pub SortComposer: Option<String>,
    pub SortTrackTitle: Option<String>,
    pub TaggingDate: Option<String>,
    pub TrackNumber: Option<u32>,
    pub TrackSubtitle: Option<String>,
    pub TrackTitle: Option<String>,
    pub TrackTotal: Option<u32>,
    pub TvEpisode: Option<u32>,
    pub TvEpisodeTitle: Option<String>,
    pub TvNetwork: Option<String>,
    pub TvSeason: Option<u32>,
    pub TvShowTitle: Option<String>,
    pub Url: Option<String>,
    pub UrlArtist: Option<String>,
    pub UrlCopyright: Option<String>,
    pub UrlInternetRadio: Option<String>,
    pub UrlLabel: Option<String>,
    pub UrlOfficial: Option<String>,
    pub UrlPayment: Option<String>,
    pub UrlPodcast: Option<String>,
    pub UrlPurchase: Option<String>,
    pub UrlSource: Option<String>,
    pub Version: Option<String>,
    pub Writer: Option<String>,

    pub AlbumCover: Option<String>,

    pub RandSeed: Option<String>,

    pub Tracks: Option<Vec<Track>>,
    pub Pauses: Option<Vec<Pause>>,

    pub Content: Option<Vec<ContentItem>>,
}

/// Ensure track paths exists, both in Tracks and in Content
///
/// Also makes sure alias references in Content map to valid tracks
fn ensure_tracks(playlist: &Playlist) -> Result<(), String> {
    if let Some(ref tracks) = playlist.Tracks {
        for track in tracks {
            if !Path::new(&track.File).exists() {
                return Err(format!("Track file does not exist: {}", track.File).into());
            }
        }
    }

    let mut aliases = HashSet::new();
    if let Some(ref tracks) = playlist.Tracks {
        for track in tracks {
            if track.Alias.is_none() {
                continue;
            }
            let alias = track.Alias.as_ref().unwrap();
            if !aliases.insert(alias.clone()) {
                return Err(format!("Duplicate track alias found: {}", alias).into());
            }
        }
    }

    if !playlist.Content.is_some() {
        return Ok(());
    }

    let content = playlist.Content.as_ref().unwrap();

    // Ensure ContentItem entries are either: path to file or exists in aliases
    for content in content {
        match content {
            ContentItem::Scalar(Some(ref path)) => {
                if path.starts_with("pause_") || path == "_" {
                    continue;
                }
                if !Path::new(path).exists() && !aliases.contains(path) {
                    return Err(format!(
                        "ContentItem value does not exist or is not a valid alias: {}",
                        path
                    )
                    .into());
                }
            }
            ContentItem::List(ref items) => {
                for item in items {
                    if let Some(ref path) = item {
                        if path.starts_with("pause_") || path == "_" {
                            continue;
                        }
                        if !Path::new(path).exists() && !aliases.contains(path) {
                            return Err(format!(
                                "ContentItem list item does not exist or is not a valid alias: {}",
                                path
                            )
                            .into());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Preprocess Content:
///
/// - Remove ContentItem elems from Content if
///     + Scalar values are empty or == "_"
///     + All values in List are empty or == "_"
/// - Resolve track alias in Content such that Content elements are valid paths
///
/// Ideally, `ensure_tracks()` should be run beforehand to make sure content entries are valid (mapped to some aliases or is valid paths)
fn preprocess_content(playlist: &mut Playlist) -> Vec<PathBuf> {
    if let Some(ref mut content) = playlist.Content {
        content.retain(|item| match item {
            ContentItem::Scalar(Some(ref value)) => !value.is_empty() && value != "_",
            ContentItem::List(ref items) => items
                .iter()
                .any(|item| item.as_ref().map_or(false, |v| !v.is_empty() && v != "_")),
            _ => false,
        });
    }

    if playlist.Content.is_none() {
        return Vec::new();
    }

    let alias_map: HashMap<String, String> = if playlist.Tracks.is_some() {
        playlist
            .Tracks
            .as_ref()
            .unwrap()
            .iter()
            .filter_map(|track| {
                track
                    .Alias
                    .as_ref()
                    .map(|alias| (alias.clone(), track.File.clone()))
            })
            .collect()
    } else {
        HashMap::new()
    };

    let mut unique_tracks: HashSet<PathBuf> = if playlist.Tracks.is_some() {
        playlist
            .Tracks
            .as_ref()
            .unwrap()
            .iter()
            .map(|track| PathBuf::from(&track.File))
            .collect()
    } else {
        HashSet::new()
    };

    // Resolve aliases in Content: operate on the inner Vec
    if let Some(ref mut content) = playlist.Content {
        for entry in content.iter_mut() {
            match entry {
                ContentItem::Scalar(ref mut value) => {
                    if let Some(ref v) = value {
                        if let Some(resolved) = alias_map.get(v) {
                            *value = Some(resolved.clone());
                        } else {
                            if !v.starts_with("pause_") && v != "_" {
                                unique_tracks.insert(PathBuf::from(v));
                            }
                        }
                    }
                }
                ContentItem::List(ref mut items) => {
                    for item in items.iter_mut() {
                        if let Some(ref v) = item {
                            if let Some(resolved) = alias_map.get(v) {
                                *item = Some(resolved.clone());
                            } else {
                                if !v.starts_with("pause_") && v != "_" {
                                    unique_tracks.insert(PathBuf::from(v));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    unique_tracks.into_iter().collect()
}

/// Preprocess a playlist and Extract unique track paths
fn parse_playlist(path: &Path) -> Result<(Option<Playlist>, Vec<PathBuf>), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read playlist {}: {}", path.display(), e))?;

    let mut playlist: Playlist = serde_yml::from_str(&content)
        .map_err(|e| format!("Failed to parse playlist {}: {}", path.display(), e))?;

    ensure_tracks(&playlist)?;
    let unique_tracks = preprocess_content(&mut playlist);

    Ok((Some(playlist), unique_tracks))
}

/// Convert a sample stamp (relative to start) into the MM:SS:FF format for cuesheet
///
/// As per standard, there are 75 frames per second
fn sample_to_msf_time(sample: u64, sample_rate: u32) -> (u32, u32, u32) {
    let total_seconds = sample as f64 / sample_rate as f64;
    // The closest MM, SS, and FF
    let minutes = total_seconds as u32 / 60;
    let seconds = total_seconds as u32 % 60;
    // Round to the nearest frame
    let frames = (total_seconds.fract() * 75.0).round() as u32;
    (minutes, seconds, frames)
}
fn format_msf_time(msf: (u32, u32, u32)) -> String {
    let (m, s, f) = msf;
    format!("{:02}:{:02}:{:02}", m, s, f)
}

fn lookup_pause_title(signature: &str, pauses: &[Pause]) -> Option<String> {
    for pause in pauses {
        if pause.Signature == signature {
            return Some(pause.Title.clone());
        }
    }
    None
}
fn lookup_track_title(path: &Path, tracks: &[Track]) -> Option<String> {
    for track in tracks {
        if Path::new(&track.File) == path {
            return Some(track.Title.clone());
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct CueData {
    pub cuesheets: Vec<String>,

    // Due to lots of limitations of Cuesheet (resolution only 75 frames per second && max 99 tracks)
    // Also create cues to append to the track
    // with Tags: TrackTitle + Comment { sample: exact sample this track start, timestamp_ms: <timestamp> }
    pub cues: Vec<symphonia::core::formats::Cue>,
}

fn append_cue_track(
    cuesheet: &mut String,
    cues: &mut Vec<symphonia::core::formats::Cue>,
    track_number: u32,
    title: &str,
    start_sample: u64,
    sample_rate: u32,
) {
    let msf_time = sample_to_msf_time(start_sample, sample_rate);
    let msf_time_str = format_msf_time(msf_time);
    // Cuesheet track numbers are 1..99 (wrap using modulo arithmetic)
    // `track_number` is a monotonically increasing counter; map it into 1..99 range
    let cuesheet_track_number = ((track_number - 1) % 99) + 1;
    cuesheet.push_str(&format!("  TRACK {:02} AUDIO\n", cuesheet_track_number));
    cuesheet.push_str(&format!("    TITLE \"{}\"\n", title));
    cuesheet.push_str(&format!("    INDEX 01 {}\n", msf_time_str));
    cuesheet.push_str(&format!("    REM STARTSAMPLE {}\n", start_sample));

    let title_tag = symphonia::core::meta::Tag {
        std_key: Some(symphonia::core::meta::StandardTagKey::TrackTitle),
        key: "TrackTitle".to_string(),
        value: symphonia::core::meta::Value::String(title.to_string()),
    };
    let title_tag_too = symphonia::core::meta::Tag {
        std_key: Some(symphonia::core::meta::StandardTagKey::TrackTitle),
        key: "Title".to_string(),
        value: symphonia::core::meta::Value::String(title.to_string()),
    };
    let comment_tag = symphonia::core::meta::Tag {
        std_key: Some(symphonia::core::meta::StandardTagKey::Comment),
        key: "Comment".to_string(),
        value: symphonia::core::meta::Value::String(format!(
            "start_sample = {}\ntimestamp_ms = {}",
            start_sample,
            (start_sample as f64 / sample_rate as f64 * 1000.0).round() as u64
        )),
    };

    let nearest_start_frame = (start_sample * 75 / sample_rate as u64) as u64;

    let cue = symphonia::core::formats::Cue {
        index: track_number,
        tags: vec![title_tag, title_tag_too, comment_tag],
        start_ts: nearest_start_frame,
        points: Vec::new(),
    };
    cues.push(cue);
}

/// Loop over playlist.Content and find the max number of element in each entry
fn find_max_elements_in_playlist(content: &[ContentItem]) -> usize {
    content
        .iter()
        .filter_map(|entry| {
            if let ContentItem::List(ref items) = entry {
                Some(items.len())
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
}

/// Generate audio data and fill an existing Vec
///
/// Spreading of channels in the output file depends on the source material:
///   - If source is mono, the data will be duplicated across all channels.
///   - If source is stereo, odd channels will contain the left channel data and even channels will contain the right channel data.
///   - If source is multi-channel, the channels will be interleaved, repeating as necessary until all output channels are filled.
fn generate_audio_data(
    audio_data: &mut Vec<Vec<f32>>,
    rng_generator: &mut StdRng,
    sample_rate: u32,
    tracks_data: &HashMap<PathBuf, tracks::TrackData>,
    playlist: &Playlist,
    playlist_file_stem: &str,
) -> Result<CueData, String> {
    let mut cuesheets: Vec<String> = Vec::new(); // All cuesheet just in case there are > 99 tracks
    let mut cues: Vec<symphonia::core::formats::Cue> = Vec::new(); // Cues to append to the track

    let content = playlist
        .Content
        .as_ref()
        .ok_or("Playlist content is empty")?;
    let n_entries = content.len();
    let n_cuesheets = (n_entries as f32 / 99.0).ceil() as usize;

    for _ in 0..n_cuesheets {
        let mut cuesheet = String::new();
        cuesheet.push_str(&format!(
            "PERFORMER \"{}\"\n",
            playlist
                .Artist
                .clone()
                .unwrap_or_else(|| "Unknown Artist".to_string())
        ));
        cuesheet.push_str(&format!(
            "TITLE \"{}\"\n",
            playlist
                .TrackTitle
                .clone()
                .unwrap_or_else(|| playlist_file_stem.to_string())
        ));
        cuesheet.push_str(&format!(
            "FILE \"{}.flac\" WAVE\n",
            playlist
                .TrackTitle
                .clone()
                .unwrap_or_else(|| playlist_file_stem.to_string())
        ));
        cuesheets.push(cuesheet);
    }

    let nchannels = audio_data.len();

    let mut counter: u32 = 0;

    for (entry_index, entry) in content.iter().enumerate() {
        let cuesheet_index = entry_index / 99;

        let last_audio_sample = audio_data
            .iter()
            .map(|channel| channel.len())
            .max()
            .unwrap_or(0) as u64;

        match entry {
            ContentItem::Scalar(Some(ref value)) => {
                if value.starts_with("pause_") || value == "_" {
                    // Look up pause title if playlist.Pauses is_some, otherwise use value
                    let mut title = if let Some(ref pauses) = playlist.Pauses {
                        lookup_pause_title(value, pauses)
                    } else {
                        Some(value.to_string())
                    };

                    let pause_data =
                        pauses::create_pause(value, title.clone(), sample_rate, rng_generator);
                    // Fill the channels
                    for channel in audio_data.iter_mut() {
                        channel.extend_from_slice(&pause_data.data);
                    }

                    if pause_data.title.is_some() {
                        title = pause_data.title;
                    }

                    counter += 1;
                    append_cue_track(
                        &mut cuesheets[cuesheet_index],
                        &mut cues,
                        counter,
                        title.as_deref().unwrap_or(&pause_data.signature),
                        last_audio_sample + 1,
                        sample_rate,
                    );
                    continue;
                }
                if let Some(track_data) = tracks_data.get(&PathBuf::from(value)) {
                    // Determine how to add track data channels
                    let track_channels = track_data.nchannels as usize;
                    match track_channels {
                        0 => {
                            // No channels, skip
                            continue;
                        }
                        1 => {
                            // Mono track, duplicate across all channels
                            for channel in audio_data.iter_mut() {
                                channel.extend_from_slice(&track_data.data[0]);
                            }
                        }
                        2 => {
                            // Stereo track, odd output channels will be source[0], even output channels will be source[1]
                            for (i, channel) in audio_data.iter_mut().enumerate() {
                                if i % 2 == 0 {
                                    channel.extend_from_slice(&track_data.data[1]);
                                } else {
                                    channel.extend_from_slice(&track_data.data[0]);
                                }
                            }
                        }
                        _ => {
                            // Multi-channel track, repeat channels until audio_data.len() is filled
                            let src_channels = track_data.nchannels;
                            let mut src_index = 0;
                            for channel in audio_data.iter_mut() {
                                channel.extend_from_slice(&track_data.data[src_index]);
                                src_index = (src_index + 1) % src_channels;
                            }
                        }
                    }
                    counter += 1;

                    let track_title = if let Some(ref tracks) = playlist.Tracks {
                        lookup_track_title(&PathBuf::from(value), tracks)
                    } else {
                        Some(track_data.meta.title.clone())
                    };

                    append_cue_track(
                        &mut cuesheets[cuesheet_index],
                        &mut cues,
                        counter,
                        track_title.as_deref().unwrap_or(&track_data.meta.title),
                        last_audio_sample + 1,
                        sample_rate,
                    );
                } else {
                    return Err(format!("Track not found in data: {}", value));
                }
            }
            ContentItem::List(ref channels) => {
                let mut entry_max_samples: usize = 0;
                let mut this_entry_data: Vec<Vec<f32>> = Vec::with_capacity(nchannels);
                let mut entry_title: Vec<String> = Vec::new(); // {[i] title + ...}
                for (i, channel) in channels.iter().enumerate() {
                    if let Some(ref value) = channel {
                        if value.starts_with("pause_") || value == "_" {
                            // Look up pause title if playlist.Pauses is_some, otherwise use value
                            let mut title = if let Some(ref pauses) = playlist.Pauses {
                                lookup_pause_title(value, pauses)
                            } else {
                                Some(value.to_string())
                            };

                            let pause_data = pauses::create_pause(
                                value,
                                title.clone(),
                                sample_rate,
                                rng_generator,
                            );

                            entry_max_samples = entry_max_samples.max(pause_data.data.len());
                            this_entry_data.push(pause_data.data);

                            if pause_data.title.is_some() {
                                title = pause_data.title;
                            }
                            entry_title.push(format!(
                                "[Ch{}] {}",
                                i + 1,
                                title.unwrap_or_else(|| pause_data.signature.clone())
                            ));
                            continue;
                        }
                        if let Some(track_data) = tracks_data.get(&PathBuf::from(value)) {
                            // Push a mono version of this track to the corresponding channel
                            let mono_data = track_data.data_as_mono();

                            entry_max_samples = entry_max_samples.max(mono_data.len());
                            this_entry_data.push(mono_data);

                            let track_title = if let Some(ref tracks) = playlist.Tracks {
                                lookup_track_title(&PathBuf::from(value), tracks)
                            } else {
                                Some(track_data.meta.title.clone())
                            };

                            entry_title.push(format!(
                                "[Ch{}] {}",
                                i + 1,
                                track_title.unwrap_or(track_data.meta.title.clone())
                            ));
                        } else {
                            return Err(format!("Track not found in data: {}", value));
                        }
                    } else {
                        this_entry_data.push(vec![0.0; entry_max_samples]);
                    }
                }
                // Fill all channels to the maximum entry length
                for channel in this_entry_data.iter_mut() {
                    channel.resize(entry_max_samples, 0.0);
                }
                // Add to audio data
                for (i, channel) in this_entry_data.iter().enumerate() {
                    audio_data[i].extend_from_slice(channel);
                }
                // Create a title for this entry
                let entry_title = entry_title.join(" + ");
                counter += 1;
                append_cue_track(
                    &mut cuesheets[cuesheet_index],
                    &mut cues,
                    counter,
                    &entry_title,
                    last_audio_sample + 1,
                    sample_rate,
                );
            }
            _ => {}
        }
    }

    for cuesheet in &mut cuesheets {
        cuesheet.insert_str(0, &format!("REM RATE {}\n", sample_rate));
        cuesheet.insert_str(0, &format!("REM TOTALSAMPLES {}\n", audio_data[0].len()));
    }
    // If there are more than 2 cuesheets, also add a REM to indicate the part number
    if n_cuesheets > 1 {
        for (i, cuesheet) in cuesheets.iter_mut().enumerate() {
            cuesheet.insert_str(0, &format!("REM CUESHEETPART {}\n", i + 1));
        }
    }

    Ok(CueData {
        cuesheets: cuesheets,
        cues: cues,
    })
}

#[tauri::command]
pub async fn audio_from_playlist(path: &Path) -> Result<Response, String> {
    let (playlist, unique_tracks) =
        parse_playlist(path).map_err(|e| format!("Error parsing playlist: {}", e))?;
    if playlist.is_none() {
        return Err("Failed to parse playlist or playlist is empty".into());
    }

    let playlist = playlist.unwrap();

    let visual = if let Some(ref cover) = playlist.AlbumCover {
        tag_n_vis::parse_visual("AlbumCover", cover.as_str())
    } else {
        None
    };

    // Normalize RandSeed into a fixed 32-byte array (zero-padded if needed)
    let randseed = if let Some(ref seed_str) = playlist.RandSeed {
        let bytes = seed_str.as_bytes();
        let mut seed_arr = [0u8; 32];
        let len = bytes.len().min(32);
        seed_arr[..len].copy_from_slice(&bytes[..len]);
        Some(seed_arr)
    } else {
        None
    };

    // A shared RNG instance to be used for all random operations in this playlist
    let mut rng = if let Some(seed_arr) = randseed {
        StdRng::from_seed(seed_arr)
    } else {
        // Default seed
        let seed = rand::random::<[u8; 32]>();
        StdRng::from_seed(seed)
    };

    // Preload tracks and extract metadata
    let mut tracks_metadata = Vec::new();
    for track_path in &unique_tracks {
        let metadata = tracks::get_basic_track_meta(track_path).map_err(|e| {
            format!(
                "Error reading track metadata for {}: {}",
                track_path.display(),
                e
            )
        })?;
        if metadata.is_none() {
            continue;
        }
        tracks_metadata.push(metadata.unwrap());
    }

    // Determine the overall track settings from the loaded metadata
    let (mut max_bit_depth, mut max_channels, mut max_sample_rate) = tracks_metadata.iter().fold(
        (0, 0, 0),
        |(max_bit_depth, max_channels, max_sample_rate), meta| {
            (
                max_bit_depth.max(meta.bit_depth),
                max_channels.max(meta.channels),
                max_sample_rate.max(meta.sample_rate),
            )
        },
    );
    if max_sample_rate == 0 {
        max_sample_rate = 44100; // default to 44.1kHz if no tracks
    }
    if max_bit_depth == 0 {
        max_bit_depth = 16; // default to 16 bits if no tracks
    }

    if playlist.Content.is_some() {
        max_channels = max_channels
            .max(find_max_elements_in_playlist(&playlist.Content.as_ref().unwrap()) as u32);
    }

    if max_channels == 0 {
        max_channels = 1; // default to mono if no tracks
    }

    let mut tracks_data: HashMap<PathBuf, tracks::TrackData> = HashMap::new();
    for track_path in &unique_tracks {
        let track_data = tracks::load_and_resample_audio(track_path, max_sample_rate)
            .map_err(|e| format!("Error loading track {}: {}", track_path.display(), e))?;
        if track_data.is_none() {
            continue;
        }
        tracks_data.insert(track_path.clone(), track_data.unwrap());
    }

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("playlist");

    let mut audio_data: Vec<Vec<f32>> = Vec::with_capacity(max_channels as usize);
    for _ in 0..max_channels {
        audio_data.push(Vec::new());
    }

    let cue_data = generate_audio_data(
        &mut audio_data,
        &mut rng,
        max_sample_rate,
        &tracks_data,
        &playlist,
        file_stem,
    )?;

    // Convert audio_data (f32 in -1.0..1.0 range) to integer PCM for writing FLAC.
    // Scale according to bit depth and clamp. Support common bit depths (16, 24, 32).
    let bits = max_bit_depth.max(16); // at least 16 bits
    let max_amplitude: i64 = match bits {
        b if b <= 16 => i16::MAX as i64,
        b if b <= 24 => (1i64 << 23) - 1,
        _ => i32::MAX as i64,
    };

    let audio_data_i32: Vec<Vec<i32>> = audio_data
        .into_iter()
        .map(|channel| {
            channel
                .into_iter()
                .map(|sample| {
                    // ensure sample is in -1.0..1.0, scale to integer range
                    let s = sample.max(-1.0).min(1.0);
                    let scaled = (s * (max_amplitude as f32)).round() as i64;
                    // clamp to the integer limits for the given depth
                    let clamped = if bits <= 16 {
                        scaled.max(i16::MIN as i64).min(i16::MAX as i64) as i32
                    } else if bits <= 24 {
                        scaled
                            .max(-((1i64 << 23) as i64))
                            .min(((1i64 << 23) - 1) as i64) as i32
                    } else {
                        scaled.max(i32::MIN as i64).min(i32::MAX as i64) as i32
                    };
                    clamped
                })
                .collect()
        })
        .collect();
    let nsamples = audio_data_i32.get(0).map(|c| c.len()).unwrap_or(0) as u64;

    let mut flac = Cursor::new(vec![]); // a FLAC file in memory

    let mut writer = FlacChannelWriter::new(
        &mut flac,
        Options::best(),
        max_sample_rate,
        max_bit_depth,
        max_channels as u8,
        Some(nsamples),
    )
    .unwrap();
    writer
        .write(&audio_data_i32)
        .map_err(|e| format!("Error writing FLAC data: {}", e))?;
    writer
        .finalize()
        .map_err(|e| format!("Error finalizing FLAC data: {}", e))?;

    flac.rewind().unwrap();

    // Add metadata to the FLAC file
    tracks::add_metadata_to_inmem_flac(&mut flac, &playlist, &cue_data, &visual)
        .map_err(|e| format!("Error adding metadata to FLAC: {}", e))?;

    flac.rewind().unwrap();

    Ok(Response::new(flac.into_inner()))
}
