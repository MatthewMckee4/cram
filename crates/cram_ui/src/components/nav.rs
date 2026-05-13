use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Response, Sense, Stroke, TextStyle, Ui,
    vec2,
};

use super::tokens;
use crate::theme::Palette;

fn palette(ui: &Ui) -> Palette {
    if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    }
}

/// shadcn-style top app bar: panel background, 20px horizontal / 12px
/// vertical padding. Pass as `frame` to a [`egui::TopBottomPanel`]. Use
/// [`paint_bottom_border`] after the panel closes to render the hairline
/// divider beneath it.
pub fn topbar_frame(dark: bool) -> Frame {
    let p = if dark { Palette::DARK } else { Palette::LIGHT };
    Frame::new()
        .fill(p.background)
        .inner_margin(Margin::symmetric(
            tokens::SPACE_5 as i8,
            tokens::SPACE_3 as i8,
        ))
        .stroke(Stroke::NONE)
}

/// Paint a 1px divider along the bottom of `rect` in the theme border color.
pub fn paint_bottom_border(ctx: &egui::Context, rect: egui::Rect) {
    let dark = ctx.style().visuals.dark_mode;
    let p = if dark { Palette::DARK } else { Palette::LIGHT };
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("__topbar_border"),
    ));
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, p.border),
    );
}

/// shadcn navigation-menu-style tab: foreground/muted text, accent-tinted
/// background when hovered, and a stronger accent when active.
pub fn nav_tab(ui: &mut Ui, label: &str, active: bool) -> Response {
    let p = palette(ui);
    let font_id = ui
        .style()
        .text_styles
        .get(&TextStyle::Button)
        .cloned()
        .unwrap_or_else(|| FontId::proportional(14.0));

    let padding = vec2(tokens::SPACE_3, tokens::SPACE_2);
    let text_color = if active {
        p.foreground
    } else {
        p.muted_foreground
    };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_id.clone(), text_color);
    let size = galley.size() + 2.0 * padding;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let bg = if active {
        p.accent
    } else if response.hovered() {
        p.muted
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(tokens::RADIUS_MD as u8), bg);
    }
    let hover_color = if response.hovered() && !active {
        p.foreground
    } else {
        text_color
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font_id,
        hover_color,
    );
    response
}
