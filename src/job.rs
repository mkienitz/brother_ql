use std::error::Error;

use image::DynamicImage;

use crate::{
    commands::*,
    image::RasterImage,
    media::{Media, MediaSettings, MediaType},
};

#[derive(PartialEq)]
pub enum CutBehavior {
    None,
    CutEach,
    CutEvery(u8),
    CutAtEnd,
}

pub struct PrintJob {
    pub no_pages: usize,
    pub image: DynamicImage,
    pub media: Media,
    pub high_dpi: bool,
    pub compressed: bool,
    pub quality_priority: bool,
    pub cut_behaviour: CutBehavior,
}

impl PrintJob {
    pub fn render(self) -> Result<Vec<u8>, Box<dyn Error>> {
        let media_settings = MediaSettings::new(&self.media);
        let raster_image = RasterImage::new(self.image, &media_settings)?;

        let mut commands = CommandBuilder::default();

        use RasterCommand::*;
        commands.add(Invalidate);
        commands.add(Initialize);
        for page_no in 0..self.no_pages {
            commands.add(SwitchDynamicCommandMode {
                command_mode: DynamicCommandMode::Raster,
            });
            // commands.add(SwitchAutomaticStatusNotificationMode { notify: true });
            commands.add(PrintInformation {
                media_settings,
                quality_priority: self.quality_priority,
                recovery_on: true,
                no_lines: raster_image.no_lines() as u32,
                first_page: page_no == 0,
            });
            commands.add(VariousMode {
                auto_cut: self.cut_behaviour != CutBehavior::None,
            });
            if let CutBehavior::CutEvery(n) = self.cut_behaviour {
                commands.add(SpecifyPageNumber { cut_every: n });
            }
            commands.add(ExpandedMode {
                two_color: media_settings.color,
                cut_at_end: self.cut_behaviour == CutBehavior::CutAtEnd,
                high_dpi: self.high_dpi,
            });
            commands.add(SpecifyMarginAmount {
                margin_size: match media_settings.media_type {
                    MediaType::Continuous => 35,
                    MediaType::DieCut { .. } => 0,
                },
            });
            commands.add(SelectCompressionMode {
                tiff_compression: self.compressed,
            });
            match &raster_image {
                RasterImage::Monochrome { black_channel } => {
                    black_channel.iter().for_each(|line| {
                        commands.add(RasterGraphicsTransfer {
                            data: line.to_vec(),
                        })
                    })
                }
                RasterImage::TwoColor {
                    black_channel,
                    red_channel,
                } => black_channel.iter().zip(red_channel.iter()).for_each(
                    |(black_line, red_line)| {
                        commands.add(TwoColorRasterGraphicsTransfer {
                            data: black_line.to_vec(),
                            color_power: ColorPower::HighEnergy,
                        });
                        commands.add(TwoColorRasterGraphicsTransfer {
                            data: red_line.to_vec(),
                            color_power: ColorPower::LowEnergy,
                        })
                    },
                ),
            };
            if page_no == self.no_pages - 1 {
                commands.add(PrintWithFeed {})
            } else {
                commands.add(Print {})
            };
        }
        Ok(commands.build())
    }
}
