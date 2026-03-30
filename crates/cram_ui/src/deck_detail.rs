use std::collections::BTreeSet;

use cram_core::Deck;
use egui::{Context, Ui};

use crate::app::{StudyMode, View};
use crate::deck_list::{due_indices, shuffled_indices_from};
use crate::style;

pub enum DeckDetailAction {
    Delete(String),
}

pub struct DeckDetailView;

impl DeckDetailView {
    pub fn show(
        ui: &mut Ui,
        ctx: &Context,
        deck: &Deck,
        view: &mut View,
        confirm_delete: &mut Option<String>,
        study_tag_filter: &mut BTreeSet<String>,
    ) -> Option<DeckDetailAction> {
        let mut action = None;

        if let Some(deck_name) = confirm_delete.clone() {
            let mut open = true;
            egui::Window::new("Delete Deck")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete \"{deck_name}\"? This cannot be undone."
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(style::destructive_button("Delete")).clicked() {
                            action = Some(DeckDetailAction::Delete(deck_name));
                            *confirm_delete = None;
                        }
                        if ui.add(style::secondary_button("Cancel")).clicked() {
                            *confirm_delete = None;
                        }
                    });
                });
            if !open {
                *confirm_delete = None;
            }
        }

        ui.vertical(|ui| {
            ui.add_space(style::SECTION_SPACING);

            ui.horizontal(|ui| {
                ui.heading(deck.name());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().button_padding = egui::vec2(12.0, 6.0);
                    if ui.add(style::destructive_button("Delete")).clicked() {
                        *confirm_delete = Some(deck.name().to_string());
                    }
                    if ui
                        .selectable_label(false, egui::RichText::new("Export").size(15.0))
                        .clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .set_file_name(format!("{}.toml", deck.name()))
                            .add_filter("TOML deck files", &["toml"])
                            .save_file()
                        && let Err(e) = cram_store::exchange::export_toml(deck, &path)
                    {
                        tracing::warn!("export failed: {e}");
                    }
                    if ui
                        .selectable_label(false, egui::RichText::new("Edit").size(15.0))
                        .clicked()
                    {
                        *view = View::Editor {
                            deck_name: deck.name().to_string(),
                            card_index: None,
                        };
                    }
                    if ui
                        .selectable_label(false, egui::RichText::new("Study (Spaced)").size(15.0))
                        .on_hover_text("Study cards due for review using spaced repetition")
                        .clicked()
                    {
                        let indices = due_indices(deck);
                        if !indices.is_empty() {
                            *view = View::Study {
                                deck_name: deck.name().to_string(),
                                card_index: 0,
                                revealed: false,
                                shuffled_indices: indices,
                                study_mode: StudyMode::SpacedRepetition,
                            };
                        }
                    }
                    if ui
                        .selectable_label(false, egui::RichText::new("Study (Random)").size(15.0))
                        .clicked()
                    {
                        let indices = deck.card_indices_matching_tags(study_tag_filter);
                        *view = View::Study {
                            deck_name: deck.name().to_string(),
                            card_index: 0,
                            revealed: false,
                            shuffled_indices: shuffled_indices_from(indices),
                            study_mode: StudyMode::Random,
                        };
                        study_tag_filter.clear();
                    }
                });
            });

            ui.separator();
            ui.add_space(style::SECTION_SPACING);

            if !deck.description().is_empty() {
                ui.label(
                    egui::RichText::new(deck.description())
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(8.0);
            }

            let total = deck.cards().len();
            ui.label(format!("{total} cards"));

            let all_tags = deck.all_tags();
            if !all_tags.is_empty() {
                ui.add_space(style::SECTION_SPACING);
                ui.label(
                    egui::RichText::new("Filter by tag")
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.horizontal_wrapped(|ui| {
                    for tag in &all_tags {
                        let selected = study_tag_filter.contains(tag);
                        if ui.selectable_label(selected, tag).clicked() {
                            if selected {
                                study_tag_filter.remove(tag);
                            } else {
                                study_tag_filter.insert(tag.clone());
                            }
                        }
                    }
                });
                if !study_tag_filter.is_empty() {
                    let filtered = deck.card_indices_matching_tags(study_tag_filter);
                    ui.label(
                        egui::RichText::new(format!("{}/{total} cards match", filtered.len()))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                }
            }
        });

        action
    }
}
