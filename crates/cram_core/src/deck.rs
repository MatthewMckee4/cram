use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub name: String,
    pub description: String,
    pub created: chrono::NaiveDate,
    pub cards: Vec<Card>,
    #[serde(default)]
    pub preamble: String,
}

impl Deck {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            created: Utc::now().date_naive(),
            cards: Vec::new(),
            preamble: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deck_is_empty() {
        let deck = Deck::new("Empty", "no cards");
        assert!(deck.cards.is_empty());
    }

    #[test]
    fn new_deck_stores_name_and_description() {
        let deck = Deck::new("Rust Basics", "Learning Rust");
        assert_eq!(deck.name, "Rust Basics");
        assert_eq!(deck.description, "Learning Rust");
    }
}
