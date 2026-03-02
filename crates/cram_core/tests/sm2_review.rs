use chrono::Utc;
use cram_core::{Card, Deck, Rating, review_card};

#[test]
fn full_review_cycle_updates_scheduling() {
    let mut deck = Deck::new("Test", "Integration test deck");
    deck.cards.push(Card::new("Front 1", "Back 1"));
    deck.cards.push(Card::new("Front 2", "Back 2"));

    let today = Utc::now().date_naive();

    // All new cards should be due today
    assert_eq!(deck.due_count(), 2);

    // Review first card as Good
    review_card(&mut deck.cards[0], Rating::Good);
    assert!(deck.cards[0].due > today);
    assert!(deck.cards[0].interval > 1.0);
    assert_eq!(deck.cards[0].reps, 1);

    // Review second card as Again — should stay due soon
    review_card(&mut deck.cards[1], Rating::Again);
    assert_eq!(deck.cards[1].interval, 1.0);
    assert_eq!(deck.cards[1].reps, 0);
}

#[test]
fn repeated_good_reviews_increase_interval() {
    let mut card = Card::new("Q", "A");

    let mut prev_interval = card.interval;
    for _ in 0..5 {
        review_card(&mut card, Rating::Good);
        assert!(card.interval >= prev_interval);
        prev_interval = card.interval;
    }

    // After 5 Good reviews, interval should be well above initial
    assert!(card.interval > 5.0);
    assert_eq!(card.reps, 5);
}

#[test]
fn again_after_many_reviews_resets_progress() {
    let mut card = Card::new("Q", "A");

    for _ in 0..5 {
        review_card(&mut card, Rating::Good);
    }
    assert!(card.interval > 5.0);

    review_card(&mut card, Rating::Again);
    assert_eq!(card.interval, 1.0);
    assert_eq!(card.reps, 0);
}

#[test]
fn ease_stays_within_bounds_under_stress() {
    let mut card = Card::new("Q", "A");

    // Hammer with Again to push ease to floor
    for _ in 0..20 {
        review_card(&mut card, Rating::Again);
    }
    assert!((card.ease - 1.3).abs() < 0.001);

    // Hammer with Easy to push ease to ceiling
    for _ in 0..20 {
        review_card(&mut card, Rating::Easy);
    }
    assert!((card.ease - 2.5).abs() < 0.001);
}

#[test]
fn mixed_ratings_produce_reasonable_scheduling() {
    let mut card = Card::new("Q", "A");

    review_card(&mut card, Rating::Good);
    review_card(&mut card, Rating::Hard);
    review_card(&mut card, Rating::Good);
    review_card(&mut card, Rating::Easy);

    assert!(card.interval > 1.0);
    assert!(card.ease >= 1.3);
    assert!(card.ease <= 2.5);
    assert_eq!(card.reps, 4);
}
