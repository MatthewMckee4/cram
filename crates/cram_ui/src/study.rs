use chrono::Utc;
use cram_core::{Deck, sm2};
use egui::{Context, Ui};

use crate::app::{StudyMode, View};
use crate::style;
use crate::texture_cache::TextureCache;

pub struct StudyContext<'a> {
    pub decks: &'a mut [Deck],
    pub deck_name: &'a str,
    pub card_index: &'a mut usize,
    pub revealed: &'a mut bool,
    pub texture_cache: &'a mut TextureCache,
    pub view: &'a mut View,
    pub session_reviewed: &'a mut u32,
    pub session_start: &'a mut Option<std::time::Instant>,
    pub shuffled_indices: &'a [usize],
    pub study_mode: StudyMode,
}

pub struct StudyView;

impl StudyView {
    pub fn show(ui: &mut Ui, ctx: &Context, sc: &mut StudyContext<'_>) {
        if sc.session_start.is_none() {
            *sc.session_start = Some(std::time::Instant::now());
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            *sc.view = View::DeckList;
            return;
        }

        let Some(deck_idx) = sc.decks.iter().position(|d| d.name() == sc.deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        if sc.decks[deck_idx].cards().is_empty() || sc.shuffled_indices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                if sc.study_mode == StudyMode::SpacedRepetition {
                    ui.heading("No cards due for review.");
                    ui.add_space(12.0);
                    ui.label("All caught up! Come back later.");
                } else {
                    ui.heading("No cards in this deck.");
                    ui.add_space(12.0);
                    ui.label("Add some cards first!");
                }
            });
            return;
        }

        let total = sc.shuffled_indices.len();
        let current_idx = (*sc.card_index).min(total.saturating_sub(1));
        let card_pos = sc.shuffled_indices[current_idx];
        let progress = format!("{}/{}", current_idx + 1, total);

        let mode_label = match sc.study_mode {
            StudyMode::Random => "Random",
            StudyMode::SpacedRepetition => "Spaced Repetition",
        };

        let card_text = if *sc.revealed {
            sc.decks[deck_idx].cards()[card_pos].back().to_string()
        } else {
            sc.decks[deck_idx].cards()[card_pos].front().to_string()
        };
        let card_source = if sc.decks[deck_idx].preamble().is_empty() {
            card_text
        } else {
            format!("{}\n{card_text}", sc.decks[deck_idx].preamble())
        };

        let dark_mode = ui.visuals().dark_mode;
        let render_result =
            sc.texture_cache
                .get_or_render(ctx, &card_source, &card_source, dark_mode);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Studying: {}", sc.deck_name));
                ui.label(
                    egui::RichText::new(format!("[{mode_label}]"))
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&progress);
                });
            });

            #[expect(clippy::cast_precision_loss)]
            let fraction = (current_idx + 1) as f32 / total as f32;
            ui.add(egui::ProgressBar::new(fraction).text(&progress));

            ui.separator();
            ui.add_space(24.0);

            style::card_frame(ui).inner_margin(24.0).show(ui, |ui| {
                ui.set_min_size(egui::vec2(ui.available_width(), 320.0));
                ui.vertical_centered(|ui| match &render_result {
                    Ok(tex) => {
                        let max_w = ui.available_width().min(600.0);
                        ui.add(egui::Image::new(tex).max_width(max_w));
                    }
                    Err(err) => {
                        ui.colored_label(egui::Color32::RED, format!("Render error: {err}"));
                        ui.label(&card_source);
                    }
                });
            });

            ui.add_space(24.0);

            if !*sc.revealed {
                ui.vertical_centered(|ui| {
                    let btn = style::accent_button("Show Answer  [Space]")
                        .min_size(egui::vec2(240.0, 44.0))
                        .corner_radius(8.0);
                    if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        *sc.revealed = true;
                    }
                });
            } else {
                match sc.study_mode {
                    StudyMode::Random => {
                        show_random_next(ui, sc, total);
                    }
                    StudyMode::SpacedRepetition => {
                        show_rating_buttons(ui, sc, deck_idx, card_pos, total);
                    }
                }
            }
        });
    }
}

fn show_random_next(ui: &mut Ui, sc: &mut StudyContext<'_>, total: usize) {
    ui.vertical_centered(|ui| {
        let btn = style::accent_button("Next  [Space]")
            .min_size(egui::vec2(240.0, 44.0))
            .corner_radius(8.0);
        if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
            advance_card(sc, total);
        }
    });
}

const RATING_BUTTON_SIZE: egui::Vec2 = egui::vec2(100.0, 44.0);

fn show_rating_buttons(
    ui: &mut Ui,
    sc: &mut StudyContext<'_>,
    deck_idx: usize,
    card_pos: usize,
    total: usize,
) {
    ui.vertical_centered(|ui| {
        ui.label("How did you do?");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 440.0).max(0.0) / 2.0);

            let ratings = [
                (
                    "Again",
                    sm2::Rating::Again,
                    egui::Color32::from_rgb(220, 50, 50),
                ),
                (
                    "Hard",
                    sm2::Rating::Hard,
                    egui::Color32::from_rgb(220, 140, 40),
                ),
                (
                    "Good",
                    sm2::Rating::Good,
                    egui::Color32::from_rgb(59, 130, 246),
                ),
                (
                    "Easy",
                    sm2::Rating::Easy,
                    egui::Color32::from_rgb(34, 160, 90),
                ),
            ];

            let keys = [
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
            ];

            for (idx, (label, rating, color)) in ratings.iter().enumerate() {
                let text = format!("{label}  [{num}]", num = idx + 1);
                let btn = egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
                    .fill(*color)
                    .corner_radius(8.0)
                    .min_size(RATING_BUTTON_SIZE);

                if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(keys[idx])) {
                    let today = Utc::now().date_naive();
                    let current_review = sc.decks[deck_idx].cards()[card_pos].review().clone();
                    let new_review = sm2::schedule(&current_review, *rating, today);
                    *sc.decks[deck_idx].cards_mut()[card_pos].review_mut() = new_review;

                    advance_card(sc, total);
                }
            }
        });
    });
}

fn advance_card(sc: &mut StudyContext<'_>, total: usize) {
    *sc.session_reviewed += 1;
    let next = *sc.card_index + 1;
    if next >= total {
        let elapsed = sc.session_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);
        *sc.view = View::SessionSummary {
            deck_name: sc.deck_name.to_string(),
            cards_reviewed: *sc.session_reviewed,
            elapsed_secs: elapsed,
        };
        *sc.session_reviewed = 0;
        *sc.session_start = None;
    } else {
        *sc.card_index = next;
    }
    *sc.revealed = false;
}
