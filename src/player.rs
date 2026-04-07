use crate::keyboard;
use crate::mapper::Mapper;
use crate::midi::MidiEvent;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("keyboard simulation error: {0}")]
    KeyboardError(#[from] keyboard::KeyboardError),
}

/// A chord represents multiple notes played simultaneously.
#[derive(Debug, Clone)]
pub struct Chord {
    /// Time in seconds from start
    pub time: f64,
    /// Duration in seconds
    pub duration: f64,
    /// Keys to press
    pub keys: Vec<String>,
}

impl Chord {
    /// Create a new chord.
    pub fn new(time: f64, keys: Vec<String>, duration: f64) -> Self {
        Chord {
            time,
            keys,
            duration,
        }
    }
}

/// Convert MIDI events to chords for playback.
pub fn events_to_chords(events: &[MidiEvent], mapper: &Mapper) -> Vec<Chord> {
    let mut chords: Vec<Chord> = Vec::new();

    // Track note-on events: note -> (time, velocity)
    let mut active_notes: HashMap<u8, (f64, u8)> = HashMap::new();

    for event in events {
        if event.is_note_on {
            // Store the note-on with its time
            active_notes.insert(event.note, (event.time, event.velocity));
        } else {
            // Note off - find matching note-on and create a chord
            if let Some((start_time, _velocity)) = active_notes.remove(&event.note) {
                if let Some(key) = mapper.note_to_key(event.note) {
                    let duration = event.time - start_time;
                    chords.push(Chord::new(start_time, vec![key.to_string()], duration));
                }
            }
        }
    }

    // Sort by time
    chords.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    // Merge chords at the same timestamp into multi-key chords
    merge_chords(chords)
}

/// Merge single-key chords at the same timestamp into multi-key chords.
fn merge_chords(chords: Vec<Chord>) -> Vec<Chord> {
    if chords.is_empty() {
        return chords;
    }

    let mut merged: Vec<Chord> = Vec::new();
    let mut current_time = chords[0].time;
    let mut current_keys: Vec<String> = Vec::new();
    let mut current_duration: f64 = 0.0;

    for chord in chords {
        // Use a small epsilon for time comparison
        let epsilon = 0.001;

        if (chord.time - current_time).abs() < epsilon {
            // Same timestamp - merge keys
            current_keys.push(chord.keys[0].clone());
            // Keep the longest duration seen so far
            current_duration = current_duration.max(chord.duration);
        } else {
            // Different timestamp - save current and start new
            if !current_keys.is_empty() {
                merged.push(Chord::new(
                    current_time,
                    current_keys.clone(),
                    current_duration,
                ));
            }
            current_time = chord.time;
            current_keys = chord.keys;
            current_duration = chord.duration;
        }
    }

    // Don't forget the last chord
    if !current_keys.is_empty() {
        merged.push(Chord::new(current_time, current_keys, current_duration));
    }

    merged
}

/// Play a sequence of chords and print keys as they play.
pub fn play_chords_with_output(chords: &[Chord]) -> Result<(), PlayerError> {
    if chords.is_empty() {
        return Ok(());
    }

    let start = Instant::now();
    let first_chord_time = chords[0].time;

    for chord in chords {
        // Calculate when this chord should start
        let target_time = first_chord_time + chord.time;
        let now = start.elapsed().as_secs_f64();

        if target_time > now {
            // Sleep until it's time
            let sleep_duration = Duration::from_secs_f64(target_time - now);
            std::thread::sleep(sleep_duration);
        }

        // Print the keys being played
        let keys_str = chord.keys.join("");
        println!("{:.2}s | {}", chord.time, keys_str);

        // Press all keys in the chord
        let keys_refs: Vec<&str> = chord.keys.iter().map(|s| s.as_str()).collect();
        keyboard::chord_press(&keys_refs, Duration::from_secs_f64(chord.duration))?;
    }

    Ok(())
}

/// Play chords in dry mode: print keys with timing but don't press them.
pub fn play_chords_dry(chords: &[Chord]) -> Result<(), PlayerError> {
    if chords.is_empty() {
        return Ok(());
    }

    println!("--- DRY MODE ---");
    let start = Instant::now();
    let first_chord_time = chords[0].time;

    for chord in chords {
        // Calculate when this chord should start
        let target_time = first_chord_time + chord.time;
        let now = start.elapsed().as_secs_f64();

        if target_time > now {
            // Sleep until it's time
            let sleep_duration = Duration::from_secs_f64(target_time - now);
            std::thread::sleep(sleep_duration);
        }

        // Print the keys being played (but don't press)
        let keys_str = chord.keys.join("");
        println!("{:.2}s | {}", chord.time, keys_str);
    }

    println!("--- Done (dry) ---");
    Ok(())
}

/// Preview: print the chord sequence without playing.
pub fn preview(chords: &[Chord], _mapper: &Mapper) {
    for chord in chords {
        let keys_str = chord.keys.join(", ");
        println!(
            "@ {:.2}s | {} | {:.2}s",
            chord.time, keys_str, chord.duration
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mapper() -> Mapper {
        Mapper::default_positional_15()
    }

    #[test]
    fn test_events_to_chords_simple() {
        let mapper = make_mapper();
        // Use positional notes 0-14 for Sky keys
        let events = vec![
            MidiEvent {
                time: 0.0,
                note: 5, // Maps to "h"
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 0.5,
                note: 5,
                is_note_on: false,
                velocity: 0,
            },
        ];

        let chords = events_to_chords(&events, &mapper);
        assert_eq!(chords.len(), 1);
        assert_eq!(chords[0].time, 0.0);
        assert_eq!(chords[0].duration, 0.5);
        assert_eq!(chords[0].keys, vec!["h"]);
    }

    #[test]
    fn test_events_to_chords_multiple_notes() {
        let mapper = make_mapper();
        let events = vec![
            // Two notes at same time: key 5 ("h") and key 9 (";")
            MidiEvent {
                time: 0.0,
                note: 5,
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 0.0,
                note: 9,
                is_note_on: true,
                velocity: 100,
            },
            // Both off at same time
            MidiEvent {
                time: 1.0,
                note: 5,
                is_note_on: false,
                velocity: 0,
            },
            MidiEvent {
                time: 1.0,
                note: 9,
                is_note_on: false,
                velocity: 0,
            },
        ];

        let chords = events_to_chords(&events, &mapper);
        assert_eq!(chords.len(), 1);
        assert_eq!(chords[0].keys, vec!["h", ";"]);
        assert_eq!(chords[0].duration, 1.0);
    }

    #[test]
    fn test_events_to_chords_unmapped_note() {
        let mapper = make_mapper();
        let events = vec![
            MidiEvent {
                time: 0.0,
                note: 5, // Mapped to "h"
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 0.0,
                note: 15, // Not in our 15-key mapping (0-14)
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 0.5,
                note: 5,
                is_note_on: false,
                velocity: 0,
            },
            MidiEvent {
                time: 0.5,
                note: 15,
                is_note_on: false,
                velocity: 0,
            },
        ];

        let chords = events_to_chords(&events, &mapper);
        // Only note 5 ("h") should be in chord, 15 is not mapped
        assert_eq!(chords.len(), 1);
        assert_eq!(chords[0].keys, vec!["h"]);
    }

    #[test]
    fn test_events_to_chords_sequential_notes() {
        let mapper = make_mapper();
        let events = vec![
            MidiEvent {
                time: 0.0,
                note: 5, // Maps to "h"
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 0.5,
                note: 5,
                is_note_on: false,
                velocity: 0,
            },
            MidiEvent {
                time: 0.5,
                note: 7, // Maps to "k"
                is_note_on: true,
                velocity: 100,
            },
            MidiEvent {
                time: 1.0,
                note: 7,
                is_note_on: false,
                velocity: 0,
            },
        ];

        let chords = events_to_chords(&events, &mapper);
        assert_eq!(chords.len(), 2);
        assert_eq!(chords[0].keys, vec!["h"]);
        assert_eq!(chords[1].keys, vec!["k"]);
    }

    #[test]
    fn test_merge_chords() {
        // Two chords at same time should merge
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.5),
            Chord::new(0.0, vec!["j".to_string()], 0.5),
        ];

        let merged = merge_chords(chords);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].keys, vec!["h", "j"]);
    }

    #[test]
    fn test_merge_chords_near_same_time() {
        // Chords very close in time should merge (within epsilon)
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.5),
            Chord::new(0.0005, vec!["j".to_string()], 0.5), // Within epsilon
        ];

        let merged = merge_chords(chords);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].keys, vec!["h", "j"]);
    }

    #[test]
    fn test_merge_chords_different_times() {
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.5),
            Chord::new(0.1, vec!["j".to_string()], 0.5), // Different time
        ];

        let merged = merge_chords(chords);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_chords_keeps_longest_duration() {
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.3),
            Chord::new(0.0, vec!["j".to_string()], 0.5), // Longer duration
        ];

        let merged = merge_chords(chords);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].duration, 0.5);
    }

    #[test]
    fn test_empty_events() {
        let mapper = make_mapper();
        let events: Vec<MidiEvent> = vec![];
        let chords = events_to_chords(&events, &mapper);
        assert!(chords.is_empty());
    }

    #[test]
    fn test_note_off_without_note_on() {
        // Note off without matching note on should be ignored
        let mapper = make_mapper();
        let events = vec![MidiEvent {
            time: 0.0,
            note: 60,
            is_note_on: false,
            velocity: 0,
        }];

        let chords = events_to_chords(&events, &mapper);
        assert!(chords.is_empty());
    }

    #[test]
    fn test_note_on_without_note_off() {
        // Note on without note off - we still create a chord with duration 0
        let mapper = make_mapper();
        let events = vec![MidiEvent {
            time: 0.0,
            note: 60,
            is_note_on: true,
            velocity: 100,
        }];

        let chords = events_to_chords(&events, &mapper);
        // Note is still tracked but duration will be 0 since no note off
        // Actually, it won't appear because we only create chords on note-off
        assert!(chords.is_empty());
    }

    #[test]
    fn test_preview() {
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.5),
            Chord::new(0.5, vec!["j".to_string(), "k".to_string()], 0.3),
        ];

        // Just verify it doesn't panic
        preview(&chords, &make_mapper());
    }

    #[test]
    fn test_play_chords_dry_empty() {
        let chords: Vec<Chord> = vec![];
        let result = play_chords_dry(&chords);
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_chords_dry_single_note() {
        let chords = vec![Chord::new(0.0, vec!["h".to_string()], 0.5)];
        let result = play_chords_dry(&chords);
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_chords_dry_multiple_chords() {
        let chords = vec![
            Chord::new(0.0, vec!["h".to_string()], 0.5),
            Chord::new(0.5, vec!["j".to_string(), "k".to_string()], 0.3),
            Chord::new(1.0, vec![";".to_string()], 0.25),
        ];
        let result = play_chords_dry(&chords);
        assert!(result.is_ok());
    }
}
