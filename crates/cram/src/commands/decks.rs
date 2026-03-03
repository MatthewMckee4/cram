use std::path::PathBuf;

use crate::settings::GlobalSettings;
use cram_core::Deck;
use cram_store::{MultiStore, SourceKind, Store, exchange};

fn store(settings: &GlobalSettings) -> anyhow::Result<Store> {
    match &settings.decks_dir {
        Some(dir) => Store::with_dir(dir.clone()),
        None => Store::new(),
    }
}

fn config_dir(settings: &GlobalSettings) -> anyhow::Result<PathBuf> {
    match &settings.decks_dir {
        Some(dir) => {
            let parent = dir
                .parent()
                .ok_or_else(|| anyhow::anyhow!("cannot determine config dir"))?;
            Ok(parent.to_path_buf())
        }
        None => {
            let dir = dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("cannot find data directory"))?
                .join("cram");
            Ok(dir)
        }
    }
}

pub fn multi_store(settings: &GlobalSettings) -> anyhow::Result<MultiStore> {
    let primary = store(settings)?;
    let cfg_dir = config_dir(settings)?;
    MultiStore::new(primary, cfg_dir)
}

pub fn list(settings: &GlobalSettings) -> anyhow::Result<()> {
    let ms = multi_store(settings)?;
    let decks = ms.load_all_decks()?;
    if decks.is_empty() {
        println!("No decks found.");
        return Ok(());
    }
    for (deck, source) in &decks {
        let suffix = match source {
            cram_store::DeckSource::Local => String::new(),
            cram_store::DeckSource::Linked(p) => format!(" [{}]", p.display()),
        };
        println!("{} ({} cards){suffix}", deck.name(), deck.cards().len());
    }
    Ok(())
}

pub fn dir(settings: &GlobalSettings) -> anyhow::Result<()> {
    let store = store(settings)?;
    println!("{}", store.data_dir().display());
    Ok(())
}

pub fn link(settings: &GlobalSettings, path: PathBuf) -> anyhow::Result<()> {
    let path = std::fs::canonicalize(&path)
        .map_err(|_| anyhow::anyhow!("path does not exist: {}", path.display()))?;
    let kind = if path.is_file() {
        SourceKind::File
    } else {
        SourceKind::Folder
    };
    let mut ms = multi_store(settings)?;
    if ms.link(path.clone(), kind)? {
        println!("Linked: {}", path.display());
    } else {
        println!("Already linked: {}", path.display());
    }
    Ok(())
}

pub fn unlink(settings: &GlobalSettings, path: PathBuf) -> anyhow::Result<()> {
    let path = if path.exists() {
        std::fs::canonicalize(&path)?
    } else {
        path
    };
    let mut ms = multi_store(settings)?;
    if ms.unlink(&path)? {
        println!("Unlinked: {}", path.display());
    } else {
        println!("Not linked: {}", path.display());
    }
    Ok(())
}

pub fn new(path: PathBuf) -> anyhow::Result<()> {
    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("toml") {
        anyhow::bail!("deck file must have a .toml extension");
    }
    if path.exists() {
        anyhow::bail!("file already exists: {}", path.display());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid file name"))?;
    let deck = Deck::new(name, "");
    let content = toml::to_string_pretty(&deck)?;
    std::fs::write(&path, content)?;
    println!("Created deck: {}", path.display());
    Ok(())
}

pub fn export(settings: &GlobalSettings, name: String, path: PathBuf) -> anyhow::Result<()> {
    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("toml") {
        anyhow::bail!("export file must have a .toml extension");
    }
    let ms = multi_store(settings)?;
    let decks = ms.load_all_decks()?;
    let deck = decks.iter().find(|(d, _)| d.name() == name).map(|(d, _)| d);
    if let Some(deck) = deck {
        exchange::export_toml(deck, &path)?;
        println!("Exported \"{}\" to {}", name, path.display());
    } else {
        anyhow::bail!("deck not found: {name}");
    }
    Ok(())
}

pub fn import(settings: &GlobalSettings, path: PathBuf) -> anyhow::Result<()> {
    if !path.is_file() {
        anyhow::bail!("file not found: {}", path.display());
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let deck = match ext {
        "toml" => exchange::import_toml(&path)?,
        "csv" => exchange::import_csv(&path)?,
        other => anyhow::bail!("unsupported file format: .{other} (expected .toml or .csv)"),
    };
    let ms = multi_store(settings)?;
    ms.primary().save_deck(&deck)?;
    println!(
        "Imported \"{}\" ({} cards)",
        deck.name(),
        deck.cards().len()
    );
    Ok(())
}

pub fn sources(settings: &GlobalSettings) -> anyhow::Result<()> {
    let ms = multi_store(settings)?;
    let srcs = ms.sources();
    if srcs.source.is_empty() {
        println!("No linked sources.");
        return Ok(());
    }
    for src in &srcs.source {
        let kind_label = match src.kind {
            SourceKind::File => "file",
            SourceKind::Folder => "folder",
        };
        let search_path = match src.kind {
            SourceKind::Folder => src.path.clone(),
            SourceKind::File => src
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| src.path.clone()),
        };
        let git_status = match cram_store::git::find_git_root(&search_path) {
            Some(root) if root == search_path => "git repo".to_string(),
            Some(root) => format!("git repo at {}", root.display()),
            None => "not a git repo".to_string(),
        };
        println!("{} ({}, {})", src.path.display(), kind_label, git_status);
    }
    Ok(())
}
