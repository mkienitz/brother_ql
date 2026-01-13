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
}

impl From<image::ImageError> for PrintJobError {
    fn from(e: image::ImageError) -> Self {
        Self::new(format!("Image error: {e}"))
    }
}

impl From<brother_ql::error::PrintJobCreationError> for PrintJobError {
    fn from(e: brother_ql::error::PrintJobCreationError) -> Self {
        Self::new(format!("{e}"))
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
