//! Printer model definitions and USB configuration

use crate::error::StatusParsingError;

/// Brother QL printer model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrinterModel {
    /// QL-700
    QL700,
    /// QL-800
    QL800,
    /// QL-810W
    QL810W,
    /// QL-820NWB
    QL820NWB,
}

impl PrinterModel {
    pub(crate) const fn pid(&self) -> u16 {
        match self {
            PrinterModel::QL700 => 0x2042,
            PrinterModel::QL800 => 0x209b,
            PrinterModel::QL810W => 0x209c,
            PrinterModel::QL820NWB => 0x209d,
        }
    }
}

impl TryFrom<u8> for PrinterModel {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x35 => Ok(Self::QL700),
            0x38 => Ok(Self::QL800),
            0x39 => Ok(Self::QL810W),
            0x41 => Ok(Self::QL820NWB),
            invalid => Err(StatusParsingError {
                reason: format!("invalid model code {invalid:#x}"),
            }),
        }
    }
}
