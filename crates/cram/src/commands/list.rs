pub fn list_decks() -> anyhow::Result<()> {
    let store = cram_store::Store::new()?;
    let decks = store.load_all_decks()?;
    if decks.is_empty() {
        println!("No decks found.");
        return Ok(());
    }
    for deck in &decks {
        println!("{} ({} cards)", deck.name, deck.cards.len());
    }
    Ok(())
}
