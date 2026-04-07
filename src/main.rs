use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process;

use crate::mapper::Mapper;
use crate::midi::parse_midi;
use crate::player::{events_to_chords, play_chords_dry, play_chords_with_output, preview};

mod keyboard;
mod mapper;
mod midi;
mod player;

#[derive(Parser)]
#[command(name = "sky-piano")]
#[command(version = "0.1.0")]
#[command(about = "Play MIDI files on Sky: Children of the Light instruments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Custom mapping file
    #[arg(short, long, default_value = "mapping.toml")]
    mapping: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a MIDI file
    Play {
        /// MIDI file to play
        file: PathBuf,

        /// Delay in seconds before starting playback (default: 3)
        #[arg(short, long, default_value = "3")]
        delay: f64,

        /// Dry run: print keys without pressing them
        #[arg(long, default_value = "false")]
        dry: bool,
    },
    /// Preview: print note sequence without playing
    Preview {
        /// MIDI file to preview
        file: PathBuf,
    },
    /// Dry run: show summary of what would be played
    DryRun {
        /// MIDI file to analyze
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let mapper = if let Some(mapping_path) = &cli.mapping {
        if mapping_path.exists() {
            match mapper::MappingConfig::from_file(mapping_path) {
                Ok(config) => mapper::Mapper::from_config(&config),
                Err(e) => {
                    eprintln!("Error loading mapping file: {}", e);
                    process::exit(1);
                }
            }
        } else {
            eprintln!("Mapping file not found: {:?}", mapping_path);
            eprintln!("Using default mapping");
            mapper::Mapper::a_minor_to_c_major()
        }
    } else {
        mapper::Mapper::a_minor_to_c_major()
    };

    match &cli.command {
        Commands::Play { file, delay, dry } => {
            if let Err(e) = run_play(file, &mapper, *delay, *dry) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::Preview { file } => {
            run_preview(file, &mapper);
        }
        Commands::DryRun { file } => {
            run_dry_run(file, &mapper);
        }
    }
}

fn run_play(
    file: &Path,
    mapper: &Mapper,
    delay: f64,
    dry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let events = parse_midi(file).map_err(|e| format!("Failed to parse MIDI: {}", e))?;

    if events.is_empty() {
        println!("No note events found in file.");
        return Ok(());
    }

    println!("Playing {}...", file.display());
    if dry {
        println!("DRY MODE - keys will only be printed, not pressed");
    }
    println!("Press Ctrl+C to stop.");

    let chords = events_to_chords(&events, mapper);
    println!("{} chords to play", chords.len());

    // Delay before starting
    println!("Starting in {:.0} seconds...", delay);
    std::thread::sleep(std::time::Duration::from_secs_f64(delay));

    if dry {
        play_chords_dry(&chords)?;
    } else {
        println!("--- Playing ---");
        play_chords_with_output(&chords)?;
        println!("--- Done ---");
    }

    Ok(())
}

fn run_preview(file: &Path, mapper: &Mapper) {
    let events = match parse_midi(file) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if events.is_empty() {
        println!("No note events found in file.");
        return;
    }

    println!("Preview: {}\n", file.display());
    println!("Time    | Keys | Duration");
    println!("--------|------|---------");

    let chords = events_to_chords(&events, mapper);
    preview(&chords, mapper);

    println!("\n{} total chords", chords.len());
}

fn run_dry_run(file: &Path, mapper: &Mapper) {
    let events = match parse_midi(file) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if events.is_empty() {
        println!("No note events found in file.");
        return;
    }

    let chords = events_to_chords(&events, mapper);

    // Count statistics
    let total_notes = events.iter().filter(|e| e.is_note_on).count();
    let unique_notes: Vec<u8> = {
        let mut notes: Vec<u8> = events
            .iter()
            .filter(|e| e.is_note_on)
            .map(|e| e.note)
            .collect();
        notes.sort();
        notes.dedup();
        notes
    };
    let chords_count = chords.len();
    let multi_key_chords = chords.iter().filter(|c| c.keys.len() > 1).count();
    let total_duration = chords.last().map(|c| c.time + c.duration).unwrap_or(0.0);

    println!("File: {}", file.display());
    println!("Total note events: {}", total_notes);
    println!("Unique notes: {}", unique_notes.len());
    println!(
        "Note names: {}",
        unique_notes
            .iter()
            .map(|n| midi::note_name(*n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Total chords: {}", chords_count);
    println!("Multi-key chords (chords): {}", multi_key_chords);
    println!("Total duration: {:.2}s", total_duration);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parsing() {
        use clap::Command;
        let cmd = Command::new("sky-piano").subcommand(Command::new("play"));
        // Just verify it doesn't panic
        let _ = cmd;
    }
}
