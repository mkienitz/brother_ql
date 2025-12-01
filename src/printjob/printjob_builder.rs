//! The core module for defining and compiling print data
use std::marker::PhantomData;

use image::DynamicImage;

use crate::{
    error::PrintJobCreationError,
    media::{Media, MediaSettings, MediaType},
    printjob::{CutBehavior, PrintJob},
};

/// Type-level marker indicating the builder has images
pub struct HasImages {}

/// Type-level marker indicating the builder has no images yet
pub struct NoImages {}

/// Builder for creating print jobs with multiple images
///
/// Uses type-state pattern to ensure at least one image is added before building.
/// The builder starts in [`NoImages`] state and transitions to [`HasImages`] after
/// the first image is added. Only builders in the [`HasImages`] state can be built.
///
/// # Example
/// ```no_run
/// # use brother_ql::{media::Media, printjob::PrintJobBuilder};
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let img1 = image::open("label1.png")?;
/// let img2 = image::open("label2.png")?;
///
/// let job = PrintJobBuilder::new(Media::C62)
///     .add_label(img1)  // Transitions to HasImages state
///     .add_label(img2)  // Can add more images
///     .high_dpi(false)
///     .cut_behavior(brother_ql::printjob::CutBehavior::CutEvery(2))
///     .build()?;        // Only available in HasImages state
/// # Ok(())
/// # }
/// ```
pub struct PrintJobBuilder<State> {
    images: Vec<DynamicImage>,
    media: Media,
    no_copies: u8,
    high_dpi: bool,
    compressed: bool,
    quality_priority: bool,
    cut_behavior: CutBehavior,
    _state: PhantomData<State>,
}

impl PrintJobBuilder<NoImages> {
    /// Create a new print job builder for the specified media type
    ///
    /// Uses default settings (see [`PrintJob`] for defaults).
    #[must_use]
    pub fn new(media: Media) -> Self {
        Self {
            images: Vec::new(),
            media,
            no_copies: 1,
            high_dpi: false,
            compressed: false,
            quality_priority: true,
            cut_behavior: match MediaSettings::from(media).media_type {
                MediaType::Continuous => CutBehavior::CutEach,
                MediaType::DieCut => CutBehavior::CutAtEnd,
            },
            _state: PhantomData,
        }
    }

    /// Add the first image (transitions to `HasImages` state)
    #[must_use]
    pub fn add_label(mut self, img: DynamicImage) -> PrintJobBuilder<HasImages> {
        self.images.push(img);
        PrintJobBuilder {
            images: self.images,
            media: self.media,
            no_copies: self.no_copies,
            high_dpi: self.high_dpi,
            compressed: self.compressed,
            quality_priority: self.quality_priority,
            cut_behavior: match MediaSettings::from(self.media).media_type {
                MediaType::Continuous => CutBehavior::CutEach,
                MediaType::DieCut => CutBehavior::CutAtEnd,
            },
            _state: PhantomData,
        }
    }

    /// Add multiple images at once
    #[must_use]
    pub fn add_labels<I: IntoIterator<Item = DynamicImage>>(mut self, imgs: I) -> Self {
        self.images.extend(imgs);
        self
    }
}

impl PrintJobBuilder<HasImages> {
    /// Add another image to the print job
    #[must_use]
    pub fn add_label(mut self, img: DynamicImage) -> Self {
        self.images.push(img);
        self
    }

    /// Add multiple images to the print job
    #[must_use]
    pub fn add_labels<I: IntoIterator<Item = DynamicImage>>(mut self, imgs: I) -> Self {
        self.images.extend(imgs);
        self
    }

    /// Build the final print job
    ///
    /// # Errors
    /// Returns an error if any image dimensions don't match the media requirements.
    pub fn build(self) -> Result<PrintJob, PrintJobCreationError> {
        PrintJob::from_images(self.images, self.media)
    }
}

impl<State> PrintJobBuilder<State> {
    /// Set the number of copies/pages to print
    ///
    /// **Default**: 1
    #[must_use]
    pub fn copies(mut self, no_copies: u8) -> Self {
        self.no_copies = no_copies;
        self
    }

    /// Enable or disable high-DPI mode (600 DPI instead of 300 DPI)
    ///
    /// When enabled, your image must be double the resolution along its length.
    /// Generally not recommended unless you need maximum quality.
    ///
    /// **Default**: `false`
    #[must_use]
    pub fn high_dpi(mut self, high_dpi: bool) -> Self {
        self.high_dpi = high_dpi;
        self
    }

    /// Enable or disable TIFF compression
    ///
    /// **Note**: Compression is not yet implemented and this setting is currently ignored.
    ///
    /// **Default**: `false`
    #[must_use]
    pub fn compressed(mut self, compressed: bool) -> Self {
        self.compressed = compressed;
        self
    }

    /// Set whether the printer should prioritize print quality over speed
    ///
    /// Has no effect on two-color printing.
    ///
    /// **Default**: `true`
    #[must_use]
    pub fn quality_priority(mut self, quality_priority: bool) -> Self {
        self.quality_priority = quality_priority;
        self
    }

    /// Set the cutting behavior for the automatic cutter unit
    ///
    /// **Default**:
    /// - `CutEach` for continuous media
    /// - `CutAtEnd` for die-cut labels
    #[must_use]
    pub fn cut_behavior(mut self, cut_behavior: CutBehavior) -> Self {
        self.cut_behavior = cut_behavior;
        self
    }
}
