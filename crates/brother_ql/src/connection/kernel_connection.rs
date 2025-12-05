//! Kernel connection support for Brother QL printers
//!
//! This module provides printer connection via the Linux kernel USB printer driver.
//! It uses standard file I/O on `/dev/usb/lpN` device files, requiring no external
//! USB library dependencies.
//!
//! # Requirements
//!
//! - Linux operating system
//! - USB printer kernel driver loaded (`usblp` module)
//! - Read/write permissions on the device file
//!
//! # Advantages
//!
//! - No external USB library dependencies (uses only standard library)
//! - Simple and lightweight
//! - Works with existing kernel driver setup
//!
//! # Permissions
//!
//! You may need to add your user to the `lp` group:
//! ```bash
//! sudo usermod -a -G lp $USER
//! # Then log out and back in
//! ```
//!
//! # Example
//!
//! ```no_run
//! # use brother_ql::{
//! #     connection::{KernelConnection, PrinterConnection},
//! #     media::Media,
//! #     printjob::PrintJob,
//! # };
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut connection = KernelConnection::open("/dev/usb/lp0")?;
//! let img = image::open("label.png")?;
//! let job = PrintJob::from_image(img, Media::C62)?;
//! connection.print(job)?;
//! # Ok(())
//! # }
//! ```

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::fd::AsFd,
    path::Path,
};

use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use tracing::debug;

use super::{PrinterConnection, printer_connection::sealed::ConnectionImpl};
use crate::error::KernelError;

/// Kernel connection to a Brother QL printer
///
/// Uses Linux kernel USB printer driver for communication.
/// Opens the device file (typically `/dev/usb/lp0`) for reading and writing.
///
/// Implements [`PrinterConnection`] trait for high-level printing operations.
///
/// # Platform Support
///
/// Currently Linux-only due to use of `nix::poll`.
pub struct KernelConnection {
    handle: File,
}

impl KernelConnection {
    /// Open a kernel connection to a Brother QL printer
    ///
    /// Opens the specified device file for bidirectional communication.
    /// Common device paths are `/dev/usb/lp0`, `/dev/usb/lp1`, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device file doesn't exist
    /// - Insufficient permissions to access the device
    /// - The device is already in use by another process
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use brother_ql::connection::KernelConnection;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let connection = KernelConnection::open("/dev/usb/lp0")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P>(path: P) -> Result<Self, KernelError>
    where
        P: AsRef<Path>,
    {
        debug!("Opening kernel connection to the printer...");
        let handle = OpenOptions::new().read(true).write(true).open(path)?;

        debug!("Successfully opened kernel device!");
        Ok(Self { handle })
    }
}

// Implement the public connection interface
impl PrinterConnection for KernelConnection {}

// Implement the private connection interface
impl ConnectionImpl for KernelConnection {
    type Error = KernelError;

    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let bytes_written = self.handle.write(data)?;
        if bytes_written != data.len() {
            return Err(KernelError::IncompleteWrite);
        }
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        // Poll for the device handle to become readable to avoid locking up in case the printer
        // is completely unresponsive (or a different device altogether)
        let mut pollfds = [PollFd::new(self.handle.as_fd(), PollFlags::POLLIN)];
        let nready = poll(&mut pollfds, PollTimeout::ZERO).unwrap_or(0);
        if nready == 0 {
            return Ok(0);
        }
        let bytes_read = self.handle.read(buffer)?;
        Ok(bytes_read)
    }
}
