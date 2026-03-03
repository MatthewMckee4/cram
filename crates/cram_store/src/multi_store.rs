use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

/// Cached entry for a single deck file, keyed by its path.
#[derive(Clone)]
struct CachedDeck {
    deck: Deck,
    source: DeckSource,
    modified: SystemTime,
}

/// Stores previously loaded decks keyed by file path, along with their
/// modification timestamps. Only files whose mtime has changed are re-read.
#[derive(Default)]
struct DeckCache {
    entries: HashMap<PathBuf, CachedDeck>,
}

/// Aggregates decks from the primary store and any linked sources (files or folders).
pub struct MultiStore {
    primary: Store,
    linked_folders: Vec<PathBuf>,
    linked_files: Vec<PathBuf>,
    sources: Sources,
    config_dir: PathBuf,
    cache: RefCell<DeckCache>,
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
            cache: RefCell::default(),
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
    ///
    /// Results are cached by file path and modification time. Only files whose
    /// mtime has changed since the last call are re-read from disk. The cache
    /// is automatically invalidated by mutating operations like `save_deck`,
    /// `delete_deck`, `link`, and `unlink`.
    pub fn load_all_decks(&self) -> Result<Vec<(Deck, DeckSource)>> {
        let mut result = Vec::new();
        let mut cache = self.cache.borrow_mut();
        let mut seen_paths: Vec<PathBuf> = Vec::new();

        for file_path in self.primary_toml_files() {
            seen_paths.push(file_path.clone());
            if let Some(entry) = cached_or_reload(&mut cache.entries, &file_path, DeckSource::Local)
            {
                result.push((entry.deck.clone(), entry.source.clone()));
            }
        }

        for dir in &self.linked_folders {
            for file_path in find_toml_files(dir) {
                seen_paths.push(file_path.clone());
                let source = DeckSource::Linked(file_path.clone());
                if let Some(entry) = cached_or_reload(&mut cache.entries, &file_path, source) {
                    result.push((entry.deck.clone(), entry.source.clone()));
                }
            }
        }

        for file_path in &self.linked_files {
            seen_paths.push(file_path.clone());
            let source = DeckSource::Linked(file_path.clone());
            if let Some(entry) = cached_or_reload(&mut cache.entries, file_path, source) {
                result.push((entry.deck.clone(), entry.source.clone()));
            }
        }

        cache.entries.retain(|k, _| seen_paths.contains(k));

        Ok(result)
    }

    /// List toml file paths in the primary store directory.
    fn primary_toml_files(&self) -> Vec<PathBuf> {
        find_toml_files(self.primary.data_dir())
    }

    /// Invalidate the deck cache so the next `load_all_decks` re-reads from disk.
    pub fn invalidate_cache(&self) {
        self.cache.borrow_mut().entries.clear();
    }

    /// Save a deck back to its source location.
    ///
    /// Invalidates the deck cache so the next `load_all_decks` picks up changes.
    pub fn save_deck(&self, deck: &Deck, source: &DeckSource) -> Result<()> {
        let result = match source {
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
        };
        self.invalidate_cache();
        result
    }

    /// Delete a deck from whichever store it lives in.
    ///
    /// Invalidates the deck cache so the next `load_all_decks` picks up changes.
    pub fn delete_deck(&self, name: &str) -> Result<()> {
        let result = self.delete_deck_inner(name);
        if result.is_ok() {
            self.invalidate_cache();
        }
        result
    }

    fn delete_deck_inner(&self, name: &str) -> Result<()> {
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
    ///
    /// Invalidates the deck cache when a new source is added.
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
        self.invalidate_cache();
        Ok(true)
    }

    /// Unlink a previously linked source (file or folder).
    ///
    /// Invalidates the deck cache when a source is removed.
    pub fn unlink(&mut self, path: &Path) -> Result<bool> {
        if !self.sources.remove(path) {
            return Ok(false);
        }
        self.sources.save(&self.config_dir)?;
        self.linked_folders.retain(|p| p != path);
        self.linked_files.retain(|p| p != path);
        self.invalidate_cache();
        Ok(true)
    }

    /// Run `git pull --ff-only` on all linked sources that are git repos.
    /// Deduplicates so each git root is only pulled once.
    ///
    /// Invalidates the deck cache since pulled changes may update deck files.
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

        self.invalidate_cache();
        results
    }

    /// Run `git pull --ff-only` on a specific path (walks up to find git root).
    ///
    /// Invalidates the deck cache since pulled changes may update deck files.
    pub fn sync(&self, path: &Path) -> git::SyncResult {
        let result = git::pull(path);
        self.invalidate_cache();
        result
    }
}

/// Return a cached entry if the file's mtime hasn't changed, otherwise reload.
/// Returns `None` if the file cannot be read (logged as a warning).
fn cached_or_reload(
    entries: &mut HashMap<PathBuf, CachedDeck>,
    path: &Path,
    source: DeckSource,
) -> Option<CachedDeck> {
    let mtime = file_modified(path);

    if let Some(cached) = entries.get(path)
        && Some(cached.modified) == mtime
    {
        return Some(cached.clone());
    }

    match load_deck_from_file(path) {
        Ok(deck) => {
            let modified = mtime.unwrap_or(SystemTime::UNIX_EPOCH);
            let entry = CachedDeck {
                deck,
                source,
                modified,
            };
            entries.insert(path.to_path_buf(), entry.clone());
            Some(entry)
        }
        Err(e) => {
            tracing::warn!("failed to load deck from {}: {e}", path.display());
            entries.remove(path);
            None
        }
    }
}

fn file_modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
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
    fn cache_returns_same_results_on_second_call() {
        let (ms, _dir) = temp_multi_store();
        ms.primary
            .save_deck(&Deck::new("cached", ""))
            .expect("save");

        let first = ms.load_all_decks().expect("first load");
        let second = ms.load_all_decks().expect("second load");
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].0.name(), second[0].0.name());
    }

    #[test]
    fn cache_detects_external_file_modification() {
        let (ms, _dir) = temp_multi_store();
        ms.primary
            .save_deck(&Deck::new("evolving", "v1"))
            .expect("save");

        let first = ms.load_all_decks().expect("first load");
        assert_eq!(first[0].0.description(), "v1");

        // Modify the file directly (simulating an external edit)
        let deck_path = ms.primary.data_dir().join("evolving.toml");
        let updated = Deck::new("evolving", "v2");
        let content = toml::to_string_pretty(&updated).expect("ser");
        // Ensure mtime changes by waiting a moment then writing
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&deck_path, content).expect("write");

        let second = ms.load_all_decks().expect("second load");
        assert_eq!(second[0].0.description(), "v2");
    }

    #[test]
    fn save_deck_invalidates_cache() {
        let (ms, _dir) = temp_multi_store();
        ms.primary
            .save_deck(&Deck::new("orig", "v1"))
            .expect("save");

        let first = ms.load_all_decks().expect("first load");
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].0.description(), "v1");

        let updated = Deck::new("orig", "v2");
        ms.save_deck(&updated, &DeckSource::Local).expect("save");

        let second = ms.load_all_decks().expect("second load");
        assert_eq!(second[0].0.description(), "v2");
    }

    #[test]
    fn delete_deck_invalidates_cache() {
        let (ms, _dir) = temp_multi_store();
        ms.primary
            .save_deck(&Deck::new("doomed", ""))
            .expect("save");

        let first = ms.load_all_decks().expect("first load");
        assert_eq!(first.len(), 1);

        ms.delete_deck("doomed").expect("delete");

        let second = ms.load_all_decks().expect("second load");
        assert!(second.is_empty());
    }

    #[test]
    fn link_invalidates_cache() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let linked_dir = dir.path().join("linked");
        std::fs::create_dir_all(&linked_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let deck = Deck::new("linked-deck", "");
        std::fs::write(
            linked_dir.join("linked-deck.toml"),
            toml::to_string_pretty(&deck).expect("ser"),
        )
        .expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");

        let before = ms.load_all_decks().expect("before link");
        assert!(before.is_empty());

        ms.link(linked_dir, SourceKind::Folder).expect("link");

        let after = ms.load_all_decks().expect("after link");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].0.name(), "linked-deck");
    }

    #[test]
    fn unlink_invalidates_cache() {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary_dir = dir.path().join("primary");
        std::fs::create_dir_all(&primary_dir).expect("mkdir");
        let linked_dir = dir.path().join("linked");
        std::fs::create_dir_all(&linked_dir).expect("mkdir");
        let config_dir = dir.path().join("config");
        std::fs::create_dir_all(&config_dir).expect("mkdir");

        let deck = Deck::new("will-unlink", "");
        std::fs::write(
            linked_dir.join("will-unlink.toml"),
            toml::to_string_pretty(&deck).expect("ser"),
        )
        .expect("write");

        let primary = Store::with_dir(primary_dir).expect("primary");
        let mut ms = MultiStore::new(primary, config_dir).expect("ms");
        ms.link(linked_dir.clone(), SourceKind::Folder)
            .expect("link");

        let before = ms.load_all_decks().expect("before unlink");
        assert_eq!(before.len(), 1);

        ms.unlink(&linked_dir).expect("unlink");

        let after = ms.load_all_decks().expect("after unlink");
        assert!(after.is_empty());
    }

    #[test]
    fn cache_prunes_deleted_files() {
        let (ms, _dir) = temp_multi_store();
        ms.primary.save_deck(&Deck::new("stays", "")).expect("save");
        ms.primary.save_deck(&Deck::new("goes", "")).expect("save");

        let first = ms.load_all_decks().expect("first load");
        assert_eq!(first.len(), 2);

        // Delete one file directly (bypassing MultiStore)
        let gone_path = ms.primary.data_dir().join("goes.toml");
        std::fs::remove_file(gone_path).expect("remove");
        ms.invalidate_cache();

        let second = ms.load_all_decks().expect("second load");
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].0.name(), "stays");
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
