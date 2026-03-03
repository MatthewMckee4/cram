use std::collections::BTreeSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::Card;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// Returns all unique tags across every card in the deck, sorted alphabetically.
    pub fn all_tags(&self) -> Vec<String> {
        let set: BTreeSet<&str> = self
            .cards
            .iter()
            .flat_map(|c| c.tags().iter().map(String::as_str))
            .collect();
        set.into_iter().map(String::from).collect()
    }

    /// Returns indices of cards that have at least one of the given tags.
    /// An empty filter set means "all cards match".
    pub fn card_indices_matching_tags(&self, tags: &BTreeSet<String>) -> Vec<usize> {
        if tags.is_empty() {
            return (0..self.cards.len()).collect();
        }
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, card)| card.tags().iter().any(|t| tags.contains(t)))
            .map(|(i, _)| i)
            .collect()
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

    #[test]
    fn all_tags_collects_unique_sorted() {
        let mut deck = Deck::new("Test", "");
        let mut c1 = Card::new("Q1", "A1");
        c1.tags_mut().push("rust".to_string());
        c1.tags_mut().push("memory".to_string());
        let mut c2 = Card::new("Q2", "A2");
        c2.tags_mut().push("rust".to_string());
        c2.tags_mut().push("async".to_string());
        deck.cards_mut().push(c1);
        deck.cards_mut().push(c2);
        assert_eq!(deck.all_tags(), vec!["async", "memory", "rust"]);
    }

    #[test]
    fn all_tags_empty_when_no_tags() {
        let mut deck = Deck::new("Test", "");
        deck.cards_mut().push(Card::new("Q", "A"));
        assert!(deck.all_tags().is_empty());
    }

    #[test]
    fn card_indices_matching_tags_returns_all_when_filter_empty() {
        let mut deck = Deck::new("Test", "");
        deck.cards_mut().push(Card::new("Q1", "A1"));
        deck.cards_mut().push(Card::new("Q2", "A2"));
        let indices = deck.card_indices_matching_tags(&BTreeSet::new());
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn card_indices_matching_tags_filters_correctly() {
        let mut deck = Deck::new("Test", "");
        let mut c1 = Card::new("Q1", "A1");
        c1.tags_mut().push("rust".to_string());
        let mut c2 = Card::new("Q2", "A2");
        c2.tags_mut().push("python".to_string());
        let mut c3 = Card::new("Q3", "A3");
        c3.tags_mut().push("rust".to_string());
        c3.tags_mut().push("async".to_string());
        deck.cards_mut().push(c1);
        deck.cards_mut().push(c2);
        deck.cards_mut().push(c3);

        let filter: BTreeSet<String> = ["rust".to_string()].into();
        assert_eq!(deck.card_indices_matching_tags(&filter), vec![0, 2]);

        let filter: BTreeSet<String> = ["python".to_string(), "async".to_string()].into();
        assert_eq!(deck.card_indices_matching_tags(&filter), vec![1, 2]);
    }
}
