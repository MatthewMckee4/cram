use cram_store::StudyStats;
use egui::Ui;

use crate::style;

pub struct StatsView;

impl StatsView {
    pub fn show(ui: &mut Ui, stats: &StudyStats) {
        ui.vertical(|ui| {
            ui.add_space(style::SECTION_SPACING);
            ui.heading("Study Statistics");
            ui.separator();
            ui.add_space(style::SECTION_SPACING);

            if stats.total_sessions() == 0 {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("No study sessions yet");
                    ui.add_space(8.0);
                    ui.label("Complete a study session to see your stats here.");
                });
                return;
            }

            show_overall_summary(ui, stats);
            ui.add_space(style::SECTION_SPACING);
            show_per_deck_table(ui, stats);
            ui.add_space(style::SECTION_SPACING);
            show_recent_sessions(ui, stats);
        });
    }
}

fn show_overall_summary(ui: &mut Ui, stats: &StudyStats) {
    style::card_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width() - 32.0);
        ui.heading("Overall");
        ui.add_space(style::ITEM_SPACING);

        egui::Grid::new("overall_stats")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Total sessions:");
                ui.label(egui::RichText::new(stats.total_sessions().to_string()).strong());
                ui.end_row();

                ui.label("Cards reviewed:");
                ui.label(egui::RichText::new(stats.total_cards_reviewed().to_string()).strong());
                ui.end_row();

                ui.label("Time spent:");
                ui.label(egui::RichText::new(format_duration(stats.total_time_secs())).strong());
                ui.end_row();
            });
    });
}

fn show_per_deck_table(ui: &mut Ui, stats: &StudyStats) {
    let summaries = stats.per_deck_summary();
    if summaries.is_empty() {
        return;
    }

    style::card_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width() - 32.0);
        ui.heading("Per Deck");
        ui.add_space(style::ITEM_SPACING);

        egui::Grid::new("per_deck_stats")
            .num_columns(4)
            .spacing([24.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Deck").strong());
                ui.label(egui::RichText::new("Sessions").strong());
                ui.label(egui::RichText::new("Cards").strong());
                ui.label(egui::RichText::new("Time").strong());
                ui.end_row();

                for summary in &summaries {
                    ui.label(&summary.deck_name);
                    ui.label(summary.sessions.to_string());
                    ui.label(summary.cards_reviewed.to_string());
                    ui.label(format_duration(summary.total_secs));
                    ui.end_row();
                }
            });
    });
}

fn show_recent_sessions(ui: &mut Ui, stats: &StudyStats) {
    let recent = stats.recent_sessions(10);
    if recent.is_empty() {
        return;
    }

    style::card_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width() - 32.0);
        ui.heading("Recent Sessions");
        ui.add_space(style::ITEM_SPACING);

        egui::Grid::new("recent_sessions")
            .num_columns(4)
            .spacing([24.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Date").strong());
                ui.label(egui::RichText::new("Deck").strong());
                ui.label(egui::RichText::new("Cards").strong());
                ui.label(egui::RichText::new("Time").strong());
                ui.end_row();

                for session in &recent {
                    ui.label(session.date.format("%Y-%m-%d").to_string());
                    ui.label(&session.deck_name);
                    ui.label(session.cards_reviewed.to_string());
                    ui.label(format_duration(session.elapsed_secs));
                    ui.end_row();
                }
            });
    });
}

fn format_duration(secs: u64) -> String {
    let mins = secs / 60;
    let remaining_secs = secs % 60;
    if mins > 0 {
        format!("{mins}m {remaining_secs}s")
    } else {
        format!("{remaining_secs}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0s");
    }

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
    }

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(125), "2m 5s");
    }

    #[test]
    fn format_duration_exact_minutes() {
        assert_eq!(format_duration(120), "2m 0s");
    }
}
