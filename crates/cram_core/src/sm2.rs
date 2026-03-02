use chrono::{Days, Utc};

use crate::{Card, Rating};

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
    use uuid::Uuid;

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
        card.ease = 2.0;
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
}
