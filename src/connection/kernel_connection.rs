//! Kernel connection support for Brother QL printers
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use tracing::{debug, info};

use crate::{
    commands::{RasterCommand, RasterCommands},
    error::{KernelError, PrintError, ProtocolError, StatusError},
    printjob::PrintJob,
    status::{Phase, StatusInformation, StatusType},
};

use super::PrinterConnection;

/// Kernel connection to a Brother QL printer
pub struct KernelConnection {
    handle: File,
}

impl KernelConnection {
    /// Open a kernel connection to a Brother QL printer
    pub fn open<P>(path: P) -> Result<Self, KernelError>
    where
        P: AsRef<Path>,
    {
        debug!("Opening kernel connection to the printer...");
        let handle = OpenOptions::new().read(true).write(true).open(path)?;

        debug!("Successfully opened kernel device!");
        Ok(Self { handle })
    }

    /// Write data to the printer
    fn write(&mut self, data: &[u8]) -> Result<(), KernelError> {
        let bytes_written = self.handle.write(data)?;
        if bytes_written != data.len() {
            return Err(KernelError::IncompleteWrite);
        }
        Ok(())
    }

    fn send_status_request(&mut self) -> Result<(), KernelError> {
        debug!("Sending status information request to the printer...");
        let status_request_bytes: Vec<u8> = RasterCommand::StatusInformationRequest.into();
        self.write(&status_request_bytes)?;
        Ok(())
    }

    /// Read status information from the printer
    pub fn get_status(&mut self) -> Result<StatusInformation, StatusError> {
        let preamble_bytes = RasterCommands::create_preamble().build();
        self.write(&preamble_bytes)?;
        self.send_status_request()?;
        self.read_status_reply()
    }

    /// Read status information but without sending init/invalidate bytes
    fn read_status_reply(&mut self) -> Result<StatusInformation, StatusError> {
        let mut read_buffer = [0u8; 32];
        self.read_exact(&mut read_buffer)?;
        let status = StatusInformation::try_from(&read_buffer[..])?;
        debug!(?status, "Printer sent status information");
        Ok(status)
    }

    /// Read raw data from the printer
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, KernelError> {
        let bytes_read = self.handle.read(buffer)?;
        Ok(bytes_read)
    }

    /// Read until the provided buffer is full
    fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), StatusError> {
        // 3000ms / 50ms = 60 retries
        const MAX_RETRIES: u8 = 60;
        const RETRY_DELAY: Duration = Duration::from_millis(50);

        let mut total_read = 0;
        let mut retries = 0;

        while total_read < buffer.len() {
            match self.read(&mut buffer[total_read..]) {
                Ok(0) => {
                    retries += 1;
                    if retries > MAX_RETRIES {
                        return Err(StatusError::NoResponse);
                    }
                    // No data available yet, wait and retry
                    std::thread::sleep(RETRY_DELAY);
                }
                Ok(n) => {
                    total_read += n;
                    retries = 0; // Reset retries on successful read
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    /// Validate that information reply matches expected state
    fn validate_status(
        status: &StatusInformation,
        expected_type: &StatusType,
        expected_phase: &Phase,
    ) -> Result<(), ProtocolError> {
        // Check if printer has errors first
        if status.has_errors() {
            return Err(ProtocolError::PrinterError(status.errors));
        }

        // Check if status type and phase match expectations
        if &status.status_type != expected_type || &status.phase != expected_phase {
            return Err(ProtocolError::UnexpectedStatus {
                expected_type: expected_type.clone(),
                expected_phase: expected_phase.clone(),
                actual_type: status.status_type.clone(),
                actual_phase: status.phase.clone(),
            });
        }

        Ok(())
    }
}

impl PrinterConnection for KernelConnection {
    fn print(&mut self, job: PrintJob) -> Result<(), PrintError> {
        info!(?job, "Starting print job...");
        let no_pages = job.page_count;
        let parts = job.into_parts();
        // Send preamble
        self.write(&parts.preamble.build())?;
        // Send status information request and validate printer is ready
        let status = self.get_status().map_err(PrintError::Status)?;
        Self::validate_status(&status, &StatusType::StatusRequestReply, &Phase::Receiving)?;
        for (page_no, page) in parts.page_data.into_iter().enumerate() {
            debug!(
                "Sending print data for page {}/{}...",
                page_no + 1,
                no_pages
            );
            self.write(&page.build())?;
            // Printer should change phase to "Printing"
            let status = self.read_status_reply().map_err(PrintError::Status)?;
            Self::validate_status(&status, &StatusType::PhaseChange, &Phase::Printing)?;
            // Printer should signal print completion
            let status = self.read_status_reply().map_err(PrintError::Status)?;
            Self::validate_status(&status, &StatusType::PrintingCompleted, &Phase::Printing)?;
            // Printer should change phase to "Receiving" again
            let status = self.read_status_reply().map_err(PrintError::Status)?;
            Self::validate_status(&status, &StatusType::PhaseChange, &Phase::Receiving)?;
            info!("Page {}/{} printed successfully!", page_no + 1, no_pages);
        }
        info!("Print job completed successfully!");
        Ok(())
    }
}
