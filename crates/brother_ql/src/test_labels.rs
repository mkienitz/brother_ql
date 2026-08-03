//! Module for creating example labels for all supported media types

use crate::error::TypstError;
use crate::media::Media;

use image::DynamicImage;
use std::sync::OnceLock;
use tracing::debug;

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::{LazyHash, Scalar};
use typst::{Library, LibraryExt};
use typst_kit::datetime::Time;
use typst_kit::fonts::FontStore;
use typst_layout::PagedDocument;
use typst_render::RenderOptions;

/// Global font cache - initialized once on first use
static FONT_STORE: OnceLock<FontStore> = OnceLock::new();

/// Typst world implementation providing file access, fonts, and compilation environment
struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,
    /// The standard library.
    library: LazyHash<Library>,
    /// Embedded fonts and their metadata.
    fonts: &'static FontStore,
    /// Datetime.
    time: Time,
}

impl TypstWrapperWorld {
    /// Creates a new Typst world with the given root directory and source content
    fn new(source: String) -> Self {
        // Get or initialize fonts once globally
        let fonts = FONT_STORE.get_or_init(|| {
            debug!("Loading embedded fonts (one-time initialization)...");
            let mut fonts = FontStore::new();
            fonts.extend(typst_kit::fonts::embedded());
            debug!("Found {} fonts:", fonts.book().families().count());
            fonts.book().families().for_each(|f| debug!("- {}", f.0));
            fonts
        });
        Self {
            library: LazyHash::new(Library::default()),
            fonts,
            source: Source::detached(source),
            time: Time::system(),
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
        self.fonts.book()
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
        self.fonts.font(id)
    }

    /// Get the current date.
    ///
    /// Optionally, a UTC offset is given.
    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
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

    let page = document.pages().first().ok_or_else(|| TypstError {
        reason: "Compiled document has no pages".to_string(),
    })?;

    let render_options = RenderOptions {
        pixel_per_pt: Scalar::new(1.0),
        ..RenderOptions::default()
    };
    let pixmap = typst_render::render(page, &render_options);
    let buf = pixmap.encode_png().map_err(|err| TypstError {
        reason: format!("PNG encoding failed: {err}"),
    })?;

    image::load_from_memory(&buf).map_err(|err| TypstError {
        reason: format!("Failed to load PNG from memory: {err}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_labels_match_media_dimensions() {
        for media in Media::iter() {
            let image = render_test_label(media).expect("test label should render");
            assert_eq!(image.width(), media.width_dots(), "{media}");
            assert_eq!(
                image.height(),
                media.length_dots().unwrap_or(300),
                "{media}"
            );
        }
    }
}
