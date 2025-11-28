//! Kernel connection support for Brother QL printers
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::fd::AsFd,
    path::Path,
};

use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use tracing::debug;

use super::{printer_connection::sealed::ConnectionImpl, PrinterConnection};
use crate::error::KernelError;

/// Kernel connection to a Brother QL printer
pub struct KernelConnection {
    handle: File,
}

impl KernelConnection {
    /// Open a kernel connection to a Brother QL printer
    ///
    /// # Errors
    /// Returns an error if:
    /// - The device file cannot be opened
    /// - Insufficient permissions to access the device
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
