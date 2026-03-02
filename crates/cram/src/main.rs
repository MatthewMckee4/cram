mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cram",
    version,
    about = "A flashcard app with Typst-powered card rendering"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
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
enum SelfCommand {
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::List) => {
            if let Err(e) = commands::list_decks() {
                eprintln!("cram: {e}");
                std::process::exit(1);
            }
        }
        Some(Command::Self_ { command }) => match command {
            SelfCommand::Update { token, prerelease } => {
                if let Err(e) = commands::self_update(token, prerelease) {
                    eprintln!("cram: {e}");
                    std::process::exit(1);
                }
            }
        },
        None => commands::launch_gui(),
    }
}
