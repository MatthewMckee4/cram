use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cram",
    version,
    about = "A flashcard app with Typst-powered card rendering"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all decks
    List,
    /// Manage the cram installation
    #[command(name = "self")]
    Self_ {
        #[command(subcommand)]
        command: SelfCommand,
    },
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
