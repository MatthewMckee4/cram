mod error;
mod world;

pub use error::RenderError;

use typst::layout::PagedDocument;
use world::CramWorld;

/// Render a Typst source string to PNG bytes at 2× pixel density.
/// The page is auto-sized to the content (not A4).
///
/// # Errors
/// Returns [`RenderError`] if the source fails to compile or produces no pages.
pub fn render(source: &str) -> Result<Vec<u8>, RenderError> {
    let preamble =
        format!("#set page(width: auto, height: auto, margin: 0.6em, fill: none)\n{source}");
    let world = CramWorld::new(&preamble);
    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|errors| {
        RenderError::Compile(
            errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    let page = document.pages.first().ok_or(RenderError::NoPages)?;
    let pixmap = typst_render::render(page, 2.0);
    pixmap.encode_png().map_err(|_| RenderError::Encode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_heading_produces_png() {
        let bytes = render("= Hello World").expect("render failed");
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[..4], b"\x89PNG");
    }

    #[test]
    fn render_math_equation() {
        let bytes = render("$ x^2 + y^2 = z^2 $").expect("math render failed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_is_compact_not_full_a4() {
        let bytes = render("= Hello").expect("render failed");
        let img = image::load_from_memory(&bytes).expect("decode failed");
        // A4 at 2x = ~1190x1684px — compact should be much smaller
        assert!(img.width() < 800, "width {} too large", img.width());
        assert!(img.height() < 400, "height {} too large", img.height());
    }

    #[test]
    fn render_body_text() {
        let bytes = render("Hello, this is *bold* and _italic_.").expect("text render failed");
        assert!(!bytes.is_empty());
    }
}
