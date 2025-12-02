use std::fmt;

use custom_debug::Debug as CustomDebug;
use image::{
    imageops::{self, BiLevel},
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, Rgb,
};

use crate::{error::PrintJobCreationError, media::Media};

type RasterLayer = Vec<[u8; 90]>;

#[derive(CustomDebug, Clone, PartialEq)]
pub(crate) enum RasterImage {
    Monochrome {
        #[debug(with = "debug_raster_layer")]
        black_layer: RasterLayer,
    },
    TwoColor {
        #[debug(with = "debug_raster_layer")]
        black_layer: RasterLayer,
        #[debug(with = "debug_raster_layer")]
        red_layer: RasterLayer,
    },
}

fn debug_raster_layer(layer: &RasterLayer, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RasterLayer({} lines)>", layer.len())
}

impl RasterImage {
    pub(crate) fn new(img: DynamicImage, media: Media) -> Result<Self, PrintJobCreationError> {
        let (width, height) = img.dimensions();
        // Always check width, for die-cut labels, also check height
        if media.width_dots() != width {
            return Err(PrintJobCreationError::DimensionMismatch {
                expected_width: media.width_dots(),
                actual_width: width,
                expected_height: None,
                actual_height: height,
            });
        }
        if let Some(length_dots) = media.length_dots() {
            if length_dots != height {
                return Err(PrintJobCreationError::DimensionMismatch {
                    expected_width: media.width_dots(),
                    actual_width: width,
                    expected_height: Some(length_dots),
                    actual_height: height,
                });
            }
        }
        Ok(if media.supports_color() {
            Self::TwoColor {
                black_layer: mask_to_raster_layer(create_mask(
                    img.clone(),
                    media.left_margin(),
                    |r, g, b| r == g && r == b && r < 200,
                )),
                red_layer: mask_to_raster_layer(create_mask(
                    img,
                    media.left_margin(),
                    |r, g, b| r > 100 && r > b && r > g,
                )),
            }
        } else {
            Self::Monochrome {
                black_layer: mask_to_raster_layer(create_mask(
                    img,
                    media.left_margin(),
                    |r, g, b| !(r == b && r == g && r == 255),
                )),
            }
        })
    }

    pub(crate) fn height(&self) -> usize {
        match self {
            RasterImage::Monochrome { black_layer } | RasterImage::TwoColor { black_layer, .. } => {
                black_layer.len()
            }
        }
    }
}

fn mask_to_raster_layer(mask: GrayImage) -> RasterLayer {
    let mut res: Vec<[u8; 90]> = mask
        .into_raw()
        .chunks_exact(720)
        .map(|line| {
            let raster_line: [u8; 90] = line
                .chunks_exact(8)
                .map(|group_of_eight| {
                    let mut res = 0;
                    group_of_eight
                        .iter()
                        .enumerate()
                        .for_each(|(i, &pixel_byte)| {
                            if pixel_byte == 0x0 {
                                res |= 1 << (7 - i);
                            }
                        });
                    res
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect("This is infallible because we ensure exact sizes");
            raster_line
        })
        .collect();
    res.reverse();
    res
}

fn create_mask(
    img: DynamicImage,
    left_margin: u32,
    print_predicate: fn(r: u8, g: u8, b: u8) -> bool,
) -> GrayImage {
    let mut rgb_image = img.into_rgb8();
    rgb_image.pixels_mut().for_each(|pixel| {
        let &mut Rgb([r, g, b]) = pixel;
        // Turn pixel white unless print predicate matches
        if !print_predicate(r, g, b) {
            *pixel = Rgb([255, 255, 255]);
        }
    });
    let mut mask = imageops::grayscale(&rgb_image);
    image::imageops::dither(&mut mask, &BiLevel);
    let (w, h) = rgb_image.dimensions();
    let right_margin = 720 - left_margin - w;
    let extended = ImageBuffer::from_fn(720, h, |x, y| {
        if (right_margin..(right_margin + w)).contains(&x) {
            *mask.get_pixel(x - right_margin, y)
        } else {
            [255].into()
        }
    });
    extended
}
