use egui::{Frame, Margin, Response, Stroke, TextEdit, Ui, vec2};

use super::tokens;
use crate::theme::Palette;

fn palette(ui: &Ui) -> Palette {
    if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    }
}

/// shadcn-style single-line input: h-9, rounded-md, 1px border, transparent
/// bg in dark mode (lighter than the card surface in light mode).
pub fn text_input(ui: &mut Ui, value: &mut String, hint: &str) -> Response {
    let p = palette(ui);
    let inner = Frame::new()
        .fill(p.input_bg)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(tokens::RADIUS_MD)
        .inner_margin(Margin::symmetric(tokens::SPACE_3 as i8, 0));
    inner
        .show(ui, |ui| {
            let avail = ui.available_width();
            ui.add_sized(
                vec2(avail, tokens::CONTROL_HEIGHT),
                TextEdit::singleline(value)
                    .hint_text(hint)
                    .frame(false)
                    .desired_width(avail)
                    .margin(Margin::ZERO)
                    .vertical_align(egui::Align::Center),
            )
        })
        .inner
}

/// Frame matching the shadcn input style — use to wrap a custom [`TextEdit`]
/// (e.g. one with a syntax-highlighting layouter) so it picks up the same
/// 1px border, `RADIUS_MD`, and padding as the standard inputs.
pub fn input_frame(ui: &Ui) -> Frame {
    let p = palette(ui);
    Frame::new()
        .fill(p.input_bg)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(tokens::RADIUS_MD)
        .inner_margin(Margin::symmetric(
            tokens::SPACE_3 as i8,
            tokens::SPACE_2 as i8,
        ))
}

/// shadcn-style multiline input. Min height ~ 5 lines.
pub fn text_area(ui: &mut Ui, value: &mut String, hint: &str, rows: usize) -> Response {
    let p = palette(ui);
    Frame::new()
        .fill(p.input_bg)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(tokens::RADIUS_MD)
        .inner_margin(Margin::symmetric(
            tokens::SPACE_3 as i8,
            tokens::SPACE_2 as i8,
        ))
        .show(ui, |ui| {
            ui.add(
                TextEdit::multiline(value)
                    .hint_text(hint)
                    .frame(false)
                    .desired_width(ui.available_width())
                    .desired_rows(rows),
            )
        })
        .inner
}
