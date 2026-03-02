use cram_core::Deck;
use cram_store::Store;
use eframe::CreationContext;
use egui::Context;

use crate::style;
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
        shuffled_indices: Vec<usize>,
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
    SessionSummary {
        deck_name: String,
        cards_reviewed: u32,
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
    preview_debounce: PreviewDebounce,
    fullscreen_preview: Option<String>,
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
            preview_debounce: PreviewDebounce::default(),
            fullscreen_preview: None,
        };
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
        egui::TopBottomPanel::top("topbar")
            .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 6)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📚 Cram");
                    ui.separator();
                    let nav = [
                        ("Decks", View::DeckList),
                        ("Stats", View::Stats),
                        ("Search", View::Search),
                    ];
                    for (label, target) in nav {
                        let active =
                            std::mem::discriminant(&self.view) == std::mem::discriminant(&target);
                        let btn = if active {
                            egui::Button::new(
                                egui::RichText::new(label).color(egui::Color32::WHITE),
                            )
                            .fill(style::ACCENT)
                            .corner_radius(style::BUTTON_RADIUS)
                        } else {
                            egui::Button::new(label).corner_radius(style::BUTTON_RADIUS)
                        };
                        if ui.add(btn).clicked() {
                            self.view = target;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon = if self.dark_mode { "☀" } else { "🌙" };
                        if ui.button(icon).clicked() {
                            self.dark_mode = !self.dark_mode;
                            self.texture_cache.clear();
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
            let bg = if self.dark_mode {
                egui::Color32::from_rgb(80, 20, 20)
            } else {
                egui::Color32::from_rgb(254, 226, 226)
            };
            egui::TopBottomPanel::bottom("errors")
                .frame(egui::Frame::new().fill(bg).inner_margin(8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::RED, err);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕").clicked() {
                                self.error_message = None;
                            }
                        });
                    });
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
                    shuffled_indices,
                } => {
                    StudyView::show(
                        ui,
                        ctx,
                        &self.decks,
                        &deck_name,
                        &mut card_index,
                        &mut revealed,
                        &mut self.texture_cache,
                        &mut self.view,
                        &mut self.session_reviewed,
                        &mut self.session_start,
                        &shuffled_indices,
                    );
                    if matches!(self.view, View::Study { .. }) {
                        self.view = View::Study {
                            deck_name,
                            card_index,
                            revealed,
                            shuffled_indices,
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
                        &mut self.fullscreen_preview,
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
                View::SessionSummary {
                    deck_name,
                    cards_reviewed,
                    elapsed_secs,
                } => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        style::card_frame(ui).show(ui, |ui| {
                            ui.set_max_width(400.0);
                            ui.vertical_centered(|ui| {
                                ui.heading("Session Complete");
                                ui.add_space(16.0);
                                ui.label(format!("Deck: {deck_name}"));
                                ui.label(format!("Cards reviewed: {cards_reviewed}"));
                                let mins = elapsed_secs / 60;
                                let secs = elapsed_secs % 60;
                                ui.label(format!("Time: {mins}m {secs}s"));
                                ui.add_space(16.0);
                                if ui.add(style::accent_button("Back to Decks")).clicked() {
                                    self.view = View::DeckList;
                                }
                            });
                        });
                    });
                }
                View::NewDeck => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        style::card_frame(ui).show(ui, |ui| {
                            ui.set_max_width(400.0);
                            ui.vertical_centered(|ui| {
                                ui.heading("New Deck");
                                ui.add_space(20.0);
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.new_deck_name);
                                });
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    if ui.add(style::accent_button("Create")).clicked()
                                        && !self.new_deck_name.is_empty()
                                    {
                                        let deck = Deck::new(self.new_deck_name.trim(), "");
                                        if let Err(e) = self.store.save_deck(&deck) {
                                            self.error_message =
                                                Some(format!("Failed to save: {e}"));
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
                            });
                        });
                    });
                }
            }
        });

        if self.fullscreen_preview.is_some() {
            self.show_fullscreen_preview(ctx);
        }
    }
}

impl CramApp {
    fn show_fullscreen_preview(&mut self, ctx: &Context) {
        let source = match &self.fullscreen_preview {
            Some(s) => s.clone(),
            None => return,
        };

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.fullscreen_preview = None;
            return;
        }

        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("fullscreen_preview_bg"))
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                ui.painter()
                    .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(180));
            });

        egui::Window::new("Card Preview")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(screen.width() * 0.85, screen.height() * 0.85))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Full-Screen Preview");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close [Esc]").clicked() {
                            self.fullscreen_preview = None;
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::both().show(ui, |ui| {
                    let key = format!("fullscreen-{source}");
                    let dark_mode = ui.visuals().dark_mode;
                    match get_or_render(ctx, &key, &source, &mut self.texture_cache, dark_mode) {
                        Ok(tex) => {
                            ui.add(egui::Image::new(&tex).max_width(ui.available_width()));
                        }
                        Err(err) => {
                            ui.colored_label(egui::Color32::RED, format!("Render error: {err}"));
                        }
                    }
                });
            });
    }
}

fn get_or_render(
    ctx: &Context,
    key: &str,
    source: &str,
    cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
    dark_mode: bool,
) -> Result<egui::TextureHandle, String> {
    if let Some(h) = cache.get(key) {
        return Ok(h.clone());
    }
    let png = cram_render::render(source, dark_mode).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&png).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let handle = ctx.load_texture(key, ci, egui::TextureOptions::LINEAR);
    cache.insert(key.to_string(), handle.clone());
    Ok(handle)
}
