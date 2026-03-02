use cram_core::Deck;
use egui::Ui;

use crate::style;

pub struct StatsView;

impl StatsView {
    pub fn show(ui: &mut Ui, decks: &[Deck]) {
        let today = chrono::Utc::now().date_naive();

        let total_cards: usize = decks.iter().map(|d| d.cards.len()).sum();
        let due_today: usize = decks.iter().map(|d| d.due_count()).sum();
        let reviewed: usize = decks
            .iter()
            .flat_map(|d| &d.cards)
            .filter(|c| c.reps > 0)
            .count();
        let retention_pct = if total_cards > 0 {
            #[expect(clippy::cast_precision_loss)]
            let pct = (total_cards - due_today) as f64 / total_cards as f64 * 100.0;
            pct
        } else {
            0.0
        };

        let streak = compute_streak(decks, today);

        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading("Statistics");
            ui.separator();
            ui.add_space(16.0);

            ui.horizontal_wrapped(|ui| {
                stat_card(ui, "Total cards", &total_cards.to_string());
                stat_card(ui, "Due today", &due_today.to_string());
                stat_card(ui, "Reviewed", &reviewed.to_string());
                stat_card(ui, "Retention", &format!("{retention_pct:.0}%"));
                stat_card(ui, "Streak", &format!("{streak} day(s)"));
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

/// Compute streak as the number of consecutive days (ending today or yesterday)
/// where at least one card was reviewed. A card was reviewed on a day if its
/// due date equals that day plus its interval (meaning it was scheduled forward
/// from that day). We approximate by checking if any card has `due - interval_days == day`.
fn compute_streak(decks: &[Deck], today: chrono::NaiveDate) -> u32 {
    use chrono::Days;

    let mut streak = 0u32;
    let mut check_day = today;

    for _ in 0..365 {
        let reviewed_on_day = decks.iter().flat_map(|d| &d.cards).any(|c| {
            if c.reps == 0 {
                return false;
            }
            let interval_days = c.interval.round() as u64;
            c.due.checked_sub_days(Days::new(interval_days)) == Some(check_day)
        });

        if reviewed_on_day {
            streak += 1;
            if let Some(prev) = check_day.checked_sub_days(Days::new(1)) {
                check_day = prev;
            } else {
                break;
            }
        } else if check_day == today {
            if let Some(prev) = check_day.checked_sub_days(Days::new(1)) {
                check_day = prev;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    streak
}
