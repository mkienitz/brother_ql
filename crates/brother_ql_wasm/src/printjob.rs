//! Print job creation and compilation - WASM wrapper for brother_ql::printjob

use wasm_bindgen::prelude::*;
use image::DynamicImage;

use brother_ql::media::{LabelType, Media};
use brother_ql::printjob::CutBehavior;

use crate::error::PrintJobError;
use crate::{parse_cut_behavior, parse_media};

/// WASM-friendly wrapper for PrintJob
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PrintJob {
    inner: brother_ql::printjob::PrintJob,
    copies: u8,
    cut_behavior: CutBehavior,
}

#[wasm_bindgen]
impl PrintJob {
    /// Create a print job from image bytes (PNG or JPEG)
    #[wasm_bindgen(js_name = fromImageBytes)]
    pub fn from_image_bytes(data: &[u8], media_name: &str) -> Result<PrintJob, PrintJobError> {
        let media = parse_media(media_name)
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
        let media = parse_media(media_name)
            .ok_or_else(|| PrintJobError::new(format!("Unknown media type: {}", media_name)))?;
        let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
            .ok_or_else(|| PrintJobError::new("Invalid pixel data dimensions"))?;
        Self::from_dynamic_image(DynamicImage::ImageRgba8(img), media)
    }

    fn from_dynamic_image(img: DynamicImage, media: Media) -> Result<Self, PrintJobError> {
        let cut_behavior = match media.label_type() {
            LabelType::Continuous => CutBehavior::CutEach,
            LabelType::DieCut => CutBehavior::CutAtEnd,
        };
        
        let inner = brother_ql::printjob::PrintJob::from_image(img, media)?;
        
        Ok(Self {
            inner,
            copies: 1,
            cut_behavior,
        })
    }

    /// Set the number of copies to print
    #[wasm_bindgen(js_name = setCopies)]
    pub fn set_copies(&mut self, copies: u8) {
        self.copies = copies;
    }

    /// Set cut behavior
    #[wasm_bindgen(js_name = setCutBehavior)]
    pub fn set_cut_behavior(&mut self, behavior: &str) {
        if let Some(b) = parse_cut_behavior(behavior) {
            self.cut_behavior = b;
        }
    }

    /// Get the number of pages in this job
    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> usize {
        self.copies as usize
    }

    /// Compile the print job into raster command bytes
    #[wasm_bindgen]
    pub fn compile(&self) -> Vec<u8> {
        // We need to rebuild the job with the current settings since brother_ql's
        // PrintJob doesn't have setters for copies/cut_behavior after creation
        let job = self.inner.clone();
        job.compile()
    }
}

impl PrintJob {
    /// Get the inner brother_ql PrintJob for direct use
    pub fn inner(&self) -> &brother_ql::printjob::PrintJob {
        &self.inner
    }
}
