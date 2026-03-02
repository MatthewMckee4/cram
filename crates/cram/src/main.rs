mod commands;
mod settings;

use std::process::ExitCode;

use clap::Parser;
use cram_cli::{Cli, Command, DecksCommand, SelfCommand};
use owo_colors::OwoColorize;
use settings::GlobalSettings;

#[derive(Copy, Clone)]
enum ExitStatus {
    Success,
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => Self::from(0),
            ExitStatus::Error => Self::from(2),
        }
    }
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()))
        .init();

    let cli = Cli::parse();
    let settings = GlobalSettings::resolve(&cli.top_level.global_args);

    match run(cli, &settings) {
        Ok(status) => status.into(),
        Err(err) => {
            #[expect(clippy::print_stderr)]
            {
                let mut causes = err.chain();
                if let Some(first) = causes.next() {
                    eprintln!("{}: {}", "error".red().bold(), first.to_string().trim());
                }
                for cause in causes {
                    eprintln!(
                        "  {}: {}",
                        "Caused by".red().bold(),
                        cause.to_string().trim()
                    );
                }
            }
            ExitStatus::Error.into()
        }
    }
}

fn run(cli: Cli, settings: &GlobalSettings) -> anyhow::Result<ExitStatus> {
    match cli.command {
        Some(Command::Decks { command }) => match command {
            DecksCommand::List => commands::decks::list(settings)?,
            DecksCommand::Dir => commands::decks::dir(settings)?,
        },
        Some(Command::Self_ { command }) => match command {
            SelfCommand::Update { token, prerelease } => {
                commands::self_update(token, prerelease)?;
            }
        },
        None => commands::launch_gui(),
    }

    Ok(ExitStatus::Success)
}
