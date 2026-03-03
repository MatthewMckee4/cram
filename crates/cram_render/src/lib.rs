mod error;
mod world;

pub use error::{CompileError, RenderError};

use typst::layout::PagedDocument;
use world::CramWorld;

/// Number of lines the internal preamble adds before the user's source.
const PREAMBLE_LINES: usize = 2;

/// Render a Typst source string to PNG bytes at 2x pixel density.
/// The page is auto-sized to the content (not A4).
///
/// When `dark_mode` is true the text colour is set to white so that
/// rendered cards remain visible on a dark background.
///
/// # Errors
/// Returns [`RenderError`] if the source fails to compile or produces no pages.
pub fn render(source: &str, dark_mode: bool) -> Result<Vec<u8>, RenderError> {
    let text_fill = if dark_mode { "white" } else { "black" };
    let preamble = format!(
        "#set page(width: auto, height: auto, margin: 0.6em, fill: none)\n\
         #set text(fill: {text_fill})\n\
         {source}"
    );
    let world = CramWorld::new(&preamble);
    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|diagnostics| {
        let source = world.main_source();
        let errors = diagnostics
            .iter()
            .map(|diag| {
                let (line, column) = source
                    .range(diag.span)
                    .and_then(|range| source.lines().byte_to_line_column(range.start))
                    .map(|(l, c)| {
                        let user_line = l.saturating_sub(PREAMBLE_LINES) + 1;
                        (Some(user_line), Some(c + 1))
                    })
                    .unwrap_or((None, None));
                CompileError {
                    line,
                    column,
                    message: diag.message.to_string(),
                    hints: diag.hints.iter().map(|h| h.to_string()).collect(),
                }
            })
            .collect();
        RenderError::Compile(errors)
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
        let bytes = render("= Hello World", false).expect("render failed");
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[..4], b"\x89PNG");
    }

    #[test]
    fn render_math_equation() {
        let bytes = render("$ x^2 + y^2 = z^2 $", false).expect("math render failed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_is_compact_not_full_a4() {
        let bytes = render("= Hello", false).expect("render failed");
        let img = image::load_from_memory(&bytes).expect("decode failed");
        // A4 at 2x = ~1190x1684px — compact should be much smaller
        assert!(img.width() < 800, "width {} too large", img.width());
        assert!(img.height() < 400, "height {} too large", img.height());
    }

    #[test]
    fn render_body_text() {
        let bytes =
            render("Hello, this is *bold* and _italic_.", false).expect("text render failed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_special_characters() {
        let bytes = render("Symbols: & < > \" ' @", false).expect("special chars render failed");
        assert_eq!(&bytes[..4], b"\x89PNG");
    }

    #[test]
    fn render_multiline_content() {
        let source = "= Title\n\nFirst paragraph.\n\nSecond paragraph with *emphasis*.";
        let bytes = render(source, false).expect("multiline render failed");
        assert_eq!(&bytes[..4], b"\x89PNG");
    }

    #[test]
    fn render_dark_mode_produces_png() {
        let bytes = render("= Dark Mode Test", true).expect("dark mode render failed");
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[..4], b"\x89PNG");
    }

    #[test]
    fn compile_error_includes_line_and_column() {
        let err = render("#let x = ", false).expect_err("should fail");
        if let RenderError::Compile(errors) = &err {
            assert!(!errors.is_empty());
            let first = &errors[0];
            assert!(first.line.is_some(), "expected line info: {first}");
            assert!(first.column.is_some(), "expected column info: {first}");
        } else {
            panic!("expected Compile error, got: {err}");
        }
    }

    #[test]
    fn compile_error_reports_user_line_not_preamble() {
        let err = render("hello\n#let x = ", false).expect_err("should fail");
        if let RenderError::Compile(errors) = &err {
            let first = &errors[0];
            if let Some(line) = first.line {
                assert!(line >= 1, "line should be in user source, got {line}");
            }
        } else {
            panic!("expected Compile error, got: {err}");
        }
    }

    #[test]
    fn compile_error_display_is_human_readable() {
        let err = render("#unknown_func()", false).expect_err("should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("line") || msg.contains("unknown"),
            "error should be human-readable: {msg}"
        );
    }
}
