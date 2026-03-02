use cram_core::{Card, Deck};
use cram_store::Store;

#[test]
fn save_load_preserves_all_card_fields() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");

    let mut deck = Deck::new("Roundtrip", "test description");
    deck.cards_mut().push(Card::new(
        "= Heading\nSome question?",
        "The answer is $x^2$",
    ));
    *deck.preamble_mut() = "#set text(size: 14pt)".to_string();

    store.save_deck(&deck).expect("save");

    let loaded = store.load_deck("Roundtrip").expect("load");
    assert_eq!(loaded.name(), "Roundtrip");
    assert_eq!(loaded.description(), "test description");
    assert_eq!(loaded.preamble(), "#set text(size: 14pt)");
    assert_eq!(loaded.cards().len(), 1);

    let c = &loaded.cards()[0];
    assert_eq!(c.front(), "= Heading\nSome question?");
    assert_eq!(c.back(), "The answer is $x^2$");
}

#[test]
fn multiple_decks_persist_independently() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");

    let mut deck_a = Deck::new("Alpha", "");
    deck_a.cards_mut().push(Card::new("Q1", "A1"));

    let mut deck_b = Deck::new("Beta", "");
    deck_b.cards_mut().push(Card::new("Q2", "A2"));
    deck_b.cards_mut().push(Card::new("Q3", "A3"));

    store.save_deck(&deck_a).expect("save A");
    store.save_deck(&deck_b).expect("save B");

    let all = store.load_all_decks().expect("load all");
    assert_eq!(all.len(), 2);

    let alpha = all.iter().find(|d| d.name() == "Alpha").expect("Alpha");
    let beta = all.iter().find(|d| d.name() == "Beta").expect("Beta");
    assert_eq!(alpha.cards().len(), 1);
    assert_eq!(beta.cards().len(), 2);
}

#[test]
fn delete_deck_removes_from_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");

    let deck = Deck::new("Ephemeral", "");
    store.save_deck(&deck).expect("save");

    assert!(store.load_deck("Ephemeral").is_ok());

    store.delete_deck("Ephemeral").expect("delete");

    assert!(store.load_deck("Ephemeral").is_err());
    assert!(store.load_all_decks().expect("load all").is_empty());
}

#[test]
fn overwrite_deck_updates_content() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");

    let mut deck = Deck::new("Mutable", "v1");
    deck.cards_mut().push(Card::new("Q", "A"));
    store.save_deck(&deck).expect("save v1");

    deck.set_description("v2");
    deck.cards_mut().push(Card::new("Q2", "A2"));
    store.save_deck(&deck).expect("save v2");

    let loaded = store.load_deck("Mutable").expect("load");
    assert_eq!(loaded.description(), "v2");
    assert_eq!(loaded.cards().len(), 2);
}
