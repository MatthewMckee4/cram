# Changelog

## Unreleased

### Added

- Initial release: pure-Rust flashcard app with Typst rendering and SM-2 spaced repetition
- `cram_core`: Card and Deck types with SM-2 scheduling algorithm
- `cram_store`: TOML-based deck persistence to `~/.local/share/cram/`
- `cram_render`: In-process Typst→PNG rendering via `typst` + `typst-render` crates
- `cram_ui`: egui GUI with DeckList, StudyView, and EditorView with live Typst preview
