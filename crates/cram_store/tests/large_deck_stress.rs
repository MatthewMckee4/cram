/// Stress tests for large decks.
///
/// These tests are marked `#[ignore]` because they are too slow for normal CI.
/// Run them explicitly with:
///
/// ```sh
/// cargo test --package cram_store -- --ignored
/// ```
use std::time::Instant;

use cram_core::{Card, Deck};
use cram_store::Store;

fn make_deck(name: &str, card_count: usize) -> Deck {
    let mut deck = Deck::new(name, format!("stress test deck with {card_count} cards"));
    for i in 0..card_count {
        deck.cards_mut()
            .push(Card::new(format!("Question #{i}"), format!("Answer #{i}")));
    }
    deck
}

fn temp_store() -> (Store, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");
    (store, dir)
}

#[test]
#[ignore]
fn save_and_load_500_cards() {
    let (store, _dir) = temp_store();
    let deck = make_deck("stress-500", 500);

    let start = Instant::now();
    store.save_deck(&deck).expect("save");
    let save_elapsed = start.elapsed();

    let start = Instant::now();
    let loaded = store.load_deck("stress-500").expect("load");
    let load_elapsed = start.elapsed();

    assert_eq!(loaded.cards().len(), 500);
    assert!(
        save_elapsed.as_secs() < 2,
        "saving 500 cards took {save_elapsed:?}"
    );
    assert!(
        load_elapsed.as_secs() < 2,
        "loading 500 cards took {load_elapsed:?}"
    );
}

#[test]
#[ignore]
fn save_and_load_1000_cards() {
    let (store, _dir) = temp_store();
    let deck = make_deck("stress-1000", 1000);

    let start = Instant::now();
    store.save_deck(&deck).expect("save");
    let save_elapsed = start.elapsed();

    let start = Instant::now();
    let loaded = store.load_deck("stress-1000").expect("load");
    let load_elapsed = start.elapsed();

    assert_eq!(loaded.cards().len(), 1000);
    assert!(
        save_elapsed.as_secs() < 3,
        "saving 1000 cards took {save_elapsed:?}"
    );
    assert!(
        load_elapsed.as_secs() < 3,
        "loading 1000 cards took {load_elapsed:?}"
    );
}

#[test]
#[ignore]
fn save_and_load_5000_cards() {
    let (store, _dir) = temp_store();
    let deck = make_deck("stress-5000", 5000);

    let start = Instant::now();
    store.save_deck(&deck).expect("save");
    let save_elapsed = start.elapsed();

    let start = Instant::now();
    let loaded = store.load_deck("stress-5000").expect("load");
    let load_elapsed = start.elapsed();

    assert_eq!(loaded.cards().len(), 5000);
    assert!(
        save_elapsed.as_secs() < 5,
        "saving 5000 cards took {save_elapsed:?}"
    );
    assert!(
        load_elapsed.as_secs() < 5,
        "loading 5000 cards took {load_elapsed:?}"
    );
}

#[test]
#[ignore]
fn search_performance_with_large_deck() {
    let (store, _dir) = temp_store();
    let mut deck = make_deck("search-stress", 5000);

    // Place a known card near the end so the search must traverse most of the deck.
    deck.cards_mut()
        .push(Card::new("Unique-Needle-XYZ", "Hidden answer"));
    store.save_deck(&deck).expect("save");

    let loaded = store.load_deck("search-stress").expect("load");
    let query = "unique-needle";

    let start = Instant::now();
    let lower_query = query.to_lowercase();
    let results: Vec<_> = loaded
        .cards()
        .iter()
        .filter(|c| {
            c.front().to_lowercase().contains(&lower_query)
                || c.back().to_lowercase().contains(&lower_query)
        })
        .collect();
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].front(), "Unique-Needle-XYZ");
    assert!(
        elapsed.as_millis() < 500,
        "searching 5000 cards took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn list_many_decks_performance() {
    let (store, _dir) = temp_store();
    for i in 0..100 {
        let mut deck = Deck::new(format!("deck-{i:03}"), "");
        deck.cards_mut()
            .push(Card::new(format!("Q{i}"), format!("A{i}")));
        store.save_deck(&deck).expect("save");
    }

    let start = Instant::now();
    let names = store.list_decks().expect("list");
    let elapsed = start.elapsed();

    assert_eq!(names.len(), 100);
    assert!(
        elapsed.as_millis() < 500,
        "listing 100 decks took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn load_all_many_decks_performance() {
    let (store, _dir) = temp_store();
    for i in 0..50 {
        let deck = make_deck(&format!("bulk-{i:03}"), 100);
        store.save_deck(&deck).expect("save");
    }

    let start = Instant::now();
    let all = store.load_all_decks().expect("load all");
    let elapsed = start.elapsed();

    assert_eq!(all.len(), 50);
    assert!(
        elapsed.as_secs() < 5,
        "loading 50 decks (100 cards each) took {elapsed:?}"
    );
}

#[test]
#[ignore]
fn overwrite_large_deck_preserves_cards() {
    let (store, _dir) = temp_store();
    let mut deck = make_deck("overwrite-stress", 2000);
    store.save_deck(&deck).expect("save v1");

    deck.set_description("updated description");
    deck.cards_mut().push(Card::new("New card", "New answer"));
    store.save_deck(&deck).expect("save v2");

    let loaded = store.load_deck("overwrite-stress").expect("load");
    assert_eq!(loaded.cards().len(), 2001);
    assert_eq!(loaded.description(), "updated description");
}
