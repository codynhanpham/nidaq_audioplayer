use clap::Parser;
use std::path::PathBuf;

use nidaq_audioplayer_lib::audio::metadata::{parse_metadata};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Metadata Extraction Tools",
    long_about = None
)]
struct Args {
    /// Path to input audio file
    #[arg(short, long)]
    input: String,

    /// Path to save output, omit or '-' to use stdout
    #[arg(short, long)]
    output: Option<String>,
}



fn main() {
    let args = Args::parse();

    let metadata = parse_metadata(&PathBuf::from(&args.input));
    if metadata.is_err() {
        eprintln!("Error parsing metadata: {}", metadata.err().unwrap());
        std::process::exit(1);
    }
    let metadata = metadata.unwrap();
    if let Some(output) = args.output {
        if output == "-" {
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        else {
            std::fs::write(output, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
    }
}
