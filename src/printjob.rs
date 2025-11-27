//! The core module for defining and compiling print data
use std::fmt;

use image::DynamicImage;

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    commands::{
        ColorPower, DynamicCommandMode, RasterCommand, RasterCommands, VariousModeSettings,
    },
    error::BQLError,
    media::{LengthInfo, Media, MediaSettings, MediaType},
    raster_image::RasterImage,
};

/// This enum specifies the cutting behavior for the generated print job.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum CutBehavior {
    /// Don't cut at all
    None,
    /// Cut after each page
    CutEach,
    /// Cut after every `n` pages. If the `no_pages` attribute of [PrintJob] is not divisible by
    /// `n` there will be added a cut at the end of the print job.
    CutEvery(u8),
    /// Cut after the last page
    CutAtEnd,
}

/// This struct defines the general settings for the generated print job.
#[derive(Clone, PartialEq, custom_debug::Debug)]
pub struct PrintJob {
    /// The amount of labels to print
    pub(crate) page_count: u8,
    /// The image to print. The required type is [DynamicImage] from the [image] crate.
    #[debug(with = debug_summarize_image)]
    pub(crate) image: DynamicImage,
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

fn debug_summarize_image(img: &DynamicImage, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<DynamicImage {}x{} px>", img.width(), img.height())
}

pub(crate) struct PrintJobParts {
    pub preamble: RasterCommands,
    pub page_data: Vec<RasterCommands>,
}

impl PrintJob {
    /// Create a new print job with defaults
    /// TODO: specify defaults in docs
    pub fn new(image: DynamicImage, media: Media) -> Result<Self, BQLError> {
        // TODO: Validate image/media compatibility
        Ok(Self {
            page_count: 1,
            image,
            media,
            high_dpi: false,
            compressed: false,
            quality_priority: true,
            cut_behavior: match MediaSettings::new(&media).media_type {
                MediaType::Continuous => CutBehavior::CutEach,
                MediaType::DieCut => CutBehavior::CutAtEnd,
            },
        })
    }

    /// Set the number of pages to print
    pub fn page_count(mut self, page_count: u8) -> Self {
        self.page_count = page_count;
        self
    }

    /// Enable or disable high-DPI mode
    pub fn high_dpi(mut self, high_dpi: bool) -> Self {
        self.high_dpi = high_dpi;
        self
    }

    /// Enable or disable compression
    pub fn compressed(mut self, compressed: bool) -> Self {
        self.compressed = compressed;
        self
    }

    /// Set whether the printer should prioritize print quality
    pub fn quality_priority(mut self, quality_priority: bool) -> Self {
        self.quality_priority = quality_priority;
        self
    }

    /// Set the cutting behavior for the automatic cutter unit
    pub fn cut_behavior(mut self, cut_behavior: CutBehavior) -> Self {
        self.cut_behavior = cut_behavior;
        self
    }

    pub(crate) fn into_parts(self) -> Result<PrintJobParts, BQLError> {
        let media_settings = MediaSettings::new(&self.media);
        let height = self.image.height();
        let raster_image = RasterImage::new(self.image, &media_settings)?;

        let mut page_data = Vec::new();

        for page_no in 0..self.page_count {
            let mut page_commands = RasterCommands::default();
            use RasterCommand::*;

            page_commands.add(SwitchDynamicCommandMode {
                command_mode: DynamicCommandMode::Raster,
            });
            page_commands.add(SwitchAutomaticStatusNotificationMode { notify: true });
            page_commands.add(PrintInformation {
                media_settings,
                quality_priority: match raster_image {
                    RasterImage::Monochrome { .. } => self.quality_priority,
                    RasterImage::TwoColor { .. } => false,
                },
                recovery_on: true,
                no_lines: height,
                first_page: page_no == 0,
            });
            page_commands.add(VariousMode(VariousModeSettings {
                auto_cut: self.cut_behavior != CutBehavior::None,
            }));
            match self.cut_behavior {
                CutBehavior::CutEvery(n) => {
                    page_commands.add(SpecifyPageNumber { cut_every: n });
                }
                CutBehavior::CutEach => {
                    page_commands.add(SpecifyPageNumber { cut_every: 1 });
                }
                _ => {}
            }
            page_commands.add(ExpandedMode {
                two_color: media_settings.color,
                cut_at_end: match self.cut_behavior {
                    CutBehavior::CutAtEnd => true,
                    CutBehavior::CutEvery(n) => !self.page_count.is_multiple_of(n),
                    _ => false,
                },
                high_dpi: self.high_dpi,
            });
            page_commands.add(SpecifyMarginAmount {
                margin_size: match media_settings.length_info {
                    LengthInfo::Endless => 35,
                    LengthInfo::Fixed { .. } => 0,
                },
            });
            page_commands.add(SelectCompressionMode {
                // TODO: Add support for compression
                tiff_compression: false,
            });
            match &raster_image {
                RasterImage::Monochrome { black_layer } => black_layer.iter().for_each(|line| {
                    page_commands.add(RasterGraphicsTransfer {
                        data: line.to_vec(),
                    })
                }),
                RasterImage::TwoColor {
                    black_layer,
                    red_layer,
                } => black_layer
                    .iter()
                    .zip(red_layer.iter())
                    .for_each(|(black_line, red_line)| {
                        page_commands.add(TwoColorRasterGraphicsTransfer {
                            data: black_line.to_vec(),
                            color_power: ColorPower::HighEnergy,
                        });
                        page_commands.add(TwoColorRasterGraphicsTransfer {
                            data: red_line.to_vec(),
                            color_power: ColorPower::LowEnergy,
                        })
                    }),
            };
            if page_no == self.page_count - 1 {
                page_commands.add(PrintWithFeed)
            } else {
                page_commands.add(Print)
            };
            page_data.push(page_commands);
        }

        Ok(PrintJobParts {
            preamble: RasterCommands::create_preamble(),
            page_data,
        })
    }

    /// Create a compiled print job from the specified settings.
    ///
    /// The resulting [`Vec<u8>`] can be directly send to your printer's serial or network interface,
    /// e.g. using `nc`.
    pub fn compile(self) -> Result<Vec<u8>, BQLError> {
        let parts = self.into_parts()?;
        let mut bytes = Vec::new();
        bytes.append(&mut parts.preamble.build());
        parts
            .page_data
            .into_iter()
            .for_each(|p| bytes.append(&mut p.build()));
        Ok(bytes)
    }

    /// Check if a specific printer model can handle this print job
    ///
    /// This validates that:
    /// - The printer supports the media type
    /// - The printer supports required features (e.g., color printing)
    /// - Any other printer-specific requirements are met
    pub fn check_printer_compatibility(
        &self,
        _model: crate::printer::PrinterModel,
    ) -> Result<(), BQLError> {
        todo!("Implement printer compatibility checks")
    }
}
