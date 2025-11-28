//! Kernel connection support for Brother QL printers
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use tracing::debug;

use crate::error::KernelError;

use super::PrinterConnection;

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

impl PrinterConnection for KernelConnection {
    type Error = KernelError;

    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let bytes_written = self.handle.write(data)?;
        if bytes_written != data.len() {
            return Err(KernelError::IncompleteWrite);
        }
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let bytes_read = self.handle.read(buffer)?;
        Ok(bytes_read)
    }
}
