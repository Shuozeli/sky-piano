use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiEvent {
    /// Time in seconds from start
    pub time: f64,
    /// MIDI note number (0-127)
    pub note: u8,
    /// True if note-on, false if note-off
    pub is_note_on: bool,
    /// Velocity (0-127), only meaningful for note-on
    pub velocity: u8,
}

/// Error types for MIDI parsing
#[derive(Debug, Error)]
pub enum MidiError {
    #[error("failed to open file: {0}")]
    FileOpen(#[from] std::io::Error),
    #[error("invalid MIDI: {0}")]
    InvalidMidi(String),
    #[error("unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),
}

/// Parse a MIDI file and return a list of note events sorted by time.
pub fn parse_midi(path: &Path) -> Result<Vec<MidiEvent>, MidiError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    parse_midi_bytes(&buffer)
}

/// Parse MIDI from bytes
pub fn parse_midi_bytes(data: &[u8]) -> Result<Vec<MidiEvent>, MidiError> {
    if data.len() < 14 {
        return Err(MidiError::InvalidMidi("File too short".to_string()));
    }

    // Check header
    if &data[0..4] != b"MThd" {
        return Err(MidiError::InvalidMidi("Missing MThd header".to_string()));
    }

    let header_len = read_u32be(&data[4..8]);
    if header_len < 6 {
        return Err(MidiError::InvalidMidi("Header too short".to_string()));
    }

    let format = read_u16be(&data[8..10]);
    let num_tracks = read_u16be(&data[10..12]);
    let division = read_u16be(&data[12..14]);

    if format > 2 {
        return Err(MidiError::UnsupportedFormat(format));
    }

    // Handle format 0 (single track) vs format 1/2 (multiple tracks)
    let mut events = Vec::new();
    let mut offset = 14.min(8 + header_len as usize);

    for _track in 0..num_tracks {
        if offset + 8 > data.len() {
            break;
        }

        if &data[offset..offset + 4] != b"MTrk" {
            return Err(MidiError::InvalidMidi("Missing MTrk header".to_string()));
        }

        let track_len = read_u32be(&data[offset + 4..offset + 8]);
        offset += 8;

        let track_end = (offset as u32 + track_len) as usize;
        if track_end > data.len() {
            return Err(MidiError::InvalidMidi(
                "Track extends past file".to_string(),
            ));
        }

        let track_events = parse_track_events(&data[offset..track_end], division)?;
        events.extend(track_events);
        offset = track_end;
    }

    // Sort by time
    events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    Ok(events)
}

fn parse_track_events(data: &[u8], division: u16) -> Result<Vec<MidiEvent>, MidiError> {
    let mut events = Vec::new();
    let mut offset = 0;
    let mut current_time: f64 = 0.0;
    let mut last_status: u8 = 0;

    while offset < data.len() {
        let (delta_time, bytes_read) = read_variable_length(&data[offset..])?;
        offset += bytes_read;

        // Convert delta time to absolute time using ticks
        // division tells us ticks per quarter note
        // Assuming 120 BPM default, microseconds per beat = 500000
        // But we don't have tempo in format 0/1 without meta events
        // Use default: 120 BPM = 500000 µs per beat
        let ticks_per_beat = division as f64;
        let micros_per_beat = 500000.0; // 120 BPM default
        let seconds_per_tick = micros_per_beat / (ticks_per_beat * 1000000.0);
        current_time += delta_time as f64 * seconds_per_tick;

        if offset >= data.len() {
            break;
        }

        let mut byte = data[offset];
        offset += 1;

        // Running status: if high bit not set, use last_status
        if byte & 0x80 == 0 {
            if last_status == 0 {
                return Err(MidiError::InvalidMidi(
                    "Running status without previous status".to_string(),
                ));
            }
            // Reuse last status, byte is data
            byte = last_status;
            offset -= 1; // Back up since we already consumed the data byte
        } else {
            last_status = byte;
        }

        let status = byte & 0xF0;
        let _channel = byte & 0x0F;

        match status {
            // Note off: 0x80 + channel, note, velocity
            0x80 if offset + 1 < data.len() => {
                let note = data[offset];
                offset += 1;
                let _velocity = data[offset];
                offset += 1;

                events.push(MidiEvent {
                    time: current_time,
                    note,
                    is_note_on: false,
                    velocity: 0,
                });
            }
            // Note on: 0x90 + channel, note, velocity
            0x90 if offset + 1 < data.len() => {
                let note = data[offset];
                offset += 1;
                let velocity = data[offset];
                offset += 1;

                // Note on with velocity 0 is treated as note off
                let is_note_on = velocity != 0;

                events.push(MidiEvent {
                    time: current_time,
                    note,
                    is_note_on,
                    velocity,
                });
            }
            // Polyphonic aftertouch / channel aftertouch - skip
            0xA0 if offset + 1 < data.len() => {
                offset += 2;
            }
            // Control change - skip
            0xB0 if offset + 1 < data.len() => {
                offset += 2;
            }
            // Program change - skip
            0xC0 if offset < data.len() => {
                offset += 1;
            }
            // Channel aftertouch - skip
            0xD0 if offset < data.len() => {
                offset += 1;
            }
            // Pitch bend - skip
            0xE0 if offset + 1 < data.len() => {
                offset += 2;
            }
            // Meta event
            0xFF => {
                if offset >= data.len() {
                    break;
                }
                let _meta_type = data[offset];
                offset += 1;
                let (len, bytes_read) = read_variable_length(&data[offset..])?;
                offset += bytes_read;

                // Skip meta event data
                offset = offset.saturating_add(len as usize);

                // Handle tempo meta event (0x51) - skip for now
                // In a full implementation, we'd track tempo changes per-track
            }
            // SysEx - skip
            0xF0 | 0xF7 => {
                let (len, bytes_read) = read_variable_length(&data[offset..])?;
                offset += bytes_read;
                offset = offset.saturating_add(len as usize);
            }
            _ => {
                // Unknown, try to skip
                if offset < data.len() {
                    offset += 1;
                }
            }
        }
    }

    Ok(events)
}

fn read_u16be(data: &[u8]) -> u16 {
    ((data[0] as u16) << 8) | (data[1] as u16)
}

fn read_u32be(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32)
}

/// Read a variable-length quantity (MIDI uses this for delta times)
fn read_variable_length(data: &[u8]) -> Result<(u32, usize), MidiError> {
    let mut result: u32 = 0;
    let mut bytes_read = 0;

    for (i, &byte) in data.iter().enumerate() {
        bytes_read += 1;
        result = (result << 7) | ((byte & 0x7F) as u32);

        if byte & 0x80 == 0 {
            return Ok((result, bytes_read));
        }

        if i >= 3 {
            return Err(MidiError::InvalidMidi(
                "Variable length quantity too long".to_string(),
            ));
        }
    }

    Err(MidiError::InvalidMidi("Unexpected end of data".to_string()))
}

/// Get note name from MIDI note number (e.g., 60 -> "C4")
pub fn note_name(note: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note / 12) as i32 - 1;
    let name_idx = (note % 12) as usize;
    format!("{}{}", names[name_idx], octave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_variable_length() {
        // 0x00 -> 0
        let (val, len) = read_variable_length(&[0x00]).unwrap();
        assert_eq!(val, 0);
        assert_eq!(len, 1);

        // 0x7F -> 127
        let (val, len) = read_variable_length(&[0x7F]).unwrap();
        assert_eq!(val, 127);
        assert_eq!(len, 1);

        // 0x81 0x00 -> 128
        let (val, len) = read_variable_length(&[0x81, 0x00]).unwrap();
        assert_eq!(val, 128);
        assert_eq!(len, 2);

        // 0x81 0x7F -> 255
        let (val, len) = read_variable_length(&[0x81, 0x7F]).unwrap();
        assert_eq!(val, 255);
        assert_eq!(len, 2);

        // 0x82 0x00 0x00 -> 256 (2 bytes consumed - 0x00 terminates)
        let (val, len) = read_variable_length(&[0x82, 0x00, 0x00]).unwrap();
        assert_eq!(val, 256);
        assert_eq!(len, 2);

        // 0x81 0x80 0x00 -> 16384 (3 bytes: 1<<14 | 0<<7 | 0)
        let (val, len) = read_variable_length(&[0x81, 0x80, 0x00]).unwrap();
        assert_eq!(val, 16384);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_note_name() {
        assert_eq!(note_name(60), "C4"); // Middle C
        assert_eq!(note_name(61), "C#4");
        assert_eq!(note_name(69), "A4"); // A440
        assert_eq!(note_name(72), "C5");
        assert_eq!(note_name(48), "C3");
        assert_eq!(note_name(0), "C-1");
    }

    #[test]
    fn test_read_u16be() {
        assert_eq!(read_u16be(&[0x00, 0x01]), 1);
        assert_eq!(read_u16be(&[0x01, 0x00]), 256);
        assert_eq!(read_u16be(&[0x00, 0x06]), 6);
    }

    #[test]
    fn test_read_u32be() {
        assert_eq!(read_u32be(&[0x00, 0x00, 0x00, 0x01]), 1);
        assert_eq!(read_u32be(&[0x00, 0x00, 0x01, 0x00]), 256);
        assert_eq!(read_u32be(&[0x00, 0x00, 0x06, 0x00]), 1536);
    }

    #[test]
    fn test_invalid_midi_too_short() {
        let result = parse_midi_bytes(&[]);
        assert!(result.is_err());

        let result = parse_midi_bytes(&[0, 1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_midi_wrong_header() {
        let data = b"MTrk\x00\x00\x00\x00";
        let result = parse_midi_bytes(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_simple_midi_parsing() {
        // Create a minimal valid MIDI file (format 0, 1 track)
        // This is a very minimal MIDI that has:
        // - MThd header
        // - One MTrk with a note on event
        let data = vec![
            // MThd
            0x4D, 0x54, 0x68, 0x64, // "MThd"
            0x00, 0x00, 0x00, 0x06, // header length = 6
            0x00, 0x00, // format = 0
            0x00, 0x01, // num tracks = 1
            0x00, 0x60, // division = 96 ticks/quarter note
            // MTrk
            0x4D, 0x54, 0x72, 0x6B, // "MTrk"
            0x00, 0x00, 0x00, 0x0B, // track length = 11
            // Delta time 0, Note On channel 0, note 60, velocity 64
            0x00, 0x90, 0x3C, 0x40,
            // Delta time 60 (1 quarter note), Note Off channel 0, note 60, velocity 0
            0x60, 0x80, 0x3C, 0x00, // End of track
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let result = parse_midi_bytes(&data);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 2);

        // First event: note on at time 0
        assert_eq!(events[0].note, 60);
        assert!(events[0].is_note_on);
        assert_eq!(events[0].velocity, 64);

        // Second event: note off at ~0.5s (96 ticks at 120 BPM = 0.5s)
        assert_eq!(events[1].note, 60);
        assert!(!events[1].is_note_on);
    }

    #[test]
    fn test_note_on_velocity_zero_is_note_off() {
        // Note on with velocity 0 should be treated as note off
        let data = vec![
            0x4D, 0x54, 0x68, 0x64, // MThd
            0x00, 0x00, 0x00, 0x06, 0x00, 0x00, // format 0
            0x00, 0x01, // 1 track
            0x00, 0x60, // division 96
            0x4D, 0x54, 0x72, 0x6B, // MTrk
            0x00, 0x00, 0x00, 0x07, 0x00, 0x90, 0x3C, 0x00, // Note on with vel 0 = note off
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let result = parse_midi_bytes(&data);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(!events[0].is_note_on); // Should be treated as note off
    }

    #[test]
    fn test_unsupported_format() {
        let data = vec![
            0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00,
            0x03, // format = 3 (unsupported)
            0x00, 0x01, 0x00, 0x60,
        ];

        let result = parse_midi_bytes(&data);
        assert!(matches!(result, Err(MidiError::UnsupportedFormat(3))));
    }
}
