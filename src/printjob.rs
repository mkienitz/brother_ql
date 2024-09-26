//! The core module for defining and compiling print data
use image::DynamicImage;

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    commands::{ColorPower, CommandBuilder, DynamicCommandMode, RasterCommand},
    error::BQLError,
    media::{Media, MediaSettings, MediaType},
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
pub struct PrintJob {
    /// The amount of replicas to print
    pub no_pages: u8,
    /// The image to print. The required type is [DynamicImage] from the [image] crate.
    pub image: DynamicImage,
    /// The paper type to use for the print job
    pub media: Media,
    /// Whether or not to use high-DPI mode. The image file will need to be double the resolution along
    /// its length. Probably not recommended.
    pub high_dpi: bool,
    /// Whether or not to use compression
    ///
    /// NOTE:
    /// Currently not respected, defaults to [false]
    pub compressed: bool,
    /// Whether or not the printer gives priority to print quality. Has no effect on two-color
    /// printing.
    pub quality_priority: bool,
    /// The selected behavior for the automatic cutter unit
    pub cut_behaviour: CutBehavior,
}

impl PrintJob {
    /// Create a compiled print job from the specified settings.
    ///
    /// The resulting [`Vec<u8>`] can be directly send to your printer's serial or network interface,
    /// e.g. using `nc`.
    pub fn compile(self) -> Result<Vec<u8>, BQLError> {
        let media_settings = MediaSettings::new(&self.media);
        let height = self.image.height();
        let raster_image = RasterImage::new(self.image, &media_settings)?;

        let mut commands = CommandBuilder::default();

        use RasterCommand::*;
        commands.add(Invalidate);
        commands.add(Initialize);
        for page_no in 0..self.no_pages {
            commands.add(SwitchDynamicCommandMode {
                command_mode: DynamicCommandMode::Raster,
            });
            commands.add(SwitchAutomaticStatusNotificationMode { notify: false });
            commands.add(PrintInformation {
                media_settings,
                quality_priority: match raster_image {
                    RasterImage::Monochrome { .. } => self.quality_priority,
                    RasterImage::TwoColor { .. } => false,
                },
                recovery_on: true,
                no_lines: height,
                first_page: page_no == 0,
            });
            commands.add(VariousMode {
                auto_cut: self.cut_behaviour != CutBehavior::None,
            });
            match self.cut_behaviour {
                CutBehavior::CutEvery(n) => {
                    commands.add(SpecifyPageNumber { cut_every: n });
                }
                CutBehavior::CutEach => {
                    commands.add(SpecifyPageNumber { cut_every: 1 });
                }
                _ => {}
            }
            commands.add(ExpandedMode {
                two_color: media_settings.color,
                cut_at_end: match self.cut_behaviour {
                    CutBehavior::CutAtEnd => true,
                    CutBehavior::CutEvery(n) => self.no_pages % n != 0,
                    _ => false,
                },
                high_dpi: self.high_dpi,
            });
            commands.add(SpecifyMarginAmount {
                margin_size: match media_settings.media_type {
                    MediaType::Continuous => 35,
                    MediaType::DieCut { .. } => 0,
                },
            });
            commands.add(SelectCompressionMode {
                // TODO: Add support for compression
                tiff_compression: false,
            });
            match &raster_image {
                RasterImage::Monochrome { black_layer } => black_layer.iter().for_each(|line| {
                    commands.add(RasterGraphicsTransfer {
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
                        commands.add(TwoColorRasterGraphicsTransfer {
                            data: black_line.to_vec(),
                            color_power: ColorPower::HighEnergy,
                        });
                        commands.add(TwoColorRasterGraphicsTransfer {
                            data: red_line.to_vec(),
                            color_power: ColorPower::LowEnergy,
                        })
                    }),
            };
            if page_no == self.no_pages - 1 {
                commands.add(PrintWithFeed)
            } else {
                commands.add(Print)
            };
        }
        Ok(commands.build())
    }
}
