use std::path::{Path, PathBuf};

use anyhow::Result;
use cram_core::Deck;

use crate::Store;
use crate::git;
use crate::sources::{SourceKind, Sources};

/// Tracks where a deck was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckSource {
    /// The primary (local) decks directory.
    Local,
    /// A linked external directory.
    Linked(PathBuf),
}

/// Aggregates decks from the primary store and any linked sources (files or folders).
pub struct MultiStore {
    primary: Store,
    linked_folders: Vec<PathBuf>,
    linked_files: Vec<PathBuf>,
    sources: Sources,
    config_dir: PathBuf,
}

impl MultiStore {
    /// Build a `MultiStore` from a primary store and a config directory
    /// where `sources.toml` lives.
    pub fn new(primary: Store, config_dir: PathBuf) -> Result<Self> {
        let sources = Sources::load(&config_dir)?;
        let mut linked_folders = Vec::new();
        let mut linked_files = Vec::new();

        for (path, kind) in sources.entries() {
            match kind {
                SourceKind::Folder => {
                    if path.is_dir() {
                        linked_folders.push(path.to_path_buf());
                    } else {
                        tracing::warn!("skipping linked folder {}: not found", path.display());
                    }
                }
                SourceKind::File => {
                    if path.exists() {
                        linked_files.push(path.to_path_buf());
                    } else {
                        tracing::warn!("skipping linked file {}: not found", path.display());
                    }
                }
            }
        }

        Ok(Self {
            primary,
            linked_folders,
            linked_files,
            sources,
            config_dir,
        })
    }

    pub fn primary(&self) -> &Store {
        &self.primary
    }

    pub fn sources(&self) -> &Sources {
        &self.sources
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Load all decks from the primary store and all linked sources.
    pub fn load_all_decks(&self) -> Result<Vec<(Deck, DeckSource)>> {
        let mut result = Vec::new();

        for deck in self.primary.load_all_decks()? {
            result.push((deck, DeckSource::Local));
        }

        for dir in &self.linked_folders {
            for file_path in find_toml_files(dir) {
                match load_deck_from_file(&file_path) {
                    Ok(deck) => result.push((deck, DeckSource::Linked(file_path))),
                    Err(e) => tracing::warn!("failed to load deck from {}: {e}", dir.display()),
                }
            }
        }

        for file_path in &self.linked_files {
            match load_deck_from_file(file_path) {
                Ok(deck) => result.push((deck, DeckSource::Linked(file_path.clone()))),
                Err(e) => tracing::warn!("failed to load deck from {}: {e}", file_path.display()),
            }
        }

        Ok(result)
    }

    /// Save a deck back to its source location.
    pub fn save_deck(&self, deck: &Deck, source: &DeckSource) -> Result<()> {
        match source {
            DeckSource::Local => self.primary.save_deck(deck),
            DeckSource::Linked(path) => {
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    let content = toml::to_string_pretty(deck)?;
                    Ok(std::fs::write(path, content)?)
                } else {
                    let store = Store::open(path.clone())?;
                    store.save_deck(deck)
                }
            }
        }
    }

    /// Delete a deck from whichever store it lives in.
    pub fn delete_deck(&self, name: &str) -> Result<()> {
        if self.primary.load_deck(name).is_ok() {
            return self.primary.delete_deck(name);
        }
        for dir in &self.linked_folders {
            for file_path in find_toml_files(dir) {
                if let Ok(deck) = load_deck_from_file(&file_path)
                    && deck.name() == name
                {
                    std::fs::remove_file(file_path)?;
                    return Ok(());
                }
            }
        }
        for file_path in &self.linked_files {
            if let Ok(deck) = load_deck_from_file(file_path)
                && deck.name() == name
            {
                std::fs::remove_file(file_path)?;
                return Ok(());
            }
        }
        anyhow::bail!("deck not found: {name}")
    }

    /// Link an external path as a deck source.
    pub fn link(&mut self, path: PathBuf, kind: SourceKind) -> Result<bool> {
        match kind {
            SourceKind::Folder => {
                if !path.is_dir() {
                    anyhow::bail!("directory does not exist: {}", path.display());
                }
            }
            SourceKind::File => {
                if !path.is_file() {
                    anyhow::bail!("file does not exist: {}", path.display());
                }
            }
        }
        if !self.sources.add(path.clone(), kind) {
            return Ok(false);
        }
        self.sources.save(&self.config_dir)?;

        match kind {
            SourceKind::Folder => {
                self.linked_folders.push(path);
            }
            SourceKind::File => {
                self.linked_files.push(path);
            }
        }
        Ok(true)
    }

    /// Unlink a previously linked source (file or folder).
    pub fn unlink(&mut self, path: &Path) -> Result<bool> {
        if !self.sources.remove(path) {
            return Ok(false);
        }
        self.sources.save(&self.config_dir)?;
        self.linked_folders.retain(|p| p != path);
        self.linked_files.retain(|p| p != path);
        Ok(true)
    }

    /// Run `git pull --ff-only` on all linked sources that are git repos.
    /// Deduplicates so each git root is only pulled once.
    pub fn sync_all(&self) -> Vec<(PathBuf, git::SyncResult)> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();

        for (path, kind) in self.sources.entries() {
            let search_path = match kind {
                SourceKind::Folder => path.to_path_buf(),
                SourceKind::File => path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| path.to_path_buf()),
            };
            let sync_target =
                git::find_git_root(&search_path).unwrap_or_else(|| search_path.clone());
            if seen.insert(sync_target.clone()) {
                let result = git::pull(&search_path);
                results.push((sync_target, result));
            }
        }

        results
    }

    /// Run `git pull --ff-only` on a specific path (walks up to find git root).
    pub fn sync(&self, path: &Path) -> git::SyncResult {
        git::pull(path)
    }
}

fn load_deck_from_file(path: &Path) -> Result<Deck> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

/// Recursively find all `.toml` files under `dir`.
pub fn find_toml_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    collect_toml_files(dir, &mut results);
    results.sort();
    results
}

fn collect_toml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_toml_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use cram_core::Deck;

    use super::*;

    fn temp_multi_store() -> (MultiStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("create primary dir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("create config dir");

        let primary = Store::with_dir(primary_dir).expect("primary store");
        let ms = MultiStore::new(primary, config_dir).expect("multi store");
        (ms, dir)
    }

    #[test]
    fn load_from_primary_only() {
        let (ms, _dir) = temp_multi_store();
        ms.primary.save_deck(&Deck::new("test", "")).expect("save");
        let decks = ms.load_all_decks().expect("load");
        assert_eq!(decks.len(), 1);
        assert_eq!(decks[0].1, DeckSource::Local);
    }

    #[test]
    fn load_from_primary_and_linked_folder() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let linked_dir = dir.path().join("linked");
        std::fs::create_dir_all(&linked_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let primary = Store::with_dir(primary_dir).expect("primary");
        primary.save_deck(&Deck::new("local", "")).expect("save");

        let deck = Deck::new("remote", "");
        let content = toml::to_string_pretty(&deck).expect("serialize");
        std::fs::write(linked_dir.join("remote.toml"), content).expect("write");

        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(linked_dir.clone(), SourceKind::Folder)
            .expect("link");

        let decks = ms.load_all_decks().expect("load");
        assert_eq!(decks.len(), 2);

        let local = decks.iter().find(|(d, _)| d.name() == "local");
        assert!(matches!(local, Some((_, DeckSource::Local))));

        let remote = decks.iter().find(|(d, _)| d.name() == "remote");
        assert!(matches!(remote, Some((_, DeckSource::Linked(_)))));
    }

    #[test]
    fn load_from_linked_folder_recursive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let linked_dir = dir.path().join("linked");
        let sub_dir = linked_dir.join("sub");
        std::fs::create_dir_all(&sub_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let top = Deck::new("top-deck", "at root");
        std::fs::write(
            linked_dir.join("top-deck.toml"),
            toml::to_string_pretty(&top).expect("ser"),
        )
        .expect("write");

        let nested = Deck::new("nested-deck", "in subdir");
        std::fs::write(
            sub_dir.join("nested-deck.toml"),
            toml::to_string_pretty(&nested).expect("ser"),
        )
        .expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(linked_dir, SourceKind::Folder).expect("link");

        let decks = ms.load_all_decks().expect("load");
        assert_eq!(decks.len(), 2);

        let top_loaded = decks.iter().find(|(d, _)| d.name() == "top-deck");
        assert!(top_loaded.is_some());

        let nested_loaded = decks.iter().find(|(d, _)| d.name() == "nested-deck");
        assert!(nested_loaded.is_some());
        if let Some((_, DeckSource::Linked(p))) = nested_loaded {
            assert!(p.ends_with("sub/nested-deck.toml"));
        }
    }

    #[test]
    fn load_from_linked_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let file_path = dir.path().join("my-deck.toml");
        let deck = Deck::new("my-deck", "file source");
        let content = toml::to_string_pretty(&deck).expect("serialize");
        std::fs::write(&file_path, content).expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(file_path.clone(), SourceKind::File).expect("link");

        let decks = ms.load_all_decks().expect("load");
        assert_eq!(decks.len(), 1);
        assert_eq!(decks[0].0.name(), "my-deck");
        assert_eq!(decks[0].0.description(), "file source");
        assert!(matches!(&decks[0].1, DeckSource::Linked(p) if *p == file_path));
    }

    #[test]
    fn save_to_linked_folder_deck() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let linked_dir = dir.path().join("linked");
        std::fs::create_dir_all(&linked_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let file_path = linked_dir.join("saved-remote.toml");
        let deck = Deck::new("saved-remote", "original");
        std::fs::write(&file_path, toml::to_string_pretty(&deck).expect("ser")).expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(linked_dir, SourceKind::Folder).expect("link");

        let mut updated = Deck::new("saved-remote", "updated");
        updated.set_description("updated");
        let source = DeckSource::Linked(file_path.clone());
        ms.save_deck(&updated, &source).expect("save");

        let reloaded: Deck =
            toml::from_str(&std::fs::read_to_string(&file_path).expect("read")).expect("parse");
        assert_eq!(reloaded.description(), "updated");
    }

    #[test]
    fn save_to_linked_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let file_path = dir.path().join("deck.toml");
        let deck = Deck::new("deck", "original");
        let content = toml::to_string_pretty(&deck).expect("serialize");
        std::fs::write(&file_path, content).expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(file_path.clone(), SourceKind::File).expect("link");

        let mut updated = Deck::new("deck", "updated");
        updated.set_description("updated");
        let source = DeckSource::Linked(file_path.clone());
        ms.save_deck(&updated, &source).expect("save");

        let reloaded: Deck =
            toml::from_str(&std::fs::read_to_string(&file_path).expect("read")).expect("parse");
        assert_eq!(reloaded.description(), "updated");
    }

    #[test]
    fn missing_linked_dir_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let mut sources = Sources::default();
        sources.add(PathBuf::from("/nonexistent/path"), SourceKind::Folder);
        sources.save(&config_dir).expect("save sources");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let ms = MultiStore::new(primary, config_dir).expect("ms");
        assert!(ms.linked_folders.is_empty());
    }

    #[test]
    fn missing_linked_file_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let mut sources = Sources::default();
        sources.add(PathBuf::from("/nonexistent/deck.toml"), SourceKind::File);
        sources.save(&config_dir).expect("save sources");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let ms = MultiStore::new(primary, config_dir).expect("ms");
        assert!(ms.linked_files.is_empty());
    }
}
