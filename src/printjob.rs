//! The core module for defining and compiling print data
use image::DynamicImage;

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    commands::{
        ColorPower, DynamicCommandMode, RasterCommand, RasterCommands, VariousModeSettings,
    },
    error::PrintJobError,
    media::{LengthInfo, Media, MediaSettings, MediaType},
    raster_image::RasterImage,
};

/// Cutting behavior for the automatic cutter unit
///
/// Controls when the printer's automatic blade cuts the media.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum CutBehavior {
    /// Don't cut at all (manual cutting required)
    None,
    /// Cut after each page
    CutEach,
    /// Cut after every `n` pages
    ///
    /// If the total page count is not divisible by `n`,
    /// an additional cut will be added at the end.
    CutEvery(u8),
    /// Cut only after the last page
    CutAtEnd,
}

/// Print job configuration with builder pattern
///
/// Create a print job using [`PrintJob::new`] with sensible defaults,
/// then customize settings using builder methods before compiling or printing.
///
/// # Example
/// ```no_run
/// # use brother_ql::printjob::{PrintJob, CutBehavior};
/// # use brother_ql::media::Media;
/// # use image::DynamicImage;
/// # fn example(image: DynamicImage) -> Result<(), brother_ql::error::PrintJobError> {
/// let job = PrintJob::new(image, Media::C62)?
///     .page_count(3)
///     .high_dpi(false)
///     .cut_behavior(CutBehavior::CutEach);
///
/// let bytes = job.compile();
/// // Send bytes to printer...
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct PrintJob {
    /// The amount of labels to print
    pub(crate) page_count: u8,
    /// The rasterized image data
    pub(crate) raster_image: RasterImage,
    /// Height of the image in pixels
    pub(crate) height: u32,
    /// The paper type to use for the print job
    pub(crate) media: Media,
    /// Whether or not to use high-DPI mode. The image file will need to be double the resolution along
    /// its length. Probably not recommended.
    pub(crate) high_dpi: bool,
    /// Whether or not to use compression
    ///
    /// NOTE:
    /// Currently not respected, defaults to [false]
    pub(crate) compressed: bool,
    /// Whether or not the printer gives priority to print quality. Has no effect on two-color
    /// printing.
    pub(crate) quality_priority: bool,
    /// The selected behavior for the automatic cutter unit
    pub(crate) cut_behavior: CutBehavior,
}

pub(crate) struct PrintJobParts {
    pub preamble: RasterCommands,
    pub page_data: Vec<RasterCommands>,
}

impl PrintJob {
    /// Create a new print job with sensible defaults
    ///
    /// # Parameters
    /// - `image`: The image to print (from the [`image`] crate)
    /// - `media`: The media/label type to use
    ///
    /// # Defaults
    /// - **Page count**: 1
    /// - **High DPI**: `false` (standard 300 DPI)
    /// - **Compressed**: `false` (compression not yet supported)
    /// - **Quality priority**: `true`
    /// - **Cut behavior**:
    ///   - `CutEach` for continuous media
    ///   - `CutAtEnd` for die-cut labels
    ///
    /// # Errors
    /// Returns an error if the image dimensions don't match the media requirements.
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::printjob::PrintJob;
    /// # use brother_ql::media::Media;
    /// # fn example() -> Result<(), brother_ql::error::PrintJobError> {
    /// let image = image::open("label.png")?;
    /// let job = PrintJob::new(image, Media::C62)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(image: DynamicImage, media: Media) -> Result<Self, PrintJobError> {
        let media_settings = MediaSettings::new(media);
        let height = image.height();
        let raster_image = RasterImage::new(image, media_settings)?;

        Ok(Self {
            page_count: 1,
            raster_image,
            height,
            media,
            high_dpi: false,
            compressed: false,
            quality_priority: true,
            cut_behavior: match media_settings.media_type {
                MediaType::Continuous => CutBehavior::CutEach,
                MediaType::DieCut => CutBehavior::CutAtEnd,
            },
        })
    }

    /// Set the number of copies/pages to print
    ///
    /// **Default**: 1
    #[must_use]
    pub fn page_count(mut self, page_count: u8) -> Self {
        self.page_count = page_count;
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

    pub(crate) fn into_parts(self) -> PrintJobParts {
        use RasterCommand as RC;

        let media_settings = MediaSettings::new(self.media);

        let mut page_data = Vec::new();

        for page_no in 0..self.page_count {
            let mut page_commands = RasterCommands::default();

            page_commands.add(RC::SwitchDynamicCommandMode {
                command_mode: DynamicCommandMode::Raster,
            });
            page_commands.add(RC::SwitchAutomaticStatusNotificationMode { notify: true });
            page_commands.add(RC::PrintInformation {
                media_settings,
                quality_priority: match self.raster_image {
                    RasterImage::Monochrome { .. } => self.quality_priority,
                    RasterImage::TwoColor { .. } => false,
                },
                recovery_on: true,
                no_lines: self.height,
                first_page: page_no == 0,
            });
            page_commands.add(RC::VariousMode(VariousModeSettings {
                auto_cut: self.cut_behavior != CutBehavior::None,
            }));
            match self.cut_behavior {
                CutBehavior::CutEvery(n) => {
                    page_commands.add(RC::SpecifyPageNumber { cut_every: n });
                }
                CutBehavior::CutEach => {
                    page_commands.add(RC::SpecifyPageNumber { cut_every: 1 });
                }
                _ => {}
            }
            page_commands.add(RC::ExpandedMode {
                two_color: media_settings.color,
                cut_at_end: match self.cut_behavior {
                    CutBehavior::CutAtEnd => true,
                    CutBehavior::CutEvery(n) => !self.page_count.is_multiple_of(n),
                    _ => false,
                },
                high_dpi: self.high_dpi,
            });
            page_commands.add(RC::SpecifyMarginAmount {
                margin_size: match media_settings.length_info {
                    LengthInfo::Endless => 35,
                    LengthInfo::Fixed { .. } => 0,
                },
            });
            page_commands.add(RC::SelectCompressionMode {
                // TODO: Add support for compression
                tiff_compression: false,
            });
            match &self.raster_image {
                RasterImage::Monochrome { black_layer } => black_layer.iter().for_each(|line| {
                    page_commands.add(RC::RasterGraphicsTransfer {
                        data: line.to_vec(),
                    });
                }),
                RasterImage::TwoColor {
                    black_layer,
                    red_layer,
                } => black_layer
                    .iter()
                    .zip(red_layer.iter())
                    .for_each(|(black_line, red_line)| {
                        page_commands.add(RC::TwoColorRasterGraphicsTransfer {
                            data: black_line.to_vec(),
                            color_power: ColorPower::HighEnergy,
                        });
                        page_commands.add(RC::TwoColorRasterGraphicsTransfer {
                            data: red_line.to_vec(),
                            color_power: ColorPower::LowEnergy,
                        });
                    }),
            }
            if page_no == self.page_count - 1 {
                page_commands.add(RC::PrintWithFeed);
            } else {
                page_commands.add(RC::Print);
            }
            page_data.push(page_commands);
        }

        PrintJobParts {
            preamble: RasterCommands::create_preamble(),
            page_data,
        }
    }

    /// Compile the print job into raster command bytes
    ///
    /// Converts the image and settings into the binary format understood
    /// by Brother QL printers. The resulting bytes can be sent directly to
    /// the printer via USB, network, or saved to a file.
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::printjob::PrintJob;
    /// # use brother_ql::media::Media;
    /// # use std::fs::File;
    /// # use std::io::Write;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let image = image::open("label.png")?;
    /// let job = PrintJob::new(image, Media::C62)?;
    /// let bytes = job.compile();
    ///
    /// // Save to file
    /// let mut file = File::create("output.bin")?;
    /// file.write_all(&bytes)?;
    ///
    /// // Or send via network: nc printer-ip 9100 < output.bin
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

    /// Check if a specific printer model can handle this print job
    ///
    /// Validates printer compatibility before printing:
    /// - The printer supports the specified media type
    /// - The printer supports required features (e.g., color printing)
    /// - Any other printer-specific requirements are met
    ///
    /// **Note**: This method is not yet implemented.
    ///
    /// # Errors
    /// Will return an error if the printer model is incompatible with the print job settings.
    pub fn check_printer_compatibility(
        &self,
        _model: crate::printer::PrinterModel,
    ) -> Result<(), PrintJobError> {
        todo!("Implement printer compatibility checks")
    }
}
