//! shadcn-inspired UI primitives for egui.
//!
//! Each submodule mirrors a shadcn component family. Compose them like you
//! would shadcn in React: a card containing inputs and buttons, badges for
//! status, separators between sections.
#![expect(
    dead_code,
    unused_imports,
    reason = "intentional component-library surface — variants are exposed even when no caller uses them yet"
)]

pub mod badge;
pub mod button;
pub mod card;
pub mod input;
pub mod nav;
pub mod separator;
pub mod tokens;

pub use badge::{BadgeVariant, badge};
pub use button::{destructive, ghost, outline, primary, secondary};
pub use card::{card_frame, surface_frame};
pub use input::{input_frame, text_area, text_input};
pub use nav::{nav_tab, paint_bottom_border, topbar_frame};
pub use separator::separator;
pub use tokens::{
    CARD_PADDING, CONTENT_PADDING, CONTROL_HEIGHT, ITEM_SPACING, RADIUS_LG, RADIUS_MD, RADIUS_SM,
    RADIUS_XL, SECTION_SPACING, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6, SPACE_8,
};
