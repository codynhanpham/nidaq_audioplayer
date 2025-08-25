use std::path::Path;

use mime::Mime;
use reqwest::blocking::Client;

use base64::{engine::general_purpose, Engine as _};
use symphonia::core::meta::StandardTagKey;

/// Given a string matching some StandardTagKey, return the corresponding StandardTagKey if it exists.
pub fn metatagstr2stdtagkey(tagkey: &str) -> Option<StandardTagKey> {
    match tagkey {
        "AcoustidFingerprint" => Some(StandardTagKey::AcoustidFingerprint),
        "AcoustidId" => Some(StandardTagKey::AcoustidId),
        "Album" => Some(StandardTagKey::Album),
        "AlbumArtist" => Some(StandardTagKey::AlbumArtist),
        "Arranger" => Some(StandardTagKey::Arranger),
        "Artist" => Some(StandardTagKey::Artist),
        "Bpm" => Some(StandardTagKey::Bpm),
        "Comment" => Some(StandardTagKey::Comment),
        "Compilation" => Some(StandardTagKey::Compilation),
        "Composer" => Some(StandardTagKey::Composer),
        "Conductor" => Some(StandardTagKey::Conductor),
        "ContentGroup" => Some(StandardTagKey::ContentGroup),
        "Copyright" => Some(StandardTagKey::Copyright),
        "Date" => Some(StandardTagKey::Date),
        "Description" => Some(StandardTagKey::Description),
        "DiscNumber" => Some(StandardTagKey::DiscNumber),
        "DiscSubtitle" => Some(StandardTagKey::DiscSubtitle),
        "DiscTotal" => Some(StandardTagKey::DiscTotal),
        "EncodedBy" => Some(StandardTagKey::EncodedBy),
        "Encoder" => Some(StandardTagKey::Encoder),
        "EncoderSettings" => Some(StandardTagKey::EncoderSettings),
        "EncodingDate" => Some(StandardTagKey::EncodingDate),
        "Engineer" => Some(StandardTagKey::Engineer),
        "Ensemble" => Some(StandardTagKey::Ensemble),
        "Genre" => Some(StandardTagKey::Genre),
        "IdentAsin" => Some(StandardTagKey::IdentAsin),
        "IdentBarcode" => Some(StandardTagKey::IdentBarcode),
        "IdentCatalogNumber" => Some(StandardTagKey::IdentCatalogNumber),
        "IdentEanUpn" => Some(StandardTagKey::IdentEanUpn),
        "IdentIsrc" => Some(StandardTagKey::IdentIsrc),
        "IdentPn" => Some(StandardTagKey::IdentPn),
        "IdentPodcast" => Some(StandardTagKey::IdentPodcast),
        "IdentUpc" => Some(StandardTagKey::IdentUpc),
        "Label" => Some(StandardTagKey::Label),
        "Language" => Some(StandardTagKey::Language),
        "License" => Some(StandardTagKey::License),
        "Lyricist" => Some(StandardTagKey::Lyricist),
        "Lyrics" => Some(StandardTagKey::Lyrics),
        "MediaFormat" => Some(StandardTagKey::MediaFormat),
        "MixDj" => Some(StandardTagKey::MixDj),
        "MixEngineer" => Some(StandardTagKey::MixEngineer),
        "Mood" => Some(StandardTagKey::Mood),
        "MovementName" => Some(StandardTagKey::MovementName),
        "MovementNumber" => Some(StandardTagKey::MovementNumber),
        "MusicBrainzAlbumArtistId" => Some(StandardTagKey::MusicBrainzAlbumArtistId),
        "MusicBrainzAlbumId" => Some(StandardTagKey::MusicBrainzAlbumId),
        "MusicBrainzArtistId" => Some(StandardTagKey::MusicBrainzArtistId),
        "MusicBrainzDiscId" => Some(StandardTagKey::MusicBrainzDiscId),
        "MusicBrainzGenreId" => Some(StandardTagKey::MusicBrainzGenreId),
        "MusicBrainzLabelId" => Some(StandardTagKey::MusicBrainzLabelId),
        "MusicBrainzOriginalAlbumId" => Some(StandardTagKey::MusicBrainzOriginalAlbumId),
        "MusicBrainzOriginalArtistId" => Some(StandardTagKey::MusicBrainzOriginalArtistId),
        "MusicBrainzRecordingId" => Some(StandardTagKey::MusicBrainzRecordingId),
        "MusicBrainzReleaseGroupId" => Some(StandardTagKey::MusicBrainzReleaseGroupId),
        "MusicBrainzReleaseStatus" => Some(StandardTagKey::MusicBrainzReleaseStatus),
        "MusicBrainzReleaseTrackId" => Some(StandardTagKey::MusicBrainzReleaseTrackId),
        "MusicBrainzReleaseType" => Some(StandardTagKey::MusicBrainzReleaseType),
        "MusicBrainzTrackId" => Some(StandardTagKey::MusicBrainzTrackId),
        "MusicBrainzWorkId" => Some(StandardTagKey::MusicBrainzWorkId),
        "Opus" => Some(StandardTagKey::Opus),
        "OriginalAlbum" => Some(StandardTagKey::OriginalAlbum),
        "OriginalArtist" => Some(StandardTagKey::OriginalArtist),
        "OriginalDate" => Some(StandardTagKey::OriginalDate),
        "OriginalFile" => Some(StandardTagKey::OriginalFile),
        "OriginalWriter" => Some(StandardTagKey::OriginalWriter),
        "Owner" => Some(StandardTagKey::Owner),
        "Part" => Some(StandardTagKey::Part),
        "PartTotal" => Some(StandardTagKey::PartTotal),
        "Performer" => Some(StandardTagKey::Performer),
        "Podcast" => Some(StandardTagKey::Podcast),
        "PodcastCategory" => Some(StandardTagKey::PodcastCategory),
        "PodcastDescription" => Some(StandardTagKey::PodcastDescription),
        "PodcastKeywords" => Some(StandardTagKey::PodcastKeywords),
        "Producer" => Some(StandardTagKey::Producer),
        "PurchaseDate" => Some(StandardTagKey::PurchaseDate),
        "Rating" => Some(StandardTagKey::Rating),
        "ReleaseCountry" => Some(StandardTagKey::ReleaseCountry),
        "ReleaseDate" => Some(StandardTagKey::ReleaseDate),
        "Remixer" => Some(StandardTagKey::Remixer),
        "ReplayGainAlbumGain" => Some(StandardTagKey::ReplayGainAlbumGain),
        "ReplayGainAlbumPeak" => Some(StandardTagKey::ReplayGainAlbumPeak),
        "ReplayGainTrackGain" => Some(StandardTagKey::ReplayGainTrackGain),
        "ReplayGainTrackPeak" => Some(StandardTagKey::ReplayGainTrackPeak),
        "Script" => Some(StandardTagKey::Script),
        "SortAlbum" => Some(StandardTagKey::SortAlbum),
        "SortAlbumArtist" => Some(StandardTagKey::SortAlbumArtist),
        "SortArtist" => Some(StandardTagKey::SortArtist),
        "SortComposer" => Some(StandardTagKey::SortComposer),
        "SortTrackTitle" => Some(StandardTagKey::SortTrackTitle),
        "TaggingDate" => Some(StandardTagKey::TaggingDate),
        "TrackNumber" => Some(StandardTagKey::TrackNumber),
        "TrackSubtitle" => Some(StandardTagKey::TrackSubtitle),
        "TrackTitle" => Some(StandardTagKey::TrackTitle),
        "TrackTotal" => Some(StandardTagKey::TrackTotal),
        "TvEpisode" => Some(StandardTagKey::TvEpisode),
        "TvEpisodeTitle" => Some(StandardTagKey::TvEpisodeTitle),
        "TvNetwork" => Some(StandardTagKey::TvNetwork),
        "TvSeason" => Some(StandardTagKey::TvSeason),
        "TvShowTitle" => Some(StandardTagKey::TvShowTitle),
        "Url" => Some(StandardTagKey::Url),
        "UrlArtist" => Some(StandardTagKey::UrlArtist),
        "UrlCopyright" => Some(StandardTagKey::UrlCopyright),
        "UrlInternetRadio" => Some(StandardTagKey::UrlInternetRadio),
        "UrlLabel" => Some(StandardTagKey::UrlLabel),
        "UrlOfficial" => Some(StandardTagKey::UrlOfficial),
        "UrlPayment" => Some(StandardTagKey::UrlPayment),
        "UrlPodcast" => Some(StandardTagKey::UrlPodcast),
        "UrlPurchase" => Some(StandardTagKey::UrlPurchase),
        "UrlSource" => Some(StandardTagKey::UrlSource),
        "Version" => Some(StandardTagKey::Version),
        "Writer" => Some(StandardTagKey::Writer),
        _ => None,
    }
}

/// Check whether the string is a Base64 encoded string with "data:{};base64,{}" pattern
fn is_base64_encoded(s: &str) -> bool {
    s.starts_with("data:")
        && s.contains(";base64,")
        && s.split(";base64,")
            .nth(1)
            .map(|data| data.len() > 0)
            .unwrap_or(false)
}

/// Decode the given Base64 encoded media. Return the MIME type and Boxed data
fn decode_base64_media(encoded: &str) -> Result<(Mime, Box<[u8]>), Box<dyn std::error::Error>> {
    // Check for "data:{};base64,{}" pattern
    if let Some((mime_str, data)) = encoded.split_once(";base64,") {
        // remove data:
        let mime_str = mime_str.trim_start_matches("data:");
        let mime: Mime = mime_str.parse::<Mime>()?;
        let decoded_data = general_purpose::STANDARD.decode(data)?;
        return Ok((mime, decoded_data.into_boxed_slice()));
    }
    Err("Invalid Base64 data".into())
}

fn get_image_mime_type(path: &str) -> Result<Option<Mime>, Box<dyn std::error::Error>> {
    if is_base64_encoded(path) {
        // Decode Base64 encoded media
        let (mime, _) = decode_base64_media(path)?;
        return Ok(Some(mime));
    }

    // If path exists on disk, check its MIME type
    if Path::new(path).exists() {
        let mime = mime_guess::from_path(path).first();
        if let Some(mime) = mime {
            return Ok(Some(mime));
        } else {
            return Ok(None);
        }
    }

    // Otherwise, assume it's a URL and try to fetch its MIME type
    let client = Client::new();
    let response = client.get(path).send()?;

    if let Some(content_type_header) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        let content_type_str = content_type_header.to_str()?;
        let mime_type = content_type_str.parse::<Mime>()?;
        Ok(Some(mime_type))
    } else {
        Ok(None)
    }
}

fn get_image_data(path: &str) -> Result<Option<Box<[u8]>>, Box<dyn std::error::Error>> {
    if is_base64_encoded(path) {
        // Decode Base64 encoded media
        let (_, data) = decode_base64_media(path)?;
        return Ok(Some(data));
    }

    // If path exists on disk, read its data
    if Path::new(path).exists() {
        let data = std::fs::read(path)?;
        return Ok(Some(data.into_boxed_slice()));
    }

    // Otherwise, assume it's a URL and try to fetch its data
    let client = Client::new();
    let response = client.get(path).send()?;

    if response.status().is_success() {
        let data = response.bytes()?.to_vec().into_boxed_slice();
        return Ok(Some(data));
    }

    Ok(None)
}

/// Given a key and some value, try and parse it into a Visual if the key matches
pub fn parse_visual(tagkey: &str, tagvalue: &str) -> Option<symphonia::core::meta::Visual> {
    match tagkey {
        "AlbumCover" => {
            let mime_type: Option<Mime> = get_image_mime_type(&tagvalue.to_string()).ok()?;
            let image_data: Option<Box<[u8]>> = get_image_data(&tagvalue.to_string()).ok()?;

            if mime_type.is_none() || image_data.is_none() {
                return None;
            }

            return Some(symphonia::core::meta::Visual {
                data: image_data.unwrap_or_else(|| Box::new([])),
                media_type: mime_type.unwrap().essence_str().to_string(),
                dimensions: None,
                bits_per_pixel: None,
                color_mode: None,
                usage: None,
                tags: Vec::new(),
            });
        }
        _ => None,
    }
}

/// Convert between symphonia Visual and flac-codec Picture
pub fn visual_to_flac_picture(
    visual: &symphonia::core::meta::Visual,
) -> flac_codec::metadata::Picture {
    flac_codec::metadata::Picture {
        picture_type: flac_codec::metadata::PictureType::FrontCover,
        media_type: visual.media_type.to_owned(),
        description: String::new(),
        width: 0,
        height: 0,
        color_depth: 0,
        colors_used: None,
        data: visual.data.to_vec(),
    }
}
