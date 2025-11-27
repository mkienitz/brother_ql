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

/// Errors related to print job validation and compatibility
///
/// These errors occur when creating or compiling a print job with invalid
/// or incompatible parameters.
///
/// # Example
/// ```no_run
/// # use brother_ql::{media::Media, printjob::PrintJob, error::PrintJobError};
/// # fn example() -> Result<(), PrintJobError> {
/// let image = image::open("label.png")?;
///
/// // This may fail with DimensionMismatch if image size doesn't match media
/// let job = PrintJob::new(&image, Media::C62)?;
/// # Ok(())
/// # }
/// ```
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

/// USB-specific connection errors
///
/// These errors occur during USB communication with the printer.
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{UsbConnection, UsbConnectionInfo},
/// #     printer::PrinterModel,
/// #     error::UsbError,
/// # };
/// # fn example() -> Result<(), UsbError> {
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
///
/// // May fail with UsbError::DeviceNotFound if printer not connected
/// let connection = UsbConnection::open(info)?;
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug)]
pub enum UsbError {
    /// USB device not found with the specified vendor and product ID
    ///
    /// This typically means:
    /// - The printer is not connected
    /// - The printer is connected but not powered on
    /// - Wrong printer model specified
    /// - USB permissions issue (may need udev rules on Linux)
    #[error("USB device not found (vendor: {vendor_id:#06x}, product: {product_id:#06x})")]
    DeviceNotFound {
        /// USB vendor ID (typically 0x04f9 for Brother)
        vendor_id: u16,
        /// USB product ID (specific to printer model)
        product_id: u16,
    },

    /// Failed to write all data to the USB device
    ///
    /// This is rare but can occur if the USB connection is interrupted
    /// during a write operation.
    #[error("Incomplete USB write: wrote {written} of {expected} bytes")]
    IncompleteWrite {
        /// Number of bytes actually written
        written: usize,
        /// Number of bytes expected to write
        expected: usize,
    },

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

/// Errors that occur when parsing status information from the printer
///
/// These errors indicate that the raw bytes received from the printer
/// could not be interpreted as valid status information.
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{UsbConnection, UsbConnectionInfo},
/// #     printer::PrinterModel,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// # let mut connection = UsbConnection::open(info)?;
/// // May fail if status bytes are malformed
/// let status = connection.get_status()?;
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug, Clone)]
#[error("Failed to parse status information: {reason}")]
pub struct StatusParsingError {
    /// Description of what's wrong with the status data
    pub reason: String,
}

/// Errors that can occur when reading status from the printer
///
/// These errors encompass all failure modes during status operations, including
/// USB communication errors, timeout waiting for printer response, and parsing
/// malformed status bytes.
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{UsbConnection, UsbConnectionInfo},
/// #     printer::PrinterModel,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// let mut connection = UsbConnection::open(info)?;
///
/// // May fail with USB, NoResponse, or Parsing errors
/// let status = connection.get_status()?;
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug)]
pub enum StatusError {
    /// USB communication error
    #[error(transparent)]
    Usb(#[from] UsbError),

    /// Printer did not respond with expected data
    ///
    /// After multiple retry attempts, the printer failed to send data.
    /// This is distinct from USB-level timeouts - the USB communication works,
    /// but the printer isn't sending data.
    #[error(
        "Printer did not respond with expected data after {attempts} attempts over {duration_ms}ms"
    )]
    NoResponse {
        /// Number of retry attempts made
        attempts: u32,
        /// Total time spent waiting in milliseconds
        duration_ms: u64,
    },

    /// Status parsing error (malformed status bytes)
    #[error(transparent)]
    Parsing(#[from] StatusParsingError),
}

/// Protocol flow errors during printer communication
///
/// These errors occur when the printer sends valid status information,
/// but at the wrong time or in an unexpected state during the print protocol.
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
/// #     media::Media,
/// #     printer::PrinterModel,
/// #     printjob::PrintJob,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// # let mut connection = UsbConnection::open(info)?;
/// # let image = image::open("label.png")?;
/// # let job = PrintJob::new(&image, Media::C62)?;
/// // May fail with ProtocolError if printer is in wrong state
/// connection.print(job)?;
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug, Clone)]
pub enum ProtocolError {
    /// Printer reported one or more error conditions
    ///
    /// The printer is in an error state and cannot print. Common errors include:
    /// - No media loaded
    /// - Cover open
    /// - Cutter jam
    /// - Media needs replacement
    ///
    /// Check the [`ErrorFlags`] to see which specific errors are active.
    #[error("Printer reported errors: {0:?}")]
    PrinterError(ErrorFlags),

    /// Printer sent unexpected status type or phase
    ///
    /// During printing, the printer should transition through specific states.
    /// This error indicates the printer sent a status update that doesn't match
    /// what was expected at this point in the protocol.
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

/// Errors that can occur during printing
///
/// This encompasses all possible errors during a print operation, including
/// USB communication issues and printer protocol errors.
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
/// #     media::Media,
/// #     printer::PrinterModel,
/// #     printjob::PrintJob,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// let mut connection = UsbConnection::open(info)?;
///
/// let image = image::open("label.png")?;
/// let job = PrintJob::new(&image, Media::C62)?;
///
/// // Can fail with USB or protocol errors
/// connection.print(job)?;
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug)]
pub enum PrintError {
    /// USB communication error
    #[error(transparent)]
    Usb(#[from] UsbError),

    /// Status reading error (communication, timeout, or parsing)
    #[error(transparent)]
    Status(StatusError),

    /// Protocol flow error (unexpected status, printer error, etc.)
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}
