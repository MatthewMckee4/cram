use cram_core::{Card, Deck};
use cram_store::Store;
use egui::{Context, Ui};

use crate::app::PreviewDebounce;
use crate::highlight::typst_layout_job;

pub struct EditorView;

impl EditorView {
    #[expect(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &mut [Deck],
        deck_name: &str,
        card_index: Option<usize>,
        store: &Store,
        texture_cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
        selected_cards: &mut std::collections::HashSet<usize>,
        preview_debounce: &mut PreviewDebounce,
    ) {
        let Some(deck) = decks.iter_mut().find(|d| d.name == deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Edit: {deck_name}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Add Card").clicked() {
                        deck.cards.push(Card::new("Front", "Back"));
                        let _ = store.save_deck(deck);
                    }
                });
            });

            ui.horizontal(|ui| {
                let all_selected =
                    !deck.cards.is_empty() && selected_cards.len() == deck.cards.len();
                if ui
                    .button(if all_selected {
                        "Deselect All"
                    } else {
                        "Select All"
                    })
                    .clicked()
                {
                    if all_selected {
                        selected_cards.clear();
                    } else {
                        *selected_cards = (0..deck.cards.len()).collect();
                    }
                }
                if !selected_cards.is_empty()
                    && ui
                        .button(format!("Delete Selected ({})", selected_cards.len()))
                        .clicked()
                {
                    let mut indices: Vec<usize> = selected_cards.iter().copied().collect();
                    indices.sort_unstable_by(|a, b| b.cmp(a));
                    for idx in indices {
                        if idx < deck.cards.len() {
                            deck.cards.remove(idx);
                        }
                    }
                    selected_cards.clear();
                    let _ = store.save_deck(deck);
                }
            });

            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_delete: Option<usize> = None;
                let mut save_now = false;
                let count = deck.cards.len();

                for i in 0..count {
                    let preview = {
                        let f = &deck.cards[i].front;
                        if f.len() > 50 {
                            format!("{}...", &f[..50])
                        } else {
                            f.clone()
                        }
                    };

                    ui.push_id(i, |ui| {
                        ui.horizontal(|ui| {
                            let mut checked = selected_cards.contains(&i);
                            if ui.checkbox(&mut checked, "").changed() {
                                if checked {
                                    selected_cards.insert(i);
                                } else {
                                    selected_cards.remove(&i);
                                }
                            }
                        });
                        egui::CollapsingHeader::new(&preview)
                            .default_open(card_index == Some(i))
                            .show(ui, |ui| {
                                let avail_w = ui.available_width();
                                let col_w = avail_w / 2.0 - 8.0;

                                ui.horizontal(|ui| {
                                    // Left column: editors
                                    ui.vertical(|ui| {
                                        ui.set_max_width(col_w);
                                        let dark = ui.visuals().dark_mode;
                                        let mut front_layouter = |ui: &egui::Ui,
                                                                    text: &dyn egui::TextBuffer,
                                                                    wrap_width: f32| {
                                            let mut job =
                                                typst_layout_job(text.as_str(), dark);
                                            job.wrap.max_width = wrap_width;
                                            ui.fonts_mut(|f| f.layout_job(job))
                                        };
                                        ui.label("Front (Typst):");
                                        ui.add(
                                            egui::TextEdit::multiline(&mut deck.cards[i].front)
                                                .font(egui::TextStyle::Monospace)
                                                .desired_rows(5)
                                                .desired_width(col_w)
                                                .layouter(&mut front_layouter),
                                        );
                                        ui.add_space(4.0);
                                        let mut back_layouter = |ui: &egui::Ui,
                                                                   text: &dyn egui::TextBuffer,
                                                                   wrap_width: f32| {
                                            let mut job =
                                                typst_layout_job(text.as_str(), dark);
                                            job.wrap.max_width = wrap_width;
                                            ui.fonts_mut(|f| f.layout_job(job))
                                        };
                                        ui.label("Back (Typst):");
                                        ui.add(
                                            egui::TextEdit::multiline(&mut deck.cards[i].back)
                                                .font(egui::TextStyle::Monospace)
                                                .desired_rows(5)
                                                .desired_width(col_w)
                                                .layouter(&mut back_layouter),
                                        );
                                        ui.add_space(4.0);
                                        ui.label("Tags (comma-separated):");
                                        let mut tags_str = deck.cards[i].tags.join(", ");
                                        if ui.text_edit_singleline(&mut tags_str).changed() {
                                            deck.cards[i].tags = tags_str
                                                .split(',')
                                                .map(|t| t.trim().to_string())
                                                .filter(|t| !t.is_empty())
                                                .collect();
                                        }
                                    });

                                    ui.separator();

                                    // Right column: preview
                                    ui.vertical(|ui| {
                                        ui.set_max_width(col_w);
                                        ui.label("Preview:");
                                        let front = deck.cards[i].front.clone();
                                        let source = preview_debounce.render_source(i, &front, ctx);
                                        let key = format!("editor-{i}-{source}");
                                        match get_or_render(ctx, &key, &source, texture_cache) {
                                            Ok(tex) => {
                                                ui.add(egui::Image::new(&tex).max_width(col_w));
                                            }
                                            Err(err) => {
                                                ui.colored_label(
                                                    egui::Color32::RED,
                                                    format!("Render error: {err}"),
                                                );
                                            }
                                        }
                                    });
                                });

                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    if ui.button("Save").clicked() {
                                        save_now = true;
                                        texture_cache.clear();
                                    }
                                    if ui.button("Delete").clicked() {
                                        to_delete = Some(i);
                                    }
                                });
                            });
                    });
                    ui.add_space(4.0);
                }

                if save_now {
                    let _ = store.save_deck(deck);
                }
                if let Some(idx) = to_delete {
                    deck.cards.remove(idx);
                    let _ = store.save_deck(deck);
                }
            });
        });
    }
}

fn get_or_render(
    ctx: &Context,
    key: &str,
    source: &str,
    cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
) -> Result<egui::TextureHandle, String> {
    if let Some(h) = cache.get(key) {
        return Ok(h.clone());
    }
    let png = cram_render::render(source).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&png).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let handle = ctx.load_texture(key, ci, egui::TextureOptions::LINEAR);
    cache.insert(key.to_string(), handle.clone());
    Ok(handle)
}
