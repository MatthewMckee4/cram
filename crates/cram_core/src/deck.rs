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
        past.due = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let mut future = Card::new("Q2", "A2");
        future.due = NaiveDate::from_ymd_opt(2099, 1, 1).unwrap();
        deck.cards.push(past);
        deck.cards.push(future);
        assert_eq!(deck.due_count(), 1);
    }
}
