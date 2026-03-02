use cram_core::{Card, Deck};
use cram_store::{DeckSource, MultiStore};
use egui::{Context, Ui};

use crate::app::PreviewDebounce;
use crate::highlight::typst_layout_job;
use crate::style;

pub struct EditorView;

impl EditorView {
    #[expect(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        decks: &mut [Deck],
        deck_name: &str,
        card_index: Option<usize>,
        multi_store: &MultiStore,
        deck_source: &DeckSource,
        texture_cache: &mut std::collections::HashMap<String, egui::TextureHandle>,
        selected_cards: &mut std::collections::HashSet<usize>,
        preview_debounce: &mut PreviewDebounce,
        fullscreen_preview: &mut Option<String>,
        save_feedback: &mut Option<std::time::Instant>,
    ) {
        let Some(deck) = decks.iter_mut().find(|d| d.name() == deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Edit: {deck_name}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(style::accent_button("+ Add Card")).clicked() {
                        deck.cards_mut().push(Card::new("Front", "Back"));
                        let _ = multi_store.save_deck(deck, deck_source);
                    }
                    if let Some(saved_at) = *save_feedback {
                        let elapsed = saved_at.elapsed();
                        if elapsed < std::time::Duration::from_secs(2) {
                            ui.label(egui::RichText::new("Saved!").color(style::ACCENT).strong());
                            ctx.request_repaint_after(std::time::Duration::from_secs(2) - elapsed);
                        } else {
                            *save_feedback = None;
                        }
                    }
                });
            });

            ui.horizontal(|ui| {
                let all_selected =
                    !deck.cards().is_empty() && selected_cards.len() == deck.cards().len();
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
                        *selected_cards = (0..deck.cards().len()).collect();
                    }
                }
                if !selected_cards.is_empty()
                    && ui
                        .add(style::destructive_button(&format!(
                            "Delete Selected ({})",
                            selected_cards.len()
                        )))
                        .clicked()
                {
                    let mut indices: Vec<usize> = selected_cards.iter().copied().collect();
                    indices.sort_unstable_by(|a, b| b.cmp(a));
                    for idx in indices {
                        if idx < deck.cards().len() {
                            deck.cards_mut().remove(idx);
                        }
                    }
                    selected_cards.clear();
                    let _ = multi_store.save_deck(deck, deck_source);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} cards", deck.cards().len()));
                });
            });

            ui.separator();

            egui::CollapsingHeader::new("Deck Preamble (shared Typst)")
                .default_open(!deck.preamble().is_empty())
                .show(ui, |ui| {
                    ui.label("This Typst code is prepended to every card when rendering:");
                    let dark = ui.visuals().dark_mode;
                    let mut preamble_layouter =
                        |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                            let mut job = typst_layout_job(text.as_str(), dark);
                            job.wrap.max_width = wrap_width;
                            ui.fonts_mut(|f| f.layout_job(job))
                        };
                    if ui
                        .add(
                            egui::TextEdit::multiline(deck.preamble_mut())
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(3)
                                .desired_width(ui.available_width())
                                .layouter(&mut preamble_layouter),
                        )
                        .changed()
                    {
                        let _ = multi_store.save_deck(deck, deck_source);
                        texture_cache.clear();
                    }
                });

            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_delete: Option<usize> = None;
                let mut save_now = false;
                let count = deck.cards().len();

                for i in 0..count {
                    let preview = {
                        let f = deck.cards()[i].front();
                        if f.len() > 50 {
                            format!("{}...", &f[..50])
                        } else {
                            f.to_string()
                        }
                    };

                    ui.push_id(i, |ui| {
                        style::card_frame(ui).show(ui, |ui| {
                            let id = ui.make_persistent_id(("card", i));
                            let default_open = card_index == Some(i);
                            let state =
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    id,
                                    default_open,
                                );
                            let is_open = state.is_open();

                            state
                                .show_header(ui, |ui| {
                                    let mut checked = selected_cards.contains(&i);
                                    if ui.checkbox(&mut checked, "").changed() {
                                        if checked {
                                            selected_cards.insert(i);
                                        } else {
                                            selected_cards.remove(&i);
                                        }
                                    }
                                    ui.label(egui::RichText::new(format!("#{}", i + 1)).strong());
                                    ui.label(egui::RichText::new(&preview).weak().italics());
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.add(style::destructive_button("Delete")).clicked()
                                            {
                                                to_delete = Some(i);
                                            }
                                            if ui.add(style::accent_button("Save")).clicked() {
                                                save_now = true;
                                                texture_cache.clear();
                                            }
                                        },
                                    );
                                })
                                .body_unindented(|ui| {
                                    if is_open {
                                        ui.separator();
                                    }
                                    let avail_w = ui.available_width();
                                    let col_w = avail_w / 2.0 - 8.0;

                                    ui.horizontal(|ui| {
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
                                                egui::TextEdit::multiline(
                                                    deck.cards_mut()[i].front_mut(),
                                                )
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
                                                egui::TextEdit::multiline(
                                                    deck.cards_mut()[i].back_mut(),
                                                )
                                                .font(egui::TextStyle::Monospace)
                                                .desired_rows(5)
                                                .desired_width(col_w)
                                                .layouter(&mut back_layouter),
                                            );
                                        });

                                        ui.separator();

                                        ui.vertical(|ui| {
                                            ui.set_max_width(col_w);
                                            let dark_mode = ui.visuals().dark_mode;

                                            ui.horizontal(|ui| {
                                                ui.label("Front Preview:");
                                                if ui.small_button("Full Screen").clicked() {
                                                    *fullscreen_preview = Some(with_preamble(
                                                        deck.preamble(),
                                                        deck.cards()[i].front(),
                                                    ));
                                                }
                                            });
                                            let front = deck.cards()[i].front().to_string();
                                            let debounced_front =
                                                preview_debounce.render_source(i, &front, ctx);
                                            let front_source =
                                                with_preamble(deck.preamble(), &debounced_front);
                                            let front_key =
                                                format!("editor-front-{i}-{front_source}");
                                            match get_or_render(
                                                ctx,
                                                &front_key,
                                                &front_source,
                                                texture_cache,
                                                dark_mode,
                                            ) {
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

                                            ui.add_space(8.0);
                                            ui.separator();
                                            ui.add_space(4.0);

                                            ui.label("Back Preview:");
                                            let back = deck.cards()[i].back().to_string();
                                            let back_debounce_key = usize::MAX - i;
                                            let debounced_back = preview_debounce.render_source(
                                                back_debounce_key,
                                                &back,
                                                ctx,
                                            );
                                            let back_source =
                                                with_preamble(deck.preamble(), &debounced_back);
                                            let back_key = format!("editor-back-{i}-{back_source}");
                                            match get_or_render(
                                                ctx,
                                                &back_key,
                                                &back_source,
                                                texture_cache,
                                                dark_mode,
                                            ) {
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
                                });
                        });
                    });
                    ui.add_space(8.0);
                }

                if save_now {
                    let _ = multi_store.save_deck(deck, deck_source);
                    *save_feedback = Some(std::time::Instant::now());
                }
                if let Some(idx) = to_delete {
                    deck.cards_mut().remove(idx);
                    let _ = multi_store.save_deck(deck, deck_source);
                }
            });
        });
    }
}

fn with_preamble(preamble: &str, body: &str) -> String {
    if preamble.is_empty() {
        body.to_string()
    } else {
        format!("{preamble}\n{body}")
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
    let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let handle = ctx.load_texture(key, ci, egui::TextureOptions::LINEAR);
    cache.insert(key.to_string(), handle.clone());
    Ok(handle)
}
