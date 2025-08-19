use std::path::{PathBuf, Path};

use super::tag_n_vis;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Track {
    pub File: String,
    pub Title: String,
    pub Alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub Tracks: Vec<Track>,

    pub Content: Option<Vec<ContentItem>>,
}

#[tauri::command]
pub async fn parse_playlist(path: &Path) -> Result<Option<Playlist>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read playlist {}: {}", path.display(), e))?;

    let playlist: Playlist = serde_yml::from_str(&content)
        .map_err(|e| format!("Failed to parse playlist {}: {}", path.display(), e))?;
    
    // // Ensure all tracks have valid file paths
    // for track in &playlist.Tracks {
    //     if !Path::new(&track.File).exists() {
    //         return Err(format!("Track file does not exist: {}", track.File).into());
    //     }
    // }

    Ok(Some(playlist))
}