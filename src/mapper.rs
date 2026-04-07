use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MapperError {
    #[error("failed to read mapping file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MappingConfig {
    #[serde(default)]
    pub mapping: HashMap<String, String>,
}

impl MappingConfig {
    /// Load mapping from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, MapperError> {
        let content = fs::read_to_string(path)?;
        let config: MappingConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub struct Mapper {
    /// Maps MIDI note number (0-127) to key string (e.g., "h", "j", ";")
    note_to_key: HashMap<u8, String>,
}

impl Mapper {
    /// Create a new mapper from a TOML config file.
    pub fn from_config(config: &MappingConfig) -> Self {
        let mut note_to_key = HashMap::new();
        for (note_str, key) in &config.mapping {
            if let Ok(note) = note_str.parse::<u8>() {
                note_to_key.insert(note, key.clone());
            }
        }
        Mapper { note_to_key }
    }

    /// Create a mapper with A minor to C major range mapping.
    ///
    /// Sky's instrument covers 15 notes: two octaves from A minor to C major.
    /// A3 (MIDI 45) to C5 (MIDI 72) spans 27 semitones but we compress to 15 keys.
    /// Keys are arranged to match Sky's 3x5 grid layout.
    ///
    /// | Sky Key | Key | Note (approx) |
    /// |---------|-----|--------------|
    /// | 0  | y | A3 |
    /// | 1  | u | B3 |
    /// | 2  | i | C4 |
    /// | 3  | o | D4 |
    /// | 4  | p | E4 |
    /// | 5  | h | F4 |
    /// | 6  | j | G4 |
    /// | 7  | k | A4 |
    /// | 8  | l | B4 |
    /// | 9  | ; | C5 |
    /// | 10 | n | D5 |
    /// | 11 | m | E5 |
    /// | 12 | , | F5 |
    /// | 13 | . | G5 |
    /// | 14 | / | A5 |
    pub fn a_minor_to_c_major() -> Self {
        let keys = [
            "y", "u", "i", "o", "p", "h", "j", "k", "l", ";", "n", "m", ",", ".", "/",
        ];

        // A minor starts at A3 (MIDI 45), C major ends at C6 (MIDI 84)
        // But we want A3 to C5 for 15 notes (two octaves: A-A with C at end)
        // A3=45, C5=72, span=27 semitones for 15 keys
        let min_note: u8 = 45; // A3
        let max_note: u8 = 72; // C5
        let range = max_note - min_note;

        let note_to_key: HashMap<u8, String> = (0u8..=127)
            .map(|note| {
                let position = if range > 0 {
                    let pos = ((note.saturating_sub(min_note) as f64) / (range as f64) * 14.0)
                        .round() as u8;
                    pos.min(14)
                } else {
                    0
                };
                (note, keys[position as usize].to_string())
            })
            .collect();

        Mapper { note_to_key }
    }

    /// Create a mapper that compresses the given note range to Sky's 15 keys.
    pub fn from_note_range(min_note: u8, max_note: u8) -> Self {
        let keys = [
            "y", "u", "i", "o", "p", "h", "j", "k", "l", ";", "n", "m", ",", ".", "/",
        ];

        let range = max_note.saturating_sub(min_note);

        let note_to_key: HashMap<u8, String> = (0u8..=127)
            .map(|note| {
                let position = if range > 0 {
                    let pos = ((note.saturating_sub(min_note) as f64) / (range as f64) * 14.0)
                        .round() as u8;
                    pos.min(14)
                } else {
                    0
                };
                (note, keys[position as usize].to_string())
            })
            .collect();

        Mapper { note_to_key }
    }

    /// Convert a MIDI note number to a keyboard key.
    /// Returns `None` if the note is not mapped.
    pub fn note_to_key(&self, note: u8) -> Option<&str> {
        self.note_to_key.get(&note).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_minor_to_c_major() {
        let mapper = Mapper::a_minor_to_c_major();

        // A3 (45) should map to "y" (first key)
        assert_eq!(mapper.note_to_key(45), Some("y"));
        // C5 (72) should map to "/" (last key)
        assert_eq!(mapper.note_to_key(72), Some("/"));
        // Middle of range: note 60 is (60-45)/27*14 ≈ 8, so "l"
        assert_eq!(mapper.note_to_key(60), Some("l"));
    }

    #[test]
    fn test_mapper_from_config() {
        let config = MappingConfig {
            mapping: [
                ("60".to_string(), "a".to_string()),
                ("61".to_string(), "s".to_string()),
            ]
            .into(),
        };

        let mapper = Mapper::from_config(&config);
        assert_eq!(mapper.note_to_key(60), Some("a"));
        assert_eq!(mapper.note_to_key(61), Some("s"));
        assert_eq!(mapper.note_to_key(62), None);
    }

    #[test]
    fn test_empty_mapping() {
        let config = MappingConfig::default();
        let mapper = Mapper::from_config(&config);

        assert_eq!(mapper.note_to_key(60), None);
    }

    #[test]
    fn test_invalid_note_in_config() {
        // Config with non-numeric key should be ignored
        let config: MappingConfig = toml::from_str(r#"mapping = { "abc" = "h" }"#).unwrap();
        let mapper = Mapper::from_config(&config);
        // Should have no mappings since "abc" is not a valid note number
        assert_eq!(mapper.note_to_key(60), None);
    }
}
