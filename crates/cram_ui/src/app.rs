use std::path::PathBuf;

use cram_core::Deck;
use cram_store::{DeckSource, MultiStore, Store};

use crate::ui_state::UiState;
use eframe::CreationContext;
use egui::Context;

use crate::editor::{EditorContext, EditorView};
use crate::sources::{SourceStatus, SourcesView, SyncTask};
use crate::study::{StudyContext, StudyView};
use crate::style;
use crate::texture_cache::TextureCache;
use crate::theme::Theme;
use crate::{deck_list::DeckListView, search::SearchView};

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
    Search,
    Sources,
    SessionSummary {
        deck_name: String,
        cards_reviewed: u32,
        elapsed_secs: u64,
    },
}

pub struct CramApp {
    multi_store: MultiStore,
    decks: Vec<(Deck, DeckSource)>,
    view: View,
    new_deck_name: String,
    texture_cache: TextureCache,
    error_message: Option<String>,
    theme: Theme,
    search_query: String,
    session_start: Option<std::time::Instant>,
    session_reviewed: u32,
    preview_debounce: PreviewDebounce,
    fullscreen_preview: Option<String>,
    sync_statuses: Vec<SourceStatus>,
    sync_task: Option<SyncTask>,
    save_feedback: Option<std::time::Instant>,
    confirm_delete_deck: Option<String>,
    last_deck: Option<String>,
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

fn build_multi_store() -> MultiStore {
    let store = Store::from_env_or_default().unwrap_or_default();
    let config_dir = config_dir_for_store(&store);
    MultiStore::new(store, config_dir).unwrap_or_else(|e| {
        tracing::warn!("failed to build multi store: {e}, falling back to primary only");
        let fallback = Store::from_env_or_default().unwrap_or_default();
        let fallback_dir = config_dir_for_store(&fallback);
        // If MultiStore::new fails again we have bigger problems, but at least try
        MultiStore::new(fallback, fallback_dir).expect("fallback multi store")
    })
}

fn config_dir_for_store(store: &Store) -> PathBuf {
    store
        .data_dir()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| store.data_dir().to_path_buf())
}

impl CramApp {
    pub fn new(cc: &CreationContext) -> Self {
        let multi_store = build_multi_store();
        let decks = multi_store.load_all_decks().unwrap_or_default();

        let ui_state = UiState::load(multi_store.config_dir()).unwrap_or_default();
        let theme = ui_state.theme.unwrap_or_default();
        cc.egui_ctx.set_visuals(theme.visuals());

        let mut app = Self {
            multi_store,
            decks,
            view: View::DeckList,
            new_deck_name: String::new(),
            texture_cache: TextureCache::new(),
            error_message: None,
            theme,
            search_query: String::new(),
            session_start: None,
            session_reviewed: 0,
            preview_debounce: PreviewDebounce::default(),
            fullscreen_preview: None,
            sync_statuses: Vec::new(),
            sync_task: None,
            save_feedback: None,
            confirm_delete_deck: None,
            last_deck: ui_state.last_deck,
        };
        if app.decks.is_empty() {
            app.seed_sample_deck();
        }
        app
    }

    fn seed_sample_deck(&mut self) {
        use cram_core::Card;
        let mut deck = Deck::new("Rust Basics", "Core Rust concepts");
        deck.cards_mut().push(Card::new(
            "= Ownership\nWhat are the three ownership rules in Rust?",
            "1. Each value has a single *owner*\n2. Only one owner at a time\n3. Owner drops → value is dropped",
        ));
        deck.cards_mut().push(Card::new(
            "= Borrowing\nWhat is the difference between `&T` and `&mut T`?",
            "- `&T` — shared reference, many allowed simultaneously\n- `&mut T` — exclusive reference, only one at a time",
        ));
        deck.cards_mut().push(Card::new(
            "= Lifetimes\nWhat does a lifetime annotation like `'a` express?",
            "It constrains how long a reference is valid, ensuring references don't outlive the data they point to.",
        ));
        deck.cards_mut().push(Card::new(
            "= Traits\nHow do you implement a trait for a type?",
            "```rust\nimpl MyTrait for MyType {\n    fn method(&self) { ... }\n}\n```",
        ));
        deck.cards_mut().push(Card::new(
            "= Closures\nWhat makes Rust closures different from regular functions?",
            "Closures *capture* their environment. They implement `Fn`, `FnMut`, or `FnOnce` depending on how they use captured variables.",
        ));
        if let Err(e) = self.multi_store.save_deck(&deck, &DeckSource::Local) {
            tracing::warn!("failed to save sample deck: {e}");
        } else {
            self.decks.push((deck, DeckSource::Local));
        }
    }

    fn reload_decks(&mut self) {
        self.decks = self.multi_store.load_all_decks().unwrap_or_default();
    }

    /// Persist current UI state (theme, last deck) to `ui_state.toml`.
    fn save_ui_state(&self) {
        let state = UiState {
            theme: Some(self.theme),
            last_deck: self.last_deck.clone(),
        };
        if let Err(e) = state.save(self.multi_store.config_dir()) {
            tracing::warn!("failed to save UI state: {e}");
        }
    }
}

impl View {
    fn deck_name(&self) -> Option<&str> {
        match self {
            View::Study { deck_name, .. } | View::Editor { deck_name, .. } => Some(deck_name),
            _ => None,
        }
    }
}

impl eframe::App for CramApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let prev_deck = self.view.deck_name().map(str::to_string);
        egui::TopBottomPanel::top("topbar")
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Cram");
                    ui.separator();
                    let nav = [
                        ("Decks", View::DeckList),
                        ("Search", View::Search),
                        ("Sources", View::Sources),
                    ];
                    for (label, target) in nav {
                        let active =
                            std::mem::discriminant(&self.view) == std::mem::discriminant(&target);
                        let text = egui::RichText::new(label).size(15.0);
                        if ui.selectable_label(active, text).clicked() {
                            self.view = target;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let prev = self.theme;
                        egui::ComboBox::from_id_salt("theme_picker")
                            .selected_text(self.theme.name())
                            .show_ui(ui, |ui| {
                                for t in Theme::ALL {
                                    ui.selectable_value(&mut self.theme, t, t.name());
                                }
                            });
                        if self.theme != prev {
                            ctx.set_visuals(self.theme.visuals());
                            self.texture_cache.clear();
                            self.save_ui_state();
                        }
                    });
                });
            });

        if let Some(err) = &self.error_message.clone() {
            let bg = if self.theme.is_dark() {
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
            egui::Frame::new()
                .inner_margin(style::CONTENT_PADDING)
                .show(ui, |ui| {
                    let view = self.view.clone();
                    match view {
                        View::DeckList => {
                            let deck_refs: Vec<(&Deck, &DeckSource)> =
                                self.decks.iter().map(|(d, s)| (d, s)).collect();
                            if let Some(name) = DeckListView::show(
                                ui,
                                ctx,
                                &deck_refs,
                                &mut self.view,
                                &mut self.new_deck_name,
                                &mut self.confirm_delete_deck,
                            ) {
                                let _ = self.multi_store.delete_deck(&name);
                                self.reload_decks();
                            }
                        }
                        View::Study {
                            deck_name,
                            mut card_index,
                            mut revealed,
                            shuffled_indices,
                        } => {
                            let deck_only: Vec<&Deck> = self.decks.iter().map(|(d, _)| d).collect();
                            let mut sc = StudyContext {
                                decks: &deck_only,
                                deck_name: &deck_name,
                                card_index: &mut card_index,
                                revealed: &mut revealed,
                                texture_cache: &mut self.texture_cache,
                                view: &mut self.view,
                                session_reviewed: &mut self.session_reviewed,
                                session_start: &mut self.session_start,
                                shuffled_indices: &shuffled_indices,
                            };
                            StudyView::show(ui, ctx, &mut sc);
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
                            let source = self
                                .decks
                                .iter()
                                .find(|(d, _)| d.name() == deck_name)
                                .map(|(_, s)| s.clone())
                                .unwrap_or(DeckSource::Local);
                            let mut deck_only: Vec<Deck> =
                                self.decks.iter().map(|(d, _)| d.clone()).collect();
                            let mut ec = EditorContext {
                                decks: &mut deck_only,
                                deck_name: &deck_name,
                                card_index,
                                multi_store: &self.multi_store,
                                deck_source: &source,
                                texture_cache: &mut self.texture_cache,
                                preview_debounce: &mut self.preview_debounce,
                                fullscreen_preview: &mut self.fullscreen_preview,
                                save_feedback: &mut self.save_feedback,
                            };
                            EditorView::show(ui, ctx, &mut ec);
                            // Write modified decks back
                            for (i, (deck, _src)) in self.decks.iter_mut().enumerate() {
                                if i < deck_only.len() {
                                    *deck = deck_only[i].clone();
                                }
                            }
                            self.view = View::Editor {
                                deck_name,
                                card_index,
                            };
                        }
                        View::Search => {
                            let deck_only: Vec<&Deck> = self.decks.iter().map(|(d, _)| d).collect();
                            if let Some((deck_name, card_index)) =
                                SearchView::show(ui, &deck_only, &mut self.search_query)
                            {
                                self.view = View::Editor {
                                    deck_name,
                                    card_index: Some(card_index),
                                };
                            }
                        }
                        View::Sources => {
                            let prev_count = self.multi_store.sources().source.len();
                            let sync_completed = SourcesView::show(
                                ui,
                                ctx,
                                &mut self.multi_store,
                                &mut self.sync_statuses,
                                &mut self.sync_task,
                                &mut self.error_message,
                            );
                            let new_count = self.multi_store.sources().source.len();
                            if prev_count != new_count || sync_completed {
                                self.reload_decks();
                            }
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
                                        ui.add_space(style::SECTION_SPACING);
                                        ui.label(format!("Deck: {deck_name}"));
                                        ui.label(format!("Cards reviewed: {cards_reviewed}"));
                                        let mins = elapsed_secs / 60;
                                        let secs = elapsed_secs % 60;
                                        ui.label(format!("Time: {mins}m {secs}s"));
                                        ui.add_space(style::SECTION_SPACING);
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
                                        ui.add_space(style::SECTION_SPACING);
                                        ui.horizontal(|ui| {
                                            ui.label("Name:");
                                            ui.text_edit_singleline(&mut self.new_deck_name);
                                        });
                                        ui.add_space(style::ITEM_SPACING);
                                        ui.horizontal(|ui| {
                                            if ui.add(style::accent_button("Create")).clicked()
                                                && !self.new_deck_name.is_empty()
                                            {
                                                let deck = Deck::new(self.new_deck_name.trim(), "");
                                                if let Err(e) = self
                                                    .multi_store
                                                    .save_deck(&deck, &DeckSource::Local)
                                                {
                                                    self.error_message =
                                                        Some(format!("Failed to save: {e}"));
                                                } else {
                                                    self.decks.push((deck, DeckSource::Local));
                                                    self.view = View::DeckList;
                                                    self.new_deck_name.clear();
                                                }
                                            }
                                            if ui.add(style::secondary_button("Cancel")).clicked() {
                                                self.view = View::DeckList;
                                            }
                                        });
                                    });
                                });
                            });
                        }
                    }
                });
        });

        if self.fullscreen_preview.is_some() {
            self.show_fullscreen_preview(ctx);
        }

        let current_deck = self.view.deck_name().map(str::to_string);
        if current_deck != prev_deck {
            self.last_deck = current_deck.or(self.last_deck.take());
            self.save_ui_state();
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
                    match self
                        .texture_cache
                        .get_or_render(ctx, &key, &source, dark_mode)
                    {
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
