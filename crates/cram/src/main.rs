mod commands;

use clap::Parser;
use cram_cli::{Cli, Command, SelfCommand};

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
