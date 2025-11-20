//! Printer model definitions and USB configuration

use std::time::Duration;

use crate::error::BQLError;

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

impl PrinterModel {
    /// Get the USB product ID for this printer model
    pub const fn product_id(self) -> u16 {
        match self {
            Self::QL800 => 0x209b,
            Self::QL810W => 0x209c,
            Self::QL820NWB => 0x209d,
        }
    }

    /// Get the USB vendor ID
    pub const fn vendor_id(self) -> u16 {
        0x04f9 // Brother
    }

    /// Get the USB interface number
    ///
    /// Uses interface 0, alternate setting 0 (the printer interface).
    pub const fn interface(self) -> u8 {
        0
    }

    /// Get the USB OUT endpoint address for writing to the printer
    ///
    /// Endpoint 2 OUT (Bulk transfer, 64 byte max packet size).
    /// Verified from lsusb output.
    pub const fn endpoint_out(self) -> u8 {
        0x02
    }

    /// Get the USB IN endpoint address for reading from the printer
    ///
    /// Endpoint 1 IN (Bulk transfer, 64 byte max packet size).
    /// Verified from lsusb output.
    pub const fn endpoint_in(self) -> u8 {
        0x81
    }

    /// Get the default timeout for USB operations
    pub const fn default_timeout(self) -> Duration {
        Duration::from_millis(5000)
    }
}

impl TryFrom<u8> for PrinterModel {
    type Error = BQLError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x38 => Ok(Self::QL800),
            0x39 => Ok(Self::QL810W),
            0x41 => Ok(Self::QL820NWB),
            invalid => Err(BQLError::MalformedStatus(format!(
                "invalid model code {invalid:#x}"
            ))),
        }
    }
}
