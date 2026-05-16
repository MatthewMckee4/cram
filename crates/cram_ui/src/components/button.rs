use egui::{Button, Color32, RichText, Stroke, Vec2, vec2};

use super::tokens;
use crate::theme::Palette;

const BUTTON_MIN_SIZE: Vec2 = vec2(72.0, tokens::CONTROL_HEIGHT);
const BUTTON_MIN_SIZE_SM: Vec2 = vec2(56.0, tokens::CONTROL_HEIGHT_SM);

/// shadcn `default` variant — solid primary background, contrasting text.
pub fn primary_button(text: &str) -> Button<'_> {
    Button::new(rich(text, Palette::DARK.primary_foreground))
        .fill(Palette::DARK.primary)
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE)
}

/// Light-mode-aware primary builder. Egui doesn't expose a way to inspect
/// theme inside a `Button<'_>`, so use this when you have a `Ui` handy.
pub fn primary(ui: &egui::Ui, text: &str) -> Button<'static> {
    let p = palette(ui);
    Button::new(rich(text, p.primary_foreground))
        .fill(p.primary)
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE)
}

/// shadcn `secondary` variant — muted surface with foreground text.
pub fn secondary(ui: &egui::Ui, text: &str) -> Button<'static> {
    let p = palette(ui);
    Button::new(rich(text, p.secondary_foreground))
        .fill(p.secondary)
        .stroke(Stroke::NONE)
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE)
}

/// shadcn `outline` variant — transparent fill, border, foreground text.
pub fn outline(ui: &egui::Ui, text: &str) -> Button<'static> {
    let p = palette(ui);
    Button::new(rich(text, p.foreground))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE)
}

/// shadcn `ghost` variant — transparent until hovered.
pub fn ghost(ui: &egui::Ui, text: &str) -> Button<'static> {
    let p = palette(ui);
    Button::new(rich(text, p.foreground))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE_SM)
}

/// shadcn `destructive` variant — red surface, light text.
pub fn destructive(ui: &egui::Ui, text: &str) -> Button<'static> {
    let p = palette(ui);
    Button::new(rich(text, p.destructive_foreground))
        .fill(p.destructive)
        .corner_radius(tokens::RADIUS_MD)
        .min_size(BUTTON_MIN_SIZE)
}

fn rich(text: &str, color: Color32) -> RichText {
    RichText::new(text).color(color).strong()
}

fn palette(ui: &egui::Ui) -> Palette {
    if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    }
}
