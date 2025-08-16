use std::io::Write;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha2::{Sha256, Digest};

use crate::audio::metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFileInfo {
    path: PathBuf,
    size: usize,
}


/// calculate SHA256 hash of the audio metadata
#[tauri::command]
pub async fn calculate_audio_metadata_hash(audio_metadata: Vec<metadata::AudioMetadata>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bincode::encode_to_vec(audio_metadata, bincode::config::standard()).unwrap());
    format!("{:x}", hasher.finalize())
}

/// Save audio metadata to a file
#[tauri::command]
pub async fn save_audio_metadata(audio_metadata: Vec<metadata::AudioMetadata>, output_path: String) -> Result<OutputFileInfo, String> {
    let output_path = PathBuf::from(output_path);

    let mut writer = match std::fs::File::create(&output_path) {
        Ok(file) => file,
        Err(error) => return Err(format!("Failed to open output file: {error:?}")),
    };

    let data = bincode::encode_to_vec(audio_metadata, bincode::config::standard())
        .map_err(|e| format!("Failed to encode metadata to vec: {e:?}"))?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data).map_err(|e| format!("Failed to compress metadata: {e:?}"))?;
    let compressed_data = encoder.finish().map_err(|e| format!("Failed to finish compression: {e:?}"))?;

    std::io::Write::write_all(&mut writer, &compressed_data).map_err(|e| format!("Failed to write metadata to file: {e:?}"))?;

    Ok(OutputFileInfo {
        path: output_path,
        size: compressed_data.len(),
    })
}

/// Load audio metadata from a file
#[tauri::command]
pub async fn load_audio_metadata(input_path: String) -> Result<Vec<metadata::AudioMetadata>, String> {
    let input_path = PathBuf::from(input_path);

    let mut reader = Vec::new();
    let mut decoder = ZlibDecoder::new(&mut reader);
    decoder.write_all(&std::fs::read(&input_path).map_err(|e| format!("Failed to read file: {e:?}"))?)
        .map_err(|e| format!("Failed to decompress metadata: {e:?}"))?;
    reader = match decoder.finish() {
        Ok(data) => data.to_vec(),
        Err(e) => return Err(format!("Failed to finish decompression: {e:?}")),
    };

    let audio_metadata: Vec<metadata::AudioMetadata> = match bincode::decode_from_slice(&reader, bincode::config::standard()) {
        Ok((data, _)) => data,
        Err(e) => return Err(format!("Failed to decode metadata from slice: {e:?}")),
    };

    Ok(audio_metadata)
}