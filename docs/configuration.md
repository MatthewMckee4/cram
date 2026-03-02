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
tags = ["memory", "borrowing"]
```

## Card fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier (auto-generated) |
| `front` | String | Typst markup for the question side |
| `back` | String | Typst markup for the answer side |
| `tags` | Array | Optional list of tag strings |

## Deck fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Deck name (also used as filename) |
| `description` | String | Optional description shown in deck list |
| `created` | Date | Creation date (YYYY-MM-DD) |
| `preamble` | String | Typst code prepended to every card when rendering |

## Linked sources

You can link external directories containing deck files so cram loads them alongside your local decks. This is useful for version-controlling decks in a git repo or sharing decks across machines.

### Linking a folder

**App**: Open the **Sources** tab in the top nav bar, enter a directory path, and click **Link Folder**.

**CLI**:

```bash
cram decks link ~/my-flashcards
```

### Listing linked sources

**App**: The **Sources** tab shows all linked directories with deck counts and git status.

**CLI**:

```bash
cram decks sources
```

### Syncing (git pull)

If a linked directory is a git repo, you can pull updates from within cram.

**App**: Click **Sync** on a source card, or **Sync All** to pull all git-repo sources at once.

**CLI**:

```bash
cram decks sync
```

Sync uses `git pull --ff-only`, which is deliberately conservative: it won't create merge commits and fails cleanly on conflicts so you can resolve them manually.

### Unlinking a folder

**App**: Click **Unlink** on a source card in the Sources tab.

**CLI**:

```bash
cram decks unlink ~/my-flashcards
```

### How it works

- Linked directories are stored in `sources.toml` alongside the `decks/` directory.
- Decks from linked sources are read-write: edits in the app save back to the linked directory.
- New decks created in the app always go to the local `decks/` directory.
- If a linked directory no longer exists at load time, it is skipped with a warning.

## Theme

Toggle between dark and light mode using the button in the top-right corner of the app. The setting is stored in memory for the current session.

## Backup

Since decks are plain TOML files, back them up by copying the data directory. You can also use `git` to version-control your decks, or link a git repo as a source to sync from within the app.
