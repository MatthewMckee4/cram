use cram_core::{Deck, Rating, review_card};
use cram_store::Store;
use egui::{Context, Ui};

pub struct StudyView;

impl StudyView {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &mut [Deck],
        deck_name: &str,
        card_index: &mut usize,
        revealed: &mut bool,
        store: &Store,
        texture_cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
    ) {
        let Some(deck) = decks.iter_mut().find(|d| d.name == deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        let today = chrono::Utc::now().date_naive();
        let due_indices: Vec<usize> = (0..deck.cards.len())
            .filter(|&i| deck.cards[i].due <= today)
            .collect();

        if due_indices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading("🎉 Nothing to review!");
                ui.add_space(12.0);
                ui.label("Come back tomorrow for more cards.");
            });
            return;
        }

        let current_idx = (*card_index).min(due_indices.len().saturating_sub(1));
        let card_pos = due_indices[current_idx];

        let progress = format!("{}/{}", current_idx + 1, due_indices.len());

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Studying: {}", deck_name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&progress);
                });
            });
            ui.separator();
            ui.add_space(16.0);

            // Card display area
            let card_source = if *revealed {
                deck.cards[card_pos].back.clone()
            } else {
                deck.cards[card_pos].front.clone()
            };

            let texture = get_or_render(ctx, &card_source, &card_source, texture_cache);

            egui::Frame::new()
                .fill(ui.visuals().window_fill)
                .corner_radius(12.0)
                .inner_margin(24.0)
                .stroke(ui.visuals().window_stroke)
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(ui.available_width(), 320.0));
                    ui.vertical_centered(|ui| {
                        if let Some(tex) = texture {
                            let max_w = ui.available_width().min(600.0);
                            ui.add(egui::Image::new(&tex).max_width(max_w));
                        } else {
                            ui.label(&card_source);
                        }
                    });
                });

            ui.add_space(16.0);

            if !*revealed {
                ui.vertical_centered(|ui| {
                    if ui.button("Show Answer  [Space]").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Space))
                    {
                        *revealed = true;
                    }
                });
            } else {
                ui.label("How well did you recall it?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let ratings = [
                        (Rating::Again, egui::Color32::from_rgb(220, 50, 50)),
                        (Rating::Hard, egui::Color32::from_rgb(220, 130, 50)),
                        (Rating::Good, egui::Color32::from_rgb(50, 150, 50)),
                        (Rating::Easy, egui::Color32::from_rgb(50, 100, 220)),
                    ];
                    for (rating, color) in ratings {
                        let label = egui::RichText::new(rating.label()).color(egui::Color32::WHITE);
                        if ui.add(egui::Button::new(label).fill(color)).clicked() {
                            review_card(&mut deck.cards[card_pos], rating);
                            if let Err(e) = store.save_deck(deck) {
                                tracing::warn!("save failed: {e}");
                            }
                            *card_index = (*card_index + 1) % due_indices.len().max(1);
                            *revealed = false;
                        }
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
) -> Option<egui::TextureHandle> {
    if let Some(h) = cache.get(key) {
        return Some(h.clone());
    }
    let png = cram_render::render(source).ok()?;
    let img = image::load_from_memory(&png).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let handle = ctx.load_texture(key, color_image, egui::TextureOptions::LINEAR);
    cache.insert(key.to_string(), handle.clone());
    Some(handle)
}
