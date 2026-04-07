# Sky Piano

Keyboard automation tool that plays MIDI songs on Sky: Children of the Light instruments by simulating keyboard input on macOS.

## Setup

```bash
# Build
cargo build --release

# Install pre-commit hooks (optional)
pip install pre-commit
pre-commit install
```

### macOS Permissions

`rdev` requires Accessibility access:
1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Add and enable your terminal app

## Usage

```bash
# Play a MIDI file (3 second delay before starting, shows keys as they play)
cargo run -- play song.mid

# Play with custom delay (e.g., 5 seconds)
cargo run -- play --delay 5 song.mid

# Dry run: print keys without pressing them (good for testing)
cargo run -- play --dry --delay 0 song.mid

# Preview: print note sequence without playing
cargo run -- preview song.mid

# Dry-run summary: show statistics about the song
cargo run -- dry-run song.mid
```

## Pre-built Songs

Included in `songs/`:
- `twinkle.mid` - Twinkle Twinkle Little Star
- `mary.mid` - Mary Had a Little Lamb
- `ode_to_joy.mid` - Ode to Joy
- `jingle.mid` - Jingle Bells
- `scale.mid` - Chromatic scale up and down
- `The_Legend_of_Heroes_Trails_in_the_Sky_The_Whereabouts_of_Light.mid` - Full song

## Key Mapping

Default 15-key chromatic mapping:

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

Customize in `mapping.toml`.

## Requirements

- macOS (keyboard simulation via `rdev`)
- Rust toolchain
- MIDI files (.mid)
