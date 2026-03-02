use std::path::PathBuf;

use anyhow::Result;
use cram_core::Deck;

use crate::error::StoreError;

pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot find data directory"))?
            .join("cram")
            .join("decks");
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Create a Store pointing at a specific directory (useful for tests).
    pub fn with_dir(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    fn deck_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(format!("{name}.toml"))
    }

    pub fn save_deck(&self, deck: &Deck) -> Result<()> {
        let content = toml::to_string_pretty(deck)?;
        Ok(std::fs::write(self.deck_path(deck.name()), content)?)
    }

    pub fn load_deck(&self, name: &str) -> Result<Deck> {
        let path = self.deck_path(name);
        if !path.exists() {
            return Err(StoreError::NotFound(name.to_string()).into());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn list_decks(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(stem.to_string());
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn delete_deck(&self, name: &str) -> Result<()> {
        let path = self.deck_path(name);
        if !path.exists() {
            return Err(StoreError::NotFound(name.to_string()).into());
        }
        Ok(std::fs::remove_file(path)?)
    }

    pub fn load_all_decks(&self) -> Result<Vec<Deck>> {
        self.list_decks()?
            .iter()
            .map(|n| self.load_deck(n))
            .collect()
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new().expect("failed to create store")
    }
}

#[cfg(test)]
mod tests {
    use cram_core::{Card, Deck};

    use super::*;

    fn temp_store() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::with_dir(dir.path().to_path_buf()).unwrap();
        (store, dir)
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (store, _dir) = temp_store();
        let mut deck = Deck::new("test", "a test deck");
        deck.cards_mut().push(Card::new("Q", "A"));
        store.save_deck(&deck).unwrap();
        let loaded = store.load_deck("test").unwrap();
        assert_eq!(loaded.name(), "test");
        assert_eq!(loaded.cards().len(), 1);
        assert_eq!(loaded.cards()[0].front(), "Q");
    }

    #[test]
    fn list_decks_returns_saved() {
        let (store, _dir) = temp_store();
        store.save_deck(&Deck::new("alpha", "")).unwrap();
        store.save_deck(&Deck::new("beta", "")).unwrap();
        let names = store.list_decks().unwrap();
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
    }

    #[test]
    fn load_missing_deck_errors() {
        let (store, _dir) = temp_store();
        assert!(store.load_deck("doesnotexist").is_err());
    }

    #[test]
    fn delete_deck_removes_file() {
        let (store, _dir) = temp_store();
        store.save_deck(&Deck::new("to_delete", "")).unwrap();
        store.delete_deck("to_delete").unwrap();
        assert!(store.load_deck("to_delete").is_err());
    }

    #[test]
    fn list_decks_empty_initially() {
        let (store, _dir) = temp_store();
        let names = store.list_decks().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn delete_missing_deck_errors() {
        let (store, _dir) = temp_store();
        assert!(store.delete_deck("nonexistent").is_err());
    }

    #[test]
    fn save_deck_with_unicode_name() {
        let (store, _dir) = temp_store();
        let deck = Deck::new("日本語テスト", "unicode description");
        store.save_deck(&deck).unwrap();
        let loaded = store.load_deck("日本語テスト").unwrap();
        assert_eq!(loaded.name(), "日本語テスト");
        assert_eq!(loaded.description(), "unicode description");
    }

    #[test]
    fn load_all_decks_returns_all() {
        let (store, _dir) = temp_store();
        store.save_deck(&Deck::new("one", "")).unwrap();
        store.save_deck(&Deck::new("two", "")).unwrap();
        store.save_deck(&Deck::new("three", "")).unwrap();
        let all = store.load_all_decks().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn overwrite_deck_preserves_name() {
        let (store, _dir) = temp_store();
        let mut deck = Deck::new("stable", "v1");
        deck.cards_mut().push(Card::new("Q", "A"));
        store.save_deck(&deck).unwrap();

        deck.set_description("v2");
        store.save_deck(&deck).unwrap();

        let loaded = store.load_deck("stable").unwrap();
        assert_eq!(loaded.name(), "stable");
        assert_eq!(loaded.description(), "v2");
    }
}
