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

    /// Create a mapper with the default 15-key chromatic mapping.
    ///
    /// | Note | Key |
    /// |------|-----|
    /// | C (60)  | h |
    /// | C# (61) | j |
    /// | D (62)  | k |
    /// | D# (63) | l |
    /// | E (64)  | ; |
    /// | F (65)  | n |
    /// | F# (66) | m |
    /// | G (67)  | , |
    /// | G# (68) | . |
    /// | A (69)  | / |
    /// | A# (70) | y |
    /// | B (71)  | u |
    /// | C2 (72) | i |
    /// | D2 (73) | o |
    /// | E2 (74) | p |
    pub fn default_chromatic_15() -> Self {
        let mapping: HashMap<u8, &str> = [
            (60, "h"),
            (61, "j"),
            (62, "k"),
            (63, "l"),
            (64, ";"),
            (65, "n"),
            (66, "m"),
            (67, ","),
            (68, "."),
            (69, "/"),
            (70, "y"),
            (71, "u"),
            (72, "i"),
            (73, "o"),
            (74, "p"),
        ]
        .into_iter()
        .collect();

        let note_to_key = mapping
            .into_iter()
            .map(|(note, key)| (note, key.to_string()))
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
    fn test_default_chromatic_15() {
        let mapper = Mapper::default_chromatic_15();

        // Test a few known mappings
        assert_eq!(mapper.note_to_key(60), Some("h")); // Middle C
        assert_eq!(mapper.note_to_key(61), Some("j")); // C#
        assert_eq!(mapper.note_to_key(64), Some(";")); // E
        assert_eq!(mapper.note_to_key(72), Some("i")); // C2

        // Unmapped note
        assert_eq!(mapper.note_to_key(0), None);
        assert_eq!(mapper.note_to_key(127), None);
        assert_eq!(mapper.note_to_key(75), None); // F2 not mapped
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
