- ALWAYS read CONTRIBUTING.md for guidelines on how to run tools
- ALWAYS attempt to add a test case for changed behavior. Get your tests to pass — if you didn't run the tests, your code does not work.
- PREFER integration tests in `it/` over unit tests
- ALWAYS run `just test` to run all tests
- ALWAYS run `uvx prek run -a` at the end of a task
- FOLLOW existing code style. Check neighboring files for patterns.
- AVOID writing significant amounts of new code. Look for existing methods and utilities first.
- AVOID using `panic!`, `unreachable!`, `.unwrap()`, unsafe code, and clippy rule ignores.
- PREFER `if let` patterns for fallibility
- PREFER `#[expect()]` over `#[allow()]` if clippy must be disabled
- PREFER let chains over nested `if let` statements
- AVOID redundant comments. The code should speak for itself.
- PREFER function comments over inline comments.

## Project Overview

**Cram** is a pure-Rust flashcard application using egui for the GUI and the typst crate
for rendering card content. Cards are written in Typst markup, rendered to PNG in-process,
and displayed as egui textures. Spaced repetition uses SM-2 scheduling.

## Architecture

Flat `crates/` workspace:

| Crate | Responsibility |
|---|---|
| `crates/cram` | Binary entry point — eframe app init |
| `crates/cram_core` | Card, Deck, Review types; SM-2 algorithm |
| `crates/cram_store` | Load/save decks from `~/.local/share/cram/` as TOML |
| `crates/cram_render` | Typst → PNG bytes via typst + typst-render crates |
| `crates/cram_ui` | All egui views: DeckList, StudyView, Editor, StatsView |

## Development Commands

```bash
just test          # run all tests with nextest
cargo build        # build
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
uvx prek run -a    # run all pre-commit hooks
```

## Data Storage

Decks stored as TOML at `~/.local/share/cram/decks/<name>.toml`:
```toml
[deck]
name = "Rust Ownership"
created = "2026-03-02"

[[cards]]
id = "uuid-here"
front = "What does the borrow checker enforce?"
back = "= Rules\n- One owner\n- Borrows don't outlive owner"
due = "2026-03-02"
interval = 1
ease = 2.5
reps = 0
```

## SM-2 Algorithm

After each card review, user rates 1-4 (Again/Hard/Good/Easy):
- 1 (Again): reset interval to 1, ease -= 0.2
- 2 (Hard): interval *= 1.2, ease -= 0.15
- 3 (Good): interval *= ease
- 4 (Easy): interval *= ease * 1.3, ease += 0.1
- ease clamped to [1.3, 2.5]
- due date = today + interval days

## Typst Rendering

```rust
// cram_render renders Typst source to PNG bytes
let png_bytes = cram_render::render(typst_source, font_size, width_px)?;
// Then in egui: upload as texture and display
```

## Code Conventions

- Edition 2024, MSRV 1.80
- No unwrap() — use anyhow::Result and ?
- No direct eprintln! — use tracing
- Strict clippy pedantic
- All errors via thiserror in library crates, anyhow in binary
