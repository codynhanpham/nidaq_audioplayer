use rand::{rngs::StdRng, Rng};

#[derive(Debug, Clone)]
pub struct PauseData {
    pub signature: String, // Generator string, eg. pause_1000ms or pause_[8000|12000]ms
    pub title: Option<String>, // Optional custom title
    pub sample_rate: u32,  // Sampling rate to generate samples at
    pub data: Vec<f32>, // Audio data samples, length == sample_rate * duration (determined from signature)
}

fn random_uint_in_range(min: u32, max: u32, rand_generator: &mut StdRng) -> u32 {
    rand_generator.random_range(min..=max)
}

/// Parses a pause signature string and returns the duration in milliseconds and an optional title
///
/// If the pause signature requested a random value, it is evaluated here as well
fn parse_pause_signature(signature: &str, rand_generator: &mut StdRng) -> (u32, Option<String>) {
    if !signature.starts_with("pause_") {
        return (0, None);
    }
    let signature = &signature[6..]; // Remove "pause_"

    // Either:
    // <duration>ms
    // <duration>ms | Title
    // [<min>|<max>]ms
    // [<min>|<max>]ms | Title

    // Split by ms and determine the Title
    let parts: Vec<&str> = signature.split("ms").collect();
    let mut title = parts.get(1).map(|s| s.trim().to_string());
    // Keep duration as string for now before parsing
    let duration = parts.get(0).map(|s| s.trim().to_string());

    if title.is_some() {
        // Trim and Remove possible leading "| "
        title = title
            .unwrap()
            .trim()
            .strip_prefix('|')
            .map(|s| s.trim().to_string());
    }

    if duration.is_none() {
        return (0, title);
    }

    let duration = duration.unwrap();
    // Duration can either be a single value or a range
    let duration = if duration.contains('|') {
        // Remove potential start [ and end ]
        let duration = duration.trim().strip_prefix('[').unwrap_or(&duration);
        let duration = duration.strip_suffix(']').unwrap_or(&duration);
        let parts: Vec<&str> = duration.split('|').collect();
        let min = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let max = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        random_uint_in_range(min, max, rand_generator)
    } else {
        let duration = duration.trim();
        duration.parse::<u32>().unwrap_or(0)
    };

    (duration, title)
}

pub fn create_pause(
    signature: &String,
    title: Option<String>,
    sample_rate: u32,
    rand_generator: &mut StdRng,
) -> PauseData {
    let (duration, overwrite_title) = parse_pause_signature(&signature, rand_generator);

    let title = if let Some(overwrite_title) = overwrite_title {
        Some(overwrite_title)
    } else {
        title
    };

    PauseData {
        signature: signature.clone(),
        title,
        sample_rate,
        data: vec![0.0; (sample_rate * duration / 1000) as usize],
    }
}
