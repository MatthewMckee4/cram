use egui::{Frame, Response, RichText, Stroke, Ui};

use super::tokens;
use crate::theme::Palette;

#[derive(Clone, Copy)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
    Destructive,
}

/// shadcn-style badge: small pill with foreground text and a tinted background.
pub fn badge(ui: &mut Ui, text: &str, variant: BadgeVariant) -> Response {
    let p = if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    };
    let (fill, fg, stroke) = match variant {
        BadgeVariant::Default => (p.primary, p.primary_foreground, Stroke::NONE),
        BadgeVariant::Secondary => (p.secondary, p.secondary_foreground, Stroke::NONE),
        BadgeVariant::Outline => (
            egui::Color32::TRANSPARENT,
            p.foreground,
            Stroke::new(1.0, p.border),
        ),
        BadgeVariant::Destructive => (p.destructive, p.destructive_foreground, Stroke::NONE),
    };
    Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(tokens::RADIUS_SM)
        .inner_margin(egui::Margin::symmetric(
            tokens::SPACE_2 as i8,
            (tokens::SPACE_1 * 0.5) as i8,
        ))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).size(12.0).strong());
        })
        .response
}
