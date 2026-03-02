use cram_core::Deck;
use cram_store::Store;
use eframe::CreationContext;
use egui::Context;

use crate::{deck_list::DeckListView, editor::EditorView, stats::StatsView, study::StudyView};

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
}

pub struct CramApp {
    store: Store,
    decks: Vec<Deck>,
    view: View,
    new_deck_name: String,
    texture_cache: std::collections::HashMap<String, egui::TextureHandle>,
    error_message: Option<String>,
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
                    );
                    self.view = View::Study {
                        deck_name,
                        card_index,
                        revealed,
                    };
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
