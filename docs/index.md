# Cram

A pure-Rust flashcard application with Typst-powered card rendering and SM-2 spaced repetition.

## Features

- Write flashcard content in [Typst](https://typst.app) markup — full math, formatting, headings
- SM-2 spaced repetition scheduling with 4-point rating scale
- Live Typst preview with syntax highlighting and 300ms debounce
- Full-screen card preview mode
- Custom Typst preamble per deck (shared imports, macros, styles)
- Dark/light theme toggle
- Import/export cards as CSV
- Card tags and bulk operations (select all, delete selected)
- Per-deck statistics with interval histogram
- Study session summary (cards reviewed, retention %, time)
- Undo last rating during study sessions
- Search across all decks
- Keyboard shortcuts: Space to reveal, 1-4 to rate, Esc to go back
- Decks stored as plain TOML files at `~/.local/share/cram/decks/`

## Quick Start

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo install --path crates/cram
cram
```

See [Installation](installation.md) for prerequisites and [Getting Started](getting-started.md) for a walkthrough.
