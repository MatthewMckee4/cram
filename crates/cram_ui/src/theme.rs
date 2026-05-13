use egui::{Color32, CornerRadius, Margin, Shadow, Stroke, Visuals, style::WidgetVisuals};
use serde::{Deserialize, Serialize};

use crate::components::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    #[allow(
        dead_code,
        reason = "public API even though only `toggled` is used today"
    )]
    pub const ALL: [Theme; 2] = [Theme::Dark, Theme::Light];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Theme::Dark)
    }

    pub fn toggled(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Dark => Palette::DARK,
            Theme::Light => Palette::LIGHT,
        }
    }

    pub fn visuals(self) -> Visuals {
        let p = self.palette();
        let mut v = if self.is_dark() {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.dark_mode = self.is_dark();
        v.override_text_color = Some(p.foreground);
        v.panel_fill = p.background;
        v.window_fill = p.card;
        v.window_stroke = Stroke::new(1.0, p.border);
        v.window_corner_radius = CornerRadius::same(tokens::RADIUS_LG as u8);
        v.window_shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(if self.is_dark() { 96 } else { 24 }),
        };
        v.popup_shadow = v.window_shadow;
        v.menu_corner_radius = CornerRadius::same(tokens::RADIUS_MD as u8);

        v.faint_bg_color = p.muted;
        v.extreme_bg_color = p.input_bg;
        v.code_bg_color = p.muted;
        v.hyperlink_color = p.primary;
        v.selection.bg_fill = p.primary.gamma_multiply(0.25);
        v.selection.stroke = Stroke::new(1.0, p.primary);
        v.warn_fg_color = if self.is_dark() {
            Color32::from_rgb(0xFB, 0xBF, 0x24)
        } else {
            Color32::from_rgb(0xCA, 0x8A, 0x04)
        };
        v.error_fg_color = p.destructive;

        let button_radius = CornerRadius::same(tokens::RADIUS_MD as u8);

        // Non-interactive widgets (separators, labels, group frames): no fill,
        // a hairline border in the divider color. Setting bg_fill to the panel
        // would paint over raised card surfaces.
        v.widgets.noninteractive = WidgetVisuals {
            bg_fill: Color32::TRANSPARENT,
            weak_bg_fill: Color32::TRANSPARENT,
            bg_stroke: Stroke::new(1.0, p.border),
            corner_radius: button_radius,
            fg_stroke: Stroke::new(1.0, p.muted_foreground),
            expansion: 0.0,
        };
        // Default state for interactive widgets (buttons, combo boxes).
        v.widgets.inactive = WidgetVisuals {
            bg_fill: p.secondary,
            weak_bg_fill: p.secondary,
            bg_stroke: Stroke::new(1.0, p.border),
            corner_radius: button_radius,
            fg_stroke: Stroke::new(1.0, p.foreground),
            expansion: 0.0,
        };
        // Hover: nudge toward accent, with a slightly stronger border.
        v.widgets.hovered = WidgetVisuals {
            bg_fill: p.accent,
            weak_bg_fill: p.accent,
            bg_stroke: Stroke::new(1.0, p.border_strong),
            corner_radius: button_radius,
            fg_stroke: Stroke::new(1.0, p.foreground),
            expansion: 0.5,
        };
        // Pressed/active: shadcn presses look like the hovered state with a
        // ring (handled via selection.stroke when focused).
        v.widgets.active = WidgetVisuals {
            bg_fill: p.accent,
            weak_bg_fill: p.accent,
            bg_stroke: Stroke::new(1.0, p.ring),
            corner_radius: button_radius,
            fg_stroke: Stroke::new(1.0, p.foreground),
            expansion: 0.0,
        };
        v.widgets.open = WidgetVisuals {
            bg_fill: p.muted,
            weak_bg_fill: p.muted,
            bg_stroke: Stroke::new(1.0, p.border_strong),
            corner_radius: button_radius,
            fg_stroke: Stroke::new(1.0, p.foreground),
            expansion: 0.0,
        };

        v
    }

    /// Apply egui-wide style: spacing, fonts, and visuals. Call on init or
    /// whenever the theme changes.
    pub fn apply(self, ctx: &egui::Context) {
        ctx.set_visuals(self.visuals());

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(tokens::SPACE_2, tokens::SPACE_2);
        style.spacing.button_padding = egui::vec2(tokens::SPACE_4, tokens::SPACE_2);
        style.spacing.menu_margin = Margin::same(tokens::SPACE_2 as i8);
        style.spacing.window_margin = Margin::same(tokens::SPACE_4 as i8);
        style.spacing.indent = tokens::SPACE_4;
        style.spacing.interact_size = egui::vec2(40.0, tokens::CONTROL_HEIGHT);

        use egui::{FontFamily, FontId, TextStyle};
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(22.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
        ]
        .into();

        ctx.set_style(style);
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// shadcn-derived palette. Names mirror the shadcn CSS variable conventions
/// (background, foreground, primary, secondary, muted, accent, etc.).
#[derive(Clone, Copy)]
#[expect(
    dead_code,
    reason = "full palette is a design-token surface for future components"
)]
pub struct Palette {
    pub background: Color32,
    pub foreground: Color32,
    pub card: Color32,
    pub card_foreground: Color32,
    pub primary: Color32,
    pub primary_foreground: Color32,
    pub secondary: Color32,
    pub secondary_foreground: Color32,
    pub muted: Color32,
    pub muted_foreground: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub destructive: Color32,
    pub destructive_foreground: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub input_bg: Color32,
    pub ring: Color32,
}

impl Palette {
    /// shadcn `zinc` dark theme, tuned for a medium-grey canvas with raised
    /// surfaces (cards lighter than the panel background, in the style of
    /// Linear / Vercel).
    pub const DARK: Palette = Palette {
        background: Color32::from_rgb(0x18, 0x18, 0x1B),
        foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        card: Color32::from_rgb(0x1F, 0x1F, 0x23),
        card_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        primary: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        primary_foreground: Color32::from_rgb(0x18, 0x18, 0x1B),
        secondary: Color32::from_rgb(0x27, 0x27, 0x2A),
        secondary_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        muted: Color32::from_rgb(0x27, 0x27, 0x2A),
        muted_foreground: Color32::from_rgb(0xA1, 0xA1, 0xAA),
        accent: Color32::from_rgb(0x2E, 0x2E, 0x33),
        accent_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        destructive: Color32::from_rgb(0xEF, 0x44, 0x44),
        destructive_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        border: Color32::from_rgb(0x32, 0x32, 0x38),
        border_strong: Color32::from_rgb(0x52, 0x52, 0x5B),
        input_bg: Color32::from_rgb(0x18, 0x18, 0x1B),
        ring: Color32::from_rgb(0xD4, 0xD4, 0xD8),
    };

    /// shadcn `zinc` light theme.
    pub const LIGHT: Palette = Palette {
        background: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        foreground: Color32::from_rgb(0x09, 0x09, 0x0B),
        card: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        card_foreground: Color32::from_rgb(0x09, 0x09, 0x0B),
        primary: Color32::from_rgb(0x18, 0x18, 0x1B),
        primary_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        secondary: Color32::from_rgb(0xF4, 0xF4, 0xF5),
        secondary_foreground: Color32::from_rgb(0x18, 0x18, 0x1B),
        muted: Color32::from_rgb(0xF4, 0xF4, 0xF5),
        muted_foreground: Color32::from_rgb(0x71, 0x71, 0x7A),
        accent: Color32::from_rgb(0xF4, 0xF4, 0xF5),
        accent_foreground: Color32::from_rgb(0x18, 0x18, 0x1B),
        destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
        destructive_foreground: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        border: Color32::from_rgb(0xE4, 0xE4, 0xE7),
        border_strong: Color32::from_rgb(0xD4, 0xD4, 0xD8),
        input_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        ring: Color32::from_rgb(0xA1, 0xA1, 0xAA),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        assert_eq!(Theme::default(), Theme::Dark);
    }

    #[test]
    fn toggle_round_trips() {
        assert_eq!(Theme::Dark.toggled(), Theme::Light);
        assert_eq!(Theme::Light.toggled(), Theme::Dark);
    }
}
