use image::{
    imageops::{self, BiLevel},
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, Rgb, RgbImage,
};
use itertools::Itertools;

use crate::{
    error::BQLError,
    media::{MediaSettings, MediaType},
};

pub enum RasterImage {
    Monochrome {
        black_channel: RasterLayer,
    },
    TwoColor {
        black_channel: RasterLayer,
        red_channel: RasterLayer,
    },
}
pub struct RasterLayer {
    pub data: Vec<[u8; 90]>,
}

impl From<GrayImage> for RasterLayer {
    fn from(value: GrayImage) -> Self {
        let mut res: Vec<[u8; 90]> = value
            .pixels()
            .chunks(720)
            .into_iter()
            .map(|line| {
                line.chunks(8)
                    .into_iter()
                    .map(|chunk| {
                        let mut res: u8 = 0;
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
        Self { data: res }
    }
}

fn create_mask(
    img: &DynamicImage,
    media_settings: &MediaSettings,
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
        let lm = media_settings.left_margin as u32;
        if (lm..(lm + w)).contains(&x) {
            *mask.get_pixel(x - media_settings.left_margin as u32, y)
        } else {
            [255].into()
        }
    });
    extended
}

impl RasterImage {
    pub fn new(img: DynamicImage, media_settings: &MediaSettings) -> Result<Self, BQLError> {
        let (w, h) = img.dimensions();
        let (width, height) = (w as usize, h as usize);
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
            let red = create_mask(&img, media_settings, |r, g, b| r > 100 && r > b && r > g);
            let _ = red.save("red.png");
            let black = create_mask(&img, media_settings, |r, g, b| r == g && r == b && r < 200);
            let _ = black.save("black.png");
            Self::TwoColor {
                black_channel: black.into(),
                red_channel: red.into(),
            }
        } else {
            let bw = create_mask(&img, media_settings, |r, g, b| {
                !(r == b && r == g && r == 255)
            });
            let _ = bw.save("bw.png");
            Self::Monochrome {
                black_channel: bw.into(),
            }
        })
    }
}
