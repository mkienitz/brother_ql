//! Print job creation and compilation

use wasm_bindgen::prelude::*;
use image::DynamicImage;

use crate::{
    commands::{ColorPower, DynamicCommandMode, RasterCommand, RasterCommands, VariousModeSettings},
    error::PrintJobError,
    media::{LabelType, Media},
    raster_image::RasterImage,
};

/// Cutting behavior for the automatic cutter unit
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum CutBehavior {
    /// Don't cut at all (manual cutting required)
    None,
    /// Cut after each page
    CutEach,
    /// Cut only after the last page
    CutAtEnd,
}

impl CutBehavior {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "None" => Some(CutBehavior::None),
            "CutEach" => Some(CutBehavior::CutEach),
            "CutAtEnd" => Some(CutBehavior::CutAtEnd),
            _ => None,
        }
    }
}

/// Print job configuration
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PrintJob {
    no_copies: u8,
    raster_images: Vec<RasterImage>,
    media: Media,
    high_dpi: bool,
    quality_priority: bool,
    cut_behavior: CutBehavior,
}

pub(crate) struct PrintJobParts {
    pub preamble: RasterCommands,
    pub page_data: Vec<RasterCommands>,
}

#[wasm_bindgen]
impl PrintJob {
    /// Create a print job from image bytes (PNG or JPEG)
    #[wasm_bindgen(js_name = fromImageBytes)]
    pub fn from_image_bytes(data: &[u8], media_name: &str) -> Result<PrintJob, PrintJobError> {
        let media = Media::from_str(media_name)
            .ok_or_else(|| PrintJobError::new(format!("Unknown media type: {}", media_name)))?;
        let img = image::load_from_memory(data)?;
        Self::from_dynamic_image(img, media)
    }

    /// Create a print job from raw RGBA pixel data
    #[wasm_bindgen(js_name = fromRgbaPixels)]
    pub fn from_rgba_pixels(
        pixels: &[u8],
        width: u32,
        height: u32,
        media_name: &str,
    ) -> Result<PrintJob, PrintJobError> {
        let media = Media::from_str(media_name)
            .ok_or_else(|| PrintJobError::new(format!("Unknown media type: {}", media_name)))?;
        let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
            .ok_or_else(|| PrintJobError::new("Invalid pixel data dimensions"))?;
        Self::from_dynamic_image(DynamicImage::ImageRgba8(img), media)
    }

    fn from_dynamic_image(img: DynamicImage, media: Media) -> Result<Self, PrintJobError> {
        let raster_image = RasterImage::new(img, media)?;
        
        Ok(Self {
            no_copies: 1,
            raster_images: vec![raster_image],
            media,
            high_dpi: false,
            quality_priority: true,
            cut_behavior: match media.label_type() {
                LabelType::Continuous => CutBehavior::CutEach,
                LabelType::DieCut => CutBehavior::CutAtEnd,
            },
        })
    }

    /// Set the number of copies to print
    #[wasm_bindgen(js_name = setCopies)]
    pub fn set_copies(&mut self, copies: u8) {
        self.no_copies = copies;
    }

    /// Set high DPI mode (600 DPI instead of 300 DPI)
    #[wasm_bindgen(js_name = setHighDpi)]
    pub fn set_high_dpi(&mut self, high_dpi: bool) {
        self.high_dpi = high_dpi;
    }

    /// Set quality priority mode
    #[wasm_bindgen(js_name = setQualityPriority)]
    pub fn set_quality_priority(&mut self, quality: bool) {
        self.quality_priority = quality;
    }

    /// Set cut behavior
    #[wasm_bindgen(js_name = setCutBehavior)]
    pub fn set_cut_behavior(&mut self, behavior: &str) {
        if let Some(b) = CutBehavior::from_str(behavior) {
            self.cut_behavior = b;
        }
    }

    /// Get the media type for this job
    #[wasm_bindgen(getter, js_name = mediaType)]
    pub fn get_media(&self) -> String {
        format!("{:?}", self.media)
    }

    /// Get the number of pages in this job
    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> usize {
        self.no_copies as usize * self.raster_images.len()
    }

    /// Compile the print job into raster command bytes
    #[wasm_bindgen]
    pub fn compile(&self) -> Vec<u8> {
        let parts = self.clone().into_parts();
        let mut bytes = Vec::new();
        bytes.append(&mut parts.preamble.build());
        parts
            .page_data
            .into_iter()
            .for_each(|p| bytes.append(&mut p.build()));
        bytes
    }
}

impl PrintJob {
    fn into_parts(self) -> PrintJobParts {
        let mut page_data = Vec::new();

        for copy_no in 0..self.no_copies {
            for (img_idx, raster_image) in self.raster_images.clone().into_iter().enumerate() {
                use RasterCommand as RC;
                let page_no = copy_no as usize * self.raster_images.len() + img_idx;

                let mut page_commands = RasterCommands::default();

                page_commands.add(RC::SwitchDynamicCommandMode {
                    command_mode: DynamicCommandMode::Raster,
                });
                page_commands.add(RC::SwitchAutomaticStatusNotificationMode { notify: true });
                page_commands.add(RC::PrintInformation {
                    media: self.media,
                    quality_priority: match &raster_image {
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
                    CutBehavior::CutEach => {
                        page_commands.add(RC::SpecifyPageNumber { cut_every: 1 });
                    }
                    _ => {}
                }
                page_commands.add(RC::ExpandedMode {
                    two_color: self.media.supports_color(),
                    cut_at_end: matches!(self.cut_behavior, CutBehavior::CutAtEnd),
                    high_dpi: self.high_dpi,
                });
                page_commands.add(RC::SpecifyMarginAmount {
                    margin_size: match self.media.label_type() {
                        LabelType::Continuous => 35,
                        LabelType::DieCut => 0,
                    },
                });
                page_commands.add(RC::SelectCompressionMode {
                    tiff_compression: false,
                });
                match &raster_image {
                    RasterImage::Monochrome { black_layer } => {
                        for line in black_layer {
                            page_commands.add(RC::RasterGraphicsTransfer {
                                data: line.to_vec(),
                            });
                        }
                    }
                    RasterImage::TwoColor { black_layer, red_layer } => {
                        black_layer.iter().zip(red_layer.iter()).for_each(
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
                        );
                    }
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
}
