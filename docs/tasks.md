# Tasks

## v1.0 — Initial Release

- [ ] Set up Rust project with Cargo.toml (rdev, midir, clap, toml)
- [ ] Implement keyboard.rs — rdev wrapper for key down/up
- [ ] Implement mapper.rs — MIDI note → keyboard key mapping from TOML
- [ ] Implement midi.rs — parse MIDI files, extract note events
- [ ] Implement player.rs — playback scheduler with timing
- [ ] Implement main.rs — CLI with play/preview/dry-run commands
- [ ] Create mapping.toml.default
- [ ] Write README.md with usage instructions
- [ ] Test on macOS with Sky game open
- [ ] Verify timing accuracy

## Future

- [ ] Velocity-based duration mapping
- [ ] Octave modifier for extended range
- [ ] Support ABC notation input
- [ ] Playlist mode
- [ ] Record mode (keyboard → MIDI)
