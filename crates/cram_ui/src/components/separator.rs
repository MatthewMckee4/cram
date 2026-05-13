use egui::Ui;

use crate::theme::Palette;

/// shadcn-style separator: 1px line in the border color.
pub fn separator(ui: &mut Ui) {
    let p = if ui.visuals().dark_mode {
        Palette::DARK
    } else {
        Palette::LIGHT
    };
    let available = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available, 1.0), egui::Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, (1.0, p.border));
}
