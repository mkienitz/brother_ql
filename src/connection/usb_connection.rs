//! USB connection support for Brother QL printers
use std::time::Duration;

use rusb::{Context, Device, DeviceHandle, UsbContext};
use tracing::{debug, info};

use crate::{
    commands::{RasterCommand, RasterCommands},
    error::{PrintError, ProtocolError, StatusError, UsbError},
    printer::PrinterModel,
    printjob::PrintJob,
    status::{Phase, StatusInformation, StatusType},
};

use super::PrinterConnection;

/// USB connection parameters for a Brother QL printer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbConnectionInfo {
    /// USB vendor ID (typically 0x04f9 for Brother Industries, Ltd)
    pub(crate) vendor_id: u16,
    /// USB product ID (specific to each printer model)
    pub(crate) product_id: u16,
    /// USB interface number (typically 0 for printers)
    pub(crate) interface: u8,
    /// USB endpoint address for writing data to the printer (OUT endpoint)
    pub(crate) endpoint_out: u8,
    /// USB endpoint address for reading data from the printer (IN endpoint)
    pub(crate) endpoint_in: u8,
    /// Timeout for USB operations
    pub(crate) timeout: Duration,
}

impl UsbConnectionInfo {
    /// Create connection info from a printer model
    ///
    /// Returns the correct USB parameters (vendor ID, product ID, endpoints, etc.)
    /// for the specified printer model. This is the recommended way to create
    /// connection info for known printer models.
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{connection::UsbConnectionInfo, printer::PrinterModel};
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// assert_eq!(info.vendor_id, 0x04f9); // Brother vendor ID
    /// ```
    #[must_use]
    pub const fn from_model(model: PrinterModel) -> Self {
        Self {
            vendor_id: 0x04f9, // Brother
            product_id: model.pid(),
            interface: 0,
            endpoint_out: 0x02,
            endpoint_in: 0x81,
            timeout: Duration::from_millis(5000),
        }
    }
}

/// USB connection to a Brother QL printer
pub struct UsbConnection {
    handle: DeviceHandle<Context>,
    interface: u8,
    timeout: Duration,
    endpoint_out: u8,
    endpoint_in: u8,
}

impl UsbConnection {
    /// Open a USB connection to a Brother QL printer
    ///
    /// Searches for a USB device matching the vendor and product IDs in the connection info,
    /// then claims the interface for exclusive access. The kernel driver is automatically
    /// detached and will be reattached when the connection is closed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - No USB device is found with the specified vendor/product ID
    /// - The USB device cannot be opened
    /// - The interface cannot be claimed
    /// - USB configuration fails
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{
    /// #     connection::{UsbConnection, UsbConnectionInfo},
    /// #     printer::PrinterModel,
    /// # };
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// let connection = UsbConnection::open(info)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(info: UsbConnectionInfo) -> Result<Self, UsbError> {
        debug!("Opening USB Connection to the printer...");
        let context = Context::new()?;
        let device = Self::find_device(&context, info.vendor_id, info.product_id)?;
        let handle = device.open()?;

        // Auto-detach and reattach kernel driver when claiming/releasing
        handle.set_auto_detach_kernel_driver(true)?;
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
        handle.set_active_configuration(1)?;
        // Claim the interface for exclusive access
        handle.claim_interface(info.interface)?;

        if let Err(e) = handle.set_alternate_setting(info.interface, 0) {
            // NOTE: Since we handle the failed alternate setting call we
            // propagate the original error instead of a possible cleanup one.
            let _ = handle.release_interface(info.interface);
            return Err(e.into());
        }

        debug!("Successfully established USB Connection!");
        Ok(Self {
            handle,
            interface: info.interface,
            timeout: info.timeout,
            endpoint_out: info.endpoint_out,
            endpoint_in: info.endpoint_in,
        })
    }

    /// Find a USB device with the specified vendor and product IDs
    fn find_device(
        context: &Context,
        vendor_id: u16,
        product_id: u16,
    ) -> Result<Device<Context>, UsbError> {
        let devices = context.devices()?;

        for device in devices.iter() {
            let descriptor = device.device_descriptor()?;
            if descriptor.vendor_id() == vendor_id && descriptor.product_id() == product_id {
                return Ok(device);
            }
        }

        Err(UsbError::DeviceNotFound {
            vendor_id,
            product_id,
        })
    }

    /// Write data to the printer
    fn write(&self, data: &[u8]) -> Result<(), UsbError> {
        let bytes_written = self
            .handle
            .write_bulk(self.endpoint_out, data, self.timeout)?;
        if bytes_written != data.len() {
            return Err(UsbError::IncompleteWrite);
        }
        Ok(())
    }

    fn send_status_request(&self) -> Result<(), UsbError> {
        debug!("Sending status information request to the printer...");
        let status_request_bytes: Vec<u8> = RasterCommand::StatusInformationRequest.into();
        self.write(&status_request_bytes)?;
        Ok(())
    }

    /// Read status information from the printer
    ///
    /// Sends a status request to the printer and returns detailed [`StatusInformation`] about:
    /// - Printer model
    /// - Current errors (if any)
    /// - Media type and dimensions
    /// - Current phase (receiving, printing, etc.)
    /// - Various mode settings
    ///
    /// This method sends the initialization preamble before requesting status.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Communication with the printer fails
    /// - The status reply is malformed or incomplete
    /// - USB timeout occurs
    ///
    /// # Example
    /// ```no_run
    /// # use brother_ql::{
    /// #     connection::{UsbConnection, UsbConnectionInfo},
    /// #     printer::PrinterModel,
    /// # };
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let info = UsbConnectionInfo::from_model(PrinterModel::QL820NWB);
    /// let mut connection = UsbConnection::open(info)?;
    ///
    /// let status = connection.get_status()?;
    /// println!("Printer: {:?}", status.model);
    /// println!("Media width: {}mm", status.media_width);
    ///
    /// if status.has_errors() {
    ///     eprintln!("Printer errors: {:?}", status.errors);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        let bytes_read = self
            .handle
            .read_bulk(self.endpoint_in, buffer, self.timeout)?;
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

impl PrinterConnection for UsbConnection {
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

impl Drop for UsbConnection {
    fn drop(&mut self) {
        // Release the interface when connection is dropped
        let _ = self.handle.release_interface(self.interface);
    }
}
