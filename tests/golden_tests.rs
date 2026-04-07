use std::fs;
use std::path::Path;
use sky_piano::mapper::Mapper;
use sky_piano::midi::parse_midi;
use sky_piano::player::events_to_chords;

/// Test that a MIDI file exports to the expected golden output.
fn test_golden_export(song_name: &str) {
    let songs_dir = Path::new("songs");
    let golden_dir = Path::new("tests/golden");
    let export_dir = Path::new("tests/export_output");

    let midi_path = songs_dir.join(format!("{}.mid", song_name));
    let golden_path = golden_dir.join(format!("{}.txt", song_name));
    let export_path = export_dir.join(format!("{}.txt", song_name));

    // Skip if golden file doesn't exist
    if !golden_path.exists() {
        println!("  (skipping - no golden file)");
        return;
    }

    // Create export dir
    fs::create_dir_all(export_dir).unwrap();

    // Parse MIDI and export
    let events = parse_midi(&midi_path).expect("Failed to parse MIDI");
    let mapper = Mapper::a_minor_to_c_major();
    let chords = events_to_chords(&events, &mapper);

    // Write export
    let mut content = String::new();
    content.push_str(&format!("# Exported from: {}\n", midi_path.display()));
    content.push_str(&format!("# Total chords: {}\n", chords.len()));
    content.push('\n');
    content.push_str("Time,Keys,Duration\n");
    for chord in &chords {
        let keys_str = chord.keys.join("");
        content.push_str(&format!("{:.2},{},{:.2}\n", chord.time, keys_str, chord.duration));
    }
    fs::write(&export_path, &content).expect("Failed to write export");

    // Compare with golden
    let golden = fs::read_to_string(&golden_path).expect("Failed to read golden file");
    assert_eq!(
        content, golden,
        "Golden test failed for {} - export differs from expected",
        song_name
    );
    println!("  ✓ {}", song_name);
}

#[test]
fn test_twinkle_twinkle() {
    test_golden_export("twinkle");
}

#[test]
fn test_mary() {
    test_golden_export("mary");
}

#[test]
fn test_ode_to_joy() {
    test_golden_export("ode_to_joy");
}

#[test]
fn test_scale() {
    test_golden_export("scale");
}

#[test]
fn test_jingle() {
    test_golden_export("jingle");
}

#[test]
fn test_legend_of_heroes() {
    test_golden_export("The_Legend_of_Heroes_Trails_in_the_Sky_The_Whereabouts_of_Light");
}
