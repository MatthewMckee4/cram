use cram_core::{Card, Deck};
use cram_store::Store;

#[test]
fn render_card_front_produces_png() {
    let bytes = cram_render::render("= Question\nWhat is Rust?", false).expect("render");
    assert!(bytes.len() > 100);
    assert_eq!(&bytes[..4], b"\x89PNG");
}

#[test]
fn render_card_with_math() {
    let bytes = cram_render::render("Euler: $e^{i pi} + 1 = 0$", false).expect("render");
    assert_eq!(&bytes[..4], b"\x89PNG");
}

#[test]
fn render_with_preamble() {
    let preamble = "#set text(size: 16pt)";
    let body = "= Hello\nWorld";
    let source = format!("{preamble}\n{body}");
    let bytes = cram_render::render(&source, false).expect("render with preamble");
    assert_eq!(&bytes[..4], b"\x89PNG");
}

#[test]
fn render_invalid_typst_returns_error() {
    let result = cram_render::render("#nonexistent-func()", false);
    assert!(result.is_err());
}

#[test]
fn end_to_end_save_render_cycle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::with_dir(dir.path().to_path_buf()).expect("store");

    let mut deck = Deck::new("RenderTest", "");
    deck.cards_mut().push(Card::new(
        "= Ownership\nWhat are the rules?",
        "1. One owner\n2. Borrows don't outlive owner",
    ));
    *deck.preamble_mut() = "#set text(size: 12pt)".to_string();
    store.save_deck(&deck).expect("save");

    let loaded = store.load_deck("RenderTest").expect("load");
    let source = format!("{}\n{}", loaded.preamble(), loaded.cards()[0].front());
    let bytes = cram_render::render(&source, false).expect("render");
    assert_eq!(&bytes[..4], b"\x89PNG");
    assert!(bytes.len() > 100);
}

#[test]
fn render_dark_mode_produces_png() {
    let bytes =
        cram_render::render("= Dark Mode\nVisible on dark background", true).expect("dark render");
    assert!(bytes.len() > 100);
    assert_eq!(&bytes[..4], b"\x89PNG");
}
