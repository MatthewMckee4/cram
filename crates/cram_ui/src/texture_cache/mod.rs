mod background;
mod cache;

pub use background::BackgroundRenderer;
pub use cache::TextureCache;

const QUANTIZE_STEP: f32 = 50.0;
const QUANTIZE_MIN: u32 = 100;

/// Rounds a logical pixel width to the nearest 50px step (minimum 100px).
/// This keeps the cache key stable across small layout fluctuations.
pub fn quantize_width(logical_px: f32) -> u32 {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let q = ((logical_px / QUANTIZE_STEP).round() as u32) * QUANTIZE_STEP as u32;
    q.max(QUANTIZE_MIN)
}
