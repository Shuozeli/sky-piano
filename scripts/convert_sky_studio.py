#!/usr/bin/env python3
"""Convert SkyStudio sheet music to our CSV format."""

import json
import sys
import os
from pathlib import Path

# Sharps for each note (0-11 semitones from C)
SHARP_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']

def midi_to_note_name(midi):
    """Convert MIDI note number to note name like C4, D#4, Bb5."""
    octave = (midi // 12) - 1
    semitone = midi % 12
    return f"{SHARP_NAMES[semitone]}{octave}"

def key_num_to_note(key_num):
    """Map SkyStudio key number (0-14) to MIDI note (45-72, A3 to C5)."""
    if key_num < 0 or key_num > 14:
        return None
    # key 0 = A3 (45), key 1 = A#3 (46), ..., key 13 = A#4 (58), key 14 = C5 (60)
    # Actually let's use a simple linear mapping: 45 + key_num for 0-13, and 60 for 14
    if key_num < 14:
        return 45 + key_num
    else:
        return 60  # C5

def parse_song_key(key_str):
    """Parse SkyStudio key like '1Key0' to (column, key_number)."""
    # Format: [column]Key[number]
    parts = key_str.replace('Key', ',').split(',')
    if len(parts) != 2:
        return None, None
    try:
        column = int(parts[0])
        key_num = int(parts[1])
        return column, key_num
    except:
        return None, None

def convert_file(input_path, output_path):
    """Convert a SkyStudio file to our CSV format."""
    # Read UTF-16 encoded JSON
    with open(input_path, 'r', encoding='utf-16') as f:
        data = json.load(f)

    # Get first song (some files have array with one song)
    if isinstance(data, list):
        song = data[0]
    else:
        song = data

    name = song.get('name', 'Unknown')
    bpm = song.get('bpm', 120)
    notes = song.get('songNotes', [])

    print(f"Converting: {name} (BPM: {bpm}, Notes: {len(notes)})")

    # Group notes by time (ms)
    time_notes = {}
    for note in notes:
        t = note['time']
        key_str = note['key']
        column, key_num = parse_song_key(key_str)
        if key_num is not None and key_num < 15:
            midi_note = key_num_to_note(key_num)
            note_name = midi_to_note_name(midi_note)
            if t not in time_notes:
                time_notes[t] = set()
            time_notes[t].add(note_name)

    # Convert to list and sort by time
    sorted_times = sorted(time_notes.keys())

    # Convert to seconds
    chords = []
    for i, t in enumerate(sorted_times):
        t_sec = t / 1000.0
        notes_at_time = sorted(time_notes[t])  # Sort for consistent output
        keys = ''.join(notes_at_time)
        # Calculate duration as time to next chord minus current time
        if i + 1 < len(sorted_times):
            next_t = sorted_times[i + 1]
            duration = (next_t - t) / 1000.0
        else:
            duration = 0.5  # default duration for last chord
        chords.append((t_sec, keys, duration))

    # Write output
    with open(output_path, 'w') as f:
        f.write(f"# {name}\n")
        f.write(f"# Converted from SkyStudio format\n")
        f.write(f"# BPM: {bpm}\n")
        f.write(f"# Total chords: {len(chords)}\n")
        f.write("\n")
        f.write("Time,Keys,Duration\n")
        for t, keys, dur in chords:
            f.write(f"{t:.2f},{keys},{dur:.2f}\n")

    print(f"  -> Wrote {len(chords)} chords to {output_path}")
    return len(chords)

def main():
    if len(sys.argv) < 3:
        print("Usage: convert_sky_studio.py <input_dir> <output_dir>")
        sys.exit(1)

    input_dir = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)

    # Find all .txt files
    txt_files = list(input_dir.glob("*.txt"))
    print(f"Found {len(txt_files)} files to convert")

    for f in txt_files:
        try:
            output_file = output_dir / f"{f.stem}.txt"
            convert_file(f, output_file)
        except Exception as e:
            print(f"  ERROR converting {f.name}: {e}")

if __name__ == "__main__":
    main()
