//! WebAssembly bindings for Brother QL label printers
//!
//! This crate provides WebAssembly bindings for compiling images to Brother QL
//! raster commands and printing via WebUSB.

use wasm_bindgen::prelude::*;

mod error;
mod printjob;
mod webusb;

pub use error::*;
pub use printjob::*;
pub use webusb::*;

// Re-export from brother_ql
pub use brother_ql::media::{LabelType, Media};
pub use brother_ql::printjob::CutBehavior;

use strum::IntoEnumIterator;

/// Initialize panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get all available media type names
#[wasm_bindgen(js_name = getAllMediaTypeNames)]
pub fn get_all_media_type_names() -> Vec<String> {
    Media::iter().map(|m| m.to_string()).collect()
}

/// Parse a media type from a string name
pub fn parse_media(s: &str) -> Option<Media> {
    Media::iter().find(|m| m.to_string() == s)
}

/// Parse cut behavior from a string
pub fn parse_cut_behavior(s: &str) -> Option<CutBehavior> {
    match s {
        "CutEach" => Some(CutBehavior::CutEach),
        "CutAtEnd" => Some(CutBehavior::CutAtEnd),
        "None" => Some(CutBehavior::None),
        _ => None,
    }
}
