//! Printer connection types and traits
//!
//! This module provides connection abstractions for communicating with Brother QL printers
//! via different transport methods.
//!
//! # Connection Types
//!
//! - [`UsbConnection`] - Direct USB communication (requires `usb` feature)
//! - [`KernelConnection`] - Uses Linux kernel USB printer driver (requires `kernel` feature, enabled by default)
//!
//! All connection types implement the [`PrinterConnection`] trait, which provides
//! high-level methods like [`print()`](PrinterConnection::print) and
//! [`get_status()`](PrinterConnection::get_status).
//!
//! # Choosing a Connection Type
//!
//! **Use [`UsbConnection`] when:**
//! - You need cross-platform support (Windows, macOS, Linux)
//! - You want full control over USB communication
//! - You need detailed device enumeration
//!
//! **Use [`KernelConnection`] when:**
//! - You're on Linux and want minimal dependencies
//! - The kernel USB printer driver (`usblp`) is already loaded
//! - You want to avoid external USB library dependencies
//!
//! # Future
//!
//! Network connection support is planned for future releases.

#[cfg(feature = "kernel")]
mod kernel_connection;
mod printer_connection;
#[cfg(feature = "usb")]
mod usb_connection;

// Re-export the trait
pub use printer_connection::PrinterConnection;

// Re-export USB types
#[cfg(feature = "usb")]
pub use usb_connection::{UsbConnection, UsbConnectionInfo};

// Re-export kernel types
#[cfg(feature = "kernel")]
pub use kernel_connection::KernelConnection;
