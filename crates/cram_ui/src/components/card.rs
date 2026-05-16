use egui::{Frame, Shadow, Stroke, Ui};

use super::tokens;
use crate::theme::Palette;

fn palette(ui: &Ui) -> Palette {
    if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    }
}

/// shadcn-style card: raised surface, 1px border, generous padding, subtle shadow.
pub fn card_frame(ui: &Ui) -> Frame {
    let p = palette(ui);
    Frame::new()
        .fill(p.card)
        .corner_radius(tokens::RADIUS_XL)
        .inner_margin(tokens::CARD_PADDING)
        .stroke(Stroke::new(1.0, p.border))
        .shadow(card_shadow(ui))
}

/// Borderless surface used inside containers that already have a border.
pub fn surface_frame(ui: &Ui) -> Frame {
    let p = palette(ui);
    Frame::new()
        .fill(p.muted)
        .corner_radius(tokens::RADIUS_MD)
        .inner_margin(tokens::SPACE_4)
}

pub fn card_shadow(ui: &Ui) -> Shadow {
    let alpha = if ui.visuals().dark_mode { 60 } else { 14 };
    Shadow {
        spread: 0,
        blur: 12,
        offset: [0, 2],
        color: egui::Color32::from_black_alpha(alpha),
    }
}
