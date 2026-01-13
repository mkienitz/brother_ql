//! Error types for the Brother QL WASM library

use wasm_bindgen::prelude::*;

/// Errors related to print job validation
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PrintJobError {
    message: String,
}

#[wasm_bindgen]
impl PrintJobError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl PrintJobError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn dimension_mismatch(
        expected_width: u32,
        actual_width: u32,
        expected_height: Option<u32>,
        actual_height: u32,
    ) -> Self {
        let height_msg = expected_height
            .map(|h| format!(", height: {h} px"))
            .unwrap_or_default();
        Self::new(format!(
            "Image dimensions ({actual_width}x{actual_height} px) don't match media requirements (width: {expected_width} px{height_msg})"
        ))
    }
}

impl From<image::ImageError> for PrintJobError {
    fn from(e: image::ImageError) -> Self {
        Self::new(format!("Image error: {e}"))
    }
}

/// USB communication errors
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct UsbError {
    message: String,
}

#[wasm_bindgen]
impl UsbError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl UsbError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<JsValue> for UsbError {
    fn from(e: JsValue) -> Self {
        let msg = e
            .as_string()
            .unwrap_or_else(|| format!("{e:?}"));
        Self::new(msg)
    }
}
