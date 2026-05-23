use cram_core::Deck;
use egui::Ui;

use crate::style;

pub struct SearchView;

impl SearchView {
    /// Returns `Some((deck_name, card_index))` when a card is clicked for editing.
    pub fn show(ui: &mut Ui, decks: &[&Deck], query: &mut String) -> Option<(String, usize)> {
        let mut navigate_to = None;

        ui.vertical(|ui| {
            ui.add_space(style::SECTION_SPACING);
            ui.heading("Search Cards");
            ui.separator();
            ui.add_space(style::SECTION_SPACING);

            crate::components::text_input(ui, query, "Search cards across all decks…");
            ui.add_space(style::SECTION_SPACING);

            let weak = ui.visuals().weak_text_color();

            if query.trim().is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(egui::RichText::new("🔍").size(56.0).color(weak));
                    ui.add_space(12.0);
                    ui.heading("Search across every deck");
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Match card fronts, backs, or tags.").color(weak));
                });
                return;
            }

            let lower_query = query.to_lowercase();
            let mut total_matches = 0usize;

            let per_deck: Vec<(&Deck, Vec<(usize, &cram_core::Card)>)> = decks
                .iter()
                .filter_map(|deck| {
                    let matches: Vec<(usize, &cram_core::Card)> = deck
                        .cards()
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| {
                            c.front().to_lowercase().contains(&lower_query)
                                || c.back().to_lowercase().contains(&lower_query)
                                || c.tags()
                                    .iter()
                                    .any(|t| t.to_lowercase().contains(&lower_query))
                        })
                        .collect();
                    if matches.is_empty() {
                        None
                    } else {
                        total_matches += matches.len();
                        Some((*deck, matches))
                    }
                })
                .collect();

            if per_deck.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.heading("No matching cards");
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!("Nothing matched \"{}\".", query.trim()))
                            .color(weak),
                    );
                });
                return;
            }

            ui.label(
                egui::RichText::new(format!(
                    "{total_matches} card{} across {} deck{}",
                    if total_matches == 1 { "" } else { "s" },
                    per_deck.len(),
                    if per_deck.len() == 1 { "" } else { "s" },
                ))
                .color(weak),
            );
            ui.add_space(style::ITEM_SPACING);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (deck, matches) in per_deck {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(deck.name()).strong().size(15.0));
                        ui.label(
                            egui::RichText::new(format!(
                                "· {} match{}",
                                matches.len(),
                                if matches.len() == 1 { "" } else { "es" },
                            ))
                            .small()
                            .color(weak),
                        );
                    });
                    ui.add_space(6.0);

                    for (card_idx, card) in matches {
                        let frame_resp = style::card_frame(ui).inner_margin(16.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(card.front()).strong());
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(card.back()).color(weak),
                                        )
                                        .truncate(),
                                    );
                                    if !card.tags().is_empty() {
                                        ui.add_space(4.0);
                                        ui.horizontal_wrapped(|ui| {
                                            for tag in card.tags() {
                                                crate::components::badge(
                                                    ui,
                                                    tag,
                                                    crate::components::BadgeVariant::Secondary,
                                                );
                                            }
                                        });
                                    }
                                });
                            });
                        });
                        let card_interact = ui
                            .interact(
                                frame_resp.response.rect,
                                egui::Id::new(("search_hit", deck.name(), card_idx)),
                                egui::Sense::click(),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if card_interact.clicked() {
                            navigate_to = Some((deck.name().to_string(), card_idx));
                        }
                        ui.add_space(style::ITEM_SPACING);
                    }
                    ui.add_space(style::SECTION_SPACING);
                }
            });
        });

        navigate_to
    }
}
