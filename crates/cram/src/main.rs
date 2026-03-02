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
            if let Err(e) = list_decks() {
                eprintln!("cram: {e}");
                std::process::exit(1);
            }
        }
        Some(Command::Self_ { command }) => match command {
            SelfCommand::Update { token, prerelease } => {
                if let Err(e) = self_update(token, prerelease) {
                    eprintln!("cram: {e}");
                    std::process::exit(1);
                }
            }
        },
        None => launch_gui(),
    }
}

fn self_update(token: Option<String>, prerelease: bool) -> anyhow::Result<()> {
    let mut updater = axoupdater::AxoUpdater::new_for("cram");

    if let Some(ref token) = token {
        updater.set_github_token(token);
    }

    if prerelease {
        updater.configure_version_specifier(axoupdater::UpdateRequest::LatestMaybePrerelease);
    }

    if let Err(e) = updater.load_receipt() {
        if matches!(
            e,
            axoupdater::errors::AxoupdateError::NoReceipt { .. }
                | axoupdater::errors::AxoupdateError::ReceiptLoadFailed { .. }
        ) {
            anyhow::bail!(
                "cram was not installed via a standalone installer, \
                 so self-update is not available.\n\
                 Update cram via the method you used to install it."
            );
        }
        return Err(e.into());
    }

    updater
        .set_current_version(env!("CARGO_PKG_VERSION").parse()?)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !updater
        .check_receipt_is_for_this_executable()
        .unwrap_or(false)
    {
        let current_exe = std::env::current_exe()?;
        eprintln!(
            "warning: the install receipt does not match this executable ({}).\n\
             You may have multiple cram installations.",
            current_exe.display()
        );
    }

    match updater.run_sync() {
        Ok(Some(result)) => {
            let tag = &result.new_version_tag;
            println!(
                "Upgraded cram from {} to {}.\n\
                 Release notes: https://github.com/MatthewMckee4/cram/releases/tag/{tag}",
                env!("CARGO_PKG_VERSION"),
                result.new_version,
            );
        }
        Ok(None) => {
            println!(
                "cram is already up to date ({}).",
                env!("CARGO_PKG_VERSION")
            );
        }
        Err(e) => {
            if is_rate_limited(&e) {
                anyhow::bail!(
                    "GitHub API rate limit exceeded. \
                     Use `cram self update --token <GITHUB_TOKEN>` to authenticate."
                );
            }
            return Err(e.into());
        }
    }

    Ok(())
}

/// Check if the error is a GitHub API rate limit (HTTP 403).
fn is_rate_limited(err: &axoupdater::errors::AxoupdateError) -> bool {
    if let axoupdater::errors::AxoupdateError::Reqwest(reqwest_err) = err
        && let Some(status) = reqwest_err.status()
    {
        return status == 403;
    }
    false
}

fn list_decks() -> anyhow::Result<()> {
    let store = cram_store::Store::new()?;
    let decks = store.load_all_decks()?;
    if decks.is_empty() {
        println!("No decks found.");
        return Ok(());
    }
    for deck in &decks {
        println!(
            "{} ({} cards, {} due)",
            deck.name,
            deck.cards.len(),
            deck.due_count()
        );
    }
    Ok(())
}

fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cram")
            .with_inner_size([960.0, 680.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Cram",
        options,
        Box::new(|cc| Ok(Box::new(cram_ui::CramApp::new(cc)))),
    ) {
        eprintln!("cram: {e}");
        std::process::exit(1);
    }
}
