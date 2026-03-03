use std::path::Path;

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A single completed study session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRecord {
    pub deck_name: String,
    pub date: NaiveDate,
    pub cards_reviewed: u32,
    pub elapsed_secs: u64,
}

/// Persistent study history stored in `study_stats.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StudyStats {
    #[serde(default)]
    pub session: Vec<SessionRecord>,
}

impl StudyStats {
    const FILENAME: &str = "study_stats.toml";

    /// Load study stats from the config directory.
    /// Returns empty stats if the file doesn't exist.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join(Self::FILENAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save study stats to the config directory.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(config_dir)?;
        let content = toml::to_string_pretty(self)?;
        Ok(std::fs::write(config_dir.join(Self::FILENAME), content)?)
    }

    /// Record a completed study session.
    pub fn record_session(
        &mut self,
        deck_name: String,
        date: NaiveDate,
        cards_reviewed: u32,
        elapsed_secs: u64,
    ) {
        self.session.push(SessionRecord {
            deck_name,
            date,
            cards_reviewed,
            elapsed_secs,
        });
    }

    /// Total number of sessions recorded.
    pub fn total_sessions(&self) -> usize {
        self.session.len()
    }

    /// Total number of cards reviewed across all sessions.
    pub fn total_cards_reviewed(&self) -> u32 {
        self.session.iter().map(|s| s.cards_reviewed).sum()
    }

    /// Total time spent studying in seconds.
    pub fn total_time_secs(&self) -> u64 {
        self.session.iter().map(|s| s.elapsed_secs).sum()
    }

    /// Per-deck summary: `(deck_name, total_sessions, total_cards, total_secs)`.
    pub fn per_deck_summary(&self) -> Vec<DeckSummary> {
        let mut map = std::collections::BTreeMap::<String, DeckSummary>::new();
        for s in &self.session {
            let entry = map
                .entry(s.deck_name.clone())
                .or_insert_with(|| DeckSummary {
                    deck_name: s.deck_name.clone(),
                    sessions: 0,
                    cards_reviewed: 0,
                    total_secs: 0,
                });
            entry.sessions += 1;
            entry.cards_reviewed += s.cards_reviewed;
            entry.total_secs += s.elapsed_secs;
        }
        map.into_values().collect()
    }

    /// Recent sessions (most recent first), up to `limit`.
    pub fn recent_sessions(&self, limit: usize) -> Vec<&SessionRecord> {
        self.session.iter().rev().take(limit).collect()
    }
}

/// Aggregated stats for a single deck.
#[derive(Debug, Clone)]
pub struct DeckSummary {
    pub deck_name: String,
    pub sessions: u32,
    pub cards_reviewed: u32,
    pub total_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> StudyStats {
        let mut stats = StudyStats::default();
        stats.record_session(
            "Rust Basics".into(),
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            10,
            120,
        );
        stats.record_session(
            "Rust Basics".into(),
            NaiveDate::from_ymd_opt(2026, 3, 2).expect("valid date"),
            5,
            60,
        );
        stats.record_session(
            "Algorithms".into(),
            NaiveDate::from_ymd_opt(2026, 3, 2).expect("valid date"),
            8,
            90,
        );
        stats
    }

    #[test]
    fn empty_stats_defaults() {
        let stats = StudyStats::default();
        assert_eq!(stats.total_sessions(), 0);
        assert_eq!(stats.total_cards_reviewed(), 0);
        assert_eq!(stats.total_time_secs(), 0);
        assert!(stats.per_deck_summary().is_empty());
        assert!(stats.recent_sessions(5).is_empty());
    }

    #[test]
    fn record_session_increments_totals() {
        let stats = sample_stats();
        assert_eq!(stats.total_sessions(), 3);
        assert_eq!(stats.total_cards_reviewed(), 23);
        assert_eq!(stats.total_time_secs(), 270);
    }

    #[test]
    fn per_deck_summary_groups_correctly() {
        let stats = sample_stats();
        let summaries = stats.per_deck_summary();
        assert_eq!(summaries.len(), 2);

        let rust = summaries.iter().find(|s| s.deck_name == "Rust Basics");
        assert!(rust.is_some());
        let rust = rust.expect("found");
        assert_eq!(rust.sessions, 2);
        assert_eq!(rust.cards_reviewed, 15);
        assert_eq!(rust.total_secs, 180);

        let algo = summaries.iter().find(|s| s.deck_name == "Algorithms");
        assert!(algo.is_some());
        let algo = algo.expect("found");
        assert_eq!(algo.sessions, 1);
        assert_eq!(algo.cards_reviewed, 8);
        assert_eq!(algo.total_secs, 90);
    }

    #[test]
    fn recent_sessions_returns_most_recent_first() {
        let stats = sample_stats();
        let recent = stats.recent_sessions(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].deck_name, "Algorithms");
        assert_eq!(recent[1].deck_name, "Rust Basics");
    }

    #[test]
    fn recent_sessions_with_large_limit() {
        let stats = sample_stats();
        let recent = stats.recent_sessions(100);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let stats = sample_stats();
        stats.save(dir.path()).expect("save");

        let loaded = StudyStats::load(dir.path()).expect("load");
        assert_eq!(loaded, stats);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let stats = StudyStats::load(dir.path()).expect("load");
        assert!(stats.session.is_empty());
    }
}
