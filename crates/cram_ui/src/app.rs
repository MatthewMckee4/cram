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
    DeckStats {
        deck_name: String,
    },
    SessionSummary {
        deck_name: String,
        cards_reviewed: u32,
        correct: u32,
        elapsed_secs: u64,
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
    selected_cards: std::collections::HashSet<usize>,
    session_start: Option<std::time::Instant>,
    session_reviewed: u32,
    session_correct: u32,
    undo_state: Option<UndoState>,
    preview_debounce: PreviewDebounce,
}

#[derive(Clone)]
pub struct UndoState {
    pub card_index: usize,
    pub interval: f64,
    pub ease: f64,
    pub reps: u32,
    pub due: chrono::NaiveDate,
}

#[derive(Default)]
pub struct PreviewDebounce {
    prev_frame_text: std::collections::HashMap<usize, String>,
    render_text: std::collections::HashMap<usize, String>,
    changed_at: std::collections::HashMap<usize, std::time::Instant>,
}

impl PreviewDebounce {
    /// Returns the source text to use for rendering the preview.
    /// Debounces updates: waits 300ms after the last keystroke before
    /// updating the render source to the new text.
    pub fn render_source(&mut self, index: usize, current: &str, ctx: &Context) -> String {
        let prev_frame = self.prev_frame_text.insert(index, current.to_string());
        let render = self
            .render_text
            .entry(index)
            .or_insert_with(|| current.to_string());

        if prev_frame.as_deref() != Some(current) {
            self.changed_at.insert(index, std::time::Instant::now());
        }

        if current != render.as_str() {
            if let Some(changed) = self.changed_at.get(&index) {
                if changed.elapsed() >= std::time::Duration::from_millis(300) {
                    *render = current.to_string();
                    self.changed_at.remove(&index);
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                }
            } else {
                *render = current.to_string();
            }
        }

        render.clone()
    }
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
            selected_cards: std::collections::HashSet::new(),
            session_start: None,
            session_reviewed: 0,
            session_correct: 0,
            undo_state: None,
            preview_debounce: PreviewDebounce::default(),
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

    fn show_deck_stats(ui: &mut egui::Ui, decks: &[Deck], deck_name: &str) {
        let Some(deck) = decks.iter().find(|d| d.name == deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading(format!("Statistics: {deck_name}"));
            ui.separator();
            ui.add_space(8.0);

            let total = deck.cards.len();
            let due = deck.due_count();
            let avg_ease = if total > 0 {
                #[expect(clippy::cast_precision_loss)]
                let avg = deck.cards.iter().map(|c| c.ease).sum::<f64>() / total as f64;
                avg
            } else {
                0.0
            };

            ui.label(format!("Total cards: {total}"));
            ui.label(format!("Due today: {due}"));
            ui.label(format!("Average ease: {avg_ease:.2}"));
            ui.add_space(12.0);

            ui.heading("Cards by interval");
            ui.add_space(4.0);

            let buckets = [
                ("New (0-1d)", 0.0_f64, 1.5),
                ("Learning (1-7d)", 1.5, 7.5),
                ("Young (7-21d)", 7.5, 21.5),
                ("Mature (21-90d)", 21.5, 90.5),
                ("Expert (90d+)", 90.5, f64::MAX),
            ];

            let max_count = buckets
                .iter()
                .map(|(_, lo, hi)| {
                    deck.cards
                        .iter()
                        .filter(|c| c.interval >= *lo && c.interval < *hi)
                        .count()
                })
                .max()
                .unwrap_or(1)
                .max(1);

            for (label, lo, hi) in buckets {
                let count = deck
                    .cards
                    .iter()
                    .filter(|c| c.interval >= lo && c.interval < hi)
                    .count();
                #[expect(clippy::cast_precision_loss)]
                let bar_frac = count as f32 / max_count as f32;
                ui.horizontal(|ui| {
                    ui.label(format!("{label:20}"));
                    let bar_width = (ui.available_width() - 60.0).max(50.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(bar_width, 16.0), egui::Sense::hover());
                    let fill_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width() * bar_frac, rect.height()),
                    );
                    ui.painter()
                        .rect_filled(fill_rect, 2.0, egui::Color32::from_rgb(50, 130, 220));
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, ui.visuals().text_color()),
                        egui::StrokeKind::Outside,
                    );
                    ui.label(count.to_string());
                });
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
                        &mut self.session_reviewed,
                        &mut self.session_correct,
                        &mut self.session_start,
                        &mut self.undo_state,
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
                        &mut self.selected_cards,
                        &mut self.preview_debounce,
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
                View::DeckStats { deck_name } => {
                    Self::show_deck_stats(ui, &self.decks, &deck_name);
                }
                View::SessionSummary {
                    deck_name,
                    cards_reviewed,
                    correct,
                    elapsed_secs,
                } => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.heading("Session Complete");
                        ui.add_space(16.0);
                        ui.label(format!("Deck: {deck_name}"));
                        ui.label(format!("Cards reviewed: {cards_reviewed}"));
                        let retention = if cards_reviewed > 0 {
                            correct as f64 / cards_reviewed as f64 * 100.0
                        } else {
                            0.0
                        };
                        ui.label(format!("Retention: {retention:.0}%"));
                        let mins = elapsed_secs / 60;
                        let secs = elapsed_secs % 60;
                        ui.label(format!("Time: {mins}m {secs}s"));
                        ui.add_space(16.0);
                        if ui.button("Back to Decks").clicked() {
                            self.view = View::DeckList;
                        }
                    });
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
