//! Printer status information parsing and types
//!
//! This module provides types and parsing for the 32-byte status packets
//! returned by Brother QL printers.

use bitflags::bitflags;

use crate::{
    commands::VariousModeSettings, error::StatusParsingError, media::MediaType,
    printer::PrinterModel,
};

bitflags! {
/// Error flags reported by the printer
///
/// A bitfield containing all active error conditions on the printer.
/// Multiple errors can be set simultaneously. Use [`StatusInformation::has_errors`]
/// to check if any errors are present.
///
/// # Common Errors
/// - **NoMediaError**: No media is loaded in the printer
/// - **CoverOpenError**: The printer cover is open
/// - **CutterJamError**: The automatic cutter is jammed
/// - **ReplaceMediaError**: The media needs to be replaced
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
/// #     printer::PrinterModel,
/// #     status::ErrorFlags,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// let mut connection = UsbConnection::open(info)?;
/// let status = connection.get_status()?;
///
/// if status.errors.contains(ErrorFlags::NoMediaError) {
///     println!("Please load media");
/// }
/// if status.errors.contains(ErrorFlags::CoverOpenError) {
///     println!("Please close the printer cover");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorFlags: u16 {
    /// No media is loaded
    const NoMediaError = 0b1 << 0;
    /// End of media reached (die-cut labels only)
    const EndOfMediaError = 0b1 << 1;
    /// Cutter is jammed
    const CutterJamError=0b1 << 2;
    /// Printer is currently in use
    const PrinterInUseError = 0b1 << 4;
    /// Printer was turned off
    const PrinterTurnedOffError = 0b1 << 5;
    /// High voltage adapter error (not used)
    const HighVoltageAdapterError = 0b1 << 6;
    /// Fan motor error (not used)
    const FanMotorError = 0b1 << 7;
    /// Media needs to be replaced
    const ReplaceMediaError = 0b1 << 8;
    /// Expansion buffer is full
    const ExpansionBufferFullError = 0b1 << 9;
    /// Communication error occurred
    const CommunicationError = 0b1 << 10;
    /// Communication buffer is full (not used)
    const CommunicationBufferFullError = 0b1 << 11;
    /// Printer cover is open
    const CoverOpenError = 0b1 << 12;
    /// Cancel key was pressed (not used)
    const CancelKeyError = 0b1 << 13;
    /// Media cannot be fed or end of media
    const FeedingError = 0b1 << 14;
    /// System error occurred
    const SystemError = 0b1 << 15;
    const _ = !0;
}
}

/// Type of status message from the printer
///
/// Indicates what kind of status update the printer is sending.
/// Different status types are sent at different points during printing:
///
/// - **`StatusRequestReply`**: Initial response to a status request
/// - **`PhaseChange`**: Printer is transitioning between receiving and printing
/// - **`PrintingCompleted`**: A page has finished printing
///
/// Error conditions are indicated through the [`ErrorFlags`] field rather
/// than through the status type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusType {
    /// Reply to a status request
    StatusRequestReply,
    /// Printing has completed
    PrintingCompleted,
    /// An error has occurred
    ErrorOccured,
    /// Printer was turned off
    TurnedOff,
    /// Notification message
    Notification,
    /// Phase change notification
    PhaseChange,
}

impl TryFrom<u8> for StatusType {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::StatusRequestReply),
            0x01 => Ok(Self::PrintingCompleted),
            0x02 => Ok(Self::ErrorOccured),
            0x04 => Ok(Self::TurnedOff),
            0x05 => Ok(Self::Notification),
            0x06 => Ok(Self::PhaseChange),
            unused @ 0x08..=0x20 => Err(StatusParsingError {
                reason: format!("{unused:#x} is an unused status type"),
            }),
            reserved @ 0x21..=0xff => Err(StatusParsingError {
                reason: format!("{reserved:#x} is a reserved status type"),
            }),
            invalid => Err(StatusParsingError {
                reason: format!("invalid status type {invalid:#x}"),
            }),
        }
    }
}

/// Current phase of the printer
///
/// The printer alternates between these two phases during operation:
///
/// - **Receiving**: Ready to receive print data
/// - **Printing**: Currently printing a page
///
/// Phase transitions are reported via [`StatusType::PhaseChange`] status updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Receiving data from the host
    Receiving,
    /// Printing a page
    Printing,
}

impl TryFrom<[u8; 3]> for Phase {
    type Error = StatusParsingError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match value {
            [0x00, 0x00, 0x00] => Ok(Self::Receiving),
            [0x01, 0x00, 0x00] => Ok(Self::Printing),
            [a, b, c] => Err(StatusParsingError {
                reason: format!("invalid phase state {a:#x}{b:x}{c:x}"),
            }),
        }
    }
}

/// Notification from the printer
///
/// Some printers may send notifications about cooling cycles.
/// Most of the time, no notification is available.
#[derive(Debug)]
pub enum Notification {
    /// No notification available
    Unavailable,
    /// Printer has started a cooling cycle
    CoolingStarted,
    /// Printer has finished a cooling cycle
    CoolingFinished,
}

impl TryFrom<u8> for Notification {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Unavailable),
            0x03 => Ok(Self::CoolingStarted),
            0x04 => Ok(Self::CoolingFinished),
            invalid => Err(StatusParsingError {
                reason: format!("invalid notification number {invalid:#x}"),
            }),
        }
    }
}

/// Status information received from the printer
///
/// Contains complete status information parsed from the 32-byte status packet
/// returned by Brother QL printers. Status information includes the printer model,
/// error conditions, media information, and the current operational phase.
///
/// # Fields Overview
/// - **model**: The specific printer model
/// - **errors**: Active error conditions (if any)
/// - **`media_width`**: Width of loaded media in millimeters
/// - **`media_type`**: Type of media (continuous or die-cut)
/// - **`media_length`**: Length in millimeters (for die-cut labels)
/// - **`status_type`**: Type of this status message
/// - **phase**: Current operational phase (receiving or printing)
///
/// # Example
/// ```no_run
/// # use brother_ql::{
/// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
/// #     printer::PrinterModel,
/// # };
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
/// let mut connection = UsbConnection::open(info)?;
///
/// let status = connection.get_status()?;
/// println!("Printer: {:?}", status.model);
/// println!("Media: {}mm wide", status.media_width);
/// println!("Phase: {:?}", status.phase);
///
/// if status.has_errors() {
///     eprintln!("Errors detected: {:?}", status.errors);
/// } else {
///     println!("Printer is ready!");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct StatusInformation {
    /// The printer model
    pub model: PrinterModel,
    /// Error flags indicating any active error conditions
    pub errors: ErrorFlags,
    /// Media width in millimeters
    pub media_width: u8,
    /// Media type (continuous or die-cut), if detected
    pub media_type: Option<MediaType>,
    /// Various mode settings active on the printer
    pub mode: VariousModeSettings,
    /// Media length in millimeters (only relevant for die-cut labels)
    pub media_length: u8,
    /// Type of this status message
    pub status_type: StatusType,
    /// Current operational phase
    pub phase: Phase,
    /// Optional notification from the printer
    pub notification: Notification,
}

impl StatusInformation {
    /// Check if any errors are present
    ///
    /// Returns `true` if the [`errors`](Self::errors) field contains any error flags.
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{
    /// #     connection::{PrinterConnection, UsbConnection, UsbConnectionInfo},
    /// #     printer::PrinterModel,
    /// # };
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// # let mut connection = UsbConnection::open(info)?;
    /// let status = connection.get_status()?;
    ///
    /// if status.has_errors() {
    ///     println!("Error flags: {:?}", status.errors);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl TryFrom<&[u8]> for StatusInformation {
    type Error = StatusParsingError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let status: &[u8; 32] = value.try_into().map_err(|_| StatusParsingError {
            reason: format!("invalid size of {}B", value.len()),
        })?;
        let check_fixed_field = |offset: usize,
                                 name: &str,
                                 expected_value: u8|
         -> Result<(), StatusParsingError> {
            if status[offset] != expected_value {
                return Err(StatusParsingError {
                    reason: format!(
                        "expected value {expected_value:#x} for field {name} at offset {offset} but was {:#x}",
                        status[offset]
                    ),
                });
            }
            Ok(())
        };
        check_fixed_field(0, "Print head mark", 0x80)?;
        check_fixed_field(1, "Size", 0x20)?;
        check_fixed_field(2, "Reserved", 0x42)?;
        check_fixed_field(3, "Series code", 0x34)?;
        let model = PrinterModel::try_from(status[4])?;
        check_fixed_field(5, "Reserved", 0x30)?;
        // NOTE: The printer replies with 0x04
        // check_fixed_field(6, "Reserved", 0x30)?;
        check_fixed_field(7, "Reserved", 0x00)?;
        let errors = ErrorFlags::from_bits_retain(u16::from_le_bytes([status[8], status[9]]));
        let media_width = status[10];
        let media_type = match status[11] {
            0x00 => None,
            other => Some(MediaType::try_from(other)?),
        };
        check_fixed_field(12, "Reserved", 0x00)?;
        check_fixed_field(13, "Reserved", 0x00)?;
        // NOTE: The printer replies with 0x15
        // check_fixed_field(14, "Reserved", 0x3f)?;
        let mode = VariousModeSettings::try_from(status[15])?;
        check_fixed_field(16, "Reserved", 0x00)?;
        let media_length = status[17];
        let status_type = StatusType::try_from(status[18])?;
        let phase_bytes: [u8; 3] = status[19..=21]
            .try_into()
            .expect("This conversion is infallible due to the earlier size assertion");
        let phase = Phase::try_from(phase_bytes)?;
        let notification = Notification::try_from(status[22])?;
        check_fixed_field(23, "Reserved", 0x00)?;
        check_fixed_field(24, "Reserved", 0x00)?;
        // Remaining 7 bytes are not specified at all
        Ok(StatusInformation {
            model,
            errors,
            media_width,
            media_type,
            mode,
            media_length,
            status_type,
            phase,
            notification,
        })
    }
}
