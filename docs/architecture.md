# Architecture

## Overview

Single binary CLI tool with 4 main modules.

## Module Responsibilities

### `keyboard.rs`
Wraps `rdev` for macOS keyboard simulation.
- `key_down(key: &str)` — simulate key press
- `key_up(key: &str)` — simulate key release

### `mapper.rs`
Converts MIDI note numbers (0-127) to keyboard key strings.
- Loads `mapping.toml` on startup
- `note_to_key(note: u8) -> Option<&str>`
- Returns `None` for unmapped notes (skipped)

### `midi.rs`
Parses MIDI files into a vector of timed events.
- `MidiEvent { time: f64, note: u8, is_note_on: bool, velocity: u8 }`
- `parse_midi(path: &Path) -> Vec<MidiEvent>`

### `player.rs`
Schedules and executes keyboard events in real time.
- `play(events: Vec<MidiEvent>, mapper: &Mapper) -> Result<()>`
- Uses `std::time::Instant` for wall-clock timing
- Handles chords by grouping same-timestamp events

### `main.rs`
CLI entry point using `clap`.
- `play` — play MIDI file with keyboard simulation
- `preview` — print note sequence without playing
- `dry-run` — show summary of what would be played

## Data Flow

```
main.rs
  └─▶ midi::parse_midi()
        └─▶ Vec<MidiEvent>
              └─▶ player::play()
                    ├─▶ mapper::note_to_key()
                    └─▶ keyboard::key_down/key_up()
```

## Configuration

`mapping.toml` structure:
```toml
[mapping]
60 = "h"   # MIDI note 60 (Middle C) → key 'h'
61 = "j"   # C# → 'j'
...
```

Default mapping embedded in binary; custom `mapping.toml` overrides.
