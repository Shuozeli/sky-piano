# Sky Piano — Design Doc

## Overview

Keyboard automation tool that plays MIDI songs on Sky: Children of the Light instruments by simulating macOS keyboard input.

**Problem:** Sky's music instruments require pressing on-screen buttons. This tool converts standard MIDI files into keyboard presses on the MacBook, allowing hands-free playback of songs.

**Scope:**
- macOS only (keyboard simulation via `rdev`)
- CLI tool, no GUI
- MIDI input, keyboard output

---

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│  MIDI File  │────▶│  midi2keys   │────▶│  rdev (keyboard)│
│  (.mid)     │     │  (Rust)      │     │  macOS events   │
└─────────────┘     └──────────────┘     └─────────────────┘
                          │
                    ┌─────▼─────┐
                    │ mapping   │
                    │.toml      │
                    └───────────┘
```

### Components

1. **MIDI Parser** — Reads `.mid` files, extracts note events (note on/off with velocity and timestamp)
2. **Note Mapper** — Converts MIDI note numbers → keyboard keys using `mapping.toml`
3. **Scheduler** — Tracks real time, fires key events at correct moments
4. **Keyboard Simulator** — Uses `rdev` crate to simulate key down/up on macOS

---

## Note Mapping (Default)

15 keys total:

| Note | Key | | Note | Key |
|------|----|-|------|-----|
| C  | `h` | | A# | `y` |
| C# | `j` | | B  | `u` |
| D  | `k` | | C2 | `i` |
| D# | `l` | | D2 | `o` |
| E  | `;` | | E2 | `p` |
| F  | `n` | | | |
| F# | `m` | | | |
| G  | `,` | | | |
| G# | `.` | | | |
| A  | `/` | | | |

MIDI note 60 (Middle C) maps to key `h`.

Users can override this mapping in `mapping.toml`.

---

## Data Flow

1. CLI parses arguments: `sky-piano play song.mid`
2. Load `mapping.toml` (or use embedded default)
3. Parse MIDI file into a list of timed events
4. Sort events by timestamp
5. Start playback loop:
   - Wait until next event time
   - If note-on: send key-down via `rdev`
   - If note-off: send key-up via `rdev`
6. Exit when all events processed or interrupted

---

## Key Implementation Details

### Timing

- Use `std::thread::sleep` with sub-millisecond precision for timing
- For chords (multiple keys at same timestamp), fire all key-downs before any key-ups
- Track elapsed time with `std::time::Instant`

### Chords

When multiple notes have the same timestamp, they form a chord:
```rust
// All keys in chord pressed simultaneously
for key in chord.keys {
    rdev::key_down(key);
}
// Small delay before releasing
thread::sleep(chord.duration);
for key in chord.keys {
    rdev::key_up(key);
}
```

### Velocity (Volume)

MIDI velocity (0-127) is ignored for now — all notes are played at fixed velocity. Future: map velocity to key press duration (louder = longer press).

### Tempo

MIDI files embed tempo (microseconds per beat). Use this for timing conversion. Fall back to default 120 BPM if not specified.

---

## File Structure

```
sky-piano/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry, argument parsing
│   ├── midi.rs          # MIDI file parsing
│   ├── mapper.rs        # Note → key mapping
│   ├── player.rs        # Playback scheduling
│   └── keyboard.rs      # rdev wrapper
├── mapping.toml.default # Default key mapping
├── docs/
│   └── design.md        # This file
└── README.md
```

---

## CLI Interface

```bash
# Play a MIDI file (3 second delay before starting, shows keys as they play)
sky-piano play song.mid

# Play with custom delay
sky-piano play --delay 5 song.mid

# Use custom mapping file
sky-piano play --mapping my-map.toml song.mid

# Preview: print note sequence without playing
sky-piano preview song.mid

# Dry run: show what would be played
sky-piano dry-run song.mid
```

### Playback Output

During playback, keys are printed with timestamps:
```
--- Playing ---
0.00s | h
0.50s | kj
1.00s | h;
--- Done ---
```

Single keys shown as-is (e.g., `h`), chords shown concatenated (e.g., `kj` for K and J pressed together).

---

## Dependencies

- `rdev` — cross-platform keyboard simulation (macOS compatible)
- `midir` or `rubber_rope` or `nom` — MIDI parsing (evaluate crates)
- `toml` or `serde_toml` — mapping config parsing
- `clap` — CLI argument parsing

---

## Future Improvements (Out of Scope for v1)

- Velocity-based duration mapping
- Octave modifier key for extended range
- GUI with visual keyboard
- Support for ABC notation input
- Song playlist mode
- Record mode: capture keyboard input → MIDI

---

## macOS Permissions

`rdev` requires Accessibility permissions. On first run, the user must:
1. Go to System Settings → Privacy & Security → Accessibility
2. Add and enable the terminal/app running `sky-piano`

---

## Testing

- Unit test MIDI parsing with sample `.mid` files
- Manual testing on macOS with Sky game open
- Verify timing accuracy with instrumentation logging
