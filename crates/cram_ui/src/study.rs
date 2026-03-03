use cram_core::Deck;
use egui::{Context, Ui};

use crate::app::View;
use crate::style;

pub struct StudyContext<'a> {
    pub decks: &'a [&'a Deck],
    pub deck_name: &'a str,
    pub card_index: &'a mut usize,
    pub revealed: &'a mut bool,
    pub texture_cache: &'a mut std::collections::HashMap<String, egui::TextureHandle>,
    pub view: &'a mut View,
    pub session_reviewed: &'a mut u32,
    pub session_start: &'a mut Option<std::time::Instant>,
    pub shuffled_indices: &'a [usize],
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

        let Some(deck) = sc.decks.iter().find(|d| d.name() == sc.deck_name) else {
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

        let total = sc.shuffled_indices.len();
        let current_idx = (*sc.card_index).min(total.saturating_sub(1));
        let card_pos = sc.shuffled_indices[current_idx];
        let progress = format!("{}/{}", current_idx + 1, total);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Studying: {}", sc.deck_name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&progress);
                });
            });

            #[expect(clippy::cast_precision_loss)]
            let fraction = (current_idx + 1) as f32 / total as f32;
            ui.add(egui::ProgressBar::new(fraction).text(&progress));

            ui.separator();
            ui.add_space(24.0);

            let card_text = if *sc.revealed {
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
                get_or_render(ctx, &card_source, &card_source, sc.texture_cache, dark_mode);

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
                ui.vertical_centered(|ui| {
                    let btn = style::accent_button("Next  [Space]")
                        .min_size(egui::vec2(240.0, 44.0))
                        .corner_radius(8.0);
                    if ui.add(btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        *sc.session_reviewed += 1;
                        let next = *sc.card_index + 1;
                        if next >= total {
                            let elapsed =
                                sc.session_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);
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
