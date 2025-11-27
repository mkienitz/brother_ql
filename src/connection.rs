//! Printer connection types and traits
//!
//! This module provides connection abstractions for communicating with Brother QL printers.
//! Currently supports USB connections, with network connections planned for the future.

mod printer_connection;
mod usb_connection;

// Re-export the trait
pub use printer_connection::PrinterConnection;

// Re-export USB types
pub use usb_connection::{UsbConnection, UsbConnectionInfo};
