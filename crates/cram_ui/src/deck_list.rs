use cram_core::Deck;
use cram_store::Store;
use egui::Ui;

use crate::app::View;

pub struct DeckListView;

impl DeckListView {
    pub fn show(
        ui: &mut Ui,
        decks: &[Deck],
        view: &mut View,
        new_deck_name: &mut String,
        _store: &Store,
    ) {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.heading("Your Decks");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("＋ New Deck").clicked() {
                        *new_deck_name = String::new();
                        *view = View::NewDeck;
                    }
                });
            });
            ui.separator();
            ui.add_space(12.0);

            if decks.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(egui::RichText::new("📚").size(64.0));
                    ui.add_space(16.0);
                    ui.heading("No decks yet");
                    ui.add_space(8.0);
                    ui.label("Create your first deck to start studying!");
                    ui.add_space(16.0);
                    if ui.button("＋ Create Deck").clicked() {
                        *new_deck_name = String::new();
                        *view = View::NewDeck;
                    }
                });
                return;
            }

            egui::Grid::new("deck_grid")
                .num_columns(3)
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (i, deck) in decks.iter().enumerate() {
                        let due = deck.due_count();
                        let total = deck.cards.len();

                        egui::Frame::new()
                            .fill(ui.visuals().extreme_bg_color)
                            .corner_radius(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(200.0);
                                ui.vertical(|ui| {
                                    ui.heading(&deck.name);
                                    if !deck.description.is_empty() {
                                        ui.label(
                                            egui::RichText::new(&deck.description)
                                                .italics()
                                                .color(ui.visuals().weak_text_color()),
                                        );
                                    }
                                    ui.label(format!("{total} cards"));
                                    if due > 0 {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 165, 0),
                                            format!("{due} due"),
                                        );
                                    } else {
                                        ui.colored_label(egui::Color32::GREEN, "Up to date ✓");
                                    }
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        if ui.button("Study").clicked() {
                                            *view = View::Study {
                                                deck_name: deck.name.clone(),
                                                card_index: 0,
                                                revealed: false,
                                            };
                                        }
                                        if ui.button("Edit").clicked() {
                                            *view = View::Editor {
                                                deck_name: deck.name.clone(),
                                                card_index: None,
                                            };
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
    }
}
