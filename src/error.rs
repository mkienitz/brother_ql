//! This module provides crate-specific error types.
use thiserror::Error;

use crate::status::ErrorFlags;

/// The crate-level error type
#[derive(Error, Debug)]
pub enum BQLError {
    /// Returned when the provided image has a size incompatible with the provided media type.
    #[error("media and image are not compatible")]
    DimensionMismatch,
    /// Returned when there is an issue with a received status information
    #[error("Received status information is malformed: {0}")]
    MalformedStatus(String),
    #[error("Received bad status during printing: {0:?}")]
    PrintingError(ErrorFlags),
    /// USB device not found
    #[cfg(feature = "usb")]
    #[error("USB device not found (vendor: {vendor_id:#06x}, product: {product_id:#06x})")]
    UsbDeviceNotFound { vendor_id: u16, product_id: u16 },
    /// USB operation failed
    #[cfg(feature = "usb")]
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),
    /// USB communication timeout
    #[cfg(feature = "usb")]
    #[error("USB operation timed out")]
    UsbTimeout,
    /// Failed to write all data to USB device
    #[cfg(feature = "usb")]
    #[error("Incomplete USB write: wrote {written} of {expected} bytes")]
    IncompleteWrite { written: usize, expected: usize },
}
