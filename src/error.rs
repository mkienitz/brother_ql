//! Error types for the Brother QL library
//!
//! This module provides precise error types for different failure scenarios:
//!
//! - [`PrintJobError`]: Validation and compatibility errors during print job creation
//! - [`UsbError`]: USB communication and device errors
//! - [`StatusParsingError`]: Status parsing errors
//! - [`StatusError`]: Errors that can occur when reading status
//! - [`ProtocolError`]: Protocol flow errors during printing
//! - [`PrintError`]: Errors that can occur during printing

use thiserror::Error;

use crate::status::ErrorFlags;

/// Errors related to print job validation
///
/// Returned by [`PrintJob::new`][crate::printjob::PrintJob::new] when image dimensions don't match media requirements.
#[derive(Error, Debug)]
pub enum PrintJobError {
    /// Image dimensions don't match the selected media type
    ///
    /// The image width must exactly match the media width in dots (pixels).
    /// For die-cut labels, the height must also match exactly.
    /// For continuous media, any height is acceptable.
    #[error("Image dimensions ({actual_width}x{actual_height} px) don't match media requirements (width: {expected_width} px{})",
        expected_height.map(|h| format!(", height: {h} px")).unwrap_or_default()
    )]
    DimensionMismatch {
        /// Expected image width in pixels (dots)
        expected_width: u32,
        /// Actual image width in pixels
        actual_width: u32,
        /// Expected image height in pixels (None for continuous media)
        expected_height: Option<u32>,
        /// Actual image height in pixels
        actual_height: u32,
    },

    /// Image I/O error from the image crate
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

/// USB communication errors
#[derive(Error, Debug)]
pub enum UsbError {
    /// USB device not found with the specified vendor and product ID
    #[error("USB device not found (vendor: {vendor_id:#06x}, product: {product_id:#06x})")]
    DeviceNotFound {
        /// USB vendor ID (typically 0x04f9 for Brother)
        vendor_id: u16,
        /// USB product ID (specific to printer model)
        product_id: u16,
    },

    /// Failed to write all data to the USB device
    ///
    /// This should never occur, but if it does, please report it as a GitHub issue
    #[error("Incomplete USB write occured! Please report this issue!")]
    IncompleteWrite,

    /// USB communication error from the rusb library
    ///
    /// Wraps errors from the underlying rusb USB library, including:
    /// - Timeout
    /// - Permission denied
    /// - Device busy
    /// - Pipe errors
    /// - Device disconnected
    /// - No device found
    ///
    /// See [`rusb::Error`] for all possible error variants.
    #[error(transparent)]
    Rusb(#[from] rusb::Error),
}

/// Kernel communication errors
#[derive(Error, Debug)]
pub enum KernelError {
    /// Kernel read/write operation failed
    #[error("Kernel IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// Failed to write all data to the kernel device
    ///
    /// This should never occur, but if it does, please report it as a GitHub issue
    #[error("Incomplete kernel write occured! Please report this issue!")]
    IncompleteWrite,
}

/// Status parsing errors
///
/// Returned when status bytes from the printer are malformed.
#[derive(Error, Debug, Clone)]
#[error("Failed to parse status information: {reason}")]
pub struct StatusParsingError {
    /// Description of what's wrong with the status data
    pub reason: String,
}

/// Status reading errors
///
/// Returned by [`get_status`](crate::connection::UsbConnection::get_status).
#[derive(Error, Debug)]
pub enum StatusError {
    /// USB communication error
    #[error(transparent)]
    Usb(#[from] UsbError),

    /// Kernel communication error
    #[error(transparent)]
    Kernel(#[from] KernelError),

    /// Printer did not respond after retries
    #[error("Printer did not respond with a status information reply after being queried")]
    NoResponse,
    /// Status parsing error (malformed status bytes)
    #[error(transparent)]
    Parsing(#[from] StatusParsingError),
}

/// Protocol flow errors during printing
///
/// Returned when the printer sends unexpected status or reports an error condition.
#[derive(Error, Debug, Clone)]
pub enum ProtocolError {
    /// Printer reported error conditions (see [`ErrorFlags`])
    #[error("Printer reported errors: {0:?}")]
    PrinterError(ErrorFlags),

    /// Printer sent unexpected status
    #[error(
        "Unexpected printer status: expected {expected_type:?}/{expected_phase:?}, got {actual_type:?}/{actual_phase:?}"
    )]
    UnexpectedStatus {
        /// Expected status type
        expected_type: crate::status::StatusType,
        /// Expected phase
        expected_phase: crate::status::Phase,
        /// Actual status type received
        actual_type: crate::status::StatusType,
        /// Actual phase received
        actual_phase: crate::status::Phase,
    },
}

/// Printing errors
///
/// Returned by [`print`](crate::connection::PrinterConnection::print).
#[derive(Error, Debug)]
pub enum PrintError {
    /// USB communication error
    #[error(transparent)]
    Usb(#[from] UsbError),

    /// Kernel communication error
    #[error(transparent)]
    Kernel(#[from] KernelError),

    /// Status reading error (communication, timeout, or parsing)
    #[error(transparent)]
    Status(StatusError),

    /// Protocol flow error (unexpected status, printer error, etc.)
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}
