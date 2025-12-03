//! Module for creating example labels for all supported media types

use crate::error::TypstError;
use crate::media::Media;

use image::DynamicImage;
use std::sync::{Arc, OnceLock};
use tracing::debug;
use typst::layout::PagedDocument;

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::fonts::{FontSearcher, FontSlot};

/// Global font cache - initialized once on first use
static FONT_DATA: OnceLock<(LazyHash<FontBook>, Arc<Vec<FontSlot>>)> = OnceLock::new();

/// Typst world implementation providing file access, fonts, and compilation environment
struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,
    /// The standard library.
    library: LazyHash<Library>,
    /// Metadata about all known fonts.
    book: LazyHash<FontBook>,
    /// Shared reference to font data (Arc allows cheap cloning)
    fonts: Arc<Vec<FontSlot>>,
    /// Datetime.
    time: time::OffsetDateTime,
}

impl TypstWrapperWorld {
    /// Creates a new Typst world with the given root directory and source content
    fn new(source: String) -> Self {
        // Get or initialize fonts once globally
        let (book, fonts) = FONT_DATA.get_or_init(|| {
            debug!("Searching for fonts (one-time initialization)...");
            let fonts = FontSearcher::new().include_system_fonts(false).search();
            debug!("Found {} fonts:", fonts.book.families().count());
            fonts.book.families().for_each(|f| debug!("- {}", f.0));
            (LazyHash::new(fonts.book), Arc::new(fonts.fonts))
        });
        Self {
            library: LazyHash::new(Library::default()),
            book: book.clone(),
            fonts: Arc::clone(fonts),
            source: Source::detached(source),
            time: time::OffsetDateTime::now_utc(),
        }
    }
}

impl typst::World for TypstWrapperWorld {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            todo!("Not implemented!")
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        todo!("Not implemented!")
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

/// Renders a test label with dimensions and media name using embedded Typst
/// For [`Continuous`](crate::media::LabelType::Continuous) labels, a height of 300px is chosen.
///
/// # Errors
///
/// Returns [`TypstError`] if Typst compilation, PNG encoding, or image loading fails
pub fn render_test_label(media: Media) -> Result<DynamicImage, TypstError> {
    let label_template = include_str!("../typst/label.typ");
    let label_call = format!(
        r#"
#label(
  width: {}pt,
  height: {}pt,
  name: "{}",
  color_support: {}
)
"#,
        media.width_dots(),
        media.length_dots().unwrap_or(300),
        media,
        media.supports_color(),
    );
    debug!("Rendering example label for {media}...");

    let world = TypstWrapperWorld::new(format!("{label_template}{label_call}"));

    let document: PagedDocument = typst::compile(&world).output.map_err(|err| TypstError {
        reason: format!("Typst compilation failed: {err:?}"),
    })?;

    let pages: Vec<_> = document.pages.iter().collect();
    let page = pages.first().ok_or_else(|| TypstError {
        reason: "Compiled document has no pages".to_string(),
    })?;

    let pixmap = typst_render::render(page, 1.0);
    let buf = pixmap.encode_png().map_err(|err| TypstError {
        reason: format!("PNG encoding failed: {err}"),
    })?;

    image::load_from_memory(&buf).map_err(|err| TypstError {
        reason: format!("Failed to load PNG from memory: {err}"),
    })
}
