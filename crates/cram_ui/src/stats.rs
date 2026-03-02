use cram_core::Deck;
use egui::Ui;

use crate::style;

pub struct StatsView;

impl StatsView {
    pub fn show(ui: &mut Ui, decks: &[&Deck]) {
        let total_decks = decks.len();
        let total_cards: usize = decks.iter().map(|d| d.cards().len()).sum();

        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading("Statistics");
            ui.separator();
            ui.add_space(16.0);

            ui.horizontal_wrapped(|ui| {
                stat_card(ui, "Total decks", &total_decks.to_string());
                stat_card(ui, "Total cards", &total_cards.to_string());
            });
        });
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str) {
    style::card_frame(ui).show(ui, |ui| {
        ui.set_min_width(100.0);
        ui.vertical_centered(|ui| {
            ui.label(label);
            ui.heading(value);
        });
    });
}
