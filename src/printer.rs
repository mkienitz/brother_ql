//! Printer model definitions and USB configuration

use crate::error::StatusParsingError;

macro_rules! printer_models {
    ($($name:ident ($pid:expr, $rcode:expr),)+) => {
        /// Brother QL printer models
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum PrinterModel {
            $(
                #[doc = stringify!($name)]
                $name,
            )+
        }

        impl PrinterModel {
            #[cfg(feature = "usb")]
            pub(crate) const fn product_id(self) -> u16 {
                match self {
                    $(Self::$name => $pid,)+
                }
            }

            #[cfg(feature = "usb")]
            pub(crate) const fn from_product_id(product_id: u16) -> Option<Self> {
                match product_id {
                    $($pid => Some(Self::$name),)+
                    _ => None,
                }
            }
        }

        impl TryFrom<u8> for PrinterModel {
            type Error = StatusParsingError;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($rcode => Ok(Self::$name),)+
                    invalid => Err(StatusParsingError {
                        reason: format!("invalid model code {invalid:#x}"),
                    }),
                }
            }
        }
    };
}

printer_models! {
    // Define all printer constants here. Usage:
    // <enum variant name> (<USB Product ID>, <Raster Model Code>)
    // - <product_id> comes from the printer's USB specification
    // - <Raster Model Code> is specified in the Raster Command Reference
    //   for the status information reply
    QL560   (0x2027, 0x31),
    QL570   (0x2028, 0x32),
    QL580N  (0x2029, 0x33),
    QL600   (0x20C0, 0x47),
    QL650TD (0x201B, 0x51),
    QL700   (0x2042, 0x35),
    QL710W  (0x2043, 0x36),
    QL720NW (0x2044, 0x37),
    QL800   (0x209b, 0x38),
    QL810W  (0x209c, 0x39),
    QL820NWB(0x209d, 0x41),
}
