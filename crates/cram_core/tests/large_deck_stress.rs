/// Stress tests for core deck and scheduling operations with large card counts.
///
/// These tests are marked `#[ignore]` because they are too slow for normal CI.
/// Run them explicitly with:
///
/// ```sh
/// cargo test --package cram_core -- --ignored
/// ```
use std::time::Instant;

use chrono::NaiveDate;
use cram_core::sm2::{Rating, is_due, schedule};
use cram_core::{Card, Deck};

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date")
}

fn make_deck(card_count: usize) -> Deck {
    let mut deck = Deck::new("stress", "");
    for i in 0..card_count {
        deck.cards_mut()
            .push(Card::new(format!("Q{i}"), format!("A{i}")));
    }
    deck
}

#[test]
#[ignore]
fn schedule_5000_cards() {
    let deck = make_deck(5000);

    let start = Instant::now();
    let states: Vec<_> = deck
        .cards()
        .iter()
        .map(|c| schedule(c.review(), Rating::Good, today()))
        .collect();
    let elapsed = start.elapsed();

    assert_eq!(states.len(), 5000);
    assert!(
        elapsed.as_millis() < 100,
        "scheduling 5000 cards took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn filter_due_cards_in_large_deck() {
    let mut deck = make_deck(5000);

    // Schedule half the cards into the future so they are not due.
    let future = NaiveDate::from_ymd_opt(2026, 12, 31).expect("valid date");
    for (i, card) in deck.cards_mut().iter_mut().enumerate() {
        if i % 2 == 0 {
            *card.review_mut() = schedule(card.review(), Rating::Easy, future);
        }
    }

    let start = Instant::now();
    let due: Vec<_> = deck
        .cards()
        .iter()
        .filter(|c| is_due(c.review(), today()))
        .collect();
    let elapsed = start.elapsed();

    assert_eq!(due.len(), 2500);
    assert!(
        elapsed.as_millis() < 100,
        "filtering due cards in 5000 took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn create_large_deck_in_memory() {
    let start = Instant::now();
    let deck = make_deck(5000);
    let elapsed = start.elapsed();

    assert_eq!(deck.cards().len(), 5000);
    assert!(
        elapsed.as_millis() < 500,
        "creating 5000 cards took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn toml_serialization_roundtrip_5000_cards() {
    let deck = make_deck(5000);

    let start = Instant::now();
    let serialized = toml::to_string_pretty(&deck).expect("serialize");
    let ser_elapsed = start.elapsed();

    let start = Instant::now();
    let deserialized: Deck = toml::from_str(&serialized).expect("deserialize");
    let de_elapsed = start.elapsed();

    assert_eq!(deserialized.cards().len(), 5000);
    assert!(
        ser_elapsed.as_secs() < 5,
        "serializing 5000 cards took {ser_elapsed:?}"
    );
    assert!(
        de_elapsed.as_secs() < 5,
        "deserializing 5000 cards took {de_elapsed:?}"
    );
}
