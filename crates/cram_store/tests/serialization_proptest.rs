use cram_core::{Card, Deck};
use proptest::prelude::*;

fn arb_card() -> impl Strategy<Value = Card> {
    (".*", ".*").prop_map(|(front, back)| Card::new(front, back))
}

fn arb_deck() -> impl Strategy<Value = Deck> {
    (".*", ".*", prop::collection::vec(arb_card(), 0..10), ".*").prop_map(
        |(name, description, cards, preamble)| {
            let mut deck = Deck::new(name, description);
            for card in cards {
                deck.cards_mut().push(card);
            }
            *deck.preamble_mut() = preamble;
            deck
        },
    )
}

/// Compare all user-visible fields of two cards (ignoring the random id assigned on construction).
fn cards_equal(a: &Card, b: &Card) -> bool {
    a.front() == b.front() && a.back() == b.back() && a.tags() == b.tags()
}

fn decks_equal(a: &Deck, b: &Deck) -> bool {
    a.name() == b.name()
        && a.description() == b.description()
        && a.created() == b.created()
        && a.preamble() == b.preamble()
        && a.cards().len() == b.cards().len()
        && a.cards()
            .iter()
            .zip(b.cards().iter())
            .all(|(ca, cb)| cards_equal(ca, cb))
}

proptest! {
    #[test]
    fn card_toml_roundtrip(card in arb_card()) {
        let serialized = toml::to_string_pretty(&card)?;
        let deserialized: Card = toml::from_str(&serialized)?;
        prop_assert_eq!(card.front(), deserialized.front());
        prop_assert_eq!(card.back(), deserialized.back());
        prop_assert_eq!(card.id(), deserialized.id());
    }

    #[test]
    fn deck_toml_roundtrip(deck in arb_deck()) {
        let serialized = toml::to_string_pretty(&deck)?;
        let deserialized: Deck = toml::from_str(&serialized)?;
        prop_assert!(decks_equal(&deck, &deserialized), "deck roundtrip mismatch");
    }

    #[test]
    fn card_roundtrip_unicode(
        front in "[\\p{L}\\p{N}\\p{P}\\p{S}\\p{Z}]{0,100}",
        back in "[\\p{L}\\p{N}\\p{P}\\p{S}\\p{Z}]{0,100}",
    ) {
        let card = Card::new(front, back);
        let serialized = toml::to_string_pretty(&card)?;
        let deserialized: Card = toml::from_str(&serialized)?;
        prop_assert_eq!(card.front(), deserialized.front());
        prop_assert_eq!(card.back(), deserialized.back());
    }

    #[test]
    fn card_roundtrip_special_toml_chars(
        front in r#"[=\[\]"'\\#\n\t\r]{1,50}"#,
        back in r#"[=\[\]"'\\#\n\t\r]{1,50}"#,
    ) {
        let card = Card::new(front, back);
        let serialized = toml::to_string_pretty(&card)?;
        let deserialized: Card = toml::from_str(&serialized)?;
        prop_assert_eq!(card.front(), deserialized.front());
        prop_assert_eq!(card.back(), deserialized.back());
    }

    #[test]
    fn deck_roundtrip_empty_strings(
        cards in prop::collection::vec(arb_card(), 0..5),
    ) {
        let mut deck = Deck::new("", "");
        for card in &cards {
            deck.cards_mut().push(card.clone());
        }
        let serialized = toml::to_string_pretty(&deck)?;
        let deserialized: Deck = toml::from_str(&serialized)?;
        prop_assert!(decks_equal(&deck, &deserialized));
    }

    #[test]
    fn deck_roundtrip_many_cards(
        cards in prop::collection::vec(arb_card(), 20..50),
    ) {
        let mut deck = Deck::new("big-deck", "lots of cards");
        for card in &cards {
            deck.cards_mut().push(card.clone());
        }
        let serialized = toml::to_string_pretty(&deck)?;
        let deserialized: Deck = toml::from_str(&serialized)?;
        prop_assert!(decks_equal(&deck, &deserialized));
    }

    #[test]
    fn deck_roundtrip_empty(name in ".*", description in ".*") {
        let deck = Deck::new(name, description);
        let serialized = toml::to_string_pretty(&deck)?;
        let deserialized: Deck = toml::from_str(&serialized)?;
        prop_assert!(decks_equal(&deck, &deserialized));
    }
}
