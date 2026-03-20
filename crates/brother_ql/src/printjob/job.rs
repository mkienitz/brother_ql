//! The core module for defining and compiling print data

use std::num::NonZeroU8;

use image::DynamicImage;

use crate::{
    commands::{
        ColorPower, DynamicCommandMode, RasterCommand, RasterCommands, VariousModeSettings,
    },
    error::PrintJobCreationError,
    media::{LabelType, Media},
    raster_image::RasterImage,
};

/// Cutting behavior for the automatic cutter unit
///
/// Controls when the printer's automatic blade cuts the media.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CutBehavior {
    /// Don't cut at all (manual cutting required)
    None,
    /// Cut after each page
    CutEach,
    /// Cut after every `n` pages
    ///
    /// If the total page count is not divisible by `n`,
    /// an additional cut will be added at the end.
    CutEvery(NonZeroU8),
    /// Cut only after the last page
    CutAtEnd,
}

/// Print job configuration
///
/// Represents a complete print job with image data and printer settings.
/// Created via the [`PrintJobBuilder`](crate::printjob::PrintJobBuilder), [`PrintJob::from_image`] or [`PrintJob::from_images`].
///
/// # Defaults
///
/// - **Copies**: 1
/// - **High DPI**: `false` (300 DPI)
/// - **Compression**: `false` (not yet implemented)
/// - **Quality priority**: `true`
/// - **Cut behavior**:
///   - `CutEach` for continuous media
///   - `CutAtEnd` for die-cut labels
#[derive(Clone, PartialEq, Debug)]
pub struct PrintJob {
    /// Number of copies to print
    pub(crate) no_copies: NonZeroU8,
    /// Rasterized image data ready for printing
    pub(crate) raster_images: Vec<RasterImage>,
    /// Media type (e.g., C62, D24)
    pub(crate) media: Media,
    /// Use high-DPI mode (600 DPI instead of 300 DPI)
    pub(crate) high_dpi: bool,
    /// Use TIFF compression (not yet implemented, currently ignored)
    pub(crate) compressed: bool,
    /// Prioritize print quality over speed (no effect on two-color printing)
    pub(crate) quality_priority: bool,
    /// Automatic cutter behavior
    pub(crate) cut_behavior: CutBehavior,
}

/// Internal representation of print job parts
pub(crate) struct PrintJobParts {
    /// Printer initialization commands
    pub preamble: RasterCommands,
    /// Commands for each page to print
    pub page_data: Vec<RasterCommands>,
}

impl PrintJob {
    /// Create a print job from a single image
    ///
    /// Uses default settings (see [`PrintJob`] for defaults).
    ///
    /// # Errors
    /// Returns an error if the image dimensions don't match the media requirements.
    pub fn from_image(image: DynamicImage, media: Media) -> Result<Self, PrintJobCreationError> {
        Self::from_images(vec![image], media)
    }

    /// Create a print job from multiple images
    ///
    /// Each image will be printed as a separate label.
    /// Uses default settings (see [`PrintJob`] for defaults).
    ///
    /// # Errors
    /// Returns an error if any image dimensions don't match the media requirements.
    pub fn from_images(
        images: Vec<DynamicImage>,
        media: Media,
    ) -> Result<Self, PrintJobCreationError> {
        let raster_images = images
            .into_iter()
            .map(|img| RasterImage::new(img, media))
            .collect::<Result<Vec<RasterImage>, _>>()?;

        Ok(Self {
            no_copies: NonZeroU8::MIN,
            raster_images,
            media,
            high_dpi: false,
            compressed: false,
            quality_priority: true,
            cut_behavior: match media.label_type() {
                LabelType::Continuous => CutBehavior::CutEach,
                LabelType::DieCut => CutBehavior::CutAtEnd,
            },
        })
    }

    pub(crate) fn page_count(&self) -> usize {
        self.no_copies.get() as usize * self.raster_images.len()
    }

    pub(crate) fn into_parts(self) -> PrintJobParts {
        let mut page_data = Vec::new();

        for copy_no in 0..self.no_copies.get() {
            for (img_idx, raster_image) in self.raster_images.iter().enumerate() {
                use RasterCommand as RC;
                let page_no = copy_no as usize * self.raster_images.len() + img_idx;

                let mut page_commands = RasterCommands::default();

                page_commands.add(RC::SwitchDynamicCommandMode {
                    command_mode: DynamicCommandMode::Raster,
                });
                page_commands.add(RC::SwitchAutomaticStatusNotificationMode { notify: true });
                page_commands.add(RC::PrintInformation {
                    media: self.media,
                    quality_priority: match *raster_image {
                        RasterImage::Monochrome { .. } => self.quality_priority,
                        RasterImage::TwoColor { .. } => false,
                    },
                    recovery_on: true,
                    #[allow(clippy::cast_possible_truncation)]
                    no_lines: raster_image.height() as u32,
                    first_page: page_no == 0,
                });
                page_commands.add(RC::VariousMode(VariousModeSettings {
                    auto_cut: self.cut_behavior != CutBehavior::None,
                }));
                match self.cut_behavior {
                    CutBehavior::CutEvery(n) => {
                        page_commands.add(RC::SpecifyPageNumber { cut_every: n.get() });
                    }
                    CutBehavior::CutEach => {
                        page_commands.add(RC::SpecifyPageNumber { cut_every: 1 });
                    }
                    _ => {}
                }
                page_commands.add(RC::ExpandedMode {
                    two_color: self.media.supports_color(),
                    cut_at_end: match self.cut_behavior {
                        CutBehavior::CutAtEnd => true,
                        CutBehavior::CutEvery(n) => !self.no_copies.get().is_multiple_of(n.get()),
                        _ => false,
                    },
                    high_dpi: self.high_dpi,
                });
                page_commands.add(RC::SpecifyMarginAmount {
                    margin_size: match self.media.label_type() {
                        LabelType::Continuous => 35,
                        LabelType::DieCut => 0,
                    },
                });
                page_commands.add(RC::SelectCompressionMode {
                    // TODO: Add support for compression
                    tiff_compression: false,
                });
                match raster_image {
                    RasterImage::Monochrome { black_layer } => {
                        for line in black_layer {
                            page_commands.add(RC::RasterGraphicsTransfer {
                                data: line.to_vec(),
                            });
                        }
                    }
                    RasterImage::TwoColor {
                        black_layer,
                        red_layer,
                    } => black_layer.iter().zip(red_layer.iter()).for_each(
                        |(black_line, red_line)| {
                            page_commands.add(RC::TwoColorRasterGraphicsTransfer {
                                data: black_line.to_vec(),
                                color_power: ColorPower::HighEnergy,
                            });
                            page_commands.add(RC::TwoColorRasterGraphicsTransfer {
                                data: red_line.to_vec(),
                                color_power: ColorPower::LowEnergy,
                            });
                        },
                    ),
                }
                if page_no == self.page_count() - 1 {
                    page_commands.add(RC::PrintWithFeed);
                } else {
                    page_commands.add(RC::Print);
                }
                page_data.push(page_commands);
            }
        }

        PrintJobParts {
            preamble: RasterCommands::create_preamble(),
            page_data,
        }
    }

    /// Compile the print job into raster command bytes
    ///
    /// Converts the image and settings into the binary format understood
    /// by Brother QL printers.
    /// This is useful if you want to send the resulting bytes directly to
    /// the printer via a TCP connection,
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::printjob::PrintJob;
    /// # use brother_ql::media::Media;
    /// # use std::fs::File;
    /// # use std::io::Write;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let image = image::open("label.png")?;
    /// let job = PrintJob::from_image(image, Media::C62)?;
    /// let bytes = job.compile();
    /// # // Save to file
    /// # let mut file = File::create("output.bin")?;
    /// # file.write_all(&bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn compile(self) -> Vec<u8> {
        let parts = self.into_parts();
        let mut bytes = Vec::new();
        bytes.append(&mut parts.preamble.build());
        parts
            .page_data
            .into_iter()
            .for_each(|p| bytes.append(&mut p.build()));
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printjob::PrintJobBuilder;
    use image::{DynamicImage, RgbImage};

    /// Create a synthetic white image matching the given media dimensions
    fn test_image(media: Media) -> DynamicImage {
        let width = media.width_dots();
        let height = media.length_dots().unwrap_or(100);
        DynamicImage::ImageRgb8(RgbImage::from_pixel(
            width,
            height,
            image::Rgb([255, 255, 255]),
        ))
    }

    #[test]
    fn builder_preserves_all_settings() {
        let n3 = NonZeroU8::new(3).unwrap();
        let job = PrintJobBuilder::new(Media::C62)
            .copies(NonZeroU8::new(5).unwrap())
            .high_dpi(true)
            .compressed(true)
            .quality_priority(false)
            .cut_behavior(CutBehavior::CutEvery(n3))
            .add_label(test_image(Media::C62))
            .build()
            .unwrap();

        assert_eq!(job.no_copies.get(), 5);
        assert!(job.high_dpi);
        assert!(job.compressed);
        assert!(!job.quality_priority);
        assert_eq!(job.cut_behavior, CutBehavior::CutEvery(n3));
    }

    #[test]
    fn builder_settings_after_add_label_preserved() {
        let job = PrintJobBuilder::new(Media::C62)
            .add_label(test_image(Media::C62))
            .copies(NonZeroU8::new(3).unwrap())
            .high_dpi(true)
            .quality_priority(false)
            .cut_behavior(CutBehavior::None)
            .build()
            .unwrap();

        assert_eq!(job.no_copies.get(), 3);
        assert!(job.high_dpi);
        assert!(!job.quality_priority);
        assert_eq!(job.cut_behavior, CutBehavior::None);
    }

    #[test]
    fn add_label_does_not_reset_cut_behavior() {
        let job = PrintJobBuilder::new(Media::C62)
            .cut_behavior(CutBehavior::CutAtEnd)
            .add_label(test_image(Media::C62))
            .build()
            .unwrap();

        assert_eq!(job.cut_behavior, CutBehavior::CutAtEnd);
    }

    #[test]
    fn cut_every_nonzero_cannot_be_zero() {
        assert!(NonZeroU8::new(0).is_none());
    }

    #[test]
    fn from_images_uses_defaults() {
        let job = PrintJob::from_image(test_image(Media::C62), Media::C62).unwrap();
        assert_eq!(job.no_copies.get(), 1);
        assert!(!job.high_dpi);
        assert!(!job.compressed);
        assert!(job.quality_priority);
        assert_eq!(job.cut_behavior, CutBehavior::CutEach); // Continuous → CutEach
    }

    #[test]
    fn die_cut_defaults_to_cut_at_end() {
        let media = Media::D29x90;
        let job = PrintJob::from_image(test_image(media), media).unwrap();
        assert_eq!(job.cut_behavior, CutBehavior::CutAtEnd);
    }

    #[test]
    fn dimension_mismatch_error() {
        let wrong_image =
            DynamicImage::ImageRgb8(RgbImage::from_pixel(100, 100, image::Rgb([255, 255, 255])));
        let result = PrintJob::from_image(wrong_image, Media::C62);
        assert!(result.is_err());
    }

    #[test]
    fn page_count_with_copies() {
        let job = PrintJobBuilder::new(Media::C62)
            .copies(NonZeroU8::new(3).unwrap())
            .add_label(test_image(Media::C62))
            .add_label(test_image(Media::C62))
            .build()
            .unwrap();

        assert_eq!(job.page_count(), 6); // 3 copies × 2 images
    }

    #[test]
    fn compile_produces_nonempty_output() {
        let job = PrintJob::from_image(test_image(Media::C62), Media::C62).unwrap();
        let bytes = job.compile();
        assert!(!bytes.is_empty());
        // Should start with the invalidate command (400 zero bytes)
        assert!(bytes.len() > 400);
        assert!(bytes[..400].iter().all(|&b| b == 0));
    }

    #[test]
    fn into_parts_page_count_matches() {
        let job = PrintJobBuilder::new(Media::C62)
            .copies(NonZeroU8::new(2).unwrap())
            .add_label(test_image(Media::C62))
            .add_label(test_image(Media::C62))
            .build()
            .unwrap();

        let expected_pages = job.page_count();
        let parts = job.into_parts();
        assert_eq!(parts.page_data.len(), expected_pages);
    }
}
