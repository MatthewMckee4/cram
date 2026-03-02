use crate::settings::GlobalSettings;

fn store(settings: &GlobalSettings) -> anyhow::Result<cram_store::Store> {
    match &settings.decks_dir {
        Some(dir) => cram_store::Store::with_dir(dir.clone()),
        None => cram_store::Store::new(),
    }
}

pub fn list(settings: &GlobalSettings) -> anyhow::Result<()> {
    let store = store(settings)?;
    let decks = store.load_all_decks()?;
    if decks.is_empty() {
        println!("No decks found.");
        return Ok(());
    }
    for deck in &decks {
        println!("{} ({} cards)", deck.name(), deck.cards().len());
    }
    Ok(())
}

pub fn dir(settings: &GlobalSettings) -> anyhow::Result<()> {
    let store = store(settings)?;
    println!("{}", store.data_dir().display());
    Ok(())
}
