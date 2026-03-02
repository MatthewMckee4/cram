use cram_core::Deck;
use cram_store::Store;
use eframe::CreationContext;
use egui::Context;

use crate::{
    deck_list::DeckListView, editor::EditorView, search::SearchView, stats::StatsView,
    study::StudyView,
};

#[derive(Default, Clone)]
pub enum View {
    #[default]
    DeckList,
    Study {
        deck_name: String,
        card_index: usize,
        revealed: bool,
    },
    Editor {
        deck_name: String,
        card_index: Option<usize>,
    },
    NewDeck,
    Stats,
    Search,
    ImportCsv {
        deck_name: String,
    },
    ExportCsv {
        deck_name: String,
    },
}

pub struct CramApp {
    store: Store,
    decks: Vec<Deck>,
    view: View,
    new_deck_name: String,
    texture_cache: std::collections::HashMap<String, egui::TextureHandle>,
    error_message: Option<String>,
    dark_mode: bool,
    search_query: String,
    csv_buffer: String,
}

impl CramApp {
    pub fn new(_cc: &CreationContext) -> Self {
        let store = Store::new().unwrap_or_default();
        let decks = store.load_all_decks().unwrap_or_default();
        let mut app = Self {
            store,
            decks,
            view: View::DeckList,
            new_deck_name: String::new(),
            texture_cache: std::collections::HashMap::new(),
            error_message: None,
            dark_mode: true,
            search_query: String::new(),
            csv_buffer: String::new(),
        };
        // Seed sample deck if nothing exists
        if app.decks.is_empty() {
            app.seed_sample_deck();
        }
        app
    }

    fn seed_sample_deck(&mut self) {
        use cram_core::Card;
        let mut deck = Deck::new("Rust Basics", "Core Rust concepts");
        deck.cards.push(Card::new(
            "= Ownership\nWhat are the three ownership rules in Rust?",
            "1. Each value has a single *owner*\n2. Only one owner at a time\n3. Owner drops → value is dropped",
        ));
        deck.cards.push(Card::new(
            "= Borrowing\nWhat is the difference between `&T` and `&mut T`?",
            "- `&T` — shared reference, many allowed simultaneously\n- `&mut T` — exclusive reference, only one at a time",
        ));
        deck.cards.push(Card::new(
            "= Lifetimes\nWhat does a lifetime annotation like `'a` express?",
            "It constrains how long a reference is valid, ensuring references don't outlive the data they point to.",
        ));
        deck.cards.push(Card::new(
            "= Traits\nHow do you implement a trait for a type?",
            "```rust\nimpl MyTrait for MyType {\n    fn method(&self) { ... }\n}\n```",
        ));
        deck.cards.push(Card::new(
            "= Closures\nWhat makes Rust closures different from regular functions?",
            "Closures *capture* their environment. They implement `Fn`, `FnMut`, or `FnOnce` depending on how they use captured variables.",
        ));
        if let Err(e) = self.store.save_deck(&deck) {
            tracing::warn!("failed to save sample deck: {e}");
        } else {
            self.decks.push(deck);
        }
    }
}

impl CramApp {
    fn show_import_csv(
        ui: &mut egui::Ui,
        decks: &mut [Deck],
        deck_name: &str,
        csv_buffer: &mut String,
        store: &Store,
        view: &mut View,
        error_message: &mut Option<String>,
    ) {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading(format!("Import CSV into: {deck_name}"));
            ui.add_space(8.0);
            ui.label("Paste CSV content below (one card per line: front,back):");
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::multiline(csv_buffer)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(12)
                    .desired_width(ui.available_width()),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Import").clicked()
                    && let Some(deck) = decks.iter_mut().find(|d| d.name == deck_name)
                {
                    let mut count = 0u32;
                    for line in csv_buffer.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        if let Some((front, back)) = line.split_once(',') {
                            let front = front.trim();
                            let back = back.trim();
                            if !front.is_empty() {
                                deck.cards.push(cram_core::Card::new(front, back));
                                count += 1;
                            }
                        }
                    }
                    if let Err(e) = store.save_deck(deck) {
                        *error_message = Some(format!("Save failed: {e}"));
                    }
                    csv_buffer.clear();
                    *view = View::Editor {
                        deck_name: deck_name.to_string(),
                        card_index: None,
                    };
                    tracing::info!("imported {count} cards into {deck_name}");
                }
                if ui.button("Cancel").clicked() {
                    csv_buffer.clear();
                    *view = View::DeckList;
                }
            });
        });
    }

    fn show_export_csv(
        ui: &mut egui::Ui,
        decks: &[Deck],
        deck_name: &str,
        csv_buffer: &mut String,
    ) {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading(format!("Export CSV: {deck_name}"));
            ui.add_space(8.0);

            if let Some(deck) = decks.iter().find(|d| d.name == deck_name) {
                if csv_buffer.is_empty() {
                    for card in &deck.cards {
                        let front = card.front.replace(',', ";");
                        let back = card.back.replace(',', ";");
                        csv_buffer.push_str(&format!("{front},{back}\n"));
                    }
                }
                ui.label("Copy the CSV content below:");
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::multiline(csv_buffer)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(12)
                        .desired_width(ui.available_width()),
                );
            } else {
                ui.label("Deck not found.");
            }
        });
    }
}

impl eframe::App for CramApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📚 Cram");
                ui.separator();
                if ui.button("Decks").clicked() {
                    self.view = View::DeckList;
                }
                if ui.button("Stats").clicked() {
                    self.view = View::Stats;
                }
                if ui.button("Search").clicked() {
                    self.view = View::Search;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if self.dark_mode { "Light" } else { "Dark" };
                    if ui.button(label).clicked() {
                        self.dark_mode = !self.dark_mode;
                        if self.dark_mode {
                            ctx.set_visuals(egui::Visuals::dark());
                        } else {
                            ctx.set_visuals(egui::Visuals::light());
                        }
                    }
                });
            });
        });

        if let Some(err) = &self.error_message.clone() {
            egui::TopBottomPanel::bottom("errors").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, err);
                if ui.button("✕").clicked() {
                    self.error_message = None;
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let view = self.view.clone();
            match view {
                View::DeckList => {
                    DeckListView::show(
                        ui,
                        &self.decks,
                        &mut self.view,
                        &mut self.new_deck_name,
                        &self.store,
                    );
                }
                View::Study {
                    deck_name,
                    mut card_index,
                    mut revealed,
                } => {
                    StudyView::show(
                        ui,
                        ctx,
                        &mut self.decks,
                        &deck_name,
                        &mut card_index,
                        &mut revealed,
                        &self.store,
                        &mut self.texture_cache,
                        &mut self.view,
                    );
                    if matches!(self.view, View::Study { .. }) {
                        self.view = View::Study {
                            deck_name,
                            card_index,
                            revealed,
                        };
                    }
                }
                View::Editor {
                    deck_name,
                    card_index,
                } => {
                    EditorView::show(
                        ui,
                        ctx,
                        &mut self.decks,
                        &deck_name,
                        card_index,
                        &self.store,
                        &mut self.texture_cache,
                    );
                    self.view = View::Editor {
                        deck_name,
                        card_index,
                    };
                }
                View::Stats => {
                    StatsView::show(ui, &self.decks);
                }
                View::Search => {
                    SearchView::show(ui, &self.decks, &mut self.search_query);
                }
                View::ImportCsv { deck_name } => {
                    Self::show_import_csv(
                        ui,
                        &mut self.decks,
                        &deck_name,
                        &mut self.csv_buffer,
                        &self.store,
                        &mut self.view,
                        &mut self.error_message,
                    );
                }
                View::ExportCsv { deck_name } => {
                    Self::show_export_csv(ui, &self.decks, &deck_name, &mut self.csv_buffer);
                }
                View::NewDeck => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.heading("New Deck");
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_deck_name);
                        });
                        ui.add_space(10.0);
                        if ui.button("Create").clicked() && !self.new_deck_name.is_empty() {
                            let deck = Deck::new(self.new_deck_name.trim(), "");
                            if let Err(e) = self.store.save_deck(&deck) {
                                self.error_message = Some(format!("Failed to save: {e}"));
                            } else {
                                self.decks.push(deck);
                                self.view = View::DeckList;
                                self.new_deck_name.clear();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.view = View::DeckList;
                        }
                    });
                }
            }
        });
    }
}
