//! Compatibility shim re-exporting the most-used `components` items under
//! the older `style::` name. New code should reach for `crate::components`
//! directly; this module exists to keep the diff manageable while the UI
//! migrates over.

use egui::{Color32, Frame, Ui};

pub use crate::components::tokens::{
    CARD_PADDING as CARD_MARGIN, CONTENT_PADDING, ITEM_SPACING, SECTION_SPACING,
};

/// Brand accent used for foreground highlights (saved chips, matched terms).
/// shadcn's neutral palette has no fixed accent hue, so we pick blue-500 —
/// the same shade used by shadcn's default chart-1 and focus rings in
/// non-neutral themes.
pub const ACCENT: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);

pub fn card_frame(ui: &Ui) -> Frame {
    crate::components::card_frame(ui)
}
