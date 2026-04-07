use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use crate::mapper::Mapper;
use crate::midi::{note_range, parse_midi, MidiEvent};
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
    /// Export: save note sequence to a file
    Export {
        /// MIDI file to export
        file: PathBuf,

        /// Output file (default: same name with .txt extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Play { file, delay, dry } => {
            if let Err(e) = run_play(file, *delay, *dry, &cli.mapping) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::Preview { file } => {
            run_preview(file, &cli.mapping);
        }
        Commands::DryRun { file } => {
            run_dry_run(file, &cli.mapping);
        }
        Commands::Export { file, output } => {
            run_export(file, output.as_deref(), &cli.mapping);
        }
    }
}

fn run_play(
    file: &Path,
    delay: f64,
    dry: bool,
    mapping: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let events = parse_midi(file).map_err(|e| format!("Failed to parse MIDI: {}", e))?;

    if events.is_empty() {
        println!("No note events found in file.");
        return Ok(());
    }

    let mapper = create_mapper(&events, mapping);

    println!("Playing {}...", file.display());
    if dry {
        println!("DRY MODE - keys will only be printed, not pressed");
    }
    println!("Press Ctrl+C to stop.");

    let chords = events_to_chords(&events, &mapper);
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

fn run_preview(file: &Path, mapping: &Option<PathBuf>) {
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

    let mapper = create_mapper(&events, mapping);

    println!("Preview: {}\n", file.display());
    println!("Time    | Keys | Duration");
    println!("--------|------|---------");

    let chords = events_to_chords(&events, &mapper);
    preview(&chords, &mapper);

    println!("\n{} total chords", chords.len());
}

fn run_dry_run(file: &Path, mapping: &Option<PathBuf>) {
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

    let mapper = create_mapper(&events, mapping);
    let chords = events_to_chords(&events, &mapper);

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

fn run_export(file: &Path, output: Option<&Path>, mapping: &Option<PathBuf>) {
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

    let mapper = create_mapper(&events, mapping);
    let chords = events_to_chords(&events, &mapper);

    // Determine output file
    let out_path = if let Some(p) = output {
        p.to_path_buf()
    } else {
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!("{}.txt", stem))
    };

    // Write export file
    let mut f = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error creating output file: {}", e);
            process::exit(1);
        }
    };

    // Header
    writeln!(f, "# Exported from: {}", file.display()).ok();
    writeln!(f, "# Total chords: {}", chords.len()).ok();
    writeln!(f).ok();
    writeln!(f, "Time,Keys,Duration").ok();

    // Chords
    for chord in &chords {
        let keys_str = chord.keys.join("");
        writeln!(f, "{:.2},{},{:.2}", chord.time, keys_str, chord.duration).ok();
    }

    println!("Exported {} chords to {}", chords.len(), out_path.display());
}

/// Create a mapper for the given MIDI events.
fn create_mapper(events: &[MidiEvent], mapping: &Option<PathBuf>) -> Mapper {
    if let Some(mapping_path) = mapping {
        if mapping_path.exists() {
            match mapper::MappingConfig::from_file(mapping_path) {
                Ok(config) => return mapper::Mapper::from_config(&config),
                Err(e) => {
                    eprintln!(
                        "Error loading mapping file: {}, using note range compression",
                        e
                    );
                }
            }
        } else {
            eprintln!(
                "Mapping file not found: {:?}, using note range compression",
                mapping_path
            );
        }
    }

    // Use the actual note range from the MIDI, compressed to Sky's range
    if let Some((min_note, max_note)) = note_range(events) {
        println!(
            "Compressing note range {}-{} to Sky keys",
            min_note, max_note
        );
        mapper::Mapper::from_note_range(min_note, max_note)
    } else {
        // Fallback to default
        mapper::Mapper::a_minor_to_c_major()
    }
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
