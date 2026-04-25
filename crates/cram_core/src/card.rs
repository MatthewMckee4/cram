use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    id: Uuid,
    front: String,
    back: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review: Option<ReviewState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    hidden: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Spaced-repetition scheduling state for a single card (SM-2 algorithm).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewState {
    /// Number of consecutive correct reviews.
    pub repetitions: u32,
    /// Inter-repetition interval in days.
    pub interval: u32,
    /// Ease factor (minimum 1.3). Default 2.5.
    pub ease_factor: f64,
    /// Date the card is next due for review. `None` means never reviewed.
    pub due_date: Option<NaiveDate>,
}

impl Default for ReviewState {
    fn default() -> Self {
        Self {
            repetitions: 0,
            interval: 0,
            ease_factor: 2.5,
            due_date: None,
        }
    }
}

impl Card {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            front: front.into(),
            back: back.into(),
            review: None,
            tags: Vec::new(),
            hidden: false,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn front(&self) -> &str {
        &self.front
    }

    pub fn front_mut(&mut self) -> &mut String {
        &mut self.front
    }

    pub fn back(&self) -> &str {
        &self.back
    }

    pub fn back_mut(&mut self) -> &mut String {
        &mut self.back
    }

    pub fn review(&self) -> Option<&ReviewState> {
        self.review.as_ref()
    }

    pub fn set_review(&mut self, state: ReviewState) {
        self.review = Some(state);
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn tags_mut(&mut self) -> &mut Vec<String> {
        &mut self.tags
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_card_has_unique_id() {
        let a = Card::new("Q", "A");
        let b = Card::new("Q", "A");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn new_card_stores_front_and_back() {
        let card = Card::new("What is Rust?", "A systems language");
        assert_eq!(card.front(), "What is Rust?");
        assert_eq!(card.back(), "A systems language");
    }

    #[test]
    fn front_mut_allows_modification() {
        let mut card = Card::new("Q", "A");
        card.front_mut().push_str(" updated");
        assert_eq!(card.front(), "Q updated");
    }

    #[test]
    fn back_mut_allows_modification() {
        let mut card = Card::new("Q", "A");
        card.back_mut().push_str(" updated");
        assert_eq!(card.back(), "A updated");
    }

    #[test]
    fn new_card_has_no_tags() {
        let card = Card::new("Q", "A");
        assert!(card.tags().is_empty());
    }

    #[test]
    fn tags_mut_allows_push() {
        let mut card = Card::new("Q", "A");
        card.tags_mut().push("rust".to_string());
        assert_eq!(card.tags(), &["rust"]);
    }

    #[test]
    fn has_tag_returns_true_for_existing_tag() {
        let mut card = Card::new("Q", "A");
        card.tags_mut().push("memory".to_string());
        assert!(card.has_tag("memory"));
        assert!(!card.has_tag("other"));
    }

    #[test]
    fn new_card_has_no_review_state() {
        let card = Card::new("Q", "A");
        assert!(card.review().is_none());
    }

    #[test]
    fn set_review_stores_state() {
        let mut card = Card::new("Q", "A");
        let state = ReviewState::default();
        card.set_review(state.clone());
        assert_eq!(card.review(), Some(&state));
    }

    #[test]
    fn new_card_is_not_hidden() {
        let card = Card::new("Q", "A");
        assert!(!card.is_hidden());
    }

    #[test]
    fn set_hidden_updates_state() {
        let mut card = Card::new("Q", "A");
        card.set_hidden(true);
        assert!(card.is_hidden());
        card.set_hidden(false);
        assert!(!card.is_hidden());
    }

    #[test]
    fn deserialize_missing_hidden_as_false() {
        let toml_str = r#"
            id = "00000000-0000-0000-0000-000000000001"
            front = "Q"
            back = "A"
        "#;
        let card: Card = toml::from_str(toml_str).expect("valid toml");
        assert!(!card.is_hidden());
    }

    #[test]
    fn hidden_roundtrips_through_toml() {
        let mut card = Card::new("Q", "A");
        card.set_hidden(true);
        let s = toml::to_string(&card).expect("serialize");
        let parsed: Card = toml::from_str(&s).expect("deserialize");
        assert!(parsed.is_hidden());
    }

    #[test]
    fn deserialize_missing_review_as_none() {
        let toml_str = r#"
            id = "00000000-0000-0000-0000-000000000001"
            front = "Q"
            back = "A"
        "#;
        let card: Card = toml::from_str(toml_str).expect("valid toml");
        assert!(card.review().is_none());
    }
}
