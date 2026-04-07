#!/usr/bin/env python3
"""Generate simple MIDI songs for Sky Piano."""

import struct
import os

def write_var_length(value):
    """Write a variable-length quantity (MIDI delta time)."""
    result = []
    while True:
        byte = value & 0x7F
        value >>= 7
        if value > 0:
            byte |= 0x80
        result.append(byte)
        if value == 0:
            break
    return bytes(reversed(result))

def write_be16(val):
    return struct.pack('>H', val)

def write_be32(val):
    return struct.pack('>I', val)

def note_to_midi(note_name, octave):
    """Convert note name + octave to MIDI note number."""
    names = {'C': 0, 'C#': 1, 'D': 2, 'D#': 3, 'E': 4, 'F': 5,
             'F#': 6, 'G': 7, 'G#': 8, 'A': 9, 'A#': 10, 'B': 11}
    return names[note_name] + (octave + 1) * 12

def create_melody(name, tempo, notes):
    """Create a complete MIDI file.
    notes: list of (note_name, octave, duration_in_beats)
    """
    division = 96  # ticks per quarter note
    micros_per_beat = int(60_000_000 / tempo)

    midi = b'MThd'
    midi += write_be32(6)
    midi += write_be16(0)  # format 0
    midi += write_be16(1)  # 1 track
    midi += write_be16(division)

    # Build track data
    track_data = b''

    # Tempo: delta 0, meta 0x51, 3 bytes
    track_data += b'\x00\xff\x51\x03'
    track_data += struct.pack('>I', micros_per_beat)[1:]

    # Track absolute time of the last event (in ticks)
    last_event_time = 0
    # Track when the NEXT note should start (in ticks)
    next_note_start = 0

    for note_name, octave, duration_beats in notes:
        midi_note = note_to_midi(note_name, octave)
        duration_ticks = int(duration_beats * division)

        # Delta = ticks from last event to this event
        delta_on = next_note_start - last_event_time
        delta_off = duration_ticks

        # Note on
        track_data += write_var_length(delta_on)
        track_data += bytes([0x90, midi_note, 100])  # channel 0, velocity 100

        # Note off (happens duration_ticks after the on)
        track_data += write_var_length(delta_off)
        track_data += bytes([0x80, midi_note, 0])

        # Update times
        last_event_time = next_note_start + duration_ticks  # time of OFF event
        next_note_start = last_event_time  # next note starts when this one ends

    # End of track
    track_data += b'\x00\xff\x2f\x00'

    # Wrap in MTrk
    track = b'MTrk'
    track += write_be32(len(track_data))
    track += track_data

    return midi + track

def save_midi(filepath, data):
    with open(filepath, 'wb') as f:
        f.write(data)
    print(f"Created: {filepath}")

def main():
    songs_dir = os.path.join(os.path.dirname(__file__), '..', 'songs')
    os.makedirs(songs_dir, exist_ok=True)

    # Twinkle Twinkle Little Star
    # C C G G A A G(4) F F E E D D C(4)
    twinkle = [
        ('C', 4, 1), ('C', 4, 1), ('G', 4, 1), ('G', 4, 1),
        ('A', 4, 1), ('A', 4, 1), ('G', 4, 2),
        ('F', 4, 1), ('F', 4, 1), ('E', 4, 1), ('E', 4, 1),
        ('D', 4, 1), ('D', 4, 1), ('C', 4, 2),
    ]
    save_midi(os.path.join(songs_dir, 'twinkle.mid'),
              create_melody('Twinkle Twinkle', 120, twinkle))

    # Mary Had a Little Lamb
    # E D C D E E E(2) D D D(2) E G G(2)
    mary = [
        ('E', 4, 1), ('D', 4, 1), ('C', 4, 1), ('D', 4, 1),
        ('E', 4, 1), ('E', 4, 1), ('E', 4, 2),
        ('D', 4, 1), ('D', 4, 1), ('D', 4, 2),
        ('E', 4, 1), ('G', 4, 1), ('G', 4, 2),
    ]
    save_midi(os.path.join(songs_dir, 'mary.mid'),
              create_melody('Mary Had a Little Lamb', 120, mary))

    # Ode to Joy
    # E E F G G F E D C C D E E D D(2)
    ode_to_joy = [
        ('E', 4, 1), ('E', 4, 1), ('F', 4, 1), ('G', 4, 1),
        ('G', 4, 1), ('F', 4, 1), ('E', 4, 1), ('D', 4, 1),
        ('C', 4, 1), ('C', 4, 1), ('D', 4, 1), ('E', 4, 1),
        ('E', 4, 2), ('D', 4, 2),
    ]
    save_midi(os.path.join(songs_dir, 'ode_to_joy.mid'),
              create_melody('Ode to Joy', 120, ode_to_joy))

    # Jingle Bells
    # E E E(2) E E E(2) E G C D E(4)
    jingle = [
        ('E', 4, 1), ('E', 4, 1), ('E', 4, 2),
        ('E', 4, 1), ('E', 4, 1), ('E', 4, 2),
        ('E', 4, 1), ('G', 4, 1), ('C', 4, 1), ('D', 4, 1),
        ('E', 4, 4),
    ]
    save_midi(os.path.join(songs_dir, 'jingle.mid'),
              create_melody('Jingle Bells', 120, jingle))

    # A simple scale up and down (good for testing)
    scale = []
    notes = [('C', 4), ('D', 4), ('E', 4), ('F', 4), ('G', 4), ('A', 4), ('B', 4), ('C', 5)]
    for n, o in notes:
        scale.append((n, o, 1))
    for n, o in reversed(notes[:-1]):
        scale.append((n, o, 1))
    save_midi(os.path.join(songs_dir, 'scale.mid'),
              create_melody('Scale', 90, scale))

    print(f"\nGenerated 5 songs in {songs_dir}/")

if __name__ == '__main__':
    main()
