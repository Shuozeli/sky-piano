# Codelab 01: Getting Started

## Goal

Get `sky-piano` building and playing a test MIDI file.

## Prerequisites

- macOS (required for keyboard simulation)
- Rust toolchain (`rustup` or `brew install rust`)
- Sky game running on another device

## Step 1: Build

```bash
cd sky-piano
cargo build --release
```

## Step 2: Grant Accessibility Permission

`rdev` requires macOS Accessibility access:

1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Click **+** and add your terminal app (e.g., Terminal, iTerm2, Alacritty)
3. Enable it

## Step 3: Find a MIDI File

Download any MIDI file, e.g.:
```bash
curl -O https://example.com/twinkle.mid
```

## Step 4: Preview the Song

```bash
cargo run -- preview twinkle.mid
```

This prints the note sequence without playing:
```
C4 @ 0.00s → h (0.50s)
G4 @ 0.50s → k (0.50s)
...
```

## Step 5: Play

Open Sky to the instrument you want to play on. Then:

```bash
cargo run -- play twinkle.mid
```

Keys will be simulated on your MacBook keyboard.

## Troubleshooting

**Keys not registering?**
- Check Accessibility permission in System Settings
- Try with `--dry-run` first to verify events are firing

**Timing off?**
- Some MIDI files have irregular timing; try another file
- Close CPU-heavy apps to reduce jitter
