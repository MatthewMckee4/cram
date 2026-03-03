use std::path::Path;

use anyhow::Result;
use cram_core::{Card, Deck};

/// Export a deck to a TOML file at the given path.
pub fn export_toml(deck: &Deck, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(deck)?;
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(std::fs::write(path, content)?)
}

/// Import a deck from a TOML file.
pub fn import_toml(path: &Path) -> Result<Deck> {
    let content = std::fs::read_to_string(path)?;
    let deck: Deck = toml::from_str(&content)?;
    Ok(deck)
}

/// Import a deck from a CSV file where each line is `front,back`.
/// The deck name is derived from the file stem.
pub fn import_csv(path: &Path) -> Result<Deck> {
    let content = std::fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported");
    let mut deck = Deck::new(name, "");
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((front, back)) = line.split_once(',') {
            let front = front.trim();
            let back = back.trim();
            if !front.is_empty() {
                deck.cards_mut().push(Card::new(front, back));
            }
        }
    }
    Ok(deck)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_toml_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test-deck.toml");

        let mut deck = Deck::new("Roundtrip", "testing export/import");
        deck.cards_mut().push(Card::new("Q1", "A1"));
        deck.cards_mut().push(Card::new("Q2", "A2"));
        *deck.preamble_mut() = "#set text(size: 14pt)".to_string();

        export_toml(&deck, &path).expect("export");
        let imported = import_toml(&path).expect("import");

        assert_eq!(imported.name(), "Roundtrip");
        assert_eq!(imported.description(), "testing export/import");
        assert_eq!(imported.preamble(), "#set text(size: 14pt)");
        assert_eq!(imported.cards().len(), 2);
        assert_eq!(imported.cards()[0].front(), "Q1");
        assert_eq!(imported.cards()[0].back(), "A1");
        assert_eq!(imported.cards()[1].front(), "Q2");
        assert_eq!(imported.cards()[1].back(), "A2");
    }

    #[test]
    fn export_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("dir").join("deck.toml");

        let deck = Deck::new("Nested", "");
        export_toml(&deck, &path).expect("export");
        assert!(path.exists());
    }

    #[test]
    fn import_csv_basic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("vocab.csv");
        std::fs::write(&path, "hello,world\nfoo,bar\n").expect("write");

        let deck = import_csv(&path).expect("import");
        assert_eq!(deck.name(), "vocab");
        assert_eq!(deck.cards().len(), 2);
        assert_eq!(deck.cards()[0].front(), "hello");
        assert_eq!(deck.cards()[0].back(), "world");
        assert_eq!(deck.cards()[1].front(), "foo");
        assert_eq!(deck.cards()[1].back(), "bar");
    }

    #[test]
    fn import_csv_skips_empty_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sparse.csv");
        std::fs::write(&path, "Q1,A1\n\n\nQ2,A2\n").expect("write");

        let deck = import_csv(&path).expect("import");
        assert_eq!(deck.cards().len(), 2);
    }

    #[test]
    fn import_csv_trims_whitespace() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("spaced.csv");
        std::fs::write(&path, "  hello , world  \n").expect("write");

        let deck = import_csv(&path).expect("import");
        assert_eq!(deck.cards()[0].front(), "hello");
        assert_eq!(deck.cards()[0].back(), "world");
    }

    #[test]
    fn import_csv_skips_lines_without_comma() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nocomma.csv");
        std::fs::write(&path, "no comma here\nQ,A\n").expect("write");

        let deck = import_csv(&path).expect("import");
        assert_eq!(deck.cards().len(), 1);
        assert_eq!(deck.cards()[0].front(), "Q");
    }

    #[test]
    fn import_csv_skips_empty_front() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("empty.csv");
        std::fs::write(&path, ",answer\nQ,A\n").expect("write");

        let deck = import_csv(&path).expect("import");
        assert_eq!(deck.cards().len(), 1);
        assert_eq!(deck.cards()[0].front(), "Q");
    }

    #[test]
    fn import_toml_invalid_file_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not valid toml [[[").expect("write");

        assert!(import_toml(&path).is_err());
    }

    #[test]
    fn import_toml_missing_file_errors() {
        let path = std::path::PathBuf::from("/nonexistent/path/deck.toml");
        assert!(import_toml(&path).is_err());
    }
}
