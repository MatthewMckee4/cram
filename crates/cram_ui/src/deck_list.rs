use std::collections::BTreeSet;

use chrono::Utc;
use cram_core::{Deck, sm2};
use cram_store::DeckSource;
use egui::{Context, Ui};

use crate::app::{StudyMode, View};
use crate::style;

pub struct DeckListView;

impl DeckListView {
    /// Returns `Some(deck_name)` when a deck deletion is confirmed.
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &[(&Deck, &DeckSource)],
        view: &mut View,
        new_deck_name: &mut String,
        confirm_delete: &mut Option<String>,
        study_tag_filter: &mut BTreeSet<String>,
    ) -> Option<String> {
        let mut deleted = None;

        if let Some(deck_name) = confirm_delete.clone() {
            let mut open = true;
            egui::Window::new("Delete Deck")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete \"{deck_name}\"? This cannot be undone."
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(style::destructive_button("Delete")).clicked() {
                            deleted = Some(deck_name);
                            *confirm_delete = None;
                        }
                        if ui.button("Cancel").clicked() {
                            *confirm_delete = None;
                        }
                    });
                });
            if !open {
                *confirm_delete = None;
            }
        }

        ui.vertical(|ui| {
            ui.add_space(style::SECTION_SPACING);
            ui.horizontal(|ui| {
                ui.heading("Your Decks");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(style::accent_button("New Deck")).clicked() {
                        *new_deck_name = String::new();
                        *view = View::NewDeck;
                    }
                });
            });
            ui.separator();
            ui.add_space(style::SECTION_SPACING);

            if decks.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(egui::RichText::new("📚").size(64.0));
                    ui.add_space(16.0);
                    ui.heading("No decks yet");
                    ui.add_space(8.0);
                    ui.label("Create your first deck to start studying!");
                    ui.add_space(16.0);
                    if ui.add(style::accent_button("＋ Create Deck")).clicked() {
                        *new_deck_name = String::new();
                        *view = View::NewDeck;
                    }
                });
                return;
            }

            egui::Grid::new("deck_grid")
                .num_columns(3)
                .spacing([20.0, 20.0])
                .show(ui, |ui| {
                    for (i, (deck, _source)) in decks.iter().enumerate() {
                        let total = deck.cards().len();
                        let all_tags = deck.all_tags();

                        style::card_frame(ui).show(ui, |ui| {
                            ui.set_min_width(240.0);
                            ui.vertical(|ui| {
                                ui.heading(deck.name());
                                if !deck.description().is_empty() {
                                    ui.label(
                                        egui::RichText::new(deck.description())
                                            .italics()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                                ui.label(format!("{total} cards"));

                                let due_count = due_card_count(deck);
                                if due_count > 0 {
                                    ui.label(
                                        egui::RichText::new(format!("{due_count} due"))
                                            .small()
                                            .color(style::ACCENT),
                                    );
                                }

                                if !all_tags.is_empty() {
                                    ui.add_space(4.0);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            egui::RichText::new("Filter tags:")
                                                .small()
                                                .color(ui.visuals().weak_text_color()),
                                        );
                                        for tag in &all_tags {
                                            let selected = study_tag_filter.contains(tag);
                                            let text = egui::RichText::new(tag).small();
                                            if ui.selectable_label(selected, text).clicked() {
                                                if selected {
                                                    study_tag_filter.remove(tag);
                                                } else {
                                                    study_tag_filter.insert(tag.clone());
                                                }
                                            }
                                        }
                                    });
                                    let filtered =
                                        deck.card_indices_matching_tags(study_tag_filter);
                                    if !study_tag_filter.is_empty() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}/{total} cards match",
                                                filtered.len()
                                            ))
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                        );
                                    }
                                }

                                ui.add_space(12.0);
                                ui.horizontal(|ui| {
                                    if ui.add(style::accent_button("Study")).clicked() {
                                        let indices =
                                            deck.card_indices_matching_tags(study_tag_filter);
                                        *view = View::Study {
                                            deck_name: deck.name().to_string(),
                                            card_index: 0,
                                            revealed: false,
                                            shuffled_indices: shuffled_indices_from(indices),
                                            study_mode: StudyMode::Random,
                                        };
                                        study_tag_filter.clear();
                                    }
                                    if ui
                                        .add(style::secondary_button("Spaced"))
                                        .on_hover_text(
                                            "Study cards due for review using spaced repetition",
                                        )
                                        .clicked()
                                    {
                                        let indices = due_indices(deck);
                                        if !indices.is_empty() {
                                            *view = View::Study {
                                                deck_name: deck.name().to_string(),
                                                card_index: 0,
                                                revealed: false,
                                                shuffled_indices: indices,
                                                study_mode: StudyMode::SpacedRepetition,
                                            };
                                        }
                                    }
                                    if ui.add(style::secondary_button("Edit")).clicked() {
                                        *view = View::Editor {
                                            deck_name: deck.name().to_string(),
                                            card_index: None,
                                        };
                                    }
                                    if ui.add(style::destructive_button("Delete")).clicked() {
                                        *confirm_delete = Some(deck.name().to_string());
                                    }
                                });
                            });
                        });

                        if (i + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        deleted
    }
}

/// Build a shuffled version of the provided card indices.
fn shuffled_indices_from(mut indices: Vec<usize>) -> Vec<usize> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::Instant::now().hash(&mut hasher);
    let mut seed = hasher.finish();
    for i in (1..indices.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        #[expect(clippy::cast_possible_truncation)]
        let j = (seed as usize) % (i + 1);
        indices.swap(i, j);
    }
    indices
}

/// Return indices of cards that are due for review today.
fn due_indices(deck: &Deck) -> Vec<usize> {
    let today = Utc::now().date_naive();
    deck.cards()
        .iter()
        .enumerate()
        .filter(|(_, card)| sm2::is_due(card.review(), today))
        .map(|(i, _)| i)
        .collect()
}

/// Count how many cards in a deck are due for review today.
fn due_card_count(deck: &Deck) -> usize {
    let today = Utc::now().date_naive();
    deck.cards()
        .iter()
        .filter(|card| sm2::is_due(card.review(), today))
        .count()
}
