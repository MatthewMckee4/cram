# Configuration

## Data storage

Cram stores decks at the platform-specific data directory:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/cram/decks/` |
| macOS | `~/Library/Application Support/cram/decks/` |
| Windows | `%APPDATA%\cram\decks\` |

Each deck is a single TOML file named after the deck (e.g., `Rust Basics.toml`).

## TOML deck format

Decks are plain TOML that you can edit directly:

```toml
name = "Rust Ownership"
description = "Core Rust memory concepts"
created = "2026-03-02"
preamble = "#set text(size: 14pt)"

[[cards]]
id = "550e8400-e29b-41d4-a716-446655440000"
front = "What does the borrow checker enforce?"
back = "= Rules\n- One owner\n- Borrows don't outlive owner"
due = "2026-03-05"
interval = 3.0
ease = 2.5
reps = 2
tags = ["memory", "borrowing"]
```

## Card fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier (auto-generated) |
| `front` | String | Typst markup for the question side |
| `back` | String | Typst markup for the answer side |
| `due` | Date | Next review date (YYYY-MM-DD) |
| `interval` | Float | Days between reviews |
| `ease` | Float | SM-2 ease factor (1.3 to 2.5) |
| `reps` | Integer | Number of successful reviews |
| `tags` | Array | Optional list of tag strings |

## Deck fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Deck name (also used as filename) |
| `description` | String | Optional description shown in deck list |
| `created` | Date | Creation date |
| `preamble` | String | Typst code prepended to every card when rendering |

## Theme

Toggle between dark and light mode using the button in the top-right corner of the app. The setting is stored in memory for the current session.

## Backup

Since decks are plain TOML files, back them up by copying the data directory. You can also use `git` to version-control your decks.
