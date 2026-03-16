use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cram_core::{Card, Deck};
use cram_store::{DeckSource, MultiStore};
use egui::{Context, Ui};

use crate::app::PreviewDebounce;
use crate::highlight::typst_layout_job;
use crate::style;
use crate::texture_cache::{TextureCache, quantize_width};

/// Estimated card body height (px) for off-screen cards with no cached measurement.
const ESTIMATED_CARD_BODY_HEIGHT: f32 = 500.0;

pub struct EditorContext<'a> {
    pub decks: &'a mut [Deck],
    pub deck_name: &'a str,
    pub card_index: Option<usize>,
    pub multi_store: &'a MultiStore,
    pub deck_source: &'a DeckSource,
    pub texture_cache: &'a mut TextureCache,
    pub preview_debounce: &'a mut PreviewDebounce,
    pub fullscreen_preview: &'a mut Option<String>,
    pub save_feedback: &'a mut Option<std::time::Instant>,
    pub tag_input: &'a mut std::collections::HashMap<usize, String>,
}

pub struct EditorView;

impl EditorView {
    pub fn show(ui: &mut Ui, ctx: &Context, ec: &mut EditorContext<'_>) {
        let Some(deck) = ec.decks.iter_mut().find(|d| d.name() == ec.deck_name) else {
            ui.label("Deck not found.");
            return;
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Edit: {}", ec.deck_name));
                if let DeckSource::Linked(path) = ec.deck_source {
                    ui.label(
                        egui::RichText::new(shorten_home(path))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(style::accent_button("+ Add Card").min_size(egui::vec2(140.0, 34.0)))
                        .clicked()
                    {
                        deck.cards_mut().push(Card::new("Front", "Back"));
                        let _ = ec.multi_store.save_deck(deck, ec.deck_source);
                    }
                    let btn_size = egui::vec2(100.0, 34.0);
                    if ui
                        .add(style::secondary_button("Collapse All").min_size(btn_size))
                        .clicked()
                    {
                        set_all_cards_open(ui, deck.cards().len(), false);
                    }
                    if ui
                        .add(style::secondary_button("Expand All").min_size(btn_size))
                        .clicked()
                    {
                        set_all_cards_open(ui, deck.cards().len(), true);
                    }
                    if let Some(saved_at) = *ec.save_feedback {
                        let elapsed = saved_at.elapsed();
                        if elapsed < std::time::Duration::from_secs(2) {
                            ui.label(egui::RichText::new("Saved!").color(style::ACCENT).strong());
                            ctx.request_repaint_after(std::time::Duration::from_secs(2) - elapsed);
                        } else {
                            *ec.save_feedback = None;
                        }
                    }
                });
            });

            ui.separator();
            ui.add_space(style::SECTION_SPACING);

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
                        let _ = ec.multi_store.save_deck(deck, ec.deck_source);
                        ec.texture_cache.clear();
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
                            let id = card_collapsing_id(i);
                            let default_open = ec.card_index == Some(i);
                            let state =
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    id,
                                    default_open,
                                );
                            let is_open = state.is_open();

                            state
                                .show_header(ui, |ui| {
                                    ui.spacing_mut().interact_size.y = 28.0;
                                    ui.label(egui::RichText::new(format!("#{}", i + 1)).strong());
                                    ui.label(egui::RichText::new(&preview).weak().italics());
                                    for tag in deck.cards()[i].tags() {
                                        ui.label(
                                            egui::RichText::new(tag).small().color(style::ACCENT),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let header_btn = egui::vec2(80.0, 28.0);
                                            if ui
                                                .add(
                                                    style::destructive_button("Delete")
                                                        .min_size(header_btn),
                                                )
                                                .clicked()
                                            {
                                                to_delete = Some(i);
                                            }
                                            if ui
                                                .add(
                                                    style::accent_button("Save")
                                                        .min_size(header_btn),
                                                )
                                                .clicked()
                                            {
                                                save_now = true;
                                                ec.texture_cache.clear();
                                            }
                                        },
                                    );
                                })
                                .body_unindented(|ui| {
                                    // Virtualize off-screen card bodies: skip all
                                    // widgets and allocate cached/estimated height.
                                    let height_id = egui::Id::new(("card_body_h", i));
                                    let cached_h: Option<f32> =
                                        ui.ctx().data(|d| d.get_temp(height_id));
                                    let h = cached_h.unwrap_or(ESTIMATED_CARD_BODY_HEIGHT);
                                    let body_rect = egui::Rect::from_min_size(
                                        ui.cursor().min,
                                        egui::vec2(ui.available_width(), h),
                                    );
                                    if !ui.is_rect_visible(body_rect) {
                                        ui.allocate_space(egui::vec2(ui.available_width(), h));
                                        return;
                                    }

                                    if is_open {
                                        ui.separator();
                                    }
                                    let avail_w = ui.available_width();
                                    let col_w = avail_w / 2.0 - 16.0;

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

                                            ui.add_space(8.0);
                                            ui.label("Tags:");
                                            ui.horizontal_wrapped(|ui| {
                                                let mut tag_to_remove = None;
                                                for (ti, tag) in
                                                    deck.cards()[i].tags().iter().enumerate()
                                                {
                                                    let chip = ui.add(
                                                        egui::Button::new(
                                                            egui::RichText::new(format!(
                                                                "{tag} \u{00d7}"
                                                            ))
                                                            .small(),
                                                        )
                                                        .corner_radius(12.0)
                                                        .fill(style::ACCENT.gamma_multiply(0.2)),
                                                    );
                                                    if chip.clicked() {
                                                        tag_to_remove = Some(ti);
                                                    }
                                                }
                                                if let Some(ti) = tag_to_remove {
                                                    deck.cards_mut()[i].tags_mut().remove(ti);
                                                    let _ = ec
                                                        .multi_store
                                                        .save_deck(deck, ec.deck_source);
                                                }
                                            });
                                            let all_tags = deck.all_tags();
                                            let buf = ec.tag_input.entry(i).or_default();
                                            let resp = ui.add(
                                                egui::TextEdit::singleline(buf)
                                                    .desired_width(col_w * 0.6)
                                                    .hint_text("Add tag..."),
                                            );
                                            let enter_pressed = resp.lost_focus()
                                                && ui
                                                    .input(|inp| inp.key_pressed(egui::Key::Enter));
                                            if !buf.is_empty() {
                                                let lower = buf.to_lowercase();
                                                let suggestions: Vec<&str> = all_tags
                                                    .iter()
                                                    .filter(|t| {
                                                        t.to_lowercase().contains(&lower)
                                                            && !deck.cards()[i].has_tag(t)
                                                    })
                                                    .map(String::as_str)
                                                    .take(5)
                                                    .collect();
                                                if !suggestions.is_empty() {
                                                    let mut picked = None;
                                                    for s in &suggestions {
                                                        if ui.small_button(*s).clicked() {
                                                            picked = Some((*s).to_string());
                                                        }
                                                    }
                                                    if let Some(tag) = picked {
                                                        if !deck.cards()[i].has_tag(&tag) {
                                                            deck.cards_mut()[i]
                                                                .tags_mut()
                                                                .push(tag);
                                                            let _ = ec
                                                                .multi_store
                                                                .save_deck(deck, ec.deck_source);
                                                        }
                                                        buf.clear();
                                                        ui.memory_mut(|m| {
                                                            m.surrender_focus(resp.id);
                                                        });
                                                    }
                                                }
                                            }
                                            if enter_pressed {
                                                let trimmed = buf.trim().to_string();
                                                if !trimmed.is_empty()
                                                    && !deck.cards()[i].has_tag(&trimmed)
                                                {
                                                    deck.cards_mut()[i].tags_mut().push(trimmed);
                                                    let _ = ec
                                                        .multi_store
                                                        .save_deck(deck, ec.deck_source);
                                                }
                                                buf.clear();
                                            }
                                        });

                                        ui.separator();

                                        ui.vertical(|ui| {
                                            ui.set_max_width(col_w);
                                            let dark_mode = ui.visuals().dark_mode;
                                            let q_col = quantize_width(col_w);
                                            let col_width_pt = q_col as f32 / 2.0;

                                            ui.horizontal(|ui| {
                                                ui.label("Front Preview:");
                                                if ui.small_button("Full Screen").clicked() {
                                                    *ec.fullscreen_preview = Some(with_preamble(
                                                        deck.preamble(),
                                                        deck.cards()[i].front(),
                                                    ));
                                                }
                                            });
                                            let front = deck.cards()[i].front().to_string();
                                            let debounced_front =
                                                ec.preview_debounce.render_source(i, &front, ctx);
                                            let front_source =
                                                with_preamble(deck.preamble(), &debounced_front);
                                            let front_key = format!(
                                                "ef-{}-{:x}@w{q_col}",
                                                deck.cards()[i].id(),
                                                hash_source(&front_source),
                                            );
                                            match ec.texture_cache.get_or_render_with_width(
                                                ctx,
                                                &front_key,
                                                &front_source,
                                                dark_mode,
                                                Some(col_width_pt),
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
                                            let debounced_back = ec.preview_debounce.render_source(
                                                back_debounce_key,
                                                &back,
                                                ctx,
                                            );
                                            let back_source =
                                                with_preamble(deck.preamble(), &debounced_back);
                                            let back_key = format!(
                                                "eb-{}-{:x}@w{q_col}",
                                                deck.cards()[i].id(),
                                                hash_source(&back_source),
                                            );
                                            match ec.texture_cache.get_or_render_with_width(
                                                ctx,
                                                &back_key,
                                                &back_source,
                                                dark_mode,
                                                Some(col_width_pt),
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

                                    // Cache measured body height for virtualization.
                                    let h = ui.min_rect().height();
                                    if h > 0.0 {
                                        ui.ctx().data_mut(|d| d.insert_temp(height_id, h));
                                    }
                                });
                        });
                    });
                    ui.add_space(style::ITEM_SPACING);
                }

                if save_now {
                    let _ = ec.multi_store.save_deck(deck, ec.deck_source);
                    *ec.save_feedback = Some(std::time::Instant::now());
                }
                if let Some(idx) = to_delete {
                    deck.cards_mut().remove(idx);
                    let _ = ec.multi_store.save_deck(deck, ec.deck_source);
                }
            });
        });
    }
}

fn shorten_home(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}

fn with_preamble(preamble: &str, body: &str) -> String {
    if preamble.is_empty() {
        body.to_string()
    } else {
        format!("{preamble}\n{body}")
    }
}

fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Stable ID for card collapsing state that is independent of UI scope.
fn card_collapsing_id(index: usize) -> egui::Id {
    egui::Id::new(("editor_card_collapse", index))
}

fn set_all_cards_open(ui: &Ui, count: usize, open: bool) {
    let ctx = ui.ctx();
    for i in 0..count {
        let id = card_collapsing_id(i);
        if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ctx, id) {
            state.set_open(open);
            state.store(ctx);
        }
    }
}

/// Resets all card collapsing states for the editor (e.g. when switching decks).
pub fn reset_card_collapse_states(ctx: &Context, count: usize) {
    for i in 0..count {
        let id = card_collapsing_id(i);
        if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ctx, id) {
            state.set_open(false);
            state.store(ctx);
        }
    }
}
