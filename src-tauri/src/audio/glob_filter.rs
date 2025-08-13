use tauri::Manager;

use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Filter audio files `recursive_level` deep in a given directory
///
/// Accepted file extensions are: `.wav`, `.flac`, `.mp3`, and `.ogg`
///
/// Args:
/// - `dir`: The directory to search for audio files.
/// - `recursive_level`: The maximum depth to search for audio files. Default: 0
#[tauri::command]
pub fn filter_audio_files(dir: &Path, recursive_level: Option<usize>) -> Vec<String> {
    let mut result = Vec::new();
    let allowed_extensions = vec!["wav", "flac", "mp3", "ogg"];

    for entry in WalkDir::new(dir)
        .max_depth(recursive_level.unwrap_or(0))
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if allowed_extensions.contains(&ext.to_str().unwrap_or_default()) {
                    if let Some(path_str) = entry.path().to_str() {
                        result.push(path_str.to_string());
                    }
                }
            }
        }
    }

    result
}

/// Given a Vec<String> of potential file/dir paths
/// validate existence and parse the direct parent (if is file) or self (if is dir)
///
/// Returns the unique directories of given `paths`
#[tauri::command]
pub fn parse_dirs_from_paths(paths: Vec<String>) -> Vec<String> {
    let mut dirs = vec![];

    for path in paths {
        let path = Path::new(&path);
        if path.exists() {
            if path.is_file() {
                if let Some(parent) = path.parent() {
                    dirs.push(parent.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                dirs.push(path.to_string_lossy().to_string());
            }
        }
    }

    dirs.into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDirStats {
    pub dir: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFilesInDir {
    pub paths: Vec<String>,
    pub stats: Vec<AudioDirStats>,
}

/// Flex search for audio files given a list of dir/files
#[tauri::command]
pub async fn flex_search_audio_files(
    paths: Vec<String>,
    recursive_level: Option<usize>,
) -> Result<AudioFilesInDir, String> {
    let mut audio_files = std::collections::HashSet::new();

    let dirs = parse_dirs_from_paths(paths);

    // Collect audio file statistics
    let mut audio_stats = Vec::new();
    for dir in dirs {
        let path = Path::new(&dir);
        let files = filter_audio_files(&path, recursive_level);
        let file_count = files.len();
        audio_stats.push(AudioDirStats {
            dir: dir,
            file_count,
        });
        audio_files.extend(files);
    }

    Ok(AudioFilesInDir {
        paths: audio_files.into_iter().collect(),
        stats: audio_stats,
    })
}
