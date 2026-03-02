use cram_core::Deck;
use egui::Ui;

use crate::style;

pub struct SearchView;

impl SearchView {
    pub fn show(ui: &mut Ui, decks: &[&Deck], query: &mut String) {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading("Search Cards");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Query:");
                ui.text_edit_singleline(query);
            });
            ui.add_space(12.0);

            if query.trim().is_empty() {
                ui.label("Type a search query to find cards across all decks.");
                return;
            }

            let lower_query = query.to_lowercase();
            let mut found = false;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for deck in decks {
                    let matches: Vec<_> = deck
                        .cards()
                        .iter()
                        .filter(|c| {
                            c.front().to_lowercase().contains(&lower_query)
                                || c.back().to_lowercase().contains(&lower_query)
                        })
                        .collect();

                    if matches.is_empty() {
                        continue;
                    }
                    found = true;

                    ui.heading(deck.name());
                    ui.separator();

                    for card in matches {
                        style::card_frame(ui).inner_margin(12.0).show(ui, |ui| {
                            ui.label(egui::RichText::new(card.front()).strong());
                            ui.label(card.back());
                        });
                        ui.add_space(4.0);
                    }
                    ui.add_space(8.0);
                }

                if !found {
                    ui.label("No matching cards found.");
                }
            });
        });
    }
}
