#!/usr/bin/env python3
"""Parse MIDI file using mido and output note events."""

import sys
import mido
from collections import defaultdict

def parse_midi_to_notes(filename):
    """Parse MIDI file and return list of (time, note, is_note_on, velocity)."""
    mid = mido.MidiFile(filename)
    
    # Convert delta to cumulative time
    for track in mid.tracks:
        tick = 0
        for msg in track:
            msg.time += tick
            tick = msg.time
    
    # Track active notes: (channel, note) -> (start_tick, velocity)
    active_notes = {}
    events = []
    
    for track in mid.tracks:
        tick = 0
        for msg in track:
            tick = msg.time
            
            if msg.type == 'note_on' and msg.velocity > 0:
                key = (msg.channel, msg.note)
                active_notes[key] = (tick, msg.velocity)
            elif msg.type == 'note_off' or (msg.type == 'note_on' and msg.velocity == 0):
                key = (msg.channel, msg.note)
                if key in active_notes:
                    start_tick, velocity = active_notes[key]
                    events.append((start_tick, msg.note, True, velocity))
                    events.append((tick, msg.note, False, 0))
                    del active_notes[key]
    
    # Sort by time
    events.sort(key=lambda x: x[0])
    
    # Convert ticks to seconds (assuming 480 ticks per beat and 120 BPM default)
    ticks_per_beat = mid.ticks_per_beat or 480
    seconds_per_beat = 0.5  # 120 BPM
    seconds_per_tick = seconds_per_beat / ticks_per_beat
    
    result = []
    for tick, note, is_on, vel in events:
        time_sec = tick * seconds_per_tick
        result.append(f"{time_sec:.4f},{note},{1 if is_on else 0},{vel}")
    
    return result

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: parse_midi.py <midi_file>", file=sys.stderr)
        sys.exit(1)
    
    notes = parse_midi_to_notes(sys.argv[1])
    for n in notes:
        print(n)
