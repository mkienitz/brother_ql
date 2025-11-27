//! Printer model definitions and USB configuration

use crate::error::StatusParsingError;

/// Brother QL printer model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrinterModel {
    /// QL-800
    QL800,
    /// QL-810W
    QL810W,
    /// QL-820NWB
    QL820NWB,
}

impl TryFrom<u8> for PrinterModel {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x38 => Ok(Self::QL800),
            0x39 => Ok(Self::QL810W),
            0x41 => Ok(Self::QL820NWB),
            invalid => Err(StatusParsingError {
                reason: format!("invalid model code {invalid:#x}"),
            }),
        }
    }
}
