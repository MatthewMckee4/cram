use std::sync::OnceLock;
use thiserror::Error;
use typst::LibraryExt as _;
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_kit::fonts::{FontSearcher, Fonts};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("typst compile error: {0}")]
    Compile(String),
    #[error("no pages in document")]
    NoPages,
    #[error("png encode failed")]
    Encode,
}

/// Render a Typst source string to PNG bytes at 2× pixel density.
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

static FONTS: OnceLock<Fonts> = OnceLock::new();

fn fonts() -> &'static Fonts {
    FONTS.get_or_init(|| FontSearcher::new().include_system_fonts(true).search())
}

struct CramWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
}

impl CramWorld {
    fn new(source_text: &str) -> Self {
        let fonts = fonts();
        let file_id = FileId::new(None, VirtualPath::new("card.typ"));
        Self {
            source: Source::new(file_id, source_text.to_string()),
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(fonts.book.clone()),
        }
    }
}

impl World for CramWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }
    }

    fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
        Err(typst::diag::FileError::NotFound(
            id.vpath().as_rootless_path().to_path_buf(),
        ))
    }

    fn font(&self, index: usize) -> Option<Font> {
        fonts().fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        use chrono::{Datelike as _, Utc};
        let now = Utc::now();
        let offset = offset.unwrap_or(0);
        let dt = now + chrono::Duration::hours(offset);
        Datetime::from_ymd(dt.year(), dt.month() as u8, dt.day() as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_heading_produces_png() {
        let bytes = render("= Hello World").expect("render failed");
        assert!(!bytes.is_empty());
        // PNG magic bytes
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
