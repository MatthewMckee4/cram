use egui::{Color32, Shadow, Stroke, Ui};

pub const CARD_RADIUS: f32 = 10.0;
pub const BUTTON_RADIUS: f32 = 8.0;
pub const CARD_MARGIN: f32 = 20.0;
pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
pub const DESTRUCTIVE: Color32 = Color32::from_rgb(220, 50, 50);
pub const CONTENT_PADDING: f32 = 24.0;
pub const SECTION_SPACING: f32 = 20.0;
pub const ITEM_SPACING: f32 = 12.0;

pub fn card_shadow() -> Shadow {
    Shadow {
        spread: 0,
        blur: 8,
        offset: [0, 2],
        color: Color32::from_black_alpha(20),
    }
}

/// Standard card frame used across all views.
pub fn card_frame(ui: &Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CARD_RADIUS)
        .inner_margin(CARD_MARGIN)
        .shadow(card_shadow())
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
}

const BUTTON_MIN_SIZE: egui::Vec2 = egui::vec2(64.0, 34.0);

/// Primary action button with accent fill and white text.
pub fn accent_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
        .fill(ACCENT)
        .corner_radius(BUTTON_RADIUS)
        .min_size(BUTTON_MIN_SIZE)
}

/// Secondary action button with consistent radius and min size.
pub fn secondary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(text)
        .corner_radius(BUTTON_RADIUS)
        .min_size(BUTTON_MIN_SIZE)
}

/// Destructive action button with red fill and white text.
pub fn destructive_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
        .fill(DESTRUCTIVE)
        .corner_radius(BUTTON_RADIUS)
        .min_size(BUTTON_MIN_SIZE)
}
