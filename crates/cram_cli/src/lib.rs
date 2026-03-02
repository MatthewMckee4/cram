use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cram",
    version,
    about = "A flashcard app with Typst-powered card rendering"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[command(flatten)]
    pub top_level: TopLevelArgs,
}

#[derive(Args)]
pub struct TopLevelArgs {
    #[command(flatten)]
    pub global_args: Box<GlobalArgs>,
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Override the directory where decks are stored
    #[arg(long, global = true, env = cram_static::EnvVars::DECKS_DIR)]
    pub decks_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage decks
    Decks {
        #[command(subcommand)]
        command: DecksCommand,
    },
    /// Manage the cram installation
    #[command(name = "self")]
    Self_ {
        #[command(subcommand)]
        command: SelfCommand,
    },
}

#[derive(Subcommand)]
pub enum DecksCommand {
    /// List all decks
    List,
    /// Print the decks directory path
    Dir,
}

#[derive(Subcommand)]
pub enum SelfCommand {
    /// Update cram to the latest version
    Update {
        /// GitHub API token for authentication (avoids rate limits)
        #[arg(long)]
        token: Option<String>,
        /// Include pre-release versions (e.g. alpha, beta, rc)
        #[arg(long)]
        prerelease: bool,
    },
}
