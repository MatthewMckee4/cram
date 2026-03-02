use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    name: String,
    description: String,
    created: chrono::NaiveDate,
    cards: Vec<Card>,
    #[serde(default)]
    preamble: String,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
    }

    pub fn created(&self) -> chrono::NaiveDate {
        self.created
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn cards_mut(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }

    pub fn preamble(&self) -> &str {
        &self.preamble
    }

    pub fn preamble_mut(&mut self) -> &mut String {
        &mut self.preamble
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deck_is_empty() {
        let deck = Deck::new("Empty", "no cards");
        assert!(deck.cards().is_empty());
    }

    #[test]
    fn new_deck_stores_name_and_description() {
        let deck = Deck::new("Rust Basics", "Learning Rust");
        assert_eq!(deck.name(), "Rust Basics");
        assert_eq!(deck.description(), "Learning Rust");
    }

    #[test]
    fn set_description_updates_value() {
        let mut deck = Deck::new("Test", "v1");
        deck.set_description("v2");
        assert_eq!(deck.description(), "v2");
    }

    #[test]
    fn cards_mut_allows_push() {
        let mut deck = Deck::new("Test", "");
        deck.cards_mut().push(Card::new("Q", "A"));
        assert_eq!(deck.cards().len(), 1);
    }

    #[test]
    fn preamble_mut_allows_modification() {
        let mut deck = Deck::new("Test", "");
        deck.preamble_mut().push_str("#set text(size: 14pt)");
        assert_eq!(deck.preamble(), "#set text(size: 14pt)");
    }
}
