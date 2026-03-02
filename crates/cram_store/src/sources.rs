use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub path: PathBuf,
    pub kind: SourceKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sources {
    #[serde(default)]
    pub source: Vec<Source>,
}

impl Sources {
    /// Load sources from the config directory.
    /// Returns an empty `Sources` if the file doesn't exist.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join("sources.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save sources to the config directory.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(config_dir)?;
        let content = toml::to_string_pretty(self)?;
        Ok(std::fs::write(config_dir.join("sources.toml"), content)?)
    }

    /// Add a source path. Returns `true` if it was actually added (not a duplicate).
    pub fn add(&mut self, path: PathBuf, kind: SourceKind) -> bool {
        if self.source.iter().any(|s| s.path == path) {
            return false;
        }
        self.source.push(Source { path, kind });
        true
    }

    /// Remove a source path. Returns `true` if it was found and removed.
    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.source.len();
        self.source.retain(|s| s.path != path);
        self.source.len() != before
    }

    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.source.iter().map(|s| s.path.as_path())
    }

    /// Iterate over sources yielding `(path, kind)` pairs.
    pub fn entries(&self) -> impl Iterator<Item = (&Path, SourceKind)> {
        self.source.iter().map(|s| (s.path.as_path(), s.kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sources = Sources::load(dir.path()).expect("load");
        assert!(sources.source.is_empty());
    }

    #[test]
    fn add_save_reload_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut sources = Sources::default();
        assert!(sources.add(PathBuf::from("/tmp/decks"), SourceKind::Folder));
        sources.save(dir.path()).expect("save");

        let reloaded = Sources::load(dir.path()).expect("load");
        assert_eq!(reloaded.source.len(), 1);
        assert_eq!(reloaded.source[0].path, PathBuf::from("/tmp/decks"));
        assert_eq!(reloaded.source[0].kind, SourceKind::Folder);
    }

    #[test]
    fn add_file_source_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut sources = Sources::default();
        assert!(sources.add(PathBuf::from("/tmp/deck.toml"), SourceKind::File));
        sources.save(dir.path()).expect("save");

        let reloaded = Sources::load(dir.path()).expect("load");
        assert_eq!(reloaded.source.len(), 1);
        assert_eq!(reloaded.source[0].path, PathBuf::from("/tmp/deck.toml"));
        assert_eq!(reloaded.source[0].kind, SourceKind::File);
    }

    #[test]
    fn add_duplicate_returns_false() {
        let mut sources = Sources::default();
        assert!(sources.add(PathBuf::from("/tmp/decks"), SourceKind::Folder));
        assert!(!sources.add(PathBuf::from("/tmp/decks"), SourceKind::Folder));
        assert_eq!(sources.source.len(), 1);
    }

    #[test]
    fn remove_existing_returns_true() {
        let mut sources = Sources::default();
        sources.add(PathBuf::from("/tmp/decks"), SourceKind::Folder);
        assert!(sources.remove(Path::new("/tmp/decks")));
        assert!(sources.source.is_empty());
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut sources = Sources::default();
        assert!(!sources.remove(Path::new("/tmp/nonexistent")));
    }
}
