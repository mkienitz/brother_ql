use image::{
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, Rgb,
    imageops::{self, BiLevel},
};
use itertools::Itertools;

use crate::{
    error::PrintJobError,
    media::{LengthInfo, MediaSettings},
};

type RasterLayer = Vec<[u8; 90]>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RasterImage {
    Monochrome {
        black_layer: RasterLayer,
    },
    TwoColor {
        black_layer: RasterLayer,
        red_layer: RasterLayer,
    },
}

impl RasterImage {
    pub(crate) fn new(
        img: DynamicImage,
        media_settings: MediaSettings,
    ) -> Result<Self, PrintJobError> {
        let (width, height) = img.dimensions();
        // Always check width, for die-cut labels, also check height
        if media_settings.width_dots != width {
            return Err(PrintJobError::DimensionMismatch {
                expected_width: media_settings.width_dots,
                actual_width: width,
                expected_height: None,
                actual_height: height,
            });
        }
        if let LengthInfo::Fixed { length_dots, .. } = media_settings.length_info {
            if length_dots != height {
                return Err(PrintJobError::DimensionMismatch {
                    expected_width: media_settings.width_dots,
                    actual_width: width,
                    expected_height: Some(length_dots),
                    actual_height: height,
                });
            }
        }
        Ok(if media_settings.color {
            Self::TwoColor {
                black_layer: mask_to_raster_layer(create_mask(
                    img.clone(),
                    media_settings.left_margin,
                    |r, g, b| r == g && r == b && r < 200,
                )),
                red_layer: mask_to_raster_layer(create_mask(
                    img,
                    media_settings.left_margin,
                    |r, g, b| r > 100 && r > b && r > g,
                )),
            }
        } else {
            Self::Monochrome {
                black_layer: mask_to_raster_layer(create_mask(
                    img,
                    media_settings.left_margin,
                    |r, g, b| !(r == b && r == g && r == 255),
                )),
            }
        })
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
                .collect_vec()
                .try_into()
                .expect("This is infallible because we ensure exact sizes");
            raster_line
        })
        .collect_vec();
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
