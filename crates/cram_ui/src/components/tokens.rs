//! Design tokens mirroring shadcn's defaults.
//!
//! Spacing follows Tailwind's 4px scale (SPACE_N = N * 4px).

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_5: f32 = 20.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;

/// Inner padding inside cards.
pub const CARD_PADDING: f32 = SPACE_6;
/// Outer page padding inside the central panel.
pub const CONTENT_PADDING: f32 = SPACE_6;
/// Gap between major sections.
pub const SECTION_SPACING: f32 = SPACE_5;
/// Gap between related items in a list/stack.
pub const ITEM_SPACING: f32 = SPACE_3;

/// shadcn `--radius-sm` ≈ 4px.
pub const RADIUS_SM: f32 = 4.0;
/// shadcn `--radius-md` ≈ 6px. Used for buttons, inputs, badges.
pub const RADIUS_MD: f32 = 6.0;
/// shadcn `--radius-lg` ≈ 8px.
pub const RADIUS_LG: f32 = 8.0;
/// shadcn `--radius-xl` ≈ 12px. Used for cards and dialogs.
pub const RADIUS_XL: f32 = 12.0;

/// Default control (button/input) height — shadcn h-9.
pub const CONTROL_HEIGHT: f32 = 36.0;
/// Small control height — shadcn h-8.
pub const CONTROL_HEIGHT_SM: f32 = 32.0;
