// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::path::PathBuf;


#[derive(Parser, Debug)]
#[command(
    version,
    about = "NI-DAQmx Audio Player CLI Tools",
    long_about = None
)]
struct Args {

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract metadata from audio files
    Metadata {
        /// Path to input audio file
        #[arg(short, long)]
        input: String,

        /// Path to save output, omit or '-' to use stdout
        #[arg(short, long)]
        output: Option<String>,
    },
}



fn main() {
    #[cfg(windows)]
    {
        use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        // we ignore the result here because
        // if the app started from a command line, like cmd or powershell,
        // it will attach sucessfully which is what we want
        // but if we were started from something like explorer,
        // it will fail to attach console which is also what we want.
        let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
    }

    let args = Args::parse();

    match &args.command {
        // If no commands or args to use CLI, launch the GUI app
        None => {
            #[cfg(windows)]
            {
                use windows::Win32::System::Console::FreeConsole;
                let _ = unsafe { FreeConsole() };
            }
            nidaq_audioplayer_lib::run()
        }
        _ => {
            // Handle subcommands and make sure output to attach shell on Windows
            let command = args.command.as_ref().unwrap();



            match &command {
                Commands::Metadata { input, output } => {
                    let metadata = nidaq_audioplayer_lib::audio::metadata::parse_metadata(&PathBuf::from(input));
                    println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
                }
            }


            std::process::exit(0);
        }
    }
}
