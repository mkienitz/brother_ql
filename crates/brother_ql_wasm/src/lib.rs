//! WebAssembly bindings for Brother QL label printers
//!
//! This crate provides WebAssembly bindings for compiling images to Brother QL
//! raster commands and printing via WebUSB.

use wasm_bindgen::prelude::*;

mod commands;
mod error;
mod media;
mod printjob;
mod raster_image;
mod webusb;

pub use error::*;
pub use media::*;
pub use printjob::*;
pub use webusb::*;

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
