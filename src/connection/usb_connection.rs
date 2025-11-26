//! USB connection support for Brother QL printers
use std::time::Duration;

use rusb::{Context, Device, DeviceHandle, UsbContext};

use crate::{
    commands::RasterCommand, error::BQLError, printer::PrinterModel, printjob::PrintJob,
    status::StatusInformation,
};

use super::PrinterConnection;

/// USB connection parameters for a Brother QL printer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbConnectionInfo {
    /// USB vendor ID (typically 0x04f9 for Brother Industries, Ltd)
    pub vendor_id: u16,
    /// USB product ID (specific to each printer model)
    pub product_id: u16,
    /// USB interface number (typically 0 for printers)
    pub interface: u8,
    /// USB endpoint address for writing data to the printer (OUT endpoint)
    pub endpoint_out: u8,
    /// USB endpoint address for reading data from the printer (IN endpoint)
    pub endpoint_in: u8,
    /// Timeout for USB operations
    pub timeout: Duration,
}

impl UsbConnectionInfo {
    /// Create connection info from a printer model
    pub const fn from_model(model: PrinterModel) -> Self {
        Self {
            vendor_id: model.vendor_id(),
            product_id: model.product_id(),
            interface: model.interface(),
            endpoint_out: model.endpoint_out(),
            endpoint_in: model.endpoint_in(),
            timeout: model.default_timeout(),
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
    pub fn open(info: UsbConnectionInfo) -> Result<Self, BQLError> {
        let context = Context::new()?;
        let device = Self::find_device(&context, info.vendor_id, info.product_id)?;
        let handle = device.open()?;

        // Auto-detach and reattach kernel driver when claiming/releasing
        handle.set_auto_detach_kernel_driver(true)?;

        // Claim the interface for exclusive access
        handle.set_active_configuration(1)?;
        handle.claim_interface(info.interface)?;

        if let Err(e) = handle.set_alternate_setting(info.interface, 0) {
            // NOTE: Since we handle the failed alternate setting call we
            // propagate the original error instead of a possible cleanup one.
            let _ = handle.release_interface(info.interface);
            return Err(e.into());
        }

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
    ) -> Result<Device<Context>, BQLError> {
        let devices = context.devices()?;

        for device in devices.iter() {
            let descriptor = device.device_descriptor()?;
            if descriptor.vendor_id() == vendor_id && descriptor.product_id() == product_id {
                return Ok(device);
            }
        }

        Err(BQLError::UsbDeviceNotFound {
            vendor_id,
            product_id,
        })
    }

    /// Set the timeout for USB operations
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Write data to the printer
    fn write(&mut self, data: &[u8]) -> Result<(), BQLError> {
        let bytes_written = self
            .handle
            .write_bulk(self.endpoint_out, data, self.timeout)?;
        if bytes_written != data.len() {
            return Err(BQLError::IncompleteWrite {
                written: bytes_written,
                expected: data.len(),
            });
        }
        Ok(())
    }

    /// Read status information from the printer
    pub fn get_status(&mut self) -> Result<StatusInformation, BQLError> {
        let invalidate_bytes: Vec<u8> = RasterCommand::Invalidate.into();
        let init_bytes: Vec<u8> = RasterCommand::Initialize.into();
        self.write(&invalidate_bytes)?;
        self.write(&init_bytes)?;
        self.read_status()
    }

    /// Read status information but without sending init/invalidate bytes
    fn read_status(&mut self) -> Result<StatusInformation, BQLError> {
        let status_request_bytes: Vec<u8> = RasterCommand::StatusInformationRequest.into();
        self.write(&status_request_bytes)?;
        let mut read_buffer = [0u8; 32];
        self.read_exact(&mut read_buffer)?;
        StatusInformation::try_from(&read_buffer[..])
    }

    /// Read raw data from the printer
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, BQLError> {
        let bytes_read = self
            .handle
            .read_bulk(self.endpoint_in, buffer, self.timeout)?;
        Ok(bytes_read)
    }

    /// Read until the provided buffer is full
    fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), BQLError> {
        let mut total_read = 0;
        let mut retries = 0;
        const RETRY_DELAY: Duration = Duration::from_millis(50);
        const MAX_RETRIES: u32 = Duration::from_secs(3).div_duration_f32(RETRY_DELAY).ceil() as u32;

        while total_read < buffer.len() {
            match self.read(&mut buffer[total_read..]) {
                Ok(0) => {
                    retries += 1;
                    if retries > MAX_RETRIES {
                        return Err(BQLError::UsbTimeout);
                    }
                    // No data available yet, wait and retry
                    std::thread::sleep(RETRY_DELAY);
                }
                Ok(n) => {
                    total_read += n;
                    retries = 0; // Reset retries on successful read
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Print data and wait for completion
    pub fn print_and_wait(&mut self, data: &[u8]) -> Result<(), BQLError> {
        // TODO: Implement print job execution
        Ok(())
    }
}

impl PrinterConnection for UsbConnection {
    fn print(&mut self, job: &PrintJob) -> Result<(), BQLError> {
        todo!()
    }
}

impl Drop for UsbConnection {
    fn drop(&mut self) {
        // Release the interface when connection is dropped
        let _ = self.handle.release_interface(self.interface);
    }
}
