use image::{imageops::BiLevel, DynamicImage, GenericImageView, ImageBuffer, Luma};
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

pub type RasterLayer = Vec<[u8; 90]>;

fn image_to_layer(buffer: ImageBuffer<Luma<u8>, Vec<u8>>) -> RasterLayer {
    let mut res: Vec<[u8; 90]> = buffer
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
    res
}

impl RasterImage {
    pub fn no_lines(&self) -> usize {
        use RasterImage::*;
        match self {
            Monochrome { black_channel } => black_channel.len(),
            TwoColor { black_channel, .. } => black_channel.len(),
        }
    }
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

        let mut bw = img.grayscale().into();
        image::imageops::dither(&mut bw, &BiLevel);
        // let _ = bw.save("bw.png");

        let extended = ImageBuffer::from_fn(720, h, |x, y| -> Luma<u8> {
            let lm: usize = media_settings.left_margin;
            if (lm..(lm + width)).contains(&(x as usize)) {
                *bw.get_pixel(x - media_settings.left_margin as u32, y)
            } else {
                [255].into()
            }
        });
        // let _ = extended.save("extended.png");

        Ok(Self::Monochrome {
            black_channel: image_to_layer(extended),
        })
    }
}
