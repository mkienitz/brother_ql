//! Printer model definitions and USB configuration

use crate::error::StatusParsingError;

/// Brother QL printer model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrinterModel {
    /// QL-560
    QL560,
    /// QL-570
    QL570,
    /// QL-580N
    QL580N,
    /// QL-600
    QL600,
    /// QL-650TD
    QL650TD,
    /// QL-700
    QL700,
    /// QL-710W
    QL710W,
    /// QL-720NW
    QL720NW,
    /// QL-800
    QL800,
    /// QL-810W
    QL810W,
    /// QL-820NWB
    QL820NWB,
}

impl PrinterModel {
    #[cfg(feature = "usb")]
    pub(crate) const fn product_id(self) -> u16 {
        match self {
            PrinterModel::QL560 => 0x2027,
            PrinterModel::QL570 => 0x2028,
            PrinterModel::QL580N => 0x2029,
            PrinterModel::QL600 => 0x20C0,
            PrinterModel::QL650TD => 0x201B,
            PrinterModel::QL700 => 0x2042,
            PrinterModel::QL710W => 0x2043,
            PrinterModel::QL720NW => 0x2044,
            PrinterModel::QL800 => 0x209b,
            PrinterModel::QL810W => 0x209c,
            PrinterModel::QL820NWB => 0x209d,
        }
    }

    #[cfg(feature = "usb")]
    pub(crate) fn from_product_id(product_id: u16) -> Option<Self> {
        match product_id {
            0x2027 => Some(PrinterModel::QL560),
            0x2028 => Some(PrinterModel::QL570),
            0x2029 => Some(PrinterModel::QL580N),
            0x20C0 => Some(PrinterModel::QL600),
            0x201B => Some(PrinterModel::QL650TD),
            0x2042 => Some(PrinterModel::QL700),
            0x2043 => Some(PrinterModel::QL710W),
            0x2044 => Some(PrinterModel::QL720NW),
            0x209b => Some(PrinterModel::QL800),
            0x209c => Some(PrinterModel::QL810W),
            0x209d => Some(PrinterModel::QL820NWB),
            _ => None,
        }
    }
}

impl TryFrom<u8> for PrinterModel {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x31 => Ok(Self::QL560),
            0x32 => Ok(Self::QL570),
            0x33 => Ok(Self::QL580N),
            0x47 => Ok(Self::QL600),
            0x51 => Ok(Self::QL650TD),
            0x35 => Ok(Self::QL700),
            0x36 => Ok(Self::QL710W),
            0x37 => Ok(Self::QL720NW),
            0x38 => Ok(Self::QL800),
            0x39 => Ok(Self::QL810W),
            0x41 => Ok(Self::QL820NWB),
            invalid => Err(StatusParsingError {
                reason: format!("invalid model code {invalid:#x}"),
            }),
        }
    }
}
