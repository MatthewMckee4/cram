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
        None => launch_gui(),
    }
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
