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

    pub fn due_cards(&self) -> Vec<&Card> {
        let today = Utc::now().date_naive();
        self.cards.iter().filter(|c| c.due <= today).collect()
    }

    pub fn due_count(&self) -> usize {
        self.due_cards().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn due_cards_filters_correctly() {
        let mut deck = Deck::new("Test", "");
        let mut past = Card::new("Q", "A");
        past.due = NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid date");
        let mut future = Card::new("Q2", "A2");
        future.due = NaiveDate::from_ymd_opt(2099, 1, 1).expect("valid date");
        deck.cards.push(past);
        deck.cards.push(future);
        assert_eq!(deck.due_count(), 1);
    }

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

    #[test]
    fn due_count_on_empty_deck() {
        let deck = Deck::new("Empty", "");
        assert_eq!(deck.due_count(), 0);
    }

    #[test]
    fn all_new_cards_are_due() {
        let mut deck = Deck::new("Test", "");
        deck.cards.push(Card::new("Q1", "A1"));
        deck.cards.push(Card::new("Q2", "A2"));
        deck.cards.push(Card::new("Q3", "A3"));
        assert_eq!(deck.due_count(), 3);
    }
}
