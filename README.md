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

Default 15-key positional mapping matching Sky's 3x5 grid:

| Sky Key | Key | | Sky Key | Key |
|---------|----|-|---------|-----|
| 0  | `y` | | 10 | `n` |
| 1  | `u` | | 11 | `m` |
| 2  | `i` | | 12 | `,` |
| 3  | `o` | | 13 | `.` |
| 4  | `p` | | 14 | `/` |
| 5  | `h` | | | |
| 6  | `j` | | | |
| 7  | `k` | | | |
| 8  | `l` | | | |
| 9  | `;` | | | |

Keyboard layout (matches Sky's 3x5 grid):
- Row 0 (top):    Y U I O P
- Row 1 (middle): H J K L ;
- Row 2 (bottom): N M , . /

Customize in `mapping.toml`.

## Requirements

- macOS (keyboard simulation via `rdev`)
- Rust toolchain
- MIDI files (.mid)
