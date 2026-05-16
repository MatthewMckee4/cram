use chrono::Utc;
use cram_core::{Deck, sm2};
use cram_store::DeckSource;
use egui::{Context, Ui};

use crate::app::View;
use crate::style;

pub enum DeckListAction {
    Import(std::path::PathBuf),
}

pub struct DeckListView;

impl DeckListView {
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &[(&Deck, &DeckSource)],
        view: &mut View,
        new_deck_name: &mut String,
    ) -> Option<DeckListAction> {
        let _ = ctx;
        let mut action = None;

        ui.vertical(|ui| {
            ui.add_space(style::SECTION_SPACING);
            ui.horizontal(|ui| {
                ui.heading("Your Decks");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().button_padding = egui::vec2(12.0, 6.0);
                    if ui
                        .selectable_label(false, egui::RichText::new("Import").size(15.0))
                        .clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Deck files", &["toml", "csv"])
                            .pick_file()
                    {
                        action = Some(DeckListAction::Import(path));
                    }
                    if ui
                        .selectable_label(false, egui::RichText::new("New Deck").size(15.0))
                        .clicked()
                    {
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
                    if ui
                        .add(crate::components::primary(ui, "＋ Create Deck"))
                        .clicked()
                    {
                        *new_deck_name = String::new();
                        *view = View::NewDeck;
                    }
                });
                return;
            }

            let spacing = 20.0;
            let min_card_width = 320.0;
            // card_frame applies inner_margin(CARD_MARGIN) on both sides, so the actual
            // column width is wider than the inner content min width.
            let min_column_width = min_card_width + 2.0 * style::CARD_MARGIN;
            let available_width = ui.available_width();
            let num_columns = ((available_width + spacing) / (min_column_width + spacing))
                .floor()
                .max(1.0) as usize;

            egui::Grid::new("deck_grid")
                .num_columns(num_columns)
                .spacing([spacing, spacing])
                .show(ui, |ui| {
                    for (i, (deck, _source)) in decks.iter().enumerate() {
                        let total = deck.cards().len();

                        let frame_resp = style::card_frame(ui).show(ui, |ui| {
                            ui.set_min_width(min_card_width);
                            ui.vertical(|ui| {
                                ui.heading(deck.name());
                                if !deck.description().is_empty() {
                                    ui.label(
                                        egui::RichText::new(deck.description())
                                            .italics()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                                ui.add_space(4.0);
                                ui.label(format!("{total} cards"));
                            });
                        });

                        let card_interact = ui
                            .interact(
                                frame_resp.response.rect,
                                egui::Id::new(("deck_card", i)),
                                egui::Sense::click(),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        if card_interact.clicked() {
                            *view = View::DeckDetail {
                                deck_name: deck.name().to_string(),
                            };
                        }

                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        action
    }
}

/// Build a shuffled version of the provided card indices.
pub(crate) fn shuffled_indices_from(mut indices: Vec<usize>) -> Vec<usize> {
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
/// Hidden cards are excluded.
pub(crate) fn due_indices(deck: &Deck) -> Vec<usize> {
    let today = Utc::now().date_naive();
    deck.cards()
        .iter()
        .enumerate()
        .filter(|(_, card)| !card.is_hidden() && sm2::is_due(card.review(), today))
        .map(|(i, _)| i)
        .collect()
}
