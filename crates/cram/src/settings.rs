use std::path::PathBuf;

use cram_cli::GlobalArgs;

pub(crate) struct GlobalSettings {
    pub(crate) decks_dir: Option<PathBuf>,
}

impl GlobalSettings {
    pub(crate) fn resolve(args: &GlobalArgs) -> Self {
        Self {
            decks_dir: args.decks_dir.clone(),
        }
    }
}
