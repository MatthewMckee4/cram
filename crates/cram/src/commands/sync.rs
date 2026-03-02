use crate::settings::GlobalSettings;
use cram_store::git::SyncResult;

pub fn sync(settings: &GlobalSettings) -> anyhow::Result<()> {
    let ms = super::decks::multi_store(settings)?;
    let results = ms.sync_all();

    if results.is_empty() {
        println!("No linked sources to sync.");
        return Ok(());
    }

    for (path, result) in &results {
        let display = path.display();
        match result {
            SyncResult::Pulled(msg) => println!("{display}: {msg}"),
            SyncResult::AlreadyUpToDate => println!("{display}: Already up to date."),
            SyncResult::NotAGitRepo => println!("{display}: Not a git repo, skipping."),
            SyncResult::Error(e) => println!("{display}: Error: {e}"),
        }
    }

    Ok(())
}
