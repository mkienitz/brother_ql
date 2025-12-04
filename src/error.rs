//! Error types for the Brother QL library
//!
//! This module provides precise error types for different failure scenarios:
//!
//! - [`PrintJobCreationError`]: Validation and compatibility errors during print job creation
//! - [`UsbError`]: USB communication and device errors (requires `usb` feature)
//! - [`KernelError`]: Kernel connection errors
//! - [`StatusParsingError`]: Status parsing errors
//! - [`StatusError`]: Errors that can occur when reading status
//! - [`ProtocolError`]: Protocol flow errors during printing
//! - [`PrintError`]: Errors that can occur during printing

use thiserror::Error;

use crate::{media::Media, status::ErrorFlags};

/// Errors related to print job validation
///
/// Returned when creating a [`PrintJob`][crate::printjob::PrintJob] when image dimensions don't match media requirements.
#[derive(Error, Debug)]
pub enum PrintJobCreationError {
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

/// Marker trait for connection error types
///
/// Currently, only [`UsbError`] and [`KernelError`] implement this trait.
pub trait ConnectionError {}

/// USB communication errors
#[cfg(feature = "usb")]
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

    /// Wraps errors from the underlying rusb USB library, including:
    ///
    /// See [`rusb::Error`] for all possible error variants.
    #[error(transparent)]
    Rusb(#[from] rusb::Error),
}

#[cfg(feature = "usb")]
impl ConnectionError for UsbError {}

/// Kernel connection errors
#[derive(Error, Debug)]
pub enum KernelError {
    /// Kernel I/O error
    #[error("Kernel IO error: {0}")]
    KernelIOError(#[from] std::io::Error),

    /// Failed to write all data to the kernel device
    ///
    /// This should never occur, but if it does, please report it as a GitHub issue
    #[error("Incomplete write occured! Please report this issue!")]
    IncompleteWrite,

    /// Kernel operation timeout
    #[error("Kernel IO operation timed out")]
    KernelIOTimeout,
}

impl ConnectionError for KernelError {}

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
/// Generic over the connection error type `E` (e.g., [`UsbError`] or [`KernelError`]).
///
/// Returned by [`get_status`](crate::connection::PrinterConnection::get_status`).
#[derive(Error, Debug)]
pub enum StatusError<E: ConnectionError> {
    /// Connection error
    #[error(transparent)]
    Connection(#[from] E),

    /// Printer did not respond after retries
    #[error("Printer did not respond with a status information reply after being queried")]
    NoResponse,

    /// Status parsing error (malformed status bytes)
    #[error(transparent)]
    Parsing(StatusParsingError),
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

    /// Media type mismatch between print job and installed media
    ///
    /// This error occurs when the printer has different media installed than what
    /// the print job requires (e.g., trying to print on 62mm tape when 29mm is loaded).
    #[error("Print job requires {expected_media:?} tape but printer reported {reported_media:?}")]
    MediaMismatch {
        /// Media type required by the print job
        expected_media: Media,
        /// Media type reported by the printer (None if unable to determine)
        reported_media: Option<Media>,
    },
}

/// Printing errors
///
/// Generic over the connection error type `E` (e.g., [`UsbError`] or [`KernelError`]).
///
/// Returned by [`print`](crate::connection::PrinterConnection::print).
///
/// This struct contains:
/// - `page`: The page number where the error occurred (0 = pre-print validation, 1+ = actual page)
/// - `kind`: The underlying error source
#[derive(Error, Debug)]
#[error("Print error on page {page_no}: {source}")]
pub struct PrintError<E: ConnectionError> {
    /// Page number where the error occurred
    ///
    /// - `0` = Error during pre-print validation (before any pages were printed)
    /// - `1+` = Error while printing the specified page
    pub page_no: u32,

    /// The specific error kind
    #[source]
    pub source: PrintErrorSource<E>,
}

impl<E: ConnectionError> PrintError<E> {
    /// Create a `PrintError` with the given source and page number
    pub(crate) fn with_page<T: Into<PrintErrorSource<E>>>(err: T, page_no: u32) -> Self {
        PrintError {
            page_no,
            source: err.into(),
        }
    }

    /// Return a closure that maps errors to `PrintError` with the given page number
    ///
    /// Useful with `map_err` when the page number is known in advance.
    pub(crate) fn err_source_mapper<S>(page_no: u32) -> impl Fn(S) -> Self
    where
        S: Into<PrintErrorSource<E>>,
    {
        move |e: S| Self {
            page_no,
            source: e.into(),
        }
    }
}

/// Specific kinds of printing errors
///
/// Generic over the connection error type `E` (e.g., [`UsbError`] or [`KernelError`]).
///
/// This enum represents the different types of errors that can occur during printing.
/// It is typically accessed via [`PrintError::source`].
#[derive(Error, Debug)]
#[error(transparent)]
pub enum PrintErrorSource<E: ConnectionError> {
    /// Connection error
    #[error(transparent)]
    Connection(#[from] E),

    /// Status reading error (communication, timeout, or parsing)
    #[error(transparent)]
    Status(#[from] StatusError<E>),

    /// Protocol flow error (unexpected status, printer error, etc.)
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

/// Test label generation error
///
/// Returned when Typst compilation, rendering, or image encoding fails during test label generation.
/// This should never occur under normal circumstances - if you encounter this error, please report it as a bug.
#[cfg(feature = "test-labels")]
#[derive(Error, Debug)]
#[error("Couldn't create test-label using typst: {reason}")]
pub struct TypstError {
    /// Reason for failed label creation
    pub reason: String,
}
