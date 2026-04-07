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

    /// Create a mapper with the default 15-key positional mapping.
    ///
    /// Maps Sky key positions 0-14 directly to keyboard keys:
    /// | Sky Key | Key |
    /// |---------|-----|
    /// | 0  | y |
    /// | 1  | u |
    /// | 2  | i |
    /// | 3  | o |
    /// | 4  | p |
    /// | 5  | h |
    /// | 6  | j |
    /// | 7  | k |
    /// | 8  | l |
    /// | 9  | ; |
    /// | 10 | n |
    /// | 11 | m |
    /// | 12 | , |
    /// | 13 | . |
    /// | 14 | / |
    pub fn default_positional_15() -> Self {
        // Keyboard keys arranged in 3x5 grid matching Sky's layout:
        // Row 0 (top):    Y U I O P
        // Row 1 (middle): H J K L ;
        // Row 2 (bottom): N M , . /
        let keys = [
            "y", "u", "i", "o", "p", "h", "j", "k", "l", ";", "n", "m", ",", ".", "/",
        ];

        let note_to_key: HashMap<u8, String> = keys
            .iter()
            .enumerate()
            .map(|(pos, key)| (pos as u8, key.to_string()))
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
    fn test_default_positional_15() {
        let mapper = Mapper::default_positional_15();

        // Test positional mappings (Sky key N -> keyboard key)
        assert_eq!(mapper.note_to_key(0), Some("y")); // Top row, leftmost
        assert_eq!(mapper.note_to_key(4), Some("p")); // Top row, rightmost
        assert_eq!(mapper.note_to_key(5), Some("h")); // Middle row, leftmost
        assert_eq!(mapper.note_to_key(9), Some(";")); // Middle row, rightmost
        assert_eq!(mapper.note_to_key(10), Some("n")); // Bottom row, leftmost
        assert_eq!(mapper.note_to_key(14), Some("/")); // Bottom row, rightmost

        // Unmapped note
        assert_eq!(mapper.note_to_key(15), None);
        assert_eq!(mapper.note_to_key(127), None);
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
