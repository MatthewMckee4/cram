use cram_core::Deck;
use egui::{Context, Ui};

use crate::app::View;
use crate::style;

pub struct StudyView;

impl StudyView {
    #[expect(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &[&Deck],
        deck_name: &str,
        card_index: &mut usize,
        revealed: &mut bool,
        texture_cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
        view: &mut View,
        session_reviewed: &mut u32,
        session_start: &mut Option<std::time::Instant>,
        shuffled_indices: &[usize],
    ) {
        if session_start.is_none() {
            *session_start = Some(std::time::Instant::now());
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            *view = View::DeckList;
            return;
        }

        let Some(deck) = decks.iter().find(|d| d.name() == deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        if deck.cards().is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading("No cards in this deck.");
                ui.add_space(12.0);
                ui.label("Add some cards first!");
            });
            return;
        }

        let total = shuffled_indices.len();
        let current_idx = (*card_index).min(total.saturating_sub(1));
        let card_pos = shuffled_indices[current_idx];
        let progress = format!("{}/{}", current_idx + 1, total);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Studying: {deck_name}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&progress);
                });
            });

            #[expect(clippy::cast_precision_loss)]
            let fraction = (current_idx + 1) as f32 / total as f32;
            ui.add(egui::ProgressBar::new(fraction).text(&progress));

            ui.separator();
            ui.add_space(16.0);

            let card_text = if *revealed {
                deck.cards()[card_pos].back().to_string()
            } else {
                deck.cards()[card_pos].front().to_string()
            };
            let card_source = if deck.preamble().is_empty() {
                card_text
            } else {
                format!("{}\n{card_text}", deck.preamble())
            };

            let dark_mode = ui.visuals().dark_mode;
            let render_result =
                get_or_render(ctx, &card_source, &card_source, texture_cache, dark_mode);

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

            ui.add_space(16.0);

            if !*revealed {
                ui.vertical_centered(|ui| {
                    let btn = style::accent_button("Show Answer  [Space]")
                        .min_size(egui::vec2(200.0, 40.0))
                        .corner_radius(8.0);
                    if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        *revealed = true;
                    }
                });
            } else {
                ui.vertical_centered(|ui| {
                    let btn = style::accent_button("Next  [Space]")
                        .min_size(egui::vec2(200.0, 40.0))
                        .corner_radius(8.0);
                    if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        *session_reviewed += 1;
                        let next = *card_index + 1;
                        if next >= total {
                            let elapsed = session_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);
                            *view = View::SessionSummary {
                                deck_name: deck_name.to_string(),
                                cards_reviewed: *session_reviewed,
                                elapsed_secs: elapsed,
                            };
                            *session_reviewed = 0;
                            *session_start = None;
                        } else {
                            *card_index = next;
                        }
                        *revealed = false;
                    }
                });
            }
        });
    }
}

fn get_or_render(
    ctx: &Context,
    key: &str,
    source: &str,
    cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
    dark_mode: bool,
) -> Result<egui::TextureHandle, String> {
    if let Some(h) = cache.get(key) {
        return Ok(h.clone());
    }
    let png = cram_render::render(source, dark_mode).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&png).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let handle = ctx.load_texture(key, color_image, egui::TextureOptions::LINEAR);
    cache.insert(key.to_string(), handle.clone());
    Ok(handle)
}
