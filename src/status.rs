//! Printer status information parsing and types
//!
//! This module provides types and parsing for the 32-byte status packets
//! returned by Brother QL printers.

use bitflags::bitflags;

use crate::{
    commands::VariousModeSettings, error::BQLError, media::MediaType, printer::PrinterModel,
};

bitflags! {
/// Error flags reported by the printer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorFlags: u16 {
    const NoMediaError = 0b1 << 0;
    const EndOfMediaError = 0b1 << 1; // Only for die-cut labels
    const CutterJamError=0b1 << 2;
    const PrinterInUseError = 0b1 << 4;
    const PrinterTurnedOffError = 0b1 << 5;
    const HighVoltageAdapterError = 0b1 << 6; // not used
    const FanMotorError = 0b1 << 7; // not used
    const ReplaceMediaError = 0b1 << 8;
    const ExpansionBufferFullError = 0b1 << 9;
    const CommunicationError = 0b1 << 10;
    const CommunicationBufferFullError = 0b1 << 11; // not used
    const CoverOpenError = 0b1 << 12;
    const CancelKeyError = 0b1 << 13; // not used
    const FeedingError = 0b1 << 14;   // Media Cannot be fed or end of media
    const SystemError = 0b1 << 15;
    const _ = !0;
}
}

/// Type of status message from the printer
#[derive(Debug)]
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
    type Error = BQLError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::StatusRequestReply),
            0x01 => Ok(Self::PrintingCompleted),
            0x02 => Ok(Self::ErrorOccured),
            0x04 => Ok(Self::TurnedOff),
            0x05 => Ok(Self::Notification),
            0x06 => Ok(Self::PhaseChange),
            unused @ 0x08..=0x20 => Err(BQLError::MalformedStatus(format!(
                "{unused:#x} is an unused status type"
            ))),
            reserved @ 0x21..=0xff => Err(BQLError::MalformedStatus(format!(
                "{reserved:#x} is a reserved status type"
            ))),
            invalid => Err(BQLError::MalformedStatus(format!(
                "invalid status type {invalid:#x}"
            ))),
        }
    }
}

/// Current phase of the printer
#[derive(Debug)]
pub enum Phase {
    /// Receiving data
    Receiving,
    /// Printing
    Printing,
}

impl TryFrom<[u8; 3]> for Phase {
    type Error = BQLError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match value {
            [0x00, 0x00, 0x00] => Ok(Self::Receiving),
            [0x01, 0x00, 0x00] => Ok(Self::Printing),
            [a, b, c] => Err(BQLError::MalformedStatus(format!(
                "invalid phase state {a:#x}{b:x}{c:x}"
            ))),
        }
    }
}

/// Notification from the printer
#[derive(Debug)]
pub enum Notification {
    /// No notification available
    Unavailable,
    /// Cooling has started
    CoolingStarted,
    /// Cooling has finished
    CoolingFinished,
}

impl TryFrom<u8> for Notification {
    type Error = BQLError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Unavailable),
            0x03 => Ok(Self::CoolingStarted),
            0x04 => Ok(Self::CoolingFinished),
            invalid => Err(BQLError::MalformedStatus(format!(
                "invalid notification number {invalid:#x}"
            ))),
        }
    }
}

/// Status information received from the printer
#[derive(Debug)]
pub struct StatusInformation {
    /// The printer model
    pub model: PrinterModel,
    /// Error flags
    pub errors: ErrorFlags,
    /// Media width in mm
    pub media_width: u8,
    /// Media type
    pub media_type: Option<MediaType>,
    /// Various mode settings
    pub mode: VariousModeSettings,
    /// Media length in mm (for die-cut labels)
    pub media_length: u8,
    /// Status type
    pub status_type: StatusType,
    /// Current phase
    pub phase: Phase,
    /// Notification
    pub notification: Notification,
}

impl StatusInformation {
    /// Check if any errors are present
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl TryFrom<&[u8]> for StatusInformation {
    type Error = BQLError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let status: &[u8; 32] = value
            .try_into()
            .map_err(|_| BQLError::MalformedStatus(format!("invalid size of {}B", value.len())))?;
        let check_fixed_field = |offset: usize,
                                 name: &str,
                                 expected_value: u8|
         -> Result<(), BQLError> {
            if status[offset] != expected_value {
                return Err(BQLError::MalformedStatus(format!(
                    "expected value {expected_value:#x} for field {name} at offset {offset} but was {:#x}",
                    status[offset]
                )));
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
