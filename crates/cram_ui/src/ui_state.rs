use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

/// Persisted UI state, stored as `ui_state.toml` in the config directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiState {
    #[serde(default)]
    pub theme: Option<Theme>,

    /// The name of the last-viewed deck.
    #[serde(default)]
    pub last_deck: Option<String>,
}

impl UiState {
    const FILE_NAME: &str = "ui_state.toml";

    /// Load UI state from the config directory.
    /// Returns a default `UiState` if the file doesn't exist.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join(Self::FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save UI state to the config directory.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(config_dir)?;
        let content = toml::to_string_pretty(self)?;
        Ok(std::fs::write(config_dir.join(Self::FILE_NAME), content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = UiState::load(dir.path()).expect("load");
        assert_eq!(state, UiState::default());
        assert!(state.theme.is_none());
        assert!(state.last_deck.is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = UiState {
            theme: Some(Theme::Nord),
            last_deck: Some("Rust Basics".to_string()),
        };
        state.save(dir.path()).expect("save");

        let reloaded = UiState::load(dir.path()).expect("load");
        assert_eq!(reloaded, state);
    }

    #[test]
    fn save_and_load_with_no_theme() {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = UiState {
            theme: None,
            last_deck: Some("My Deck".to_string()),
        };
        state.save(dir.path()).expect("save");

        let reloaded = UiState::load(dir.path()).expect("load");
        assert_eq!(reloaded, state);
    }

    #[test]
    fn save_and_load_with_no_last_deck() {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = UiState {
            theme: Some(Theme::Dracula),
            last_deck: None,
        };
        state.save(dir.path()).expect("save");

        let reloaded = UiState::load(dir.path()).expect("load");
        assert_eq!(reloaded, state);
    }

    #[test]
    fn save_creates_config_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("nested").join("config");
        let state = UiState {
            theme: Some(Theme::Dark),
            last_deck: None,
        };
        state.save(&nested).expect("save");
        assert!(nested.join("ui_state.toml").exists());
    }

    #[test]
    fn overwrite_preserves_latest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let first = UiState {
            theme: Some(Theme::Dark),
            last_deck: Some("Old Deck".to_string()),
        };
        first.save(dir.path()).expect("save");

        let second = UiState {
            theme: Some(Theme::Light),
            last_deck: Some("New Deck".to_string()),
        };
        second.save(dir.path()).expect("save");

        let reloaded = UiState::load(dir.path()).expect("load");
        assert_eq!(reloaded, second);
    }

    #[test]
    fn load_partial_toml_uses_defaults_for_missing_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("ui_state.toml"), "theme = \"nord\"\n").expect("write");

        let state = UiState::load(dir.path()).expect("load");
        assert_eq!(state.theme, Some(Theme::Nord));
        assert!(state.last_deck.is_none());
    }

    #[test]
    fn load_empty_toml_returns_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("ui_state.toml"), "").expect("write");

        let state = UiState::load(dir.path()).expect("load");
        assert_eq!(state, UiState::default());
    }
}
