use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::Utc;
use cram_core::{Deck, sm2};
use egui::{Context, TextureHandle, Ui};

use crate::app::{SessionSummaryState, StudyMode, View};
use crate::style;
use crate::texture_cache::{BackgroundRenderer, TextureCache, quantize_width};

/// Transient state for an active study session. Created when the user enters
/// study mode and dropped when they leave or finish all cards.
pub struct StudySession {
    /// When the current session began.
    pub start: std::time::Instant,
    /// Number of cards reviewed so far in this session.
    pub reviewed: u32,
    /// Pre-renders upcoming card textures on background threads.
    pub bg_renderer: BackgroundRenderer,
}

impl StudySession {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            reviewed: 0,
            bg_renderer: BackgroundRenderer::new(),
        }
    }
}

/// Borrowed references to the state needed for rendering a single frame
/// of the study view. Reconstructed each frame from [`crate::app::CramApp`] fields.
pub struct StudyContext<'a> {
    /// The single deck currently being studied.
    pub deck: &'a mut Deck,
    /// Name of the deck (used for display and view transitions).
    pub deck_name: &'a str,
    /// Position within the shuffled card order.
    pub card_index: &'a mut usize,
    /// Whether the answer side of the current card is visible.
    pub revealed: &'a mut bool,
    /// Shared texture cache for rendered card images.
    pub texture_cache: &'a mut TextureCache,
    /// The active study session holding timing and background rendering state.
    pub session: &'a mut StudySession,
    /// Current application view; mutated on navigation (Escape, session end).
    pub view: &'a mut View,
    /// Card indices in the order they will be presented. Mutable so that
    /// deleting the current card can remove its entry and shift higher indices.
    pub shuffled_indices: &'a mut Vec<usize>,
    /// Whether this is a random or spaced-repetition session.
    pub study_mode: StudyMode,
}

/// Renders the study view for a single frame: card content,
/// and action buttons. Handles keyboard input for navigation and rating.
pub fn show_study(ui: &mut Ui, ctx: &Context, sc: &mut StudyContext<'_>) {
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        *sc.view = View::DeckDetail {
            deck_name: sc.deck_name.to_string(),
        };
        return;
    }

    if sc.deck.cards().is_empty() || sc.shuffled_indices.is_empty() {
        show_empty_state(ui, sc.study_mode);
        return;
    }

    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) && *sc.card_index > 0 {
        *sc.card_index -= 1;
        *sc.revealed = false;
        return;
    }

    let total = sc.shuffled_indices.len();
    let current_idx = (*sc.card_index).min(total.saturating_sub(1));
    let card_pos = sc.shuffled_indices[current_idx];

    if card_pos >= sc.deck.cards().len() {
        *sc.view = View::DeckDetail {
            deck_name: sc.deck_name.to_string(),
        };
        return;
    }

    let params = RenderParams {
        dark_mode: ui.visuals().dark_mode,
        quantized: quantize_width(ui.available_width() - 50.0),
        width_pt: quantize_width(ui.available_width() - 50.0) as f32 / 2.0,
    };

    sc.session.bg_renderer.poll_completed(sc.texture_cache, ctx);

    prerender_all(
        sc.deck,
        sc.shuffled_indices,
        current_idx,
        &params,
        sc.texture_cache,
        &mut sc.session.bg_renderer,
    );

    let card_source = build_card_source(sc.deck, card_pos, *sc.revealed);
    let cache_key = make_cache_key(&card_source, params.quantized);
    let render_result = sc.texture_cache.get_or_render_with_width(
        ctx,
        &cache_key,
        &card_source,
        params.dark_mode,
        Some(params.width_pt),
    );

    let mut delete_requested = false;
    let mut edit_requested = false;

    ui.vertical(|ui| {
        show_header(
            ui,
            sc.deck_name,
            sc.study_mode,
            current_idx,
            total,
            &mut edit_requested,
            &mut delete_requested,
        );

        ui.separator();
        ui.add_space(24.0);

        show_card(ui, &render_result, &card_source);

        ui.add_space(24.0);

        if !*sc.revealed {
            show_reveal_button(ui, sc.revealed);
        } else {
            match sc.study_mode {
                StudyMode::Random => show_random_next(ui, sc, total),
                StudyMode::SpacedRepetition => {
                    show_rating_buttons(ui, sc, card_pos, total);
                }
            }
        }
    });

    if edit_requested {
        *sc.view = View::Editor {
            deck_name: sc.deck_name.to_string(),
            card_index: Some(card_pos),
        };
    } else if delete_requested {
        delete_current_card(sc, card_pos, current_idx);
    }
}

/// Builds the Typst source for a card, combining the deck preamble with
/// either the front only or front + divider + back when revealed.
fn build_card_source(deck: &Deck, card_pos: usize, revealed: bool) -> String {
    let card_text = if revealed {
        format!(
            "{}\n\n#line(length: 100%, stroke: 0.5pt + luma(180))\n\n{}",
            deck.cards()[card_pos].front(),
            deck.cards()[card_pos].back()
        )
    } else {
        deck.cards()[card_pos].front().to_string()
    };
    if deck.preamble().is_empty() {
        card_text
    } else {
        format!("{}\n{card_text}", deck.preamble())
    }
}

/// Produces a deterministic cache key from the Typst source content and
/// quantized render width.
fn make_cache_key(source: &str, quantized: u32) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("study-{:x}@w{quantized}", hasher.finish())
}

fn show_empty_state(ui: &mut Ui, study_mode: StudyMode) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        if study_mode == StudyMode::SpacedRepetition {
            ui.heading("No cards due for review.");
            ui.add_space(12.0);
            ui.label("All caught up! Come back later.");
        } else {
            ui.heading("No cards in this deck.");
            ui.add_space(12.0);
            ui.label("Add some cards first!");
        }
    });
}

fn show_header(
    ui: &mut Ui,
    deck_name: &str,
    study_mode: StudyMode,
    current_idx: usize,
    total: usize,
    edit_requested: &mut bool,
    delete_requested: &mut bool,
) {
    let progress = format!("{}/{}", current_idx + 1, total);
    let mode_label = match study_mode {
        StudyMode::Random => "Random",
        StudyMode::SpacedRepetition => "Spaced Repetition",
    };

    ui.horizontal(|ui| {
        ui.label(format!("Studying: {deck_name}"));
        ui.label(
            egui::RichText::new(format!("[{mode_label}]"))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(&progress);
            let header_btn = egui::vec2(70.0, 24.0);
            if ui
                .add(crate::components::destructive(ui, "Delete").min_size(header_btn))
                .clicked()
            {
                *delete_requested = true;
            }
            if ui
                .add(crate::components::outline(ui, "Edit").min_size(header_btn))
                .clicked()
            {
                *edit_requested = true;
            }
        });
    });
}

fn show_card(ui: &mut Ui, render_result: &Result<TextureHandle, String>, card_source: &str) {
    style::card_frame(ui).inner_margin(24.0).show(ui, |ui| {
        ui.set_min_size(egui::vec2(ui.available_width(), 320.0));
        ui.vertical_centered(|ui| match render_result {
            Ok(tex) => {
                ui.add(egui::Image::new(tex).max_width(ui.available_width()));
            }
            Err(err) => {
                ui.colored_label(egui::Color32::RED, format!("Render error: {err}"));
                ui.label(card_source);
            }
        });
    });
}

fn show_reveal_button(ui: &mut Ui, revealed: &mut bool) {
    ui.vertical_centered(|ui| {
        let btn = crate::components::primary(ui, "Show Answer  [Space / Right]")
            .min_size(egui::vec2(240.0, 44.0))
            .corner_radius(8.0);
        if ui.add(btn).clicked()
            || ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::ArrowRight))
        {
            *revealed = true;
        }
    });
}

fn show_random_next(ui: &mut Ui, sc: &mut StudyContext<'_>, total: usize) {
    ui.vertical_centered(|ui| {
        let btn = crate::components::primary(ui, "Next  [Space / Right]")
            .min_size(egui::vec2(240.0, 44.0))
            .corner_radius(8.0);
        if ui.add(btn).clicked()
            || ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::ArrowRight))
        {
            advance_card(sc, total);
        }
    });
}

const RATING_BUTTON_SIZE: egui::Vec2 = egui::vec2(100.0, 44.0);

fn show_rating_buttons(ui: &mut Ui, sc: &mut StudyContext<'_>, card_pos: usize, total: usize) {
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
                    let current_review = sc.deck.cards()[card_pos].review();
                    let new_review = sm2::schedule(current_review, *rating, today);
                    sc.deck.cards_mut()[card_pos].set_review(new_review);

                    advance_card(sc, total);
                }
            }
        });
    });
}

/// Removes the current card from the deck and from the shuffled order,
/// shifting later indices down by one. Ends the session if no cards remain.
fn delete_current_card(sc: &mut StudyContext<'_>, card_pos: usize, current_idx: usize) {
    sc.deck.cards_mut().remove(card_pos);
    sc.shuffled_indices.remove(current_idx);
    for idx in sc.shuffled_indices.iter_mut() {
        if *idx > card_pos {
            *idx -= 1;
        }
    }

    let total = sc.shuffled_indices.len();
    if total == 0 {
        let elapsed = sc.session.start.elapsed().as_secs();
        *sc.view = View::SessionSummary(SessionSummaryState {
            deck_name: sc.deck_name.to_string(),
            cards_reviewed: sc.session.reviewed,
            elapsed_secs: elapsed,
        });
    } else if *sc.card_index >= total {
        *sc.card_index = total - 1;
    }
    *sc.revealed = false;
}

/// Moves to the next card or ends the session if all cards are done.
fn advance_card(sc: &mut StudyContext<'_>, total: usize) {
    sc.session.reviewed += 1;
    let next = *sc.card_index + 1;
    if next >= total {
        let elapsed = sc.session.start.elapsed().as_secs();
        *sc.view = View::SessionSummary(SessionSummaryState {
            deck_name: sc.deck_name.to_string(),
            cards_reviewed: sc.session.reviewed,
            elapsed_secs: elapsed,
        });
    } else {
        *sc.card_index = next;
    }
    *sc.revealed = false;
}

/// Parameters for rendering a card texture (used by [`prerender_all`]).
struct RenderParams {
    dark_mode: bool,
    quantized: u32,
    width_pt: f32,
}

/// Queues background renders for all cards in priority order:
/// current revealed first, then the next card, then remaining cards.
fn prerender_all(
    deck: &Deck,
    shuffled_indices: &[usize],
    current_idx: usize,
    params: &RenderParams,
    texture_cache: &TextureCache,
    bg_renderer: &mut BackgroundRenderer,
) {
    let total = shuffled_indices.len();
    let mut targets = Vec::with_capacity(total * 2);

    let cur_pos = shuffled_indices[current_idx];
    targets.push((cur_pos, true));

    if current_idx + 1 < total {
        let next_pos = shuffled_indices[current_idx + 1];
        targets.push((next_pos, false));
        targets.push((next_pos, true));
    }

    for (i, &pos) in shuffled_indices.iter().enumerate() {
        if i == current_idx || i == current_idx + 1 {
            continue;
        }
        targets.push((pos, false));
        targets.push((pos, true));
    }

    for (pos, revealed) in targets {
        let source = build_card_source(deck, pos, revealed);
        let key = make_cache_key(&source, params.quantized);
        bg_renderer.request_render(
            texture_cache,
            key,
            source,
            params.dark_mode,
            Some(params.width_pt),
        );
    }
}
