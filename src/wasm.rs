//! This module exposes an API for the NPM package and can be ignored by the Rust consumers.
use wasm_bindgen::prelude::*;

use crate::{media::Media, printjob};

/// This enum specifies the cutting behavior for the generated print job.
#[wasm_bindgen]
pub enum CutBehavior {
    /// Cut after each page
    CutEach,
    /// Cut after the last page
    CutAtEnd,
    /// Don't cut at all
    None,
}

/// Compile a image to binary raster commands using the specified settings
///
/// # Arguments
///
/// * `image`           - The image to print
/// * `noPages`         - The amount of replicas to print
/// * `media`           - The paper type used
/// * `highDPI`         - Whether or not to use high-DPI mode
/// * `compressed`      - Whether or not to use compression - currently unsupported!
/// * `qualityPriority` - Whether or not the printer gives priority to print quality.
///                     - Has no effect on print quality.
/// * `cutBehavior`     - The selected behavior for the automatic cutter unit
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn compile(
    image: &[u8],
    noPages: u8,
    media: Media,
    highDPI: bool,
    compressed: bool,
    qualityPriority: bool,
    cutBehavior: CutBehavior,
) -> Result<Vec<u8>, JsError> {
    let job = printjob::PrintJob {
        no_pages: noPages,
        image: image::load_from_memory(image)?,
        media,
        high_dpi: highDPI,
        compressed,
        quality_priority: qualityPriority,
        cut_behaviour: match cutBehavior {
            CutBehavior::CutEach => printjob::CutBehavior::CutEach,
            CutBehavior::CutAtEnd => printjob::CutBehavior::CutAtEnd,
            CutBehavior::None => printjob::CutBehavior::None,
        },
    };
    let data = job.compile()?;
    Ok(data)
}
