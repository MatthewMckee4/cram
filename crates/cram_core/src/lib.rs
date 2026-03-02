use chrono::{Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub front: String,
    pub back: String,
    pub due: NaiveDate,
    pub interval: f64,
    pub ease: f64,
    pub reps: u32,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Card {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            front: front.into(),
            back: back.into(),
            due: Utc::now().date_naive(),
            interval: 1.0,
            ease: 2.5,
            reps: 0,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub name: String,
    pub description: String,
    pub created: NaiveDate,
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            created: Utc::now().date_naive(),
            cards: Vec::new(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn label(self) -> &'static str {
        match self {
            Self::Again => "Again",
            Self::Hard => "Hard",
            Self::Good => "Good",
            Self::Easy => "Easy",
        }
    }
}

/// Apply SM-2 algorithm to a card based on the rating.
pub fn review_card(card: &mut Card, rating: Rating) {
    let today = Utc::now().date_naive();

    match rating {
        Rating::Again => {
            card.interval = 1.0;
            card.ease = (card.ease - 0.2).max(1.3);
            card.reps = 0;
        }
        Rating::Hard => {
            card.interval = (card.interval * 1.2).max(1.0);
            card.ease = (card.ease - 0.15).max(1.3);
            card.reps += 1;
        }
        Rating::Good => {
            card.interval = (card.interval * card.ease).max(1.0);
            card.reps += 1;
        }
        Rating::Easy => {
            card.interval = (card.interval * card.ease * 1.3).max(1.0);
            card.ease = (card.ease + 0.1).min(2.5);
            card.reps += 1;
        }
    }

    let days = card.interval.round() as u64;
    card.due = today.checked_add_days(Days::new(days)).unwrap_or(today);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_card() -> Card {
        Card {
            id: Uuid::new_v4(),
            front: "Q".into(),
            back: "A".into(),
            due: Utc::now().date_naive(),
            interval: 2.0,
            ease: 2.5,
            reps: 1,
            tags: Vec::new(),
        }
    }

    #[test]
    fn again_resets_interval() {
        let mut card = fresh_card();
        review_card(&mut card, Rating::Again);
        assert_eq!(card.interval, 1.0);
        assert_eq!(card.reps, 0);
    }

    #[test]
    fn again_reduces_ease() {
        let mut card = fresh_card();
        review_card(&mut card, Rating::Again);
        assert!((card.ease - 2.3).abs() < 0.001);
    }

    #[test]
    fn ease_floor_enforced() {
        let mut card = fresh_card();
        card.ease = 1.3;
        review_card(&mut card, Rating::Again);
        assert!((card.ease - 1.3).abs() < 0.001);
    }

    #[test]
    fn good_increases_interval() {
        let mut card = fresh_card();
        let old = card.interval;
        review_card(&mut card, Rating::Good);
        assert!(card.interval > old);
    }

    #[test]
    fn easy_increases_ease() {
        let mut card = fresh_card();
        card.ease = 2.0; // below ceiling so Easy can increase it
        let old_ease = card.ease;
        review_card(&mut card, Rating::Easy);
        assert!(card.ease > old_ease);
    }

    #[test]
    fn ease_ceiling_enforced() {
        let mut card = fresh_card();
        card.ease = 2.5;
        review_card(&mut card, Rating::Easy);
        assert!(card.ease <= 2.5);
    }

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

    #[test]
    fn new_card_defaults() {
        let card = Card::new("front", "back");
        assert_eq!(card.ease, 2.5);
        assert_eq!(card.interval, 1.0);
        assert_eq!(card.reps, 0);
        assert!(card.tags.is_empty());
    }
}
