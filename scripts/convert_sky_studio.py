#!/usr/bin/env python3
"""Convert SkyStudio sheet music to our CSV format."""

import json
import sys
import os
from pathlib import Path

# Sky piano key mapping - the 15 keys arranged in 3x5 grid
# SkyStudio uses 1Key0-1Key14, 2Key0-2Key14, 3Key0-3Key14 (columns x keys)
# But Sky piano uses single row of 15 keys: y,u,i,o,p,h,j,k,l,;,n,m,,,.,

# Map column+key to Sky piano key
# Column 1: 1Key0-1Key14 -> y,u,i,o,p (row 1, keys 0-4), h,j,k,l,; (row 2, keys 5-9), n,m,,,.,/ (row 3, keys 10-14)
# Column 2: 2Key0-2Key14 -> same physical keys as column 1 but different position
# Column 3: 3Key0-3Key14 -> same physical keys as column 1 but different position
#
# For simplicity, we just use the key_number (0-14) to map to Sky keys
# key_number 0-4 -> first 5 keys, 5-9 -> next 5, 10-14 -> last 5

SKY_KEYS = ['y', 'u', 'i', 'o', 'p', 'h', 'j', 'k', 'l', ';', 'n', 'm', ',', '.', '/']

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
        if key_num is not None and key_num < len(SKY_KEYS):
            sky_key = SKY_KEYS[key_num]
            if t not in time_notes:
                time_notes[t] = set()
            time_notes[t].add(sky_key)
    
    # Convert to list and sort by time
    # Time in SkyStudio is in ms, convert to seconds
    # BPM is given, but we'll assume time is already scaled appropriately
    sorted_times = sorted(time_notes.keys())
    
    # Calculate time in seconds (assuming time is in ms and BPM affects speed)
    # For now, just use raw time values / 1000
    # Actually, looking at the data, time seems to be in some unit
    # Let's try: time / 1000 / (bpm / 60) or just time / 1000
    # The BPM gives us the tempo - higher BPM = faster playback
    
    # Actually, let's look at typical values. BPM 180 means 180 beats per minute
    # If time is in ms and we want seconds: time_ms / 1000
    # But the beats depend on BPM... Let's just use simple conversion
    
    # Looking at Flower Dance (BPM 180), first notes at time 0, 666, 1332...
    # 666ms = about 2/3 second which is reasonable for 180 BPM (beat = 333ms)
    # So time seems to be in ms already
    
    # Convert to seconds
    prev_time = 0
    chords = []
    for t in sorted_times:
        t_sec = t / 1000.0
        keys = ''.join(sorted(time_notes[t]))
        # Calculate duration as time to next chord minus current time
        idx = sorted_times.index(t)
        if idx + 1 < len(sorted_times):
            next_t = sorted_times[idx + 1]
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
