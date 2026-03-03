use std::sync::OnceLock;

use typst::LibraryExt as _;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_kit::fonts::{FontSearcher, Fonts};

static FONTS: OnceLock<Fonts> = OnceLock::new();

pub(crate) fn fonts() -> &'static Fonts {
    FONTS.get_or_init(|| FontSearcher::new().include_system_fonts(true).search())
}

pub(crate) struct CramWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
}

impl CramWorld {
    pub(crate) fn new(source_text: &str) -> Self {
        let fonts = fonts();
        let file_id = FileId::new(None, VirtualPath::new("card.typ"));
        Self {
            source: Source::new(file_id, source_text.to_string()),
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(fonts.book.clone()),
        }
    }

    /// Return the main source, used to resolve span locations in diagnostics.
    pub(crate) fn main_source(&self) -> &Source {
        &self.source
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
