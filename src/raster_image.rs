use image::{
    imageops::{self, BiLevel},
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, Rgb, RgbImage,
};
use itertools::Itertools;

use crate::{
    error::BQLError,
    media::{MediaSettings, MediaType},
};

type RasterLayer = Vec<[u8; 90]>;

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
    pub(crate) fn new(img: DynamicImage, media_settings: &MediaSettings) -> Result<Self, BQLError> {
        let (width, height) = img.dimensions();
        // Always check width, for die-cut labels, also check height
        if media_settings.width_dots != width {
            return Err(BQLError::DimensionMismatch);
        }
        if let MediaType::DieCut { length_dots, .. } = media_settings.media_type {
            if length_dots != height {
                return Err(BQLError::DimensionMismatch);
            }
        }
        Ok(if media_settings.color {
            Self::TwoColor {
                black_layer: mask_to_raster_layer(create_mask(
                    &img,
                    media_settings.left_margin,
                    |r, g, b| r == g && r == b && r < 200,
                )),
                red_layer: mask_to_raster_layer(create_mask(
                    &img,
                    media_settings.left_margin,
                    |r, g, b| r > 100 && r > b && r > g,
                )),
            }
        } else {
            Self::Monochrome {
                black_layer: mask_to_raster_layer(create_mask(
                    &img,
                    media_settings.left_margin,
                    |r, g, b| !(r == b && r == g && r == 255),
                )),
            }
        })
    }
}

fn mask_to_raster_layer(mask: GrayImage) -> RasterLayer {
    let mut res: Vec<[u8; 90]> = mask
        .pixels()
        .chunks(720)
        .into_iter()
        .map(|line| {
            line.chunks(8)
                .into_iter()
                .map(|chunk| {
                    let mut res = 0;
                    chunk.enumerate().for_each(|(i, px)| {
                        if px.0[0] == 0 {
                            res |= 1 << (7 - i);
                        }
                    });
                    res
                })
                .collect_vec()
                .try_into()
                .unwrap()
        })
        .collect_vec();
    res.reverse();
    res
}

fn create_mask(
    img: &DynamicImage,
    left_margin: u32,
    filter: fn(r: u8, g: u8, b: u8) -> bool,
) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut filtered = RgbImage::new(w, h);
    img.to_rgb8()
        .pixels()
        .zip(filtered.pixels_mut())
        .for_each(|(ipx, fpx)| {
            let &Rgb(chs @ [r, g, b]) = ipx;
            fpx.0 = if filter(r, g, b) {
                chs
            } else {
                [255, 255, 255]
            };
        });
    let mut mask = imageops::grayscale(&filtered);
    image::imageops::dither(&mut mask, &BiLevel);
    let extended = ImageBuffer::from_fn(720, h, |x, y| {
        if (left_margin..(left_margin + w)).contains(&x) {
            *mask.get_pixel(x - left_margin, y)
        } else {
            [255].into()
        }
    });
    extended
}
